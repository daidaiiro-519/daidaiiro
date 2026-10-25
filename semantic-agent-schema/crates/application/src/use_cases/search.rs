//! search ・ get

use crate::ports::contracts::Contracts;
use crate::use_cases::Outcome;
use serde_json::json;

/// 文書の1つの場所の値を返す。pointer が空なら全体
pub fn get(contracts: &dyn Contracts, path: &str, pointer: &str) -> Outcome {
    let document = match contracts.read(path) {
        Ok(Some(d)) => d,
        Ok(None) => return Outcome::refused("", &format!("{path} が無い")),
        Err(e) => return Outcome::refused("", &e),
    };
    match document.pointer(pointer) {
        Some(value) => Outcome::checked(
            &[],
            json!({"path": path, "pointer": pointer, "value": value}),
        ),
        None => Outcome::refused(pointer, &format!("{pointer} が無い")),
    }
}
