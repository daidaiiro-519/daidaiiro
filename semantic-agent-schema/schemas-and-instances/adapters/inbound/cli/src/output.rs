//! 結果を JSON の1行にする（operation-result.schema.json の validation ・ change ・ error）。

use schemas_and_instances_conformance_application::use_cases::documents::{
    ChangeResult, Refused, Validation,
};
use schemas_and_instances_conformance_domain::values::{Counts, RuleResult, Verdict};
use schemas_and_instances_operation_recording_application::ports::WriteRecord;
use schemas_and_instances_operation_recording_application::use_cases::history::Interleaved;
use serde_json::{Map, Value, json};

fn counts(c: &Counts) -> Value {
    let mut m = Map::new();
    for v in Verdict::ALL {
        m.insert(v.as_str().to_string(), json!(c.get(v)));
    }
    Value::Object(m)
}

fn rule_result(r: &RuleResult) -> Value {
    json!({
        "rule": r.rule(),
        "pointer": r.pointer(),
        "by": r.by().as_str(),
        "result": r.verdict().as_str(),
        "reason": r.reason(),
    })
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
                "results": t.results.iter().map(rule_result).collect::<Vec<_>>(),
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

/// 記録にある、対象を書いた操作1件（間に入った変更の1件）。
fn write_record(w: &WriteRecord) -> Value {
    // 記録の欄の値は JSON の文字列で持つので、読み直して値にする
    let value = |t: &Option<String>| {
        t.as_deref()
            .and_then(|t| serde_json::from_str::<Value>(t).ok())
    };
    // 読む順（だれが ・ 何のために ・ いつ ・ どの欄を）に並べる
    let mut v = json!({"operation": w.operation, "session": w.session, "kind": w.row.kind});
    if let Some(i) = &w.row.intent {
        v["intent"] = json!(i);
    }
    v["at"] = json!(w.at);
    if let Some(b) = &w.row.before {
        v["before"] = json!({"fingerprint": b});
    }
    v["after"] = json!({"fingerprint": w.row.after});
    v["changes"] = w
        .row
        .fields
        .iter()
        .map(|f| {
            let mut c = json!({"pointer": f.pointer, "change": f.change});
            if let Some(b) = value(&f.before) {
                c["before"] = b;
            }
            if let Some(a) = value(&f.after) {
                c["after"] = a;
            }
            c
        })
        .collect();
    v
}

fn interleaved(i: &Interleaved) -> Value {
    let mut v = json!({
        "expected": i.expected,
        "current": i.current,
        "untracked": i.untracked,
        "operations": i.operations.iter().map(write_record).collect::<Vec<_>>(),
    });
    if let Some(w) = &i.last_recorded {
        v["last_recorded"] = json!({
            "operation": w.operation,
            "session": w.session,
            "at": w.at,
            "after": {"fingerprint": w.row.after},
        });
    }
    v
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
    if let Some(s) = &r.session {
        out["session"] = json!(s);
    }
    if let Some(o) = &r.operation {
        out["operation"] = json!(o);
    }
    if let Some(i) = &r.interleaved {
        out["interleaved"] = interleaved(i);
    }
    println!("{out}");
}

/// 作成か更新の結果を出す。
pub fn print_change(c: &ChangeResult) {
    let mut diff = json!({
        "source": "sas",
        "after": {"fingerprint": c.after.as_str()},
        "changes": c.diff.changes().iter().map(|f| {
            let mut v = json!({"pointer": f.pointer(), "change": f.change().as_str()});
            if let Some(b) = f.before() {
                v["before"] = b.clone();
            }
            if let Some(a) = f.after() {
                v["after"] = a.clone();
            }
            v
        }).collect::<Vec<_>>(),
    });
    if let Some(b) = &c.before {
        diff["before"] = json!({"fingerprint": b.as_str()});
    }
    let out = json!({
        "$schema": "urn:semantic-agent-schema:base:operation-result:v1",
        "kind": "change",
        "session": c.session,
        "operation": c.operation,
        "result": c.written.as_str(),
        "replayed": c.replayed,
        "target": {"kind": c.kind.as_str(), "path": c.path, "concrete": c.concrete},
        "diff": diff,
        "stale_rules": c.stale.iter().map(|s| json!({
            "rule": s.rule, "pointer": s.pointer, "operation": s.operation,
        })).collect::<Vec<_>>(),
        "violated_instances": c.violated.iter().map(|v| json!({
            "path": v.path,
            "results": v.results.iter().map(rule_result).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    });
    println!("{out}");
}
