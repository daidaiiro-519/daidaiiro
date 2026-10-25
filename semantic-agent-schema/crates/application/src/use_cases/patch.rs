//! patch（検査①：適用する前に入力を見る）

use crate::ports::contracts::Contracts;
use crate::use_cases::Outcome;
use crate::use_cases::validate::write_checked;
use semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema;
use serde_json::{Value, json};

/// Concrete に JSON Patch を適用する。検査①で外れたら断る。検査②で、この書き込みが新しく持ち込んだ機械の violated があれば書かずに返す
pub fn patch(contracts: &dyn Contracts, path: &str, ops: &Value) -> Outcome {
    let document = match contracts.read(path) {
        Ok(Some(d)) => d,
        Ok(None) => return Outcome::refused("", &format!("{path} が無い")),
        Err(e) => return Outcome::refused("", &e),
    };
    let concrete = match ConcreteSchema::from_document(document.clone()) {
        Ok(c) => c,
        Err(e) => return Outcome::refused("", &e),
    };
    let next = match concrete.patched(ops) {
        Ok(n) => n,
        Err(r) => return Outcome::refused(r.pointer(), r.reason()),
    };
    write_checked(
        contracts,
        path,
        Some(&document),
        next.document(),
        json!({"path": path}),
    )
}
