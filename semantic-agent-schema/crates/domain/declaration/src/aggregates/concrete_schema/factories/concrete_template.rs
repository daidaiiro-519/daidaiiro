//! Concrete の雛形

use crate::aggregates::abstract_schema::AbstractSchema;
use crate::aggregates::concrete_schema::ConcreteSchema;
use crate::values::urn::Urn;
use serde_json::{Value, json};

/// Concrete の雛形：Abstract の必須の欄だけを持ち、そのまま Abstract に照らして通る Concrete を作る
pub fn concrete_template(id: &Urn, of: &AbstractSchema) -> ConcreteSchema {
    let document: Value = json!({
        "$schema": of.id().to_string(),
        "$id": id.to_string(),
        "title": id.kind(),
        "description": "",
        "type": "object",
        "properties": {}
    });
    ConcreteSchema {
        id: id.clone(),
        abstract_id: of.id().clone(),
        document,
    }
}
