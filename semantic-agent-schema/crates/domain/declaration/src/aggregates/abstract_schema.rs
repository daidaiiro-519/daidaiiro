//! Semantic Abstract Schema（セマンティックアブストラクトスキーマ）

use crate::values::urn::Urn;
use serde_json::Value;

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
}

#[cfg(test)]
mod tests {
    use super::AbstractSchema;

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
