//! Semantic Abstract Schema（セマンティックアブストラクトスキーマ）

use crate::values::urn::Urn;
use serde_json::{Value, json};

/// Abstract が根と欄に足す9つの x-
pub const X_KEYWORDS: [&str; 9] = [
    "x-instance-path",
    "x-instance-label",
    "x-delegates",
    "x-extensions",
    "x-prompt",
    "x-relationship",
    "x-derived-from",
    "x-decided-by",
    "x-published",
];

/// 実行ファイルに入れて出す Abstract の版（閉じた一覧の中身）
const SHIPPED: [&str; 2] = [
    include_str!("../../../../../schemas/v1/semantic-abstract-schema.json"),
    include_str!("../../../../../schemas/v1/glossary.schema.json"),
];

/// Semantic Abstract Schema（セマンティックアブストラクトスキーマ）：Concrete が $schema に書くメタスキーマの1つの版
#[derive(Debug, Clone, PartialEq)]
pub struct AbstractSchema {
    id: Urn,
    document: Value,
}

impl AbstractSchema {
    /// 実行ファイルに入れて出す版の一覧（閉じた一覧）。不変条件 I2
    ///
    /// # Errors
    /// 入れて出す版が JSON でないか、$id が URN でないとき（作り方の誤り）
    pub fn shipped() -> Result<Vec<AbstractSchema>, String> {
        SHIPPED.iter().map(|text| Self::from_text(text)).collect()
    }

    fn from_text(text: &str) -> Result<Self, String> {
        let document: Value =
            serde_json::from_str(text).map_err(|e| format!("Abstract が JSON でない：{e}"))?;
        let id = document
            .get("$id")
            .and_then(Value::as_str)
            .ok_or("Abstract に $id が無い")?;
        Ok(AbstractSchema {
            id: Urn::parse(id)?,
            document,
        })
    }

    /// 名前（$id）
    pub fn id(&self) -> &Urn {
        &self.id
    }

    /// 文書（JSON Schema）
    pub fn document(&self) -> &Value {
        &self.document
    }

    /// Abstract の本体（Concrete が $schema に書くメタスキーマ）か。ほかは Abstract のファイル（glossary など）
    pub fn is_body(&self) -> bool {
        self.id.kind() == "semantic-abstract-schema"
    }

    /// 本体の根に書けるもののうち、文書にまだ無いものを、案内（説明 ・ 読み方 ・ 書くときの rule の id）と一緒に返す。
    /// x- の案内は本体の x-extensions、ほかは本体の properties が持つ
    pub fn unfilled(&self, document: &Value) -> Vec<Value> {
        let extensions = self.document.get("x-extensions");
        self.document
            .get("properties")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .filter(|(name, _)| document.get(name.as_str()).is_none())
            .map(|(name, property)| {
                // 本体の中の $ref（#/$defs/…）は、指した先の説明と案内を読む
                let target = property
                    .get("$ref")
                    .and_then(Value::as_str)
                    .and_then(|r| r.strip_prefix('#'))
                    .and_then(|p| self.document.pointer(p))
                    .unwrap_or(property);
                let (description, prompt) = match extensions.and_then(|x| x.get(name.as_str())) {
                    Some(x) => (x.get("description"), x.get("prompt")),
                    None => (
                        property.get("description").or(target.get("description")),
                        property.get("x-prompt").or(target.get("x-prompt")),
                    ),
                };
                let rules: Vec<Value> = prompt
                    .and_then(|p| p.get("write"))
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|r| r.get("id").cloned())
                    .collect();
                json!({
                    "name": name,
                    "description": description.cloned().unwrap_or(Value::Null),
                    "means": prompt.and_then(|p| p.pointer("/read/means")).cloned().unwrap_or(Value::Null),
                    "rules": rules
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::AbstractSchema;

    /// traces: U1:事後
    #[test]
    fn unfilled_lists_x_keywords_with_their_guidance() {
        let body = AbstractSchema::shipped().unwrap().remove(0);
        let left = body.unfilled(&serde_json::json!({"title": "t"}));
        let path = left
            .iter()
            .find(|u| u["name"] == "x-instance-path")
            .unwrap();
        assert!(path["description"].is_string() && left.iter().all(|u| u["name"] != "title"));
    }

    /// traces: I2
    #[test]
    fn shipped_list_has_the_abstract_v1() {
        let list = AbstractSchema::shipped().unwrap();
        assert!(
            list.iter().any(|a| a.id().to_string()
                == "urn:semantic-agent-schema:base:semantic-abstract-schema:v1")
        );
    }
}
