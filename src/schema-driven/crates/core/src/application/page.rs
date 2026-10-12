//! ページの生成（page.schema.json）。
//! 節を並べ、show ・ each を適用し、式を描画の文脈で評価し、部品の描画はカタログ（ポート Design）を呼び出す。
//! 文字は必ずエスケープし、部品のプレースホルダーと渡した名前が合わなければエラーにする。

use crate::application::view::{to_text, truthy, ViewEngine, ViewError};
use crate::domain::design_system;
use crate::domain::schema::Schema;
use crate::domain::values::Html;
use crate::ports::Renderer;
use crate::ports::{Design, Frame, Query};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// 部品の HTML を、id → 中身に分ける。
fn parse_parts(source: &str) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    let mut rest = source;
    while let Some(start) = rest.find("<template id=\"") {
        let after = &rest[start + "<template id=\"".len()..];
        let id_end = after.find('"').ok_or("部品の id が閉じていない")?;
        let id = after[..id_end].to_owned();
        let body_start = after[id_end..]
            .find('>')
            .ok_or("部品の開きタグが閉じていない")?
            + id_end
            + 1;
        let body_end = after[body_start..]
            .find("</template>")
            .ok_or_else(|| format!("部品 {id} が閉じていない"))?
            + body_start;
        if out
            .insert(id.clone(), after[body_start..body_end].to_owned())
            .is_some()
        {
            return Err(format!("部品 {id} が2つある"));
        }
        rest = &after[body_end + "</template>".len()..];
    }
    Ok(out)
}

/// 部品のプレースホルダー {{名前}} の一覧。
fn slots_of(template: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = template;
    while let Some(open) = rest.find("{{") {
        let after = &rest[open + 2..];
        match after.find("}}") {
            Some(close) => {
                out.insert(after[..close].to_owned());
                rest = &after[close + 2..];
            }
            None => break,
        }
    }
    out
}

/// ページテンプレートと部品のカタログから、ページを生成する。
pub struct PageRenderer<Q> {
    engine: ViewEngine<Q>,
    design: Arc<dyn Design>,
    parts: BTreeMap<String, String>,
    components: BTreeSet<String>,
    tokens: String,
}

impl<Q: Query> PageRenderer<Q> {
    pub fn new(engine: ViewEngine<Q>, design: Arc<dyn Design>) -> Result<Self, ViewError> {
        let parts = parse_parts(design.parts()).map_err(ViewError)?;
        let components = design
            .components()
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        let tokens = design_system::css(design.design_system());
        Ok(Self {
            engine,
            design,
            parts,
            components,
            tokens,
        })
    }

    /// 描画の文脈（contexts の1件）に、ページテンプレートを適用して1ページを生成する。schema はこのインスタンスのスキーマの名前。
    pub fn render(&self, page: &Value, context: &Value, schema: &str) -> Result<Html, ViewError> {
        let mut run = Run {
            page_renderer: self,
            schema,
            current: context.clone(),
        };
        let err = ViewError;
        let head = page.get("head").cloned().unwrap_or(Value::Null);
        let mut badges = Html::default();
        for badge in head
            .get("badges")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            badges.push(run.node(badge).map_err(err)?);
        }
        let lead = match head.get("lead") {
            Some(node) => run.node(node).map_err(err)?,
            None => Html::default(),
        };
        let mut sections = Html::default();
        for section in page
            .get("sections")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(show) = section.get("show").and_then(Value::as_str) {
                if !truthy(&run.eval(show).map_err(err)?) {
                    continue;
                }
            }
            let mut body = Html::default();
            for node in section
                .get("body")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                body.push(run.node(node).map_err(err)?);
            }
            let heading = section.get("heading").and_then(Value::as_str).unwrap_or("");
            sections.push(self.design.section(heading, body, &run).map_err(err)?);
        }
        // 題は、スキーマの根の x-view の label。無ければ id。
        let this = context.get("this").cloned().unwrap_or(Value::Null);
        let title = self.engine.title(schema, &this).unwrap_or_else(|| {
            this.get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        });
        let frame = Frame {
            title,
            badges,
            lead,
            sections,
        };
        self.design.page(&frame, &run).map_err(err)
    }
}

/// 1ページを生成するあいだの状態。今の位置（current）は each と node_at で替わる。
struct Run<'renderer, Q> {
    page_renderer: &'renderer PageRenderer<Q>,
    schema: &'renderer str,
    current: Value,
}

impl<Q: Query> Run<'_, Q> {
    fn eval(&self, expr: &str) -> Result<Value, String> {
        self.page_renderer
            .engine
            .evaluate(self.schema, expr, &self.current)
            .map_err(|error| format!("式 {expr}: {}", error.0))
    }

    fn component(&mut self, node: &Value) -> Result<Html, String> {
        if let Some(show) = node.get("show").and_then(Value::as_str) {
            if !truthy(&self.eval(show)?) {
                return Ok(Html::default());
            }
        }
        if let Some(each) = node.get("each").and_then(Value::as_str) {
            let list = self.eval(each)?;
            let mut once = node.clone();
            if let Some(object) = once.as_object_mut() {
                object.remove("each");
                object.remove("show");
            }
            let mut out = Html::default();
            for item in list.as_array().cloned().unwrap_or_default() {
                out.push(self.node_at(&once, &item)?);
            }
            return Ok(out);
        }
        let name = node.get("component").and_then(Value::as_str).unwrap_or("");
        if !self.page_renderer.components.contains(name) {
            return Err(format!("知らないコンポーネント: {name}"));
        }
        let design = self.page_renderer.design.clone();
        design.render(name, node, self)
    }
}

impl<Q: Query> Renderer for Run<'_, Q> {
    fn value(&mut self, expr: &Value) -> Result<Value, String> {
        match expr {
            Value::String(expr_text) => self.eval(expr_text),
            Value::Object(object) if !object.contains_key("component") => {
                Ok(object.get("text").cloned().unwrap_or(Value::Null))
            }
            Value::Null => Ok(Value::Null),
            _ => Err("コンポーネントは値にできない".into()),
        }
    }

    fn node(&mut self, node: &Value) -> Result<Html, String> {
        match node {
            Value::String(expr_text) => Ok(Html::escape(&to_text(&self.eval(expr_text)?))),
            Value::Object(object) if object.contains_key("component") => self.component(node),
            Value::Object(object) => Ok(Html::escape(
                object.get("text").and_then(Value::as_str).unwrap_or(""),
            )),
            Value::Null => Ok(Html::default()),
            other => Err(format!("要素の形が違う: {other}")),
        }
    }

    fn node_at(&mut self, node: &Value, at: &Value) -> Result<Html, String> {
        let saved = std::mem::replace(&mut self.current, at.clone());
        let out = self.node(node);
        self.current = saved;
        out
    }

    fn part(&self, id: &str, slots: &[(&str, Html)]) -> Result<Html, String> {
        let template = self
            .page_renderer
            .parts
            .get(id)
            .ok_or_else(|| format!("部品 {id} が無い"))?;
        let want = slots_of(template);
        let given: BTreeSet<String> = slots.iter().map(|(name, _)| (*name).to_owned()).collect();
        if want != given {
            let missing: Vec<_> = want.difference(&given).cloned().collect();
            let extra: Vec<_> = given.difference(&want).cloned().collect();
            return Err(format!(
                "部品 {id} のプレースホルダーが合わない（足りない {missing:?}、余る {extra:?}）"
            ));
        }
        let mut out = template.clone();
        for (name, html) in slots {
            out = out.replace(&format!("{{{{{name}}}}}"), html.as_str());
        }
        Ok(Html::trusted(out))
    }

    fn text(&self, text: &str) -> Html {
        Html::escape(text)
    }

    fn svg(&self, svg: &str) -> Result<Html, String> {
        let lower = svg.to_lowercase();
        if !lower.trim_start().starts_with("<svg") {
            return Err("SVG の文字列が <svg で始まっていない".into());
        }
        let has_event_attribute = lower
            .split(|character: char| character.is_whitespace())
            .any(|word| word.starts_with("on") && word.contains('='));
        if lower.contains("<script") || lower.contains("javascript:") || has_event_attribute {
            return Err("SVG に script ・ イベントの属性 ・ javascript: が含まれている".into());
        }
        Ok(Html::trusted(svg.to_owned()))
    }

    fn tokens(&self) -> Html {
        Html::trusted(self.page_renderer.tokens.clone())
    }
}

/// ページテンプレートの部品を、カタログの名前と入力の形で検査する。見つかった順に返す。
pub fn page_errors(design: &dyn Design, page: &Value) -> Vec<String> {
    let shapes: BTreeMap<String, Value> = design.components().into_iter().collect();
    let mut out = Vec::new();
    walk(page, "", &shapes, &mut out);
    out
}

fn walk(value: &Value, at: &str, shapes: &BTreeMap<String, Value>, out: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            if let Some(name) = object.get("component").and_then(Value::as_str) {
                match shapes.get(name) {
                    None => out.push(format!("{at}: 知らないコンポーネント {name}")),
                    Some(shape) => match Schema::new(name, shape.clone(), vec![]).validate(value) {
                        Ok(validation) => {
                            for unfilled in &validation.unfilled {
                                out.push(format!("{at}: {name} に {} が無い", unfilled.property()));
                            }
                            for error in &validation.errors {
                                out.push(format!(
                                    "{at}: {name} の {} {}",
                                    error.property(),
                                    error.reason()
                                ));
                            }
                        }
                        Err(error) => out.push(format!(
                            "{at}: {name} の入力の形が壊れている（{}）",
                            error.0
                        )),
                    },
                }
            }
            for (key, child) in object {
                walk(child, &format!("{at}/{key}"), shapes, out);
            }
        }
        Value::Array(elements) => {
            for (position, child) in elements.iter().enumerate() {
                walk(child, &format!("{at}/{position}"), shapes, out);
            }
        }
        _ => {}
    }
}

/// 基盤の文書（schemas/document.schema.json）の決まったページテンプレート。何も指定が無いときに使う。
/// 題は x-view の label（title）、題の上に badges、題の下に lead。目次 ・ 節とブロック ・ 出どころは、部品 document が描画する。
/// 節の見出しを空にして、部品 document の出力をそのまま置く。
pub fn document_page_template() -> Value {
    serde_json::json!({
        "kind": "document",
        "head": {
            "badges": [{"component": "badges", "show": "this.badges", "value": "this.badges"}],
            "lead": {"component": "inline", "show": "this.lead", "value": "this.lead"}
        },
        "sections": [{"heading": "", "body": [{"component": "document", "value": "this"}]}]
    })
}

#[cfg(test)]
mod tests {
    //! ページの生成のテスト。
    //! テスト用の小さな具体（TestDesign）が、ポート Design にコンポーネントと部品の実装を渡す。
    //! 基盤は節を並べ、show ・ each を適用し、式を描画の文脈で評価し、文字をエスケープし、プレースホルダーの過不足をエラーにする。

    use super::{page_errors, PageRenderer};
    use crate::application::context::contexts;
    use crate::application::view::{Instance, ViewEngine};
    use crate::domain::check::Doc;
    use crate::domain::schema::Schema;
    use crate::domain::values::JsonValue;
    use crate::tests::{at, Answer, TableQuery, TestDesign};
    use serde_json::{json, Value};
    use std::sync::Arc;

    const PARTS: &str = r#"<template id="page"><main><h1>{{title}}</h1>{{badges}}{{lead}}{{sections}}</main></template>
<template id="section"><section><h2>{{heading}}</h2>{{body}}</section></template>
<template id="pill"><span class="pill {{tone}}">{{text}}</span></template>
<template id="table"><table><tr>{{head}}</tr>{{body}}</table></template>
<template id="th"><th>{{text}}</th></template>
<template id="tr"><tr>{{cells}}</tr></template>
<template id="td"><td>{{text}}</td></template>
<template id="lead"><p>{{text}}</p></template>"#;

    /// テンプレートは色を使わないので、トークンは最小にする。
    const MINI_SYSTEM: &str = r##"{"scope": ":root",
 "tokens": {"base": {"color": {"ink-900": "#111d1a", "mist-100": "#e6eeeb"}},
            "semantic": {"light": {"ink": "ink-900"}, "dark": {"ink": "mist-100"}}, "component": {}},
 "components": [{"name": "pill", "states": {"default": {"text": "ink"}}},
                {"name": "table", "states": {"default": {"text": "ink"}}}]}"##;

    fn design(parts: &str) -> TestDesign {
        TestDesign {
            parts: parts.to_owned(),
            system: MINI_SYSTEM.to_owned(),
        }
    }

    /// 見本のページテンプレートの式と答え。
    fn answers() -> Vec<(&'static str, Answer)> {
        vec![
            ("this.header.level", |value, _| {
                Ok(at(value, "/this/header/level"))
            }),
            ("this.header.name", |value, _| {
                Ok(at(value, "/this/header/name"))
            }),
            ("this.steps", |value, _| Ok(at(value, "/this/steps"))),
            ("this.tags", |value, _| Ok(at(value, "/this/tags"))),
            ("@", |value, _| Ok(value.clone())),
            ("'出ない'", |_, _| Ok(json!("出ない"))),
            ("links[?at=='/rules'].target", |value, _| {
                Ok(value["links"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|link| link["at"] == "/rules")
                    .map(|link| link["target"].clone())
                    .collect())
            }),
            ("name(@)", |value, functions| {
                functions.call("name", std::slice::from_ref(value))
            }),
            ("header.name", |value, _| Ok(at(value, "/header/name"))),
            ("name", |value, _| Ok(at(value, "/name"))),
        ]
    }

    fn uc_schema() -> Value {
        json!({"x-view": {"label": "header.name"},
               "properties": {"rules": {"x-ref": {"to": "rule"}}}})
    }

    fn rule_schema() -> Value {
        json!({"x-view": {"label": "name"}})
    }

    fn instances() -> Vec<Value> {
        vec![
            json!({"id": "UC-1", "kind": "uc", "header": {"name": "注文 <する>", "level": "user"},
                   "tags": ["甲", "乙"], "rules": ["R-1", "R-2"], "steps": []}),
            json!({"id": "R-1", "kind": "rule", "name": "在庫がある", "owner": "倉庫"}),
            json!({"id": "R-2", "kind": "rule", "name": "支払える", "owner": "経理"}),
        ]
    }

    fn schema_name(value: &Value) -> String {
        format!("{}.schema.json", value["kind"].as_str().unwrap())
    }

    /// 見本のインスタンス（UC-1）のページを生成する。
    fn render_uc(page: Value, parts: &str) -> Result<String, String> {
        let uc = Schema::new("uc.schema.json", uc_schema(), vec![]);
        let rule = Schema::new("rule.schema.json", rule_schema(), vec![]);
        let values = instances();
        let docs: Vec<Doc> = values
            .iter()
            .map(|value| Doc {
                path: format!("{}.json", value["id"].as_str().unwrap()),
                value: value.clone(),
                hash: JsonValue::new(&value.to_string()).hash(),
                schema: [&uc, &rule][usize::from(value["kind"] == "rule")],
            })
            .collect();
        let ctx = contexts(&docs).remove(0);
        let list = values
            .iter()
            .map(|value| Instance {
                value: value.clone(),
                schema: schema_name(value),
            })
            .collect();
        let engine = ViewEngine::new(
            TableQuery::new(answers()),
            vec![
                (
                    "uc.schema.json".into(),
                    Schema::new("uc.schema.json", uc_schema(), vec![]),
                ),
                (
                    "rule.schema.json".into(),
                    Schema::new("rule.schema.json", rule_schema(), vec![]),
                ),
            ],
            list,
        );
        let renderer =
            PageRenderer::new(engine, Arc::new(design(parts))).map_err(|error| error.0)?;
        renderer
            .render(&page, &ctx, "uc.schema.json")
            .map(|html| html.as_str().to_owned())
            .map_err(|error| error.0)
    }

    #[test]
    fn frame_title_comes_from_label_and_sections_follow_show() {
        let page = json!({"kind": "uc", "head": {"lead": "this.header.level"}, "sections": [
            {"heading": "出る節", "body": [{"text": "固定"}]},
            {"heading": "出ない節", "show": "this.steps", "body": [{"text": "x"}]}]});
        assert_eq!(
            render_uc(page, PARTS).unwrap(),
            "<main><h1>注文 &lt;する&gt;</h1><p>user</p><section><h2>出る節</h2>固定</section></main>"
        );
    }

    #[test]
    fn expression_nodes_are_escaped_and_arrays_joined() {
        let page = json!({"kind": "uc", "sections": [{"heading": "h", "body": ["this.header.name", "this.tags"]}]});
        let html = render_uc(page, PARTS).unwrap();
        assert!(
            html.contains("<section><h2>h</h2>注文 &lt;する&gt;甲 ・ 乙</section>"),
            "{html}"
        );
    }

    #[test]
    fn component_each_and_show_are_applied_by_the_base() {
        let page = json!({"kind": "uc", "sections": [{"heading": "h", "body": [
            {"component": "pill", "each": "this.tags", "value": "@", "tone": "accent"},
            {"component": "pill", "show": "this.steps", "value": "'出ない'"}]}]});
        let html = render_uc(page, PARTS).unwrap();
        assert!(
            html.contains(
                r#"<span class="pill accent">甲</span><span class="pill accent">乙</span></section>"#
            ),
            "{html}"
        );
    }

    #[test]
    fn table_renders_column_nodes_against_each_row() {
        let page = json!({"kind": "uc", "sections": [{"heading": "h", "body": [
            {"component": "table", "rows": "links[?at=='/rules'].target",
             "cols": [{"head": "ルール", "value": "name(@)"},
                      {"head": "バッジ", "value": {"component": "pill", "value": "@"}}]}]}]});
        let html = render_uc(page, PARTS).unwrap();
        assert!(
            html.contains(
                r#"<table><tr><th>ルール</th><th>バッジ</th></tr><tr><td>在庫がある</td><td><span class="pill plain">R-1</span></td></tr><tr><td>支払える</td><td><span class="pill plain">R-2</span></td></tr></table>"#
            ),
            "{html}"
        );
    }

    #[test]
    fn part_slots_must_match_exactly() {
        let missing = r#"<template id="page"><main>{{title}}{{sections}}</main></template>
<template id="section"><section>{{heading}}{{body}}</section></template>
<template id="lead"><p>{{text}}</p></template>"#;
        let page = json!({"kind": "uc", "sections": [{"heading": "h", "body": [{"text": "x"}]}]});
        let err = render_uc(page, missing).unwrap_err();
        assert!(err.contains("badges"), "{err}");
    }

    #[test]
    fn unknown_component_is_an_error() {
        let page =
            json!({"kind": "uc", "sections": [{"heading": "h", "body": [{"component": "nope"}]}]});
        // 具体に渡す前に、基盤が登録されていない名前を止める
        assert_eq!(
            render_uc(page, PARTS).unwrap_err(),
            "知らないコンポーネント: nope"
        );
    }

    #[test]
    fn page_errors_report_unknown_components_and_input_shape() {
        let page = json!({"kind": "uc", "sections": [{"heading": "h", "body": [
            {"component": "nope"},
            {"component": "pill"},
            {"component": "table", "rows": "x", "cols": [{"head": "h", "value": {"component": "pill", "value": "@", "tone": 1}}]}]}]});
        let errors = page_errors(&design(PARTS), &page);
        assert_eq!(errors.len(), 3, "{errors:?}");
        assert!(errors[0].contains("nope"));
        assert!(errors[1].contains("pill") && errors[1].contains("value"));
        assert!(errors[2].contains("tone"));
    }
}
