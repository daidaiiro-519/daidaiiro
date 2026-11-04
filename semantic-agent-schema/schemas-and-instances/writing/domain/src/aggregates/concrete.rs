//! 集約 Concrete：使う側が Abstract に従って書く schema。
//!
//! 自明として最初から domain に置いた（探索の記録 step-01 に根拠）。版は閉じた一覧から申告する。
//! 書く条件（ステップ2）：x-published: true の欄を同じ版のまま変えない。
//! Concrete に書ける x- の閉じた一覧は、Abstract の語彙（JSON Schema の propertyNames）が見る。

use crate::values::WriteViolation;
use serde_json::Value;

/// Semantic Agent Schema が入れて出す Abstract の版（閉じた一覧）。これに無い $schema は取りに行かずに断る。
pub const ABSTRACT_VERSIONS: [&str; 1] =
    ["urn:semantic-agent-schema:published-language:abstract:v1"];

/// 方言（Concrete の入れ子の x- の形を決める）。Concrete は Abstract の本体と、この方言の両方に照らす。
pub const DIALECT: &str = "urn:semantic-agent-schema:published-language:dialect:v1";

/// Concrete の文書。
#[derive(Debug, Clone, PartialEq)]
pub struct ConcreteSchema {
    document: Value,
}

impl ConcreteSchema {
    /// 文書から作る。$schema が閉じた一覧のどれかであるものだけを Concrete とみなす。
    #[must_use]
    pub fn read(document: &Value) -> Option<Self> {
        let declared = document.get("$schema")?.as_str()?;
        ABSTRACT_VERSIONS.contains(&declared).then(|| Self {
            document: document.clone(),
        })
    }

    /// $schema が Abstract の版の形（urn:semantic-agent-schema:published-language:abstract:v…）をしているか。
    /// 閉じた一覧に無い版を見分けるために使う。
    #[must_use]
    pub fn declares_abstract(document: &Value) -> bool {
        document
            .get("$schema")
            .and_then(Value::as_str)
            .is_some_and(|s| {
                s.starts_with("urn:semantic-agent-schema:published-language:abstract:")
            })
    }

    /// $id（インスタンスが $schema で指す名前）。
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.document.get("$id").and_then(Value::as_str)
    }

    /// 文書。
    #[must_use]
    pub const fn document(&self) -> &Value {
        &self.document
    }

    /// 文書のどこかで使っている x- のキーと、その場所。
    #[must_use]
    pub fn used_keywords(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        walk_keys(&self.document, "", &mut out);
        out
    }
}

impl ConcreteSchema {
    /// この中身を書いてよいか（ステップ2）。before は直す前の中身（作成なら None）。
    /// 直す前に x-published: true だった欄（入れ子 ・ items ・ $defs の中も）は、$id（版）を変えずには変えない ・ 消さない。
    ///
    /// # Errors
    /// 公開した欄を同じ版のまま変えたとき（最初に見つけた1つ）。
    pub fn check_write(&self, before: Option<&Self>) -> Result<(), WriteViolation> {
        let Some(b) = before else {
            return Ok(());
        };
        if b.id() != self.id() {
            return Ok(());
        }
        for (keyword, at) in b.used_keywords() {
            if keyword != "x-published" {
                continue;
            }
            let field = at.strip_suffix("/x-published").unwrap_or_default();
            let was = b.document.pointer(field);
            if was
                .and_then(|f| f.get("x-published"))
                .and_then(Value::as_bool)
                == Some(true)
                && self.document.pointer(field) != was
            {
                return Err(WriteViolation::new(
                    "published_changed",
                    field,
                    &format!(
                        "{field} は x-published: true の欄で、同じ版のあいだ変えない。変えるときは $id の版を上げる（いまの版：{}）",
                        self.id().unwrap_or("（無し）")
                    ),
                ));
            }
        }
        Ok(())
    }
}

fn walk_keys(v: &Value, at: &str, out: &mut Vec<(String, String)>) {
    match v {
        Value::Object(m) => {
            for (k, child) in m {
                let here = format!("{at}/{}", escape(k));
                if k.starts_with("x-") {
                    out.push((k.clone(), here.clone()));
                }
                // x- の値の中と、const ・ default ・ examples の中は値なので降りない
                if !k.starts_with("x-") && k != "const" && k != "default" && k != "examples" {
                    walk_keys(child, &here, out);
                }
            }
        }
        Value::Array(a) => {
            for (i, child) in a.iter().enumerate() {
                walk_keys(child, &format!("{at}/{i}"), out);
            }
        }
        _ => {}
    }
}

/// JSON Pointer の1段を書く（~ と / を逃がす）。
#[must_use]
pub fn escape(s: &str) -> String {
    s.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests {
    use super::ConcreteSchema;
    use serde_json::json;

    #[test]
    fn only_listed_versions_are_concrete() {
        let ok = json!({"$schema": "urn:semantic-agent-schema:published-language:abstract:v1"});
        let other = json!({"$schema": "urn:semantic-agent-schema:published-language:abstract:v9"});
        assert!(ConcreteSchema::read(&ok).is_some());
        assert!(ConcreteSchema::read(&other).is_none());
        assert!(ConcreteSchema::declares_abstract(&other));
    }

    #[test]
    fn published_field_needs_new_version() {
        let abs = "urn:semantic-agent-schema:published-language:abstract:v1";
        let before = ConcreteSchema::read(&json!({"$schema": abs, "$id": "urn:x:v1",
            "properties": {"s": {"x-published": true, "type": "string"}, "t": {"type": "string"}}}))
        .unwrap();
        let other = ConcreteSchema::read(&json!({"$schema": abs, "$id": "urn:x:v1",
            "properties": {"s": {"x-published": true, "type": "string"}, "t": {"type": "number"}}}))
        .unwrap();
        let changed = ConcreteSchema::read(&json!({"$schema": abs, "$id": "urn:x:v1",
            "properties": {"s": {"x-published": true, "type": "array"}}}))
        .unwrap();
        let next = ConcreteSchema::read(&json!({"$schema": abs, "$id": "urn:x:v2",
            "properties": {"s": {"x-published": true, "type": "array"}}}))
        .unwrap();
        assert!(other.check_write(Some(&before)).is_ok());
        assert_eq!(
            changed.check_write(Some(&before)).err().map(|v| v.code()),
            Some("published_changed")
        );
        assert!(next.check_write(Some(&before)).is_ok());
        assert!(changed.check_write(None).is_ok());
    }

    #[test]
    fn nested_published_field_needs_new_version_too() {
        let abs = "urn:semantic-agent-schema:published-language:abstract:v1";
        let doc = |t: &str| {
            ConcreteSchema::read(&json!({"$schema": abs, "$id": "urn:x:v1",
                "properties": {"steps": {"type": "array", "items": {"properties": {
                    "kind": {"x-published": true, "type": t}}}}},
                "$defs": {"d": {"x-published": true, "type": t}}}))
            .unwrap()
        };
        let before = doc("string");
        let changed = doc("number");
        let v = changed.check_write(Some(&before)).err().unwrap();
        assert_eq!(v.code(), "published_changed");
        assert_eq!(v.pointer(), "/properties/steps/items/properties/kind");
        assert!(doc("string").check_write(Some(&before)).is_ok());
    }
}
