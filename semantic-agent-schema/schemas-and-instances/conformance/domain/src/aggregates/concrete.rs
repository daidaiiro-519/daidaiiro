//! 集約 Concrete：使う側が Abstract に従って書く schema。
//!
//! 自明として最初から domain に置いた（探索の記録 step-01 に根拠）。版は閉じた一覧から申告し、
//! 本体（Abstract）の x- の名前を足さず、使う x- は本体か自分の x-extensions にあるものだけ。
//! 書く条件（ステップ2）：公開した欄（x-published: true）を同じ版のまま変えない。

use crate::values::{By, RuleResult, Verdict, WriteViolation};
use serde_json::Value;
use std::collections::BTreeSet;

/// SAS が入れて出す Abstract の版（閉じた一覧）。これに無い $schema は取りに行かずに断る。
pub const ABSTRACT_VERSIONS: [&str; 1] =
    ["urn:semantic-agent-schema:base:semantic-abstract-schema:v1"];

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

    /// $schema が Abstract の版の形（urn:semantic-agent-schema:base:semantic-abstract-schema:v…）をしているか。
    /// 閉じた一覧に無い版を見分けるために使う。
    #[must_use]
    pub fn declares_abstract(document: &Value) -> bool {
        document
            .get("$schema")
            .and_then(Value::as_str)
            .is_some_and(|s| {
                s.starts_with("urn:semantic-agent-schema:base:semantic-abstract-schema:")
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

    /// 自分の x-extensions に足した名前。
    #[must_use]
    pub fn own_extensions(&self) -> BTreeSet<String> {
        extension_names(&self.document)
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

    /// 機械の rule のうち、集約が守る3つを判定する。知らない id は None。
    #[must_use]
    pub fn judge(
        &self,
        rule: &str,
        pointer: &str,
        abstract_names: &BTreeSet<String>,
    ) -> Option<Vec<RuleResult>> {
        match rule {
            "closed-list" => Some(vec![RuleResult::new(
                rule,
                pointer,
                By::Machine,
                Verdict::Verified,
                "申告した版は閉じた一覧にある",
            )]),
            "no-abstract-names" => Some(self.no_abstract_names(abstract_names)),
            "abstract-keywords-closed" => Some(self.keywords_closed(abstract_names)),
            _ => None,
        }
    }

    fn no_abstract_names(&self, abstract_names: &BTreeSet<String>) -> Vec<RuleResult> {
        let clash: Vec<RuleResult> = self
            .own_extensions()
            .intersection(abstract_names)
            .map(|name| {
                RuleResult::new(
                    "no-abstract-names",
                    &format!("/x-extensions/{}", escape(name)),
                    By::Machine,
                    Verdict::Violated,
                    &format!("{name} は Abstract の本体がすでに持つ名前"),
                )
            })
            .collect();
        if clash.is_empty() {
            vec![RuleResult::new(
                "no-abstract-names",
                "/x-extensions",
                By::Machine,
                Verdict::Verified,
                "本体と同じ名前を足していない",
            )]
        } else {
            clash
        }
    }

    fn keywords_closed(&self, abstract_names: &BTreeSet<String>) -> Vec<RuleResult> {
        let own = self.own_extensions();
        let unknown: Vec<RuleResult> = self
            .used_keywords()
            .into_iter()
            .filter(|(k, _)| !abstract_names.contains(k) && !own.contains(k))
            .map(|(k, at)| {
                RuleResult::new(
                    "abstract-keywords-closed",
                    &at,
                    By::Machine,
                    Verdict::Violated,
                    &format!("{k} は本体にも、このスキーマの x-extensions にも無い"),
                )
            })
            .collect();
        if unknown.is_empty() {
            vec![RuleResult::new(
                "abstract-keywords-closed",
                "",
                By::Machine,
                Verdict::Verified,
                "使っている x- はどれも定義がある",
            )]
        } else {
            unknown
        }
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
                        "{at} は x-published: true の欄で、ほかの文脈が読む。変えるときは $id の版を上げる（いまの版：{}）",
                        self.id().unwrap_or("（無し）")
                    ),
                ));
            }
        }
        Ok(())
    }

    /// この Concrete を消してよいか（ステップ4）。x-published: true の根の欄は、ほかの文脈が読むので消さない。
    /// 従うインスタンスや、この Concrete を指す別の Concrete は集約の外なので、ここでは見ない（削除の手続きが見る）。
    ///
    /// # Errors
    /// 公開した欄を持つとき（最初に見つけた1つ）。
    pub fn check_delete(&self) -> Result<(), WriteViolation> {
        for (name, field) in self
            .document
            .get("properties")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            if field.get("x-published").and_then(Value::as_bool) == Some(true) {
                let at = format!("/properties/{}", escape(name));
                return Err(WriteViolation::new(
                    "published_changed",
                    &at,
                    &format!(
                        "{at} は x-published: true の欄で、ほかの文脈が読む。公開した版（{}）は消さない",
                        self.id().unwrap_or("（無し）")
                    ),
                ));
            }
        }
        Ok(())
    }
}

/// 文書の根の x-extensions に並ぶ名前（Abstract の本体にも Concrete にも使う）。
#[must_use]
pub fn extension_names(document: &Value) -> BTreeSet<String> {
    document
        .get("x-extensions")
        .and_then(Value::as_object)
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
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
    use crate::values::Verdict;
    use serde_json::json;
    use std::collections::BTreeSet;

    fn names() -> BTreeSet<String> {
        ["x-extensions", "x-prompt", "x-relationship"]
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn only_listed_versions_are_concrete() {
        let ok = json!({"$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1"});
        let other =
            json!({"$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v9"});
        assert!(ConcreteSchema::read(&ok).is_some());
        assert!(ConcreteSchema::read(&other).is_none());
        assert!(ConcreteSchema::declares_abstract(&other));
    }

    #[test]
    fn clash_with_abstract_name_is_violated() {
        let c = ConcreteSchema::read(&json!({
            "$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1",
            "x-extensions": {"x-prompt": {}}
        }))
        .unwrap();
        let r = c.judge("no-abstract-names", "", &names()).unwrap();
        assert_eq!(r[0].verdict(), Verdict::Violated);
        assert_eq!(r[0].pointer(), "/x-extensions/x-prompt");
    }

    #[test]
    fn unknown_keyword_is_violated_and_own_is_allowed() {
        let c = ConcreteSchema::read(&json!({
            "$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1",
            "x-extensions": {"x-mine": {}},
            "properties": {"a": {"x-mine": 1, "x-other": 2}}
        }))
        .unwrap();
        let r = c.judge("abstract-keywords-closed", "", &names()).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].pointer(), "/properties/a/x-other");
    }

    #[test]
    fn published_field_needs_new_version() {
        let abs = "urn:semantic-agent-schema:base:semantic-abstract-schema:v1";
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
    fn a_published_concrete_is_not_deleted() {
        let open = ConcreteSchema::read(
            &json!({"$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1",
            "$id": "urn:semantic-agent-schema:t:uc:v1", "properties": {"goal": {"type": "string"}}}),
        );
        assert!(open.is_some_and(|c| c.check_delete().is_ok()));
        let published = ConcreteSchema::read(
            &json!({"$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1",
            "$id": "urn:semantic-agent-schema:t:uc:v1", "properties": {"goal": {"type": "string", "x-published": true}}}),
        );
        let refused = published.map(|c| c.check_delete());
        assert!(
            matches!(refused, Some(Err(v)) if v.code() == "published_changed" && v.pointer() == "/properties/goal")
        );
    }
}
