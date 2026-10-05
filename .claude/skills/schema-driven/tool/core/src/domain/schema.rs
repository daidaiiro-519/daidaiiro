//! スキーマ（VO-2 が指す JSON Schema）による検証と、x-prompt の取り出し。入出力を持たない。

use crate::domain::values::{Unfilled, ValidationError};
use serde_json::Value;

/// 検証結果（情報の別名 TERM-9）。未記入は検証エラーと分けて返す（ACDR 0118）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Validation {
    pub errors: Vec<ValidationError>,
    pub unfilled: Vec<Unfilled>,
}

/// スキーマそのものが壊れていて、検証器を作れない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaError(pub String);

/// 読み込んだスキーマ。本体と、`$ref` の先になる同じディレクトリのスキーマを持つ。
pub struct Schema {
    name: String,
    root: Value,
    siblings: Vec<(String, Value)>,
}

const BASE: &str = "file:///schemas/";

impl Schema {
    /// `name` はスキーマのファイル名、`siblings` は同じディレクトリのスキーマ（ファイル名と内容）。
    pub fn new(name: &str, root: Value, siblings: Vec<(String, Value)>) -> Self {
        Self {
            name: name.to_owned(),
            root,
            siblings,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn root(&self) -> &Value {
        &self.root
    }

    /// インスタンスを検証する。必須のプロパティが無いことは未記入として分ける。
    pub fn validate(&self, instance: &Value) -> Result<Validation, SchemaError> {
        let mut registry = jsonschema::Registry::new();
        for (name, value) in &self.siblings {
            registry = registry
                .add(format!("{BASE}{name}"), value.clone())
                .map_err(|e| SchemaError(e.to_string()))?;
        }
        let registry = registry.prepare().map_err(|e| SchemaError(e.to_string()))?;
        let validator = jsonschema::options()
            .offline()
            .with_registry(&registry)
            .with_base_uri(format!("{BASE}{}", self.name))
            .build(&self.root)
            .map_err(|e| SchemaError(e.to_string()))?;
        let mut out = Validation::default();
        for err in validator.iter_errors(instance) {
            let at = err.instance_path().to_string();
            if let jsonschema::error::ValidationErrorKind::Required { property } = err.kind() {
                let name = property
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| property.to_string());
                let pointer = format!("{at}/{}", name.replace('~', "~0").replace('/', "~1"));
                if let Ok(u) = Unfilled::new(&pointer) {
                    out.unfilled.push(u);
                }
            } else if let Ok(e) = ValidationError::new(&at, &err.to_string()) {
                out.errors.push(e);
            }
        }
        out.unfilled.sort_by(|a, b| a.property().cmp(b.property()));
        Ok(out)
    }

    /// プロパティ（JSON Pointer）の x-prompt を返す。プロパティがスキーマに無いか、x-prompt を持たなければ None。
    pub fn prompt(&self, pointer: &str) -> Option<Value> {
        let mut node = &self.root;
        let mut doc = &self.root;
        for raw in pointer.split('/').skip(1) {
            let seg = raw.replace("~1", "/").replace("~0", "~");
            let (n, d) = self.resolve(node, doc);
            node = n;
            doc = d;
            node = if seg.chars().all(|c| c.is_ascii_digit()) && node.get("items").is_some() {
                node.get("items")?
            } else {
                node.get("properties")?.get(&seg)?
            };
        }
        if pointer.is_empty() {
            return None;
        }
        if let Some(p) = node.get("x-prompt") {
            return Some(p.clone());
        }
        let (n, _) = self.resolve(node, doc);
        n.get("x-prompt").cloned()
    }

    /// `$ref` をたどる。同じファイル（`#/…`）と、同じディレクトリのスキーマ（`名前#/…`）だけを解く。
    pub(crate) fn resolve<'a>(
        &'a self,
        mut node: &'a Value,
        mut doc: &'a Value,
    ) -> (&'a Value, &'a Value) {
        for _ in 0..16 {
            let Some(r) = node.get("$ref").and_then(Value::as_str) else {
                break;
            };
            let (file, frag) = r.split_once('#').unwrap_or((r, ""));
            if !file.is_empty() {
                match self.siblings.iter().find(|(n, _)| n == file) {
                    Some((_, v)) => doc = v,
                    None => break,
                }
            }
            match doc.pointer(frag) {
                Some(v) => node = v,
                None => break,
            }
        }
        (node, doc)
    }
}
