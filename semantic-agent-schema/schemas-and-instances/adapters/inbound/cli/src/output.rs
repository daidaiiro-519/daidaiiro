//! 結果を JSON の1行にする（operation-result.schema.json の validation ・ change ・ error）。

use recording_application::ports::WriteRecord;
use recording_application::use_cases::history::Interleaved;
use serde_json::{Map, Value, json};
use validation_application::use_cases::validate::{Refused, Validation};
use validation_domain::values::{Counts, RuleResult, Verdict};
use writing_application::use_cases::write::{ChangeResult, Deletion};
use writing_domain::values::Diff;

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
            let mut o = json!({
                "path": t.path,
                "kind": t.kind.as_str(),
                "schema": t.schema,
                "fingerprint": t.fingerprint.as_str(),
                "operation": t.operation,
                "results": t.results.iter().map(rule_result).collect::<Vec<_>>(),
                "counts": counts(&t.counts),
            });
            if v.stale_only {
                o["unchanged_rules"] = json!(t.unchanged_rules);
            }
            o
        })
        .collect();
    let mut out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:operation-result:v1",
        "kind": "validation",
        "parent_session": v.parent,
        "operations": v.targets.iter().map(|t| t.operation.clone()).collect::<Vec<_>>(),
        "targets": targets,
        "counts": counts(&v.counts),
        "untracked": v.untracked.iter().map(|u| json!({
            "path": u.path,
            "last_operation": u.last_operation,
            "expected": u.expected,
            "actual": u.actual,
        })).collect::<Vec<_>>(),
    });
    if v.stale_only {
        out["stale_only"] = json!(true);
        out["unchanged_targets"] = json!(v.unchanged_targets);
    }
    if let Some(u) = v.untracked.first() {
        out["hint"] = json!(format!(
            "SAS を通らない変更がある。意図を付けて取り込む：sas update {} --adopt --intent \"…\"",
            if v.untracked.len() == 1 {
                u.path.as_str()
            } else {
                "<経路>"
            }
        ));
    }
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
    let mut v = json!({"operation": w.operation, "parent_session": w.parent, "kind": w.row.kind, "source": w.row.source});
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
            if f.before_unavailable {
                c["before_unavailable"] = json!(true);
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
            "parent_session": w.parent,
            "at": w.at,
            "after": {"fingerprint": w.row.after},
        });
    }
    v
}

/// 始められなかった理由を出す。
pub fn print_refused(r: &Refused) {
    let mut out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:operation-result:v1",
        "kind": "error",
        "code": r.code,
        "message": r.message,
    });
    if let Some(p) = &r.path {
        out["path"] = json!(p);
    }
    if let Some(o) = &r.operation {
        out["operation"] = json!(o);
    }
    if let Some(i) = &r.interleaved {
        out["interleaved"] = interleaved(i);
    }
    if let Some(h) = &r.hint {
        out["hint"] = json!(h);
    }
    if let Some(blocked) = &r.blocked {
        out["blocked"] = blocked
            .iter()
            .map(|b| json!({"path": b.path, "code": b.code, "message": b.message, "hint": b.hint}))
            .collect();
    }
    println!("{out}");
}

fn changes(d: &Diff) -> Value {
    d.changes()
        .iter()
        .map(|f| {
            let mut v = json!({"pointer": f.pointer(), "change": f.change().as_str()});
            if let Some(b) = f.before() {
                v["before"] = b.clone();
            }
            if let Some(a) = f.after() {
                v["after"] = a.clone();
            }
            v
        })
        .collect()
}

/// 削除の結果を出す。diff は消す前の状態（根の欄ごとの remove）で、事後を持たない。
pub fn print_deletion(d: &Deletion) {
    let out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:operation-result:v1",
        "kind": "change",
        "parent_session": d.parent,
        "operation": d.operation,
        "result": "deleted",
        "replayed": d.replayed,
        "target": {"kind": d.kind.as_str(), "path": d.path, "concrete": d.concrete},
        "diff": {
            "source": "sas",
            "before": {"fingerprint": d.before.as_str()},
            "changes": changes(&d.diff),
        },
        "references": d.references.iter().map(|r| json!({
            "operation": r.operation,
            "cause": d.operation,
            "target": {"kind": "instance", "path": r.path, "concrete": r.concrete},
            "diff": {
                "source": "sas",
                "before": {"fingerprint": r.before.as_str()},
                "after": {"fingerprint": r.after.as_str()},
                "changes": changes(&r.diff),
            },
        })).collect::<Vec<_>>(),
        "stale_rules": [],
        "violated_instances": [],
    });
    println!("{out}");
}

/// 作成か更新の結果を出す。
pub fn print_change(c: &ChangeResult) {
    let mut diff = json!({
        "source": if c.outside { "outside" } else { "sas" },
        "after": {"fingerprint": c.after.as_str()},
        "changes": c.diff.changes().iter().map(|f| {
            let mut v = json!({"pointer": f.pointer(), "change": f.change().as_str()});
            if let Some(b) = f.before() {
                v["before"] = b.clone();
            }
            if f.before_unavailable() {
                v["before_unavailable"] = json!(true);
            }
            if let Some(a) = f.after() {
                v["after"] = a.clone();
            }
            v
        }).collect::<Vec<_>>(),
    });
    if let Some(b) = &c.before {
        diff["before"] = json!({"fingerprint": b.as_str()});
        if let Some(v) = &c.before_commit {
            diff["before"]["commit"] = json!(v);
        }
    }
    if c.outside && c.before.is_some() {
        diff["before_unavailable"] = json!(c.diff.before_unavailable());
    }
    let out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:operation-result:v1",
        "kind": "change",
        "parent_session": c.parent,
        "operation": c.operation,
        "result": if c.outside { "adopted" } else { c.written.as_str() },
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
