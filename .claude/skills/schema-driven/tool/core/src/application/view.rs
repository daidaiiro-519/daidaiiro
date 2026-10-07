//! x-view の文の型（references/view.schema.json、ACDR 0129）。インスタンスの値を文にする。
//! {…} の中は普通の JMESPath で、出ていく側のポート Query に基盤の9つの関数を渡して評価する。

use crate::domain::schema::Schema;
use crate::ports::outbound::{Functions, Query, FUNCTIONS};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;
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

impl fmt::Display for ViewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "x-view を文にできない：{}", self.0)
    }
}

impl std::error::Error for ViewError {}

struct Inner {
    query: Arc<dyn Query>,
    schemas: BTreeMap<String, Schema>,
    instances: Vec<Instance>,
}

/// x-view の文の型を適用するエンジン。
#[derive(Clone)]
pub struct ViewEngine {
    inner: Arc<Inner>,
    /// 具体が足した関数（ACDR 0132）。
    extra: Option<Arc<dyn Functions>>,
}

/// 値を文字にする。null は空、配列は「 ・ 」でつなぐ。
pub fn to_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        Value::Array(elements) => elements
            .iter()
            .map(to_text)
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join(" ・ "),
        other => other.to_string(),
    }
}

/// JMESPath の真偽（null ・ false ・ 空の文字 ・ 空の配列 ・ 空のオブジェクトは偽）。
pub fn truthy(value: &Value) -> bool {
    match value {
        Value::Null | Value::Bool(false) => false,
        Value::String(text) => !text.is_empty(),
        Value::Array(elements) => !elements.is_empty(),
        Value::Object(object) => !object.is_empty(),
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
        let character = chars[i];
        if character == '{' && chars.get(i + 1) == Some(&'{') {
            text.push('{');
            i += 2;
        } else if character == '}' && chars.get(i + 1) == Some(&'}') {
            text.push('}');
            i += 2;
        } else if character == '{' {
            if !text.is_empty() {
                out.push((false, std::mem::take(&mut text)));
            }
            let mut depth = 1;
            let mut quote: Option<char> = None;
            let mut expr = String::new();
            i += 1;
            while i < chars.len() {
                let inner = chars[i];
                match quote {
                    Some(open_quote) if inner == open_quote => quote = None,
                    Some(_) => {}
                    None if inner == '\'' || inner == '`' || inner == '"' => quote = Some(inner),
                    None if inner == '{' => depth += 1,
                    None if inner == '}' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    None => {}
                }
                expr.push(inner);
                i += 1;
            }
            if depth != 0 {
                return Err(ViewError(format!("文の型の {{ が閉じていない: {template}")));
            }
            out.push((true, expr.trim().to_owned()));
            i += 1;
        } else {
            text.push(character);
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
        if let Some(name) = extra
            .names()
            .into_iter()
            .find(|name| FUNCTIONS.contains(&name.as_str()))
        {
            return Err(ViewError(format!("基盤の関数と同じ名前は足せない: {name}")));
        }
        Ok(Self {
            extra: Some(extra),
            ..self
        })
    }

    /// 基盤の関数（と具体が足した関数）を使って、式を値で評価する。ページテンプレートの式に使う。
    pub fn evaluate(&self, schema: &str, expr: &str, value: &Value) -> Result<Value, ViewError> {
        let host = Arc::new(Host {
            engine: self.clone(),
            schema: schema.to_owned(),
            x_view: Value::Null,
        });
        self.inner
            .query
            .evaluate(expr, value, host)
            .map_err(|error| ViewError(error.0))
    }

    /// インスタンスの題。スキーマの根の x-view の label をインスタンスに当てた名前。label が無いか、空なら None。
    pub fn title(&self, schema: &str, value: &Value) -> Option<String> {
        let instance = Instance {
            value: value.clone(),
            schema: schema.to_owned(),
        };
        self.label_at(&instance, &[])
    }

    /// スキーマの場所（JSON Pointer）にある x-view で、値を文にする。x-view が無ければ値をそのまま文字にする。
    pub fn render(&self, schema: &str, pointer: &str, value: &Value) -> Result<String, ViewError> {
        let loaded = self
            .inner
            .schemas
            .get(schema)
            .ok_or_else(|| ViewError(format!("スキーマが無い: {schema}")))?;
        let node = loaded
            .root()
            .pointer(pointer)
            .ok_or_else(|| ViewError(format!("{schema}#{pointer} が無い")))?;
        match node.get("x-view") {
            Some(x_view) => self.render_view(schema, x_view, value),
            None => Ok(to_text(value)),
        }
    }

    fn render_view(
        &self,
        schema: &str,
        x_view: &Value,
        value: &Value,
    ) -> Result<String, ViewError> {
        if x_view.get("hidden") == Some(&Value::Bool(true)) {
            return Ok(String::new());
        }
        let host = Arc::new(Host {
            engine: self.clone(),
            schema: schema.to_owned(),
            x_view: x_view.clone(),
        });
        let template = match self.pick(x_view.get("cases"), value, &host)? {
            Some(template) => template,
            None => match x_view.get("text").and_then(Value::as_str) {
                Some(template) => template.to_owned(),
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
                .map_err(|error| ViewError(error.0))?;
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
                let evaluated = self
                    .inner
                    .query
                    .evaluate(&part, value, host.clone())
                    .map_err(|error| ViewError(error.0))?;
                out.push_str(&to_text(&evaluated));
            } else {
                out.push_str(&part);
            }
        }
        Ok(out)
    }

    /// 参照の指す先を探し、その形の x-view の label で名前を返す。見つからなければ参照の値そのもの。
    fn name_of(&self, reference: &str) -> String {
        let instances = &self.inner.instances;
        let id_of = |value: &Value| value.get("id").and_then(Value::as_str).map(str::to_owned);
        if let Some(instance) = instances
            .iter()
            .find(|instance| id_of(&instance.value).as_deref() == Some(reference))
        {
            return self
                .label_at(instance, &[])
                .unwrap_or_else(|| reference.to_owned());
        }
        let (owner, item) = match reference.split_once('.') {
            Some((owner_id, item_id)) => (Some(owner_id), item_id),
            None => (None, reference),
        };
        for instance in instances {
            if owner.is_some_and(|owner_id| id_of(&instance.value).as_deref() != Some(owner_id)) {
                continue;
            }
            if let Some(path) = find_item(&instance.value, item, &mut Vec::new()) {
                return self
                    .label_at(instance, &path)
                    .unwrap_or_else(|| item.to_owned());
            }
        }
        reference.to_owned()
    }

    /// インスタンスの中の場所 path にある値の形の x-view の label を、その値で評価した名前。
    fn label_at(&self, instance: &Instance, path: &[String]) -> Option<String> {
        let schema = self.inner.schemas.get(&instance.schema)?;
        let mut node = schema.root();
        let mut doc = schema.root();
        let mut value = &instance.value;
        for segment in path {
            let (resolved_node, resolved_doc) = schema.resolve(node, doc);
            doc = resolved_doc;
            node = match segment.parse::<usize>() {
                Ok(position) => {
                    value = value.get(position)?;
                    resolved_node.get("items")?
                }
                Err(_) => {
                    value = value.get(segment)?;
                    resolved_node.get("properties")?.get(segment)?
                }
            };
        }
        let label = node
            .get("x-view")
            .and_then(|view| view.get("label"))
            .or_else(|| {
                schema
                    .resolve(node, doc)
                    .0
                    .get("x-view")
                    .and_then(|view| view.get("label"))
            })?
            .as_str()?
            .to_owned();
        let found = self.inner.query.search(&label, value).ok()?;
        let text = to_text(&found);
        (!text.is_empty()).then_some(text)
    }
}

/// インスタンスの中で、id が item のオブジェクトの場所を探す。
fn find_item(value: &Value, item: &str, path: &mut Vec<String>) -> Option<Vec<String>> {
    match value {
        Value::Object(object) => {
            if !path.is_empty() && object.get("id").and_then(Value::as_str) == Some(item) {
                return Some(path.clone());
            }
            for (key, child) in object {
                path.push(key.clone());
                if let Some(found) = find_item(child, item, path) {
                    return Some(found);
                }
                path.pop();
            }
            None
        }
        Value::Array(elements) => {
            for (position, child) in elements.iter().enumerate() {
                path.push(position.to_string());
                if let Some(found) = find_item(child, item, path) {
                    return Some(found);
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
    x_view: Value,
}

/// 配列なら要素ごとに適用する。
fn each(value: &Value, apply: &dyn Fn(&Value) -> Result<Value, String>) -> Result<Value, String> {
    match value {
        Value::Array(elements) => elements
            .iter()
            .map(apply)
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        other => apply(other),
    }
}

fn arg(args: &[Value], position: usize) -> &Value {
    args.get(position).unwrap_or(&Value::Null)
}

impl Functions for Host {
    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = FUNCTIONS.iter().map(|name| name.to_string()).collect();
        if let Some(extra) = &self.engine.extra {
            names.extend(extra.names());
        }
        names
    }

    fn call(&self, name: &str, args: &[Value]) -> Result<Value, String> {
        let text_of = |value: &Value| to_text(value);
        match name {
            "name" => each(arg(args, 0), &|value| {
                Ok(Value::String(self.engine.name_of(&text_of(value))))
            }),
            "label" => each(arg(args, 0), &|value| {
                let id = value.get("id").map(to_text).unwrap_or_default();
                Ok(Value::String(if id.is_empty() {
                    String::new()
                } else {
                    self.engine.name_of(&id)
                }))
            }),
            "map" => {
                let table = text_of(arg(args, 0));
                let table_entries = self
                    .x_view
                    .get("maps")
                    .and_then(|maps| maps.get(&table))
                    .cloned()
                    .unwrap_or_default();
                each(arg(args, 1), &|value| {
                    Ok(table_entries
                        .get(text_of(value))
                        .cloned()
                        .unwrap_or(Value::String(String::new())))
                })
            }
            "view" => {
                let location = text_of(arg(args, 0));
                let (file, pointer) = match location.split_once('#') {
                    Some((file_name, fragment)) if !file_name.is_empty() => {
                        (file_name.to_owned(), fragment.to_owned())
                    }
                    Some((_, fragment)) => (self.schema.clone(), fragment.to_owned()),
                    None => (self.schema.clone(), location.clone()),
                };
                each(arg(args, 1), &|value| {
                    self.engine
                        .render(&file, &pointer, value)
                        .map(Value::String)
                        .map_err(|error| error.0)
                })
            }
            "part" => {
                let part_name = text_of(arg(args, 0));
                let part = self
                    .x_view
                    .get("parts")
                    .and_then(|parts| parts.get(&part_name))
                    .cloned()
                    .ok_or(format!("parts に {part_name} が無い"))?;
                let host = Arc::new(Host {
                    engine: self.engine.clone(),
                    schema: self.schema.clone(),
                    x_view: self.x_view.clone(),
                });
                let value = arg(args, 1);
                let template = match &part {
                    Value::String(text) => Some(text.clone()),
                    array @ Value::Array(_) => self
                        .engine
                        .pick(Some(array), value, &host)
                        .map_err(|error| error.0)?,
                    _ => None,
                };
                match template {
                    Some(template) => self
                        .engine
                        .fill(&template, value, &host)
                        .map(Value::String)
                        .map_err(|error| error.0),
                    None => Ok(Value::String(String::new())),
                }
            }
            "quote" => each(arg(args, 0), &|value| {
                Ok(Value::String(format!("「{}」", text_of(value))))
            }),
            other => match &self.engine.extra {
                Some(extra) => extra.call(other, args),
                None => Err(format!("知らない関数: {other}")),
            },
        }
    }
}
