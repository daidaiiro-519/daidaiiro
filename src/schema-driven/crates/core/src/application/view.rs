//! x-view の文の型（schemas/view.schema.json）。インスタンスの値を文にする。
//! {…} の中は普通の JMESPath で、出ていく側のポート Query に基盤の9つの関数を渡して評価する。

use crate::domain::schema::Schema;
use crate::ports::Functions;
use crate::ports::Query;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

/// JMESPath に足す基盤の関数（view.schema.json の functions）。
pub const FUNCTIONS: [&str; 6] = ["name", "label", "map", "view", "part", "quote"];

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

struct Inner<Q> {
    query: Q,
    schemas: BTreeMap<String, Schema>,
    instances: Vec<Instance>,
}

/// x-view の文の型を適用するエンジン。
pub struct ViewEngine<Q> {
    inner: Arc<Inner<Q>>,
}

impl<Q> Clone for ViewEngine<Q> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
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

impl<Q: Query> ViewEngine<Q> {
    pub fn new(query: Q, schemas: Vec<(String, Schema)>, instances: Vec<Instance>) -> Self {
        Self {
            inner: Arc::new(Inner {
                query,
                schemas: schemas.into_iter().collect(),
                instances,
            }),
        }
    }

    /// 基盤の関数を使って、式を値で評価する。ページテンプレートの式に使う。
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
        host: &Arc<Host<Q>>,
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

    fn fill(
        &self,
        template: &str,
        value: &Value,
        host: &Arc<Host<Q>>,
    ) -> Result<String, ViewError> {
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
struct Host<Q> {
    engine: ViewEngine<Q>,
    schema: String,
    x_view: Value,
}

/// 配列なら要素ごとに適用する。
fn each(value: &Value, apply: impl Fn(&Value) -> Result<Value, String>) -> Result<Value, String> {
    match value {
        Value::Array(elements) => elements
            .iter()
            .map(&apply)
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        other => apply(other),
    }
}

fn arg(args: &[Value], position: usize) -> &Value {
    args.get(position).unwrap_or(&Value::Null)
}

impl<Q: Query> Functions for Host<Q> {
    fn names(&self) -> Vec<String> {
        FUNCTIONS.iter().map(|name| name.to_string()).collect()
    }

    fn call(&self, name: &str, args: &[Value]) -> Result<Value, String> {
        let text_of = |value: &Value| to_text(value);
        match name {
            "name" => each(arg(args, 0), |value| {
                Ok(Value::String(self.engine.name_of(&text_of(value))))
            }),
            "label" => each(arg(args, 0), |value| {
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
                each(arg(args, 1), |value| {
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
                each(arg(args, 1), |value| {
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
            "quote" => each(arg(args, 0), |value| {
                Ok(Value::String(format!("「{}」", text_of(value))))
            }),
            other => Err(format!("知らない関数: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    //! x-view の文の型（schemas/view.schema.json）のテスト。
    //! {…} の中の式は、式の表で答える Query の代役が答える。関数の呼び出しを含む式は、代役が基盤の関数（Functions）を呼ぶ。
    //! 本物の JMESPath と組み合わせた確かめは、adapters の jmespath.rs の単体テストが持つ。

    use super::{Instance, ViewEngine, ViewError};
    use crate::domain::schema::Schema;
    use crate::tests::{at, join, Answer, TableQuery};
    use serde_json::{json, Value};

    /// 見本の式と答え。関数の呼び出しは、渡された基盤の関数を呼ぶ。
    fn answers() -> Vec<(&'static str, Answer)> {
        vec![
            ("min", |value, _| Ok(at(value, "/min"))),
            ("max", |value, _| Ok(at(value, "/max"))),
            ("tags", |value, _| Ok(at(value, "/tags"))),
            ("none", |value, _| Ok(at(value, "/none"))),
            ("min == `1` && max == `1`", |value, _| {
                Ok(json!(value["min"] == json!(1) && value["max"] == json!(1)))
            }),
            ("max == null", |value, _| Ok(json!(value["max"].is_null()))),
            ("target", |value, _| Ok(at(value, "/target"))),
            ("value", |value, _| Ok(at(value, "/value"))),
            ("op == 'le'", |value, _| Ok(json!(value["op"] == "le"))),
            ("op == 'not_empty'", |value, _| {
                Ok(json!(value["op"] == "not_empty"))
            }),
            ("map('agg', agg)", |value, functions| {
                functions.call("map", &[json!("agg"), at(value, "/agg")])
            }),
            ("part('subject', @)", |value, functions| {
                functions.call("part", &[json!("subject"), value.clone()])
            }),
            ("part('predicate', @)", |value, functions| {
                functions.call("part", &[json!("predicate"), value.clone()])
            }),
            ("names | quote(@) | join('', @)", |value, functions| {
                Ok(join("", &functions.call("quote", &[at(value, "/names")])?))
            }),
            ("clause(s)", |value, functions| {
                functions.call("clause", &[at(value, "/s")])
            }),
            ("header.name", |value, _| Ok(at(value, "/header/name"))),
            ("word", |value, _| Ok(at(value, "/word"))),
            ("refs | name(@) | join('、', @)", |value, functions| {
                Ok(join("、", &functions.call("name", &[at(value, "/refs")])?))
            }),
            ("a", |value, _| Ok(at(value, "/a"))),
            ("b", |value, _| Ok(at(value, "/b"))),
            ("view('/$defs/inner', cond)", |value, functions| {
                functions.call("view", &[json!("/$defs/inner"), at(value, "/cond")])
            }),
        ]
    }

    fn engine(schema: Value, instances: Vec<Value>) -> ViewEngine<TableQuery> {
        let test_schema = Schema::new("t.schema.json", schema, vec![]);
        let list = instances
            .into_iter()
            .map(|value| Instance {
                value,
                schema: "t.schema.json".into(),
            })
            .collect();
        ViewEngine::new(
            TableQuery::new(answers()),
            vec![("t.schema.json".into(), test_schema)],
            list,
        )
    }

    fn render(view_engine: &ViewEngine<TableQuery>, pointer: &str, value: Value) -> String {
        view_engine
            .render("t.schema.json", pointer, &value)
            .unwrap()
    }

    #[test]
    fn text_fills_expressions_and_formats_null_and_arrays() {
        let view_engine = engine(
            json!({"$defs": {"m": {"x-view": {"text": "{min}件以上 {tags} [{none}]"}}}}),
            vec![],
        );
        assert_eq!(
            render(
                &view_engine,
                "/$defs/m",
                json!({"min": 0, "tags": ["a", "b"]})
            ),
            "0件以上 a ・ b []"
        );
    }

    #[test]
    fn cases_pick_first_matching_when() {
        let view_engine = engine(
            json!({"$defs": {"m": {"x-view": {"cases": [
                {"when": "min == `1` && max == `1`", "text": "1つ"},
                {"when": "max == null", "text": "{min}件以上"}], "text": "{min}〜{max}件"}}}}),
            vec![],
        );
        assert_eq!(
            render(&view_engine, "/$defs/m", json!({"min": 1, "max": 1})),
            "1つ"
        );
        assert_eq!(
            render(&view_engine, "/$defs/m", json!({"min": 0, "max": null})),
            "0件以上"
        );
        assert_eq!(
            render(&view_engine, "/$defs/m", json!({"min": 2, "max": 5})),
            "2〜5件"
        );
    }

    #[test]
    fn parts_and_maps() {
        let view_engine = engine(
            json!({"$defs": {"c": {"x-view": {
                "maps": {"agg": {"count": "の件数"}},
                "parts": {"subject": "{target}{map('agg', agg)}",
                          "predicate": [{"when": "op == 'le'", "text": "は{value}以下"}, {"when": "op == 'not_empty'", "text": "は空でない"}]},
                "text": "{part('subject', @)}{part('predicate', @)}"}}}}),
            vec![],
        );
        assert_eq!(
            render(
                &view_engine,
                "/$defs/c",
                json!({"target": "エラー", "agg": "count", "op": "le", "value": 0})
            ),
            "エラーの件数は0以下"
        );
        assert_eq!(
            render(
                &view_engine,
                "/$defs/c",
                json!({"target": "値", "op": "not_empty"})
            ),
            "値は空でない"
        );
    }

    #[test]
    fn quote_wraps_each_element() {
        let view_engine = engine(
            json!({"$defs": {"q": {"x-view": {"text": "{names | quote(@) | join('', @)}が無い"}}}}),
            vec![],
        );
        assert_eq!(
            render(&view_engine, "/$defs/q", json!({"names": ["甲", "乙"]})),
            "「甲」「乙」が無い"
        );
    }

    #[test]
    fn a_function_outside_the_base_is_an_error() {
        let schema = json!({"$defs": {"c": {"x-view": {"text": "{clause(s)}"}}}});
        let view_engine = engine(schema, vec![]);
        assert_eq!(
            view_engine
                .render("t.schema.json", "/$defs/c", &json!({"s": "値は空"}))
                .unwrap_err(),
            ViewError("知らない関数: clause".into())
        );
    }

    #[test]
    fn name_uses_label_of_referenced_instance_or_item() {
        let schema = json!({
            "x-view": {"label": "header.name"},
            "properties": {"terms": {"items": {"x-view": {"label": "word"}}}},
            "$defs": {"r": {"x-view": {"text": "{refs | name(@) | join('、', @)}"}}}
        });
        let instances = vec![
            json!({"id": "UC-1", "kind": "uc", "header": {"name": "注文する"}}),
            json!({"id": "GLO-1", "kind": "glossary", "terms": [{"id": "TERM-1", "word": "注文"}]}),
        ];
        let view_engine = engine(schema, instances);
        assert_eq!(
            render(
                &view_engine,
                "/$defs/r",
                json!({"refs": ["UC-1", "GLO-1.TERM-1", "TERM-1", "UC-9"]})
            ),
            "注文する、注文、注文、UC-9"
        );
    }

    #[test]
    fn view_renders_value_with_another_x_view() {
        let view_engine = engine(
            json!({"$defs": {
                "inner": {"x-view": {"text": "{a}は{b}"}},
                "outer": {"x-view": {"text": "{view('/$defs/inner', cond)}なら、終わる"}}}}),
            vec![],
        );
        assert_eq!(
            render(
                &view_engine,
                "/$defs/outer",
                json!({"cond": {"a": "値", "b": "空"}})
            ),
            "値は空なら、終わる"
        );
    }

    #[test]
    fn hidden_renders_nothing_and_unreadable_expression_is_error() {
        let view_engine = engine(
            json!({"$defs": {"h": {"x-view": {"hidden": true}}, "bad": {"x-view": {"text": "{a[}"}}}}),
            vec![],
        );
        assert_eq!(render(&view_engine, "/$defs/h", json!({"a": 1})), "");
        assert_eq!(
            view_engine
                .render("t.schema.json", "/$defs/bad", &json!({"a": 1}))
                .unwrap_err(),
            ViewError("表に無い式: a[".into())
        );
    }

    #[test]
    fn view_error_describes_itself_without_closing_punctuation() {
        let error: Box<dyn std::error::Error> = Box::new(ViewError("{ が閉じていない".into()));
        assert_eq!(error.to_string(), "x-view を文にできない：{ が閉じていない");
    }
}
