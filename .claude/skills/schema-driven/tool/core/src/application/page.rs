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
        let q = after.find('"').ok_or("部品の id が閉じていない")?;
        let id = after[..q].to_owned();
        let body_start =
            after[q..].find('>').ok_or("部品の開きタグが閉じていない")? + q + 1;
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
    while let Some(i) = rest.find("{{") {
        let after = &rest[i + 2..];
        match after.find("}}") {
            Some(j) => {
                out.insert(after[..j].to_owned());
                rest = &after[j + 2..];
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
        let components = design.components().into_iter().map(|(n, _)| n).collect();
        Ok(Self {
            engine,
            design,
            parts,
            components,
        })
    }

    /// 描画の文脈（contexts の1件）に、ページテンプレートを適用して1ページを生成する。schema はこのインスタンスのスキーマの名前。
    pub fn render(&self, page: &Value, context: &Value, schema: &str) -> Result<Html, ViewError> {
        let mut r = Run {
            pr: self,
            schema,
            tones: page.get("tones").cloned().unwrap_or(Value::Null),
            current: context.clone(),
        };
        let err = ViewError;
        let head = page.get("head").cloned().unwrap_or(Value::Null);
        let mut badges = Html::default();
        for b in head
            .get("badges")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            badges.push(r.node(b).map_err(err)?);
        }
        let lead = match head.get("lead") {
            Some(n) => r.node(n).map_err(err)?,
            None => Html::default(),
        };
        let mut sections = Html::default();
        for s in page
            .get("sections")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(show) = s.get("show").and_then(Value::as_str) {
                if !truthy(&r.eval(show).map_err(err)?) {
                    continue;
                }
            }
            let mut body = Html::default();
            for n in s
                .get("body")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                body.push(r.node(n).map_err(err)?);
            }
            let heading = s.get("heading").and_then(Value::as_str).unwrap_or("");
            sections.push(self.design.section(heading, body, &r).map_err(err)?);
        }
        let title = context
            .pointer("/this/id")
            .and_then(Value::as_str)
            .map(|id| self.engine.name(id))
            .unwrap_or_default();
        let frame = Frame {
            title,
            badges,
            lead,
            sections,
        };
        self.design.page(&frame, &r).map_err(err)
    }
}

/// 1ページを生成するあいだの状態。今の位置（current）は each と node_at で替わる。
struct Run<'a> {
    pr: &'a PageRenderer,
    schema: &'a str,
    tones: Value,
    current: Value,
}

impl Run<'_> {
    fn eval(&self, expr: &str) -> Result<Value, String> {
        self.pr
            .engine
            .evaluate(self.schema, expr, &self.current)
            .map_err(|e| format!("式 {expr}: {}", e.0))
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
            if let Some(m) = once.as_object_mut() {
                m.remove("each");
                m.remove("show");
            }
            let mut out = Html::default();
            for item in list.as_array().cloned().unwrap_or_default() {
                out.push(self.node_at(&once, &item)?);
            }
            return Ok(out);
        }
        let name = node.get("component").and_then(Value::as_str).unwrap_or("");
        if !self.pr.components.contains(name) {
            return Err(format!("知らないコンポーネント: {name}"));
        }
        let design = self.pr.design.clone();
        design.render(name, node, self)
    }
}

impl Renderer for Run<'_> {
    fn value(&mut self, expr: &Value) -> Result<Value, String> {
        match expr {
            Value::String(e) => self.eval(e),
            Value::Object(m) if !m.contains_key("component") => {
                Ok(m.get("text").cloned().unwrap_or(Value::Null))
            }
            Value::Null => Ok(Value::Null),
            _ => Err("コンポーネントは値にできない".into()),
        }
    }

    fn node(&mut self, node: &Value) -> Result<Html, String> {
        match node {
            Value::String(e) => Ok(Html::escape(&to_text(&self.eval(e)?))),
            Value::Object(m) if m.contains_key("component") => self.component(node),
            Value::Object(m) => Ok(Html::escape(
                m.get("text").and_then(Value::as_str).unwrap_or(""),
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
            .pr
            .parts
            .get(id)
            .ok_or_else(|| format!("部品 {id} が無い"))?;
        let want = slots_of(template);
        let given: BTreeSet<String> = slots.iter().map(|(n, _)| (*n).to_owned()).collect();
        if want != given {
            let missing: Vec<_> = want.difference(&given).cloned().collect();
            let extra: Vec<_> = given.difference(&want).cloned().collect();
            return Err(format!(
                "部品 {id} のプレースホルダーが合わない（足りない {missing:?}、余る {extra:?}）"
            ));
        }
        let mut out = template.clone();
        for (n, h) in slots {
            out = out.replace(&format!("{{{{{n}}}}}"), h.as_str());
        }
        Ok(Html::trusted(out))
    }

    fn text(&self, text: &str) -> Html {
        Html::escape(text)
    }

    fn tone(&self, table: &str, value: &Value) -> Option<String> {
        self.tones
            .get(table)
            .and_then(|t| t.get(to_text(value)))
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

fn walk(v: &Value, at: &str, shapes: &BTreeMap<String, Value>, out: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            if let Some(name) = m.get("component").and_then(Value::as_str) {
                match shapes.get(name) {
                    None => out.push(format!("{at}: 知らないコンポーネント {name}")),
                    Some(shape) => match Schema::new(name, shape.clone(), vec![]).validate(v) {
                        Ok(res) => {
                            for u in &res.unfilled {
                                out.push(format!("{at}: {name} に {} が無い", u.property()));
                            }
                            for e in &res.errors {
                                out.push(format!(
                                    "{at}: {name} の {} {}",
                                    e.property(),
                                    e.reason()
                                ));
                            }
                        }
                        Err(e) => {
                            out.push(format!("{at}: {name} の入力の形が壊れている（{}）", e.0))
                        }
                    },
                }
            }
            for (k, c) in m {
                walk(c, &format!("{at}/{k}"), shapes, out);
            }
        }
        Value::Array(a) => {
            for (i, c) in a.iter().enumerate() {
                walk(c, &format!("{at}/{i}"), shapes, out);
            }
        }
        _ => {}
    }
}
