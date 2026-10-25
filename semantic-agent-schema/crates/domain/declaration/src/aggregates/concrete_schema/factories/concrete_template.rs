//! Concrete の雛形

use crate::aggregates::abstract_schema::AbstractSchema;
use crate::aggregates::concrete_schema::ConcreteSchema;
use crate::values::urn::Urn;
use serde_json::{Map, Value, json};

/// 作った直後から置く入れ物。ほとんどの Concrete が使い、空でも Abstract に合う
const CONTAINERS: [&str; 4] = ["properties", "$defs", "x-delegates", "x-extensions"];

/// Concrete の雛形：Abstract の版と名前（$schema ・ $id）と空の入れ物（properties ・ $defs ・ x-delegates ・ x-extensions）を入れ、
/// 作る人が渡した値（create-request）を重ねた Concrete を作る
///
/// # Errors
/// 渡した値が object でないか、$schema ・ $id を含むとき（名前は引数で決める）
pub fn concrete_template(
    id: &Urn,
    of: &AbstractSchema,
    value: &Value,
) -> Result<ConcreteSchema, String> {
    let given = value.as_object().ok_or("作る値は object で渡す")?;
    if given.contains_key("$schema") || given.contains_key("$id") {
        return Err("$schema と $id は渡さない（--owner と --kind で決まる）".to_string());
    }
    let mut document: Map<String, Value> = Map::new();
    document.insert("$schema".to_string(), json!(of.id().to_string()));
    document.insert("$id".to_string(), json!(id.to_string()));
    for c in CONTAINERS {
        document.insert(c.to_string(), json!({}));
    }
    document.extend(given.clone());
    Ok(ConcreteSchema {
        id: id.clone(),
        abstract_id: of.id().clone(),
        document: Value::Object(document),
    })
}

#[cfg(test)]
mod tests {
    use super::concrete_template;
    use crate::aggregates::abstract_schema::AbstractSchema;
    use crate::values::urn::Urn;
    use serde_json::json;

    /// traces: U1:事後
    #[test]
    fn template_has_the_four_containers() {
        let of = AbstractSchema::shipped().unwrap().remove(0);
        let c = concrete_template(&Urn::new("o", "r", 1).unwrap(), &of, &json!({"title": "t"}))
            .unwrap();
        assert_eq!(c.document()["x-extensions"], json!({}));
    }

    /// traces: U1:事後
    #[test]
    fn given_value_overrides_a_container() {
        let of = AbstractSchema::shipped().unwrap().remove(0);
        let c = concrete_template(
            &Urn::new("o", "r", 1).unwrap(),
            &of,
            &json!({"properties": {"a": {}}}),
        )
        .unwrap();
        assert_eq!(c.document()["properties"], json!({"a": {}}));
    }
}
