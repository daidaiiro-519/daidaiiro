//! ページの生成（3c。page.schema.json、ボード schema-driven-build の論点5、ACDR 0132 ・ 0133）。
//! 節を並べ、show ・ each を適用し、式を描画の文脈で評価し、コンポーネントの描画は具体の Design を呼び出す。
//! 文字は必ずエスケープし、部品のプレースホルダーと渡した名前が合わなければエラーにする。

use crate::application::view::{to_text, truthy, ViewEngine, ViewError};
use crate::domain::schema::Schema;
use crate::domain::values::Html;
use crate::ports::outbound::{Design, Frame, Renderer};
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

/// ページテンプレートと具体のデザインから、ページを生成する。
pub struct PageRenderer {
    engine: ViewEngine,
    design: Arc<dyn Design>,
    parts: BTreeMap<String, String>,
    components: BTreeSet<String>,
}

impl PageRenderer {
    pub fn new(engine: ViewEngine, design: Arc<dyn Design>) -> Result<Self, ViewError> {
        let parts = parse_parts(design.parts()).map_err(ViewError)?;
        let components = design
            .components()
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        Ok(Self {
            engine,
            design,
            parts,
            components,
        })
    }

    /// 描画の文脈（contexts の1件）に、ページテンプレートを適用して1ページを生成する。schema はこのインスタンスのスキーマの名前。
    pub fn render(&self, page: &Value, context: &Value, schema: &str) -> Result<Html, ViewError> {
        let mut run = Run {
            page_renderer: self,
            schema,
            tones: page.get("tones").cloned().unwrap_or(Value::Null),
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
struct Run<'renderer> {
    page_renderer: &'renderer PageRenderer,
    schema: &'renderer str,
    tones: Value,
    current: Value,
}

impl Run<'_> {
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

impl Renderer for Run<'_> {
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

    fn tone(&self, table: &str, value: &Value) -> Option<String> {
        self.tones
            .get(table)
            .and_then(|tones| tones.get(to_text(value)))
            .and_then(Value::as_str)
            .map(str::to_owned)
    }
}

/// ページテンプレートのコンポーネントを、具体が登録した名前と入力の形で検査する。見つかった順に返す。
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

/// 基盤の文書（references/document.schema.json）の決まったページテンプレート。何も指定が無いときに使う（UC-6）。
/// 題は x-view の label（title）、題の上に badges、題の下に lead。目次 ・ 節とブロック ・ 出どころは、コンポーネント document が描画する。
pub fn document_page_template() -> Value {
    serde_json::json!({
        "kind": "document",
        "head": {
            "badges": [{"component": "badges", "show": "this.badges", "value": "this.badges"}],
            "lead": {"component": "inline", "show": "this.lead", "value": "this.lead"}
        },
        "sections": [{"heading": "文書", "body": [{"component": "document", "value": "this"}]}]
    })
}
