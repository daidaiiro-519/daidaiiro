//! インスタンス。欄は pub(in crate::aggregates::instance)
pub mod factories;

use semantic_agent_schema_declaration::values::patch::{self, Refusal};
use semantic_agent_schema_declaration::values::urn::Urn;
use serde_json::{Value, json};

/// インスタンス：Concrete 1つに従う（$schema）、kind と id で指される1件
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    pub(in crate::aggregates::instance) schema: Urn,
    pub(in crate::aggregates::instance) kind: String,
    pub(in crate::aggregates::instance) id: String,
    pub(in crate::aggregates::instance) document: Value,
}

impl Instance {
    /// 文書から読む。不変条件 I8（従う Concrete を $schema で1つ申告する）
    ///
    /// # Errors
    /// $schema が URN でないか、kind か id が文字列でないとき
    pub fn from_document(document: Value) -> Result<Self, String> {
        let text = |k: &str| document.get(k).and_then(Value::as_str).map(str::to_string);
        let schema = Urn::parse(&text("$schema").ok_or("$schema が無い")?)?;
        let kind = text("kind").ok_or("kind が無い")?;
        let id = text("id").ok_or("id が無い")?;
        Ok(Instance {
            schema,
            kind,
            id,
            document,
        })
    }

    /// 従う Concrete
    pub fn schema(&self) -> &Urn {
        &self.schema
    }

    /// 種類
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// id
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 文書
    pub fn document(&self) -> &Value {
        &self.document
    }

    /// JSON Patch を適用したインスタンスを返す。検査①：$schema ・ kind ・ id を変える op は断り、自分は変えない。不変条件 I11（id は使い直さない）
    ///
    /// # Errors
    /// 断ったとき
    pub fn patched(&self, ops: &Value) -> Result<Instance, Refusal> {
        for (n, op) in ops.as_array().into_iter().flatten().enumerate() {
            let path = op.get("path").and_then(Value::as_str).unwrap_or_default();
            let kind = op.get("op").and_then(Value::as_str).unwrap_or_default();
            if kind != "test" && (path == "/kind" || path == "/id") {
                return Err(Refusal::new(
                    &format!("/{n}"),
                    "kind と id は変えない（不要になったら superseded にして作り直す）",
                ));
            }
        }
        let document = patch::apply(&self.document, ops)?;
        Ok(Instance {
            schema: self.schema.clone(),
            kind: self.kind.clone(),
            id: self.id.clone(),
            document,
        })
    }

    /// 置き場所の型（x-instance-path）の {名前} を、このインスタンスの欄の値で埋めた置き場所。埋められない名前があれば None
    pub fn expected_path(&self, template: &str) -> Option<String> {
        let mut out = String::new();
        let mut rest = template;
        while let Some(start) = rest.find('{') {
            out.push_str(&rest[..start]);
            let end = rest[start..].find('}')? + start;
            let name = &rest[start + 1..end];
            match self.document.get(name)? {
                Value::String(s) => out.push_str(s),
                Value::Number(n) => out.push_str(&n.to_string()),
                _ => return None,
            }
            rest = &rest[end + 1..];
        }
        out.push_str(rest);
        Some(out)
    }

    /// 必須の欄（required）のうち、無いか空（空の文字列 ・ 配列 ・ オブジェクト）のもの。不変条件 I12
    pub fn empty_required(&self, required: &[String]) -> Vec<String> {
        required
            .iter()
            .filter(|name| match self.document.get(name.as_str()) {
                None | Some(Value::Null) => true,
                Some(Value::String(s)) => s.is_empty(),
                Some(Value::Array(a)) => a.is_empty(),
                Some(Value::Object(o)) => o.is_empty(),
                _ => false,
            })
            .cloned()
            .collect()
    }

    /// Concrete の根の欄のうち、まだ書いていないものを、案内（説明 ・ 読み方 ・ 書くときの rule の id）と一緒に返す
    pub fn unfilled(&self, concrete: &Value) -> Vec<Value> {
        concrete
            .get("properties")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .filter(|(name, _)| self.document.get(name.as_str()).is_none())
            .map(|(name, f)| {
                let rules: Vec<Value> = f
                    .pointer("/x-prompt/write")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|r| r.get("id").cloned())
                    .collect();
                json!({
                    "name": name,
                    "description": f.get("description").cloned().unwrap_or(Value::Null),
                    "means": f.pointer("/x-prompt/read/means").cloned().unwrap_or(Value::Null),
                    "rules": rules
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Instance;
    use serde_json::json;

    fn instance() -> Instance {
        Instance::from_document(json!({"$schema": "urn:semantic-agent-schema:t:record:v1", "kind": "record", "id": "r-1", "no": "0199", "slug": "依存", "why": ""})).unwrap()
    }

    /// traces: U5:事後
    #[test]
    fn instance_declares_one_concrete() {
        assert_eq!(
            instance().schema().to_string(),
            "urn:semantic-agent-schema:t:record:v1"
        );
    }

    /// traces: U6:失敗時
    #[test]
    fn changing_the_id_is_refused() {
        assert!(
            instance()
                .patched(&json!([{"op": "replace", "path": "/id", "value": "r-2"}]))
                .is_err()
        );
    }

    /// traces: U5:事後
    #[test]
    fn path_template_is_filled_with_field_values() {
        assert_eq!(
            instance().expected_path(".rec/{no}-{slug}/r.json"),
            Some(".rec/0199-依存/r.json".to_string())
        );
    }

    /// traces: I12
    #[test]
    fn empty_and_missing_required_fields_are_listed() {
        assert_eq!(
            instance().empty_required(&["no".into(), "why".into(), "decision".into()]),
            vec!["why", "decision"]
        );
    }
}
