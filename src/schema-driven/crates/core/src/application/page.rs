//! ページの生成（page.schema.json）。
//! 節を並べ、show ・ each を適用し、式を描画の文脈で評価し、concrete が渡したカタログのテンプレートへ値を入れる。
//! 部品の名前も見た目も知らない。部品の呼び出しは、同じ名前のテンプレートの {{入力の名前}} へ入力の値を入れるだけである。
//! 文字は必ずエスケープする。SVG を埋めるのは、入力の形が contentMediaType で SVG と宣言した入力だけである。

use crate::application::view::{to_text, truthy, ViewEngine, ViewError};
use crate::domain::catalog::{parse_parts, shapes, slots_of, CONTROL, PAGE, PAGE_SLOTS, SECTION};
use crate::domain::design_system;
use crate::domain::schema::Schema;
use crate::domain::values::Html;
use crate::ports::{Catalog, Query};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// 入力の形が、値を SVG として埋めると宣言しているか。
const SVG: &str = "image/svg+xml";

/// 検査を通ったカタログを、描画に使う形にしたもの。
pub struct Parts {
    templates: BTreeMap<String, String>,
    shapes: BTreeMap<String, Value>,
    tokens: String,
    style: String,
    script: String,
}

impl Parts {
    /// カタログを読み取る。テンプレートが読めなければエラー（形の検査は domain::catalog::check が持つ）。
    pub fn new(catalog: &Catalog) -> Result<Self, String> {
        let schema: Value =
            serde_json::from_str(&catalog.schema).map_err(|error| error.to_string())?;
        Ok(Self {
            templates: parse_parts(&catalog.parts)?,
            shapes: shapes(&schema),
            tokens: design_system::css(&catalog.design_system),
            style: catalog.style.clone(),
            script: catalog.script.clone(),
        })
    }

    /// テンプレートのプレースホルダーへ値を入れる。値の無いプレースホルダーは空にする。
    /// 1回だけ左から読むので、入れた値の中の {{…}} は置き換えない。
    fn fill(&self, id: &str, values: &BTreeMap<String, Html>) -> Result<Html, String> {
        let template = self
            .templates
            .get(id)
            .ok_or_else(|| format!("テンプレート {id} が無い"))?;
        let mut out = String::new();
        let mut rest = template.as_str();
        while let Some(open) = rest.find("{{") {
            out.push_str(&rest[..open]);
            let after = &rest[open + 2..];
            let Some(close) = after.find("}}") else {
                out.push_str(&rest[open..]);
                rest = "";
                break;
            };
            if let Some(value) = values.get(&after[..close]) {
                out.push_str(value.as_str());
            }
            rest = &after[close + 2..];
        }
        out.push_str(rest);
        Ok(Html::trusted(out))
    }
}

/// ページテンプレートと部品のカタログから、ページを生成する。
pub struct PageRenderer<Q> {
    engine: ViewEngine<Q>,
    parts: Parts,
}

impl<Q: Query> PageRenderer<Q> {
    pub fn new(engine: ViewEngine<Q>, parts: Parts) -> Self {
        Self { engine, parts }
    }

    /// 描画の文脈（contexts の1件）に、ページテンプレートを適用して1ページを生成する。schema はこのインスタンスのスキーマの名前。
    pub fn render(&self, page: &Value, context: &Value, schema: &str) -> Result<Html, ViewError> {
        let mut run = Run {
            page_renderer: self,
            schema,
            current: context.clone(),
        };
        let err = ViewError;
        let mut values = BTreeMap::new();
        for (slot, node) in page
            .get("head")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            values.insert(slot.clone(), run.slot(node, None).map_err(err)?);
        }
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
            let text =
                |key: &str| Html::escape(section.get(key).and_then(Value::as_str).unwrap_or(""));
            let section_values = BTreeMap::from([
                ("heading".to_owned(), text("heading")),
                ("help".to_owned(), text("help")),
                (
                    "body".to_owned(),
                    run.slot(&section["body"], None).map_err(err)?,
                ),
            ]);
            sections.push(self.parts.fill(SECTION, &section_values).map_err(err)?);
        }
        // 題は、スキーマの根の x-view の label。無ければ id。
        let this = context.get("this").cloned().unwrap_or(Value::Null);
        let title = self.engine.title(schema, &this).unwrap_or_else(|| {
            this.get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        });
        let given = [
            Html::escape(&title),
            Html::trusted(self.parts.tokens.clone()),
            Html::trusted(self.parts.style.clone()),
            Html::trusted(self.parts.script.clone()),
            sections,
        ];
        values.extend(PAGE_SLOTS.iter().map(|slot| (*slot).to_owned()).zip(given));
        self.parts.fill(PAGE, &values).map_err(err)
    }
}

/// 1ページを生成するあいだの状態。今の位置（current）は each で替わる。
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

    /// 要素1つ：文字列は式の値を文字に、{"text": …} は決まった文字に、component を持つものは部品にする。
    fn node(&mut self, node: &Value) -> Result<Html, String> {
        match node {
            Value::String(expr_text) => Ok(Html::escape(&to_text(&self.eval(expr_text)?))),
            Value::Object(object) if object.contains_key("component") => self.component(object),
            Value::Object(object) => Ok(Html::escape(
                object.get("text").and_then(Value::as_str).unwrap_or(""),
            )),
            Value::Null => Ok(Html::default()),
            other => Err(format!("要素の形が違う: {other}")),
        }
    }

    /// プレースホルダー1つに入れる値。並びは要素ごとに描いてつなぐ。shape が SVG を宣言していれば、式の値を SVG として埋める。
    fn slot(&mut self, value: &Value, shape: Option<&Value>) -> Result<Html, String> {
        if shape.and_then(|shape| shape.get("contentMediaType")) == Some(&Value::from(SVG)) {
            let svg = match value {
                Value::String(expr_text) => to_text(&self.eval(expr_text)?),
                other => other
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned(),
            };
            return svg_html(&svg);
        }
        match value {
            Value::Array(nodes) => {
                let mut out = Html::default();
                for node in nodes {
                    out.push(self.node(node)?);
                }
                Ok(out)
            }
            other => self.node(other),
        }
    }

    fn component(&mut self, node: &Map<String, Value>) -> Result<Html, String> {
        if let Some(show) = node.get("show").and_then(Value::as_str) {
            if !truthy(&self.eval(show)?) {
                return Ok(Html::default());
            }
        }
        if let Some(each) = node.get("each").and_then(Value::as_str) {
            let list = self.eval(each)?;
            let mut once = node.clone();
            once.remove("each");
            once.remove("show");
            let mut out = Html::default();
            for item in list.as_array().cloned().unwrap_or_default() {
                let saved = std::mem::replace(&mut self.current, item);
                let drawn = self.component(&once);
                self.current = saved;
                out.push(drawn?);
            }
            return Ok(out);
        }
        let name = node.get("component").and_then(Value::as_str).unwrap_or("");
        let Some(shape) = self.page_renderer.parts.shapes.get(name).cloned() else {
            return Err(format!("知らない部品: {name}"));
        };
        let mut values = BTreeMap::new();
        for (input, value) in node
            .iter()
            .filter(|(key, _)| !CONTROL.contains(&key.as_str()))
        {
            let input_shape = shape.pointer(&format!("/properties/{input}"));
            values.insert(input.clone(), self.slot(value, input_shape)?);
        }
        self.page_renderer.parts.fill(name, &values)
    }
}

/// SVG の文字列をページに埋める。script ・ イベントの属性（on…）・ javascript: を含むなら断る。
fn svg_html(svg: &str) -> Result<Html, String> {
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

/// ページテンプレートを、カタログの部品の名前と入力の形、ページのテンプレートのプレースホルダーで検査する。見つかった順に返す。
pub fn page_errors(parts: &Parts, page: &Value) -> Vec<String> {
    let mut out = Vec::new();
    let page_slots = parts
        .templates
        .get(PAGE)
        .map(|template| slots_of(template))
        .unwrap_or_default();
    for slot in page
        .get("head")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(slot, _)| slot)
    {
        if !page_slots.contains(slot) {
            out.push(format!(
                "/head/{slot}: カタログのテンプレート {PAGE} に {{{{{slot}}}}} が無い"
            ));
        }
    }
    walk(page, "", &parts.shapes, &mut out);
    out
}

fn walk(value: &Value, at: &str, shapes: &BTreeMap<String, Value>, out: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            if let Some(name) = object.get("component").and_then(Value::as_str) {
                match shapes.get(name) {
                    None => out.push(format!("{at}: 知らない部品 {name}")),
                    Some(shape) => input_errors(name, shape, object, at, out),
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

/// 部品の呼び出し1つの入力を、入力の形で検査する。基盤が読む欄（component ・ show ・ each）は入力に含めない。
fn input_errors(
    name: &str,
    shape: &Value,
    object: &Map<String, Value>,
    at: &str,
    out: &mut Vec<String>,
) {
    let input: Map<String, Value> = object
        .iter()
        .filter(|(key, _)| !CONTROL.contains(&key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    let known = shape.get("properties").and_then(Value::as_object);
    for key in input.keys() {
        if !known.is_some_and(|properties| properties.contains_key(key)) {
            out.push(format!("{at}: 部品 {name} に入力 {key} は無い"));
        }
    }
    match Schema::new(name, shape.clone(), vec![]).validate(&Value::Object(input)) {
        Ok(validation) => {
            for unfilled in &validation.unfilled {
                out.push(format!(
                    "{at}: 部品 {name} に {} が無い",
                    unfilled.property()
                ));
            }
            for error in &validation.errors {
                out.push(format!(
                    "{at}: 部品 {name} の {} {}",
                    error.property(),
                    error.reason()
                ));
            }
        }
        Err(error) => out.push(format!(
            "{at}: 部品 {name} の入力の形が壊れている（{}）",
            error.0
        )),
    }
}

#[cfg(test)]
mod tests {
    //! ページの生成のテスト。試験用の小さなカタログ（crate::tests::small_catalog）で描く。
    //! 基盤は節を並べ、show ・ each を適用し、式を描画の文脈で評価し、文字をエスケープし、テンプレートへ値を入れる。

    use super::{page_errors, PageRenderer, Parts};
    use crate::application::context::contexts;
    use crate::application::view::{Instance, ViewEngine};
    use crate::domain::check::Doc;
    use crate::domain::schema::Schema;
    use crate::domain::values::JsonValue;
    use crate::tests::{at, small_catalog, Answer, TableQuery};
    use serde_json::{json, Value};

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
            ("this.figure", |value, _| Ok(at(value, "/this/figure"))),
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

    fn instances(figure: &str) -> Vec<Value> {
        vec![
            json!({"id": "UC-1", "kind": "uc", "header": {"name": "注文 <する>", "level": "user"},
                   "tags": ["甲", "乙"], "rules": ["R-1", "R-2"], "steps": [], "figure": figure}),
            json!({"id": "R-1", "kind": "rule", "name": "在庫がある", "owner": "倉庫"}),
            json!({"id": "R-2", "kind": "rule", "name": "支払える", "owner": "経理"}),
        ]
    }

    fn schema_name(value: &Value) -> String {
        format!("{}.schema.json", value["kind"].as_str().unwrap())
    }

    /// 見本のインスタンス（UC-1）のページを、試験用の小さなカタログで生成する。
    fn render_with(page: Value, figure: &str) -> Result<String, String> {
        let uc = Schema::new("uc.schema.json", uc_schema(), vec![]);
        let rule = Schema::new("rule.schema.json", rule_schema(), vec![]);
        let values = instances(figure);
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
        let renderer = PageRenderer::new(engine, Parts::new(&small_catalog()).unwrap());
        renderer
            .render(&page, &ctx, "uc.schema.json")
            .map(|html| html.as_str().to_owned())
            .map_err(|error| error.0)
    }

    fn render_uc(page: Value) -> Result<String, String> {
        render_with(page, "")
    }

    #[test]
    fn title_comes_from_label_head_fills_page_slots_and_sections_follow_show() {
        let page = json!({"kind": "uc", "head": {"lead": "this.header.level"}, "sections": [
            {"heading": "出る節", "body": [{"text": "固定"}]},
            {"heading": "出ない節", "show": "this.steps", "body": [{"text": "x"}]}]});
        let html = render_uc(page).unwrap();
        assert!(
            html.ends_with("<main><h1>注文 &lt;する&gt;</h1><p>user</p><section><h2>出る節</h2>固定</section></main>"),
            "{html}"
        );
    }

    #[test]
    fn the_page_carries_the_tokens_style_and_script_of_the_catalog() {
        let page = json!({"kind": "uc", "sections": [{"heading": "h", "body": [{"text": "x"}]}]});
        let html = render_uc(page).unwrap();
        assert!(
            html.starts_with("<style>:root{--ink:#111d1a;--paper:#f7f8f7}.pill{color:var(--ink)}</style><script>/* 操作 */</script>"),
            "{html}"
        );
    }

    #[test]
    fn expression_nodes_are_escaped_and_arrays_joined() {
        let page = json!({"kind": "uc", "sections": [{"heading": "h", "body": ["this.header.name", "this.tags"]}]});
        let html = render_uc(page).unwrap();
        assert!(
            html.contains("<section><h2>h</h2>注文 &lt;する&gt;甲 ・ 乙</section>"),
            "{html}"
        );
    }

    #[test]
    fn component_inputs_fill_the_placeholders_of_the_same_name() {
        let page = json!({"kind": "uc", "sections": [{"heading": "h", "body": [
            {"component": "pill", "each": "this.tags", "text": "@", "tone": {"text": "accent"}},
            {"component": "pill", "show": "this.steps", "text": "'出ない'"},
            {"component": "pill", "text": {"text": "{{tone}}"}}]}]});
        let html = render_uc(page).unwrap();
        assert!(
            html.contains(
                r#"<b class="pill accent">甲</b><b class="pill accent">乙</b><b class="pill ">{{tone}}</b></section>"#
            ),
            "{html}"
        );
    }

    #[test]
    fn nested_parts_repeat_against_each_row() {
        let page = json!({"kind": "uc", "sections": [{"heading": "h", "body": [
            {"component": "list", "items": [
                {"component": "pill", "each": "links[?at=='/rules'].target", "text": "name(@)"}]}]}]});
        let html = render_uc(page).unwrap();
        assert!(
            html.contains(
                r#"<ul><b class="pill ">在庫がある</b><b class="pill ">支払える</b></ul>"#
            ),
            "{html}"
        );
    }

    #[test]
    fn an_input_declared_as_svg_is_embedded_after_the_check() {
        let page = json!({"kind": "uc", "sections": [{"heading": "h", "body": [
            {"component": "figure", "svg": "this.figure"}]}]});
        let html = render_with(page.clone(), "<svg viewBox=\"0 0 1 1\"></svg>").unwrap();
        assert!(
            html.contains(r#"<figure><svg viewBox="0 0 1 1"></svg></figure>"#),
            "{html}"
        );
        assert_eq!(
            render_with(page, "<svg onload=\"alert(1)\"></svg>").unwrap_err(),
            "SVG に script ・ イベントの属性 ・ javascript: が含まれている"
        );
    }

    #[test]
    fn unknown_component_is_an_error() {
        let page =
            json!({"kind": "uc", "sections": [{"heading": "h", "body": [{"component": "nope"}]}]});
        assert_eq!(render_uc(page).unwrap_err(), "知らない部品: nope");
    }

    #[test]
    fn page_errors_report_unknown_parts_inputs_and_head_slots() {
        let page = json!({"kind": "uc", "head": {"badge": "this.tags"}, "sections": [{"heading": "h", "body": [
            {"component": "nope"},
            {"component": "pill"},
            {"component": "pill", "text": "@", "color": "@"},
            {"component": "list", "items": [{"component": "pill", "text": "@", "tone": 1}]}]}]});
        let errors = page_errors(&Parts::new(&small_catalog()).unwrap(), &page);
        assert_eq!(
            errors,
            vec![
                "/head/badge: カタログのテンプレート page に {{badge}} が無い",
                "/sections/0/body/0: 知らない部品 nope",
                "/sections/0/body/1: 部品 pill に /text が無い",
                "/sections/0/body/2: 部品 pill に入力 color は無い",
                "/sections/0/body/3/items/0: 部品 pill の /tone 型が string でない（値：1）",
            ]
        );
    }
}
