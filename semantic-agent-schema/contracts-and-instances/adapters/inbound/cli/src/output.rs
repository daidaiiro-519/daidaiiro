//! 結果を JSON の1行にする（operation-result.schema.json の validation と error）。

use contracts_and_instances_conformance_application::use_cases::documents::{Refused, Validation};
use contracts_and_instances_conformance_domain::values::{Counts, Verdict};
use serde_json::{Map, Value, json};

fn counts(c: &Counts) -> Value {
    let mut m = Map::new();
    for v in Verdict::ALL {
        m.insert(v.as_str().to_string(), json!(c.get(v)));
    }
    Value::Object(m)
}

/// 検証の結果を出す。
pub fn print_validation(v: &Validation) {
    let targets: Vec<Value> = v
        .targets
        .iter()
        .map(|t| {
            json!({
                "path": t.path,
                "kind": t.kind.as_str(),
                "schema": t.schema,
                "fingerprint": t.fingerprint.as_str(),
                "operation": t.operation,
                "results": t.results.iter().map(|r| json!({
                    "rule": r.rule(),
                    "pointer": r.pointer(),
                    "by": r.by().as_str(),
                    "result": r.verdict().as_str(),
                    "reason": r.reason(),
                })).collect::<Vec<_>>(),
                "counts": counts(&t.counts),
            })
        })
        .collect();
    let out = json!({
        "$schema": "urn:semantic-agent-schema:base:operation-result:v1",
        "kind": "validation",
        "session": v.session,
        "operations": v.targets.iter().map(|t| t.operation.clone()).collect::<Vec<_>>(),
        "targets": targets,
        "counts": counts(&v.counts),
    });
    println!("{out}");
}

/// 始められなかった理由を出す。
pub fn print_refused(r: &Refused) {
    let mut out = json!({
        "$schema": "urn:semantic-agent-schema:base:operation-result:v1",
        "kind": "error",
        "code": r.code,
        "message": r.message,
    });
    if let Some(p) = &r.path {
        out["path"] = json!(p);
    }
    println!("{out}");
}
