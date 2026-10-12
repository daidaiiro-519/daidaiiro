//! core の単体テストが共有する代役。ポートをメモリ上で実装する（core の中のテストの補助は、ここ1か所に置く）。
//! - MemoryFiles：ファイルを、パス → 中身の表で持つ Files と Schemas
//! - TableQuery：式（文字列）→ 答えの関数 の表で答える Query。表に無い式は「読めない」にする
//! - small_catalog ・ TestDesign：試験用の小さな部品のカタログと、それを渡す Design

use crate::domain::values::JsonValue;
use crate::ports::Functions;
use crate::ports::{
    Catalog, Design, Files, Query, QueryError, ReadError, Schemas, WriteError, WriteIf,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// ファイルを、パス → 中身の表で持つ。複製しても同じ表を指す。
#[derive(Clone, Default)]
pub struct MemoryFiles(Arc<Mutex<BTreeMap<String, String>>>);

impl MemoryFiles {
    /// JSON の値を、パスごとに文字にして置く。
    pub fn with(files: &[(&str, Value)]) -> Self {
        let memory = Self::default();
        for (path, value) in files {
            memory.put(path, &value.to_string());
        }
        memory
    }

    pub fn put(&self, path: &str, text: &str) {
        self.0
            .lock()
            .unwrap()
            .insert(path.to_owned(), text.to_owned());
    }

    pub fn get(&self, path: &str) -> Option<String> {
        self.0.lock().unwrap().get(path).cloned()
    }

    /// 置いてあるすべてのパス（名前の順）。
    pub fn paths(&self) -> Vec<String> {
        self.0.lock().unwrap().keys().cloned().collect()
    }

    fn under(&self, dir: &str) -> Vec<String> {
        let prefix = format!("{}/", dir.trim_end_matches('/'));
        self.paths()
            .into_iter()
            .filter(|path| path.starts_with(&prefix))
            .collect()
    }
}

impl Files for MemoryFiles {
    fn exists(&self, path: &str) -> bool {
        self.get(path).is_some()
    }

    fn read(&self, path: &str) -> Result<String, ReadError> {
        self.get(path)
            .ok_or_else(|| ReadError(format!("{path}: 無い")))
    }

    fn list(&self, dir: &str) -> Result<Vec<String>, ReadError> {
        let depth = dir.trim_end_matches('/').matches('/').count() + 1;
        Ok(self
            .under(dir)
            .into_iter()
            .filter(|path| path.matches('/').count() == depth)
            .collect())
    }

    fn list_tree(&self, dir: &str) -> Result<Vec<String>, ReadError> {
        Ok(self.under(dir))
    }

    fn write(&self, path: &str, content: &str, cond: WriteIf) -> Result<(), WriteError> {
        let current = self.get(path);
        let allowed = match (&cond, &current) {
            (WriteIf::Absent, None) => true,
            (WriteIf::Unchanged(expected), Some(text)) => JsonValue::new(text).hash() == *expected,
            _ => false,
        };
        if !allowed {
            return Err(WriteError::Conflict);
        }
        self.put(path, content);
        Ok(())
    }

    fn remove(&self, path: &str) -> Result<(), WriteError> {
        self.0
            .lock()
            .unwrap()
            .remove(path)
            .map(|_| ())
            .ok_or_else(|| WriteError::Unwritable(format!("{path}: 無い")))
    }
}

/// 同じフォルダの `*.schema.json` を返す。
impl Schemas for MemoryFiles {
    fn referenced(&self, schema_path: &str) -> Result<Vec<(String, String)>, ReadError> {
        let dir = schema_path
            .rsplit_once('/')
            .map(|(dir, _)| dir)
            .unwrap_or(".");
        Ok(self
            .list(dir)?
            .into_iter()
            .filter(|path| path.ends_with(".schema.json"))
            .map(|path| {
                let name = path.rsplit('/').next().unwrap_or(&path).to_owned();
                (name, self.get(&path).unwrap_or_default())
            })
            .collect())
    }
}

/// 式に答える関数。値と、基盤の関数（search では関数の無いもの）を受け取る。
pub type Answer = fn(&Value, &dyn Functions) -> Result<Value, String>;

/// 式（文字列）→ 答えの関数 の表で答える Query の代役。表に無い式は「読めない」にする。
#[derive(Clone)]
pub struct TableQuery(Arc<Vec<(&'static str, Answer)>>);

impl TableQuery {
    pub fn new(table: Vec<(&'static str, Answer)>) -> Self {
        Self(Arc::new(table))
    }

    fn answer(&self, expression: &str) -> Result<Answer, QueryError> {
        self.0
            .iter()
            .find(|(known, _)| *known == expression)
            .map(|(_, answer)| *answer)
            .ok_or_else(|| QueryError(format!("表に無い式: {expression}")))
    }
}

/// search で渡す、関数を持たない Functions。
struct NoFunctions;

impl Functions for NoFunctions {
    fn names(&self) -> Vec<String> {
        Vec::new()
    }
    fn call(&self, name: &str, _: &[Value]) -> Result<Value, String> {
        Err(format!("知らない関数: {name}"))
    }
}

impl Query for TableQuery {
    fn search(&self, expression: &str, json: &Value) -> Result<Value, QueryError> {
        self.answer(expression)?(json, &NoFunctions).map_err(QueryError)
    }

    fn parse(&self, expression: &str) -> Result<(), QueryError> {
        self.answer(expression).map(|_| ())
    }

    fn evaluate(
        &self,
        expression: &str,
        json: &Value,
        functions: Arc<dyn Functions>,
    ) -> Result<Value, QueryError> {
        self.answer(expression)?(json, &*functions).map_err(QueryError)
    }
}

/// 値の中の JSON Pointer の場所の値。無ければ null。
pub fn at(value: &Value, pointer: &str) -> Value {
    value.pointer(pointer).cloned().unwrap_or(Value::Null)
}

/// 文字の配列をつなぐ（JMESPath の join）。
pub fn join(separator: &str, value: &Value) -> Value {
    let parts: Vec<&str> = value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    Value::String(parts.join(separator))
}

/// 試験用の小さな部品のカタログ。部品は pill（text ・ tone）・ list（items）・ figure（svg）の3つで、
/// ページのテンプレートは head の lead を {{lead}} に置く。部品の名前は、試験のこのカタログだけが持つ。
pub fn small_catalog() -> Catalog {
    Catalog {
        schema: json!({
            "$schema": "schema-driven/catalog@v1", "$id": "example/catalog@v1",
            "$defs": {
                "pill": {"type": "object", "description": "短い分類", "required": ["text"],
                         "properties": {"text": {}, "tone": {"type": "string"}},
                         "x-states": {"default": "押せない"}},
                "list": {"type": "object", "description": "並び", "required": ["items"],
                         "properties": {"items": {"type": "array"}},
                         "x-states": {"default": "押せない"}},
                "figure": {"type": "object", "description": "図", "required": ["svg"],
                           "properties": {"svg": {"type": "string", "contentMediaType": "image/svg+xml"}},
                           "x-states": {"default": "押せない"}}
            }
        })
        .to_string(),
        parts: r#"<template id="page"><style>{{tokens}}{{style}}</style><script>{{script}}</script><main><h1>{{title}}</h1><p>{{lead}}</p>{{sections}}</main></template>
<template id="section"><section><h2>{{heading}}</h2>{{body}}</section></template>
<template id="pill"><b class="pill {{tone}}">{{text}}</b></template>
<template id="list"><ul>{{items}}</ul></template>
<template id="figure"><figure>{{svg}}</figure></template>"#
            .to_owned(),
        style: ".pill{color:var(--ink)}".to_owned(),
        script: "/* 操作 */".to_owned(),
        design_system: json!({
            "scope": ":root",
            "tokens": {"ink": "#111d1a", "paper": "#f7f8f7"},
            "contrast": [{"foreground": "ink", "background": "paper", "use": "text"}]
        })
        .to_string(),
    }
}

/// 部品のカタログの供給元の代役。None なら「カタログが無い」を返す。
pub struct TestDesign(pub Option<Catalog>);

impl Design for TestDesign {
    fn catalog(&self) -> Result<Catalog, ReadError> {
        self.0
            .clone()
            .ok_or_else(|| ReadError("カタログが無い".into()))
    }
}
