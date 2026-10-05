//! x-view の文の型（references/view.schema.json、ACDR 0129）。インスタンスの値を文にする。
//! {…} の中は普通の JMESPath で、出ていく側のポート Query に基盤の9つの関数を渡して評価する。

use crate::domain::schema::Schema;
use crate::ports::outbound::{Functions, Query, FUNCTIONS};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

/// 文にする対象のインスタンス1件と、そのスキーマの名前。
#[derive(Debug, Clone)]
pub struct Instance {
    pub value: Value,
    pub schema: String,
}

/// 文にできなかった理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewError(pub String);

struct Inner {
    query: Arc<dyn Query>,
    schemas: BTreeMap<String, Schema>,
    instances: Vec<Instance>,
}

/// x-view の文の型を当てる道具。
#[derive(Clone)]
pub struct ViewEngine {
    inner: Arc<Inner>,
    /// 具体が足した関数（ACDR 0132）。
    extra: Option<Arc<dyn Functions>>,
}

/// 値を文字にする。null は空、配列は「 ・ 」でつなぐ。
pub fn to_text(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Array(a) => a
            .iter()
            .map(to_text)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ・ "),
        other => other.to_string(),
    }
}

/// JMESPath の真偽（null ・ false ・ 空の文字 ・ 空の配列 ・ 空のオブジェクトは偽）。
pub fn truthy(v: &Value) -> bool {
    match v {
        Value::Null | Value::Bool(false) => false,
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
        _ => true,
    }
}

/// 文の型を、決まった文字と {…} の式に分ける。{{ と }} は文字の { と }。
fn split(template: &str) -> Result<Vec<(bool, String)>, ViewError> {
    let chars: Vec<char> = template.chars().collect();
    let mut out = Vec::new();
    let mut text = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '{' && chars.get(i + 1) == Some(&'{') {
            text.push('{');
            i += 2;
        } else if c == '}' && chars.get(i + 1) == Some(&'}') {
            text.push('}');
            i += 2;
        } else if c == '{' {
            if !text.is_empty() {
                out.push((false, std::mem::take(&mut text)));
            }
            let mut depth = 1;
            let mut quote: Option<char> = None;
            let mut expr = String::new();
            i += 1;
            while i < chars.len() {
                let d = chars[i];
                match quote {
                    Some(q) if d == q => quote = None,
                    Some(_) => {}
                    None if d == '\'' || d == '`' || d == '"' => quote = Some(d),
                    None if d == '{' => depth += 1,
                    None if d == '}' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    None => {}
                }
                expr.push(d);
                i += 1;
            }
            if depth != 0 {
                return Err(ViewError(format!("文の型の {{ が閉じていない: {template}")));
            }
            out.push((true, expr.trim().to_owned()));
            i += 1;
        } else {
            text.push(c);
            i += 1;
        }
    }
    if !text.is_empty() {
        out.push((false, text));
    }
    Ok(out)
}

impl ViewEngine {
    pub fn new(
        query: Arc<dyn Query>,
        schemas: Vec<(String, Schema)>,
        instances: Vec<Instance>,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                query,
                schemas: schemas.into_iter().collect(),
                instances,
            }),
            extra: None,
        }
    }

    /// 具体の関数を足す（ACDR 0132）。基盤の関数と同じ名前は足せない。
    pub fn with_functions(self, extra: Arc<dyn Functions>) -> Result<Self, ViewError> {
        if let Some(n) = extra
            .names()
            .into_iter()
            .find(|n| FUNCTIONS.contains(&n.as_str()))
        {
            return Err(ViewError(format!("基盤の関数と同じ名前は足せない: {n}")));
        }
        Ok(Self {
            extra: Some(extra),
            ..self
        })
    }

    /// スキーマの場所（JSON Pointer）にある x-view で、値を文にする。x-view が無ければ値をそのまま文字にする。
    pub fn render(&self, schema: &str, pointer: &str, value: &Value) -> Result<String, ViewError> {
        let s = self
            .inner
            .schemas
            .get(schema)
            .ok_or_else(|| ViewError(format!("スキーマが無い: {schema}")))?;
        let node = s
            .root()
            .pointer(pointer)
            .ok_or_else(|| ViewError(format!("{schema}#{pointer} が無い")))?;
        match node.get("x-view") {
            Some(xv) => self.render_view(schema, xv, value),
            None => Ok(to_text(value)),
        }
    }

    fn render_view(&self, schema: &str, xv: &Value, value: &Value) -> Result<String, ViewError> {
        if xv.get("hidden") == Some(&Value::Bool(true)) {
            return Ok(String::new());
        }
        let host = Arc::new(Host {
            engine: self.clone(),
            schema: schema.to_owned(),
            xv: xv.clone(),
        });
        let template = match self.pick(xv.get("cases"), value, &host)? {
            Some(t) => t,
            None => match xv.get("text").and_then(Value::as_str) {
                Some(t) => t.to_owned(),
                None => return Ok(to_text(value)),
            },
        };
        self.fill(&template, value, &host)
    }

    /// cases（または parts の配列）を上から当て、最初に当たった text を返す。
    fn pick(
        &self,
        cases: Option<&Value>,
        value: &Value,
        host: &Arc<Host>,
    ) -> Result<Option<String>, ViewError> {
        for case in cases.and_then(Value::as_array).into_iter().flatten() {
            let when = case.get("when").and_then(Value::as_str).unwrap_or("false");
            let hit = self
                .inner
                .query
                .evaluate(when, value, host.clone())
                .map_err(|e| ViewError(e.0))?;
            if truthy(&hit) {
                return Ok(case.get("text").and_then(Value::as_str).map(str::to_owned));
            }
        }
        Ok(None)
    }

    fn fill(&self, template: &str, value: &Value, host: &Arc<Host>) -> Result<String, ViewError> {
        let mut out = String::new();
        for (is_expr, part) in split(template)? {
            if is_expr {
                let v = self
                    .inner
                    .query
                    .evaluate(&part, value, host.clone())
                    .map_err(|e| ViewError(e.0))?;
                out.push_str(&to_text(&v));
            } else {
                out.push_str(&part);
            }
        }
        Ok(out)
    }

    /// 参照の指す先を探し、その形の x-view の label で名前を返す。見つからなければ参照の値そのもの。
    fn name_of(&self, reference: &str) -> String {
        let inst = &self.inner.instances;
        let id_of = |v: &Value| v.get("id").and_then(Value::as_str).map(str::to_owned);
        if let Some(i) = inst
            .iter()
            .find(|i| id_of(&i.value).as_deref() == Some(reference))
        {
            return self
                .label_at(i, &[])
                .unwrap_or_else(|| reference.to_owned());
        }
        let (owner, item) = match reference.split_once('.') {
            Some((o, it)) => (Some(o), it),
            None => (None, reference),
        };
        for i in inst {
            if owner.is_some_and(|o| id_of(&i.value).as_deref() != Some(o)) {
                continue;
            }
            if let Some(path) = find_item(&i.value, item, &mut Vec::new()) {
                return self.label_at(i, &path).unwrap_or_else(|| item.to_owned());
            }
        }
        reference.to_owned()
    }

    /// インスタンスの中の場所 path にある値の形の x-view の label を、その値に当てた名前。
    fn label_at(&self, inst: &Instance, path: &[String]) -> Option<String> {
        let schema = self.inner.schemas.get(&inst.schema)?;
        let mut node = schema.root();
        let mut doc = schema.root();
        let mut value = &inst.value;
        for seg in path {
            let (n, d) = schema.resolve(node, doc);
            doc = d;
            node = match seg.parse::<usize>() {
                Ok(i) => {
                    value = value.get(i)?;
                    n.get("items")?
                }
                Err(_) => {
                    value = value.get(seg)?;
                    n.get("properties")?.get(seg)?
                }
            };
        }
        let label = node
            .get("x-view")
            .and_then(|x| x.get("label"))
            .or_else(|| {
                schema
                    .resolve(node, doc)
                    .0
                    .get("x-view")
                    .and_then(|x| x.get("label"))
            })?
            .as_str()?
            .to_owned();
        let v = self.inner.query.search(&label, value).ok()?;
        let text = to_text(&v);
        (!text.is_empty()).then_some(text)
    }
}

/// インスタンスの中で、id が item のオブジェクトの場所を探す。
fn find_item(v: &Value, item: &str, path: &mut Vec<String>) -> Option<Vec<String>> {
    match v {
        Value::Object(m) => {
            if !path.is_empty() && m.get("id").and_then(Value::as_str) == Some(item) {
                return Some(path.clone());
            }
            for (k, c) in m {
                path.push(k.clone());
                if let Some(p) = find_item(c, item, path) {
                    return Some(p);
                }
                path.pop();
            }
            None
        }
        Value::Array(a) => {
            for (i, c) in a.iter().enumerate() {
                path.push(i.to_string());
                if let Some(p) = find_item(c, item, path) {
                    return Some(p);
                }
                path.pop();
            }
            None
        }
        _ => None,
    }
}

/// 基盤の6つの関数と、具体が足した関数。1つの x-view（maps と parts）を文脈に持つ。
struct Host {
    engine: ViewEngine,
    schema: String,
    xv: Value,
}

/// 配列なら要素ごとに当てる。
fn each(v: &Value, f: &dyn Fn(&Value) -> Result<Value, String>) -> Result<Value, String> {
    match v {
        Value::Array(a) => a
            .iter()
            .map(f)
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        other => f(other),
    }
}

fn arg(args: &[Value], i: usize) -> &Value {
    args.get(i).unwrap_or(&Value::Null)
}

impl Functions for Host {
    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = FUNCTIONS.iter().map(|n| n.to_string()).collect();
        if let Some(extra) = &self.engine.extra {
            names.extend(extra.names());
        }
        names
    }

    fn call(&self, name: &str, args: &[Value]) -> Result<Value, String> {
        let s = |v: &Value| to_text(v);
        match name {
            "name" => each(arg(args, 0), &|v| {
                Ok(Value::String(self.engine.name_of(&s(v))))
            }),
            "label" => each(arg(args, 0), &|v| {
                let id = v.get("id").map(to_text).unwrap_or_default();
                Ok(Value::String(if id.is_empty() {
                    String::new()
                } else {
                    self.engine.name_of(&id)
                }))
            }),
            "map" => {
                let table = s(arg(args, 0));
                let t = self
                    .xv
                    .get("maps")
                    .and_then(|m| m.get(&table))
                    .cloned()
                    .unwrap_or_default();
                each(arg(args, 1), &|v| {
                    Ok(t.get(s(v)).cloned().unwrap_or(Value::String(String::new())))
                })
            }
            "view" => {
                let loc = s(arg(args, 0));
                let (file, pointer) = match loc.split_once('#') {
                    Some((f, p)) if !f.is_empty() => (f.to_owned(), p.to_owned()),
                    Some((_, p)) => (self.schema.clone(), p.to_owned()),
                    None => (self.schema.clone(), loc.clone()),
                };
                each(arg(args, 1), &|v| {
                    self.engine
                        .render(&file, &pointer, v)
                        .map(Value::String)
                        .map_err(|e| e.0)
                })
            }
            "part" => {
                let pname = s(arg(args, 0));
                let part = self
                    .xv
                    .get("parts")
                    .and_then(|p| p.get(&pname))
                    .cloned()
                    .ok_or(format!("parts に {pname} が無い"))?;
                let host = Arc::new(Host {
                    engine: self.engine.clone(),
                    schema: self.schema.clone(),
                    xv: self.xv.clone(),
                });
                let value = arg(args, 1);
                let template = match &part {
                    Value::String(t) => Some(t.clone()),
                    arr @ Value::Array(_) => {
                        self.engine.pick(Some(arr), value, &host).map_err(|e| e.0)?
                    }
                    _ => None,
                };
                match template {
                    Some(t) => self
                        .engine
                        .fill(&t, value, &host)
                        .map(Value::String)
                        .map_err(|e| e.0),
                    None => Ok(Value::String(String::new())),
                }
            }
            "quote" => each(arg(args, 0), &|v| {
                Ok(Value::String(format!("「{}」", s(v))))
            }),
            other => match &self.engine.extra {
                Some(extra) => extra.call(other, args),
                None => Err(format!("知らない関数: {other}")),
            },
        }
    }
}
