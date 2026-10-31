//! search ・ get

use crate::ports::contracts::Contracts;
use crate::use_cases::Outcome;
use semantic_agent_schema_declaration::values::etag::ETag;
use serde_json::json;

/// 文書の1つの場所の値と、文書の ETag（書くときに --if-match へ渡す）を返す。pointer が空なら全体
pub fn get(contracts: &dyn Contracts, path: &str, pointer: &str) -> Outcome {
    let document = match contracts.read(path) {
        Ok(Some(d)) => d,
        Ok(None) => return Outcome::refused("", &format!("{path} が無い")),
        Err(e) => return Outcome::refused("", &e),
    };
    match document.pointer(pointer) {
        Some(value) => Outcome::checked(
            &[],
            json!({"path": path, "pointer": pointer, "value": value, "etag": ETag::of(&document).as_str()}),
        ),
        None => Outcome::refused(pointer, &format!("{pointer} が無い")),
    }
}
