//! glossary

use crate::values::urn::Urn;
use serde_json::{Value, json};

/// glossary（用語集）：文脈の言葉（term）を $defs に置き、$anchor で名前を付ける
#[derive(Debug, Clone, PartialEq)]
pub struct Glossary {
    id: Urn,
    document: Value,
}

/// glossary の Abstract のファイルの名前
pub const GLOSSARY_SCHEMA: &str = "urn:semantic-agent-schema:base:glossary:v1";

impl Glossary {
    /// term の無い glossary を作る
    ///
    /// # Errors
    /// 名前の種類が glossary でないとき
    pub fn empty(id: &Urn) -> Result<Self, String> {
        if id.kind() != "glossary" {
            return Err(format!(
                "glossary の種類は glossary（{} ではない）",
                id.kind()
            ));
        }
        let document = json!({"$schema": GLOSSARY_SCHEMA, "$id": id.to_string(), "title": id.kind(), "$defs": {}});
        Ok(Glossary {
            id: id.clone(),
            document,
        })
    }

    /// 文書から読む
    ///
    /// # Errors
    /// $id が無いか、種類が glossary の URN でないとき
    pub fn from_document(document: Value) -> Result<Self, String> {
        let id = Urn::parse(
            document
                .get("$id")
                .and_then(Value::as_str)
                .ok_or("$id が無い")?,
        )?;
        if id.kind() != "glossary" {
            return Err(format!("{id} は glossary でない"));
        }
        Ok(Glossary { id, document })
    }

    /// 名前（$id）
    pub fn id(&self) -> &Urn {
        &self.id
    }

    /// 文書
    pub fn document(&self) -> &Value {
        &self.document
    }

    /// その名前（$anchor）の term があるか
    pub fn defines(&self, name: &str) -> bool {
        self.document
            .get("$defs")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .any(|(_, t)| t.get("$anchor").and_then(Value::as_str) == Some(name))
    }
}

#[cfg(test)]
mod tests {
    use super::Glossary;
    use crate::values::urn::Urn;
    use serde_json::json;

    /// traces: I4
    #[test]
    fn anchor_names_the_term() {
        let g = Glossary::from_document(json!({"$id": "urn:semantic-agent-schema:a:glossary:v1", "$defs": {"price": {"$anchor": "Price"}}})).unwrap();
        assert!(g.defines("Price") && !g.defines("price"));
    }

    /// traces: I4
    #[test]
    fn glossary_must_be_of_kind_glossary() {
        assert!(Glossary::empty(&Urn::new("a", "record", 1).unwrap()).is_err());
    }
}
