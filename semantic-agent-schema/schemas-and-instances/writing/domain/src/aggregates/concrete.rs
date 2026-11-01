//! 集約 Concrete：使う側が Abstract に従って書く schema。
//!
//! 自明として最初から domain に置いた（探索の記録 step-01 に根拠）。版は閉じた一覧から申告する。
//! 書く条件（ステップ2）：公開した欄（x-published: true）を同じ版のまま変えない。
//! 本体（Abstract）の x- の名前を足さない ・ 使う x- は本体か自分の x-extensions にあるもの、の判定は
//! 検証のモジュール（validation-domain の services::judge_concrete）が持つ。

use crate::values::WriteViolation;
use serde_json::Value;

/// SAS が入れて出す Abstract の版（閉じた一覧）。これに無い $schema は取りに行かずに断る。
pub const ABSTRACT_VERSIONS: [&str; 1] =
    ["urn:semantic-agent-schema:published-language:abstract:v1"];

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

    /// 申告した Abstract の版。
    #[must_use]
    pub fn declared(&self) -> &str {
        self.document
            .get("$schema")
            .and_then(Value::as_str)
            .unwrap_or_default()
    }

    /// 文書のどこかで使っている x- のキーと、その場所。
    #[must_use]
    pub fn used_keywords(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        walk_keys(&self.document, "", &mut out);
        out
    }

    /// x- の名前を使っているか。
    #[must_use]
    pub fn uses(&self, keyword: &str) -> bool {
        self.used_keywords().iter().any(|(k, _)| k == keyword)
    }
}

impl ConcreteSchema {
    /// この中身を書いてよいか（ステップ2）。before は直す前の中身（作成なら None）。
    /// 直す前に x-published: true だった根の欄は、$id（版）を変えずには変えない ・ 消さない。
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
        let now = self.document.get("properties");
        for (name, field) in b
            .document
            .get("properties")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            if field.get("x-published").and_then(Value::as_bool) == Some(true)
                && now.and_then(|p| p.get(name)) != Some(field)
            {
                let at = format!("/properties/{}", escape(name));
                return Err(WriteViolation::new(
                    "published_changed",
                    &at,
                    &format!(
                        "{at} は x-published: true の欄で、同じ版のあいだ変えない。変えるときは $id の版を上げる（いまの版：{}）",
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
                // x-extensions の中身と、x- の値の中は、名前の並びや値なので降りない
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
}
