//! インスタンス（作る ・ 取り込む）

use crate::aggregates::instance::Instance;
use semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema;
use serde_json::{Map, Value, json};

/// インスタンスの雛形：Concrete の名前（$schema）・ 種類（kind の const）・ id を入れ、作る人が渡した値を重ねる
///
/// # Errors
/// Concrete に kind の const が無いか、渡した値が object でないか、$schema ・ kind ・ id を含むとき
pub fn instance_template(
    concrete: &ConcreteSchema,
    id: &str,
    value: &Value,
) -> Result<Instance, String> {
    let kind = concrete
        .document()
        .pointer("/properties/kind/const")
        .and_then(Value::as_str)
        .ok_or("Concrete の根の properties に kind の const が無い（インスタンスを作るスキーマでない）")?;
    let given = value.as_object().ok_or("作る値は object で渡す")?;
    if ["$schema", "kind", "id"]
        .iter()
        .any(|k| given.contains_key(*k))
    {
        return Err("$schema ・ kind ・ id は渡さない（--schema と create が決める）".to_string());
    }
    let mut document: Map<String, Value> = Map::new();
    document.insert("$schema".to_string(), json!(concrete.id().to_string()));
    document.insert("kind".to_string(), json!(kind));
    document.insert("id".to_string(), json!(id));
    document.extend(given.clone());
    Ok(Instance {
        schema: concrete.id().clone(),
        kind: kind.to_string(),
        id: id.to_string(),
        document: Value::Object(document),
    })
}

#[cfg(test)]
mod tests {
    use super::instance_template;
    use semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema;
    use serde_json::json;

    fn concrete() -> ConcreteSchema {
        ConcreteSchema::from_document(json!({"$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1", "$id": "urn:semantic-agent-schema:t:record:v1", "properties": {"kind": {"const": "record"}}})).unwrap()
    }

    /// traces: U5:事後
    #[test]
    fn template_takes_kind_from_the_concrete() {
        let i = instance_template(&concrete(), "r-1", &json!({"title": "t"})).unwrap();
        assert_eq!(
            (i.kind(), i.document()["$schema"].as_str()),
            ("record", Some("urn:semantic-agent-schema:t:record:v1"))
        );
    }

    /// traces: U5:失敗時
    #[test]
    fn value_with_id_is_refused() {
        assert!(instance_template(&concrete(), "r-1", &json!({"id": "x"})).is_err());
    }
}
