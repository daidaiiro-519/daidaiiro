//! patch（検査①：適用する前に入力を見る）

use crate::ports::contracts::Contracts;
use crate::ports::instances::Instances;
use crate::use_cases::Outcome;
use crate::use_cases::validate::{is_declaration, write_checked};
use semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema;
use semantic_agent_schema_instance::aggregates::instance::Instance;
use serde_json::{Value, json};

/// Concrete かインスタンスに JSON Patch を適用する。検査①で外れたら断る。
/// 検査②で、この書き込みが新しく持ち込んだ機械の violated があれば書かずに返す
pub fn patch(
    contracts: &dyn Contracts,
    instances: &dyn Instances,
    path: &str,
    ops: &Value,
) -> Outcome {
    let document = match instances.read(path) {
        Ok(Some(d)) => d,
        Ok(None) => return Outcome::refused("", &format!("{path} が無い")),
        Err(e) => return Outcome::refused("", &e),
    };
    let next = if is_declaration(&document) {
        ConcreteSchema::from_document(document.clone())
            .map_err(|e| (String::new(), e))
            .and_then(|c| {
                c.patched(ops)
                    .map(|n| n.document().clone())
                    .map_err(|r| (r.pointer().to_string(), r.reason().to_string()))
            })
    } else {
        Instance::from_document(document.clone())
            .map_err(|e| (String::new(), e))
            .and_then(|i| {
                i.patched(ops)
                    .map(|n| n.document().clone())
                    .map_err(|r| (r.pointer().to_string(), r.reason().to_string()))
            })
    };
    let next = match next {
        Ok(n) => n,
        Err((at, why)) => return Outcome::refused(&at, &why),
    };
    let declaration = is_declaration(&document);
    write_checked(
        contracts,
        path,
        Some(&document),
        &next,
        json!({"path": path}),
        &|doc| {
            if declaration {
                contracts.write(path, doc)
            } else {
                instances.write(path, doc)
            }
        },
    )
}
