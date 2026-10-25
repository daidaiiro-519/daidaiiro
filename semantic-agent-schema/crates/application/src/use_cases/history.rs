//! history

use crate::use_cases::{Outcome, Ports};
use semantic_agent_schema_declaration::values::target::Target;
use serde_json::{Value, json};

/// 対象を変えた変更の記録を、意図の判定と一緒に古い順に返す（使う場面 U13。いまは手元の記録だけ）
pub fn history(ports: &Ports<'_>, path: &str) -> Outcome {
    let document = match ports.instances.read(path) {
        Ok(d) => d,
        Err(e) => return Outcome::refused("", &e),
    };
    let target = Target::of_document(path, document.as_ref());
    let records = match ports.records.touching(&target) {
        Ok(r) => r,
        Err(e) => return Outcome::refused("", &e),
    };
    let mut rows: Vec<Value> = Vec::new();
    for r in &records {
        let judgment = match ports.judgments.intent_of(r.id()) {
            Ok(j) => j.map(|j| json!({"by": j.by().as_str(), "result": j.result().as_str(), "meaning": j.meaning(), "reason": j.reason()})),
            Err(e) => return Outcome::refused("", &e),
        };
        let fields: Vec<&str> = r
            .items()
            .iter()
            .filter(|i| i.target() == &target)
            .filter_map(|i| i.field())
            .collect();
        rows.push(json!({"id": r.id().as_str(), "purpose_ref": r.purpose_ref(), "intent": r.intent(), "writer_kind": r.writer().as_str(), "fields": fields, "intent_judgment": judgment}));
    }
    Outcome::checked(&[], json!({"target": target.to_json(), "changes": rows}))
}
