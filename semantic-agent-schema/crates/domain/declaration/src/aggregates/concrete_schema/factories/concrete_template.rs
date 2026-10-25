//! Concrete の雛形

use crate::aggregates::abstract_schema::AbstractSchema;
use crate::aggregates::concrete_schema::ConcreteSchema;
use crate::values::urn::Urn;
use serde_json::{Map, Value, json};

/// Concrete の雛形：Abstract の版と名前（$schema ・ $id）を入れ、作る人が渡した値（create-request）を重ねた Concrete を作る
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
    document.extend(given.clone());
    Ok(ConcreteSchema {
        id: id.clone(),
        abstract_id: of.id().clone(),
        document: Value::Object(document),
    })
}
