//! 結果を JSON の1行にする（command-result.schema.json の validation ・ change ・ render ・ session ・ operation ・ search ・
//! error と、procedure.schema.json の手順）。

use recording_application::ports::{
    OperationDetail, OperationSummary, RuleRow, SessionRow, WriteRecord,
};
use recording_application::use_cases::history::Interleaved;
use rendering_application::use_cases::design::DESIGN_SYSTEM;
use rendering_application::use_cases::design::DesignValidation;
use rendering_application::use_cases::render::Rendered;
use search_application::use_cases::search::Found;
use serde_json::{Map, Value, json};
use validation_application::use_cases::validate::{Refused, Validation};
use validation_domain::values::{Counts, RuleResult, Verdict};
use writing_application::use_cases::write::{ChangeResult, Deletion};
use writing_domain::aggregates::ABSTRACT_VERSIONS;
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

/// 手順（procedure.schema.json）を1行で出す。
pub fn print_procedure(procedure: &Value) {
    println!("{procedure}");
}

/// sas init で登録したセッションを出す。
pub fn print_session(row: &SessionRow) {
    let out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
        "kind": "session",
        "id": row.id,
        "tool": row.tool,
        "folder": row.folder,
        "via": row.via,
    });
    println!("{out}");
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
                "hash": t.hash.as_str(),
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
        "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
        "kind": "validation",
        "session": v.session,
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

/// デザインシステムの検証の結果を出す（検証と同じ形。種類は design-system）。
pub fn print_design_validation(v: &DesignValidation) {
    let targets: Vec<Value> = v
        .targets
        .iter()
        .map(|t| {
            json!({
                "path": t.path,
                "kind": "design-system",
                "schema": "urn:semantic-agent-schema:published-language:design-system:v1",
                "hash": t.hash.as_str(),
                "operation": t.operation,
                "results": t.results.iter().map(rule_result).collect::<Vec<_>>(),
                "counts": counts(&t.counts),
            })
        })
        .collect();
    let out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
        "kind": "validation",
        "session": v.session,
        "operations": v.targets.iter().map(|t| t.operation.clone()).collect::<Vec<_>>(),
        "targets": targets,
        "counts": counts(&v.counts),
        "untracked": [],
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
    let mut v = json!({"operation": w.operation, "session": w.session, "kind": w.row.kind, "source": w.row.source});
    if let Some(i) = &w.row.intent {
        v["intent"] = json!(i);
    }
    v["at"] = json!(w.at);
    if let Some(b) = &w.row.before {
        v["before"] = json!({"hash": b});
    }
    v["after"] = json!({"hash": w.row.after});
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
            "session": w.session,
            "at": w.at,
            "after": {"hash": w.row.after},
        });
    }
    v
}

/// 始められなかった理由を出す。
pub fn print_refused(r: &Refused) {
    let mut out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
        "kind": "error",
        "code": r.code,
        "message": r.message,
    });
    if let Some(p) = &r.path {
        out["path"] = json!(p);
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
    if let Some(missing) = &r.missing {
        out["missing"] = json!(missing.as_slice());
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
        "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
        "kind": "change",
        "session": d.session,
        "operation": d.operation,
        "result": "deleted",
        "replayed": d.replayed,
        "target": {"kind": d.kind.as_str(), "path": d.path, "concrete": d.concrete},
        "diff": {
            "source": "sas",
            "before": {"hash": d.before.as_str()},
            "changes": changes(&d.diff),
        },
        "stale_rules": [],
        "violated_instances": [],
    });
    println!("{out}");
}

/// 作成か更新の結果を出す。
pub fn print_change(c: &ChangeResult) {
    println!("{}", change_json(c));
}

fn change_json(c: &ChangeResult) -> Value {
    let mut diff = json!({
        "source": if c.outside { "outside" } else { "sas" },
        "after": {"hash": c.after.as_str()},
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
        diff["before"] = json!({"hash": b.as_str()});
        if let Some(v) = &c.before_commit {
            diff["before"]["commit"] = json!(v);
        }
    }
    if c.outside && c.before.is_some() {
        diff["before_unavailable"] = json!(c.diff.before_unavailable());
    }
    let out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
        "kind": "change",
        "session": c.session,
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
    out
}

/// 描画の結果を1行で出す。
pub fn print_render(r: &Rendered) {
    let pages: Vec<Value> = r
        .pages
        .iter()
        .map(|p| json!({"target": p.target, "page_path": p.page_path, "views": p.views}))
        .collect();
    let out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
        "kind": "render",
        "design_system": {
            "name": r.design_system.name,
            "version": r.design_system.version,
            "hash": r.design_system.hash,
        },
        "pages": pages,
    });
    println!("{out}");
}

/// 記録にある操作1件の要約（sas log の一覧の1行）。
pub fn print_operation(o: &OperationSummary) {
    let mut out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
        "kind": "operation",
        "operation": o.operation,
        "at": o.at,
        "op": o.op,
        "path": o.path,
        "session": o.session.as_ref().map(|s| json!({"id": s.id, "tool": s.tool, "folder": s.folder})),
    });
    if let Some(i) = &o.intent {
        out["intent"] = json!(i);
    }
    if o.op == "validate" {
        let mut m = Map::new();
        for v in Verdict::ALL {
            let n = o
                .counts
                .iter()
                .find(|(name, _)| name == v.as_str())
                .map_or(0, |(_, n)| *n);
            m.insert(v.as_str().to_string(), json!(n));
        }
        out["counts"] = Value::Object(m);
    }
    println!("{out}");
}

/// 従う schema から、対象の種類（Concrete ・ インスタンス ・ デザインシステム）を決める。
fn kind_of(schema: &str) -> &'static str {
    if ABSTRACT_VERSIONS.contains(&schema) {
        "concrete"
    } else if schema == DESIGN_SYSTEM {
        "design-system"
    } else {
        "instance"
    }
}

/// 記録の rule の結果1件を、検証の結果と同じ形にする。
fn rule_row(r: &RuleRow) -> Value {
    json!({"rule": r.rule, "pointer": r.pointer, "by": r.by, "result": r.result, "reason": r.reason})
}

/// 記録にある操作1件の中身（sas log --operation）。書いたとき ・ 検証したときと同じ change ・ validation の形で出し、
/// 記録した日時 at と意図 intent を足す。記録にあるのは差分と判定だけなので、stale_rules と violated_instances は空。
pub fn print_operation_detail(d: &OperationDetail) {
    let out = match d {
        OperationDetail::Change {
            operation,
            at,
            session,
            row,
        } => {
            // 記録の欄の値は JSON の文字列で持つので、読み直して値にする
            let value = |t: &Option<String>| {
                t.as_deref()
                    .and_then(|t| serde_json::from_str::<Value>(t).ok())
            };
            let outside = row.source == "outside";
            let changes: Vec<Value> = row
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
            let mut diff = json!({"source": row.source});
            if let Some(b) = &row.before {
                diff["before"] = json!({"hash": b});
            }
            if let Some(a) = &row.after {
                diff["after"] = json!({"hash": a});
            }
            diff["changes"] = json!(changes);
            if outside && row.kind == "update" {
                diff["before_unavailable"] = json!(row.fields.iter().any(|f| f.before_unavailable));
            }
            let result = match (outside, row.kind.as_str()) {
                (true, _) => "adopted",
                (false, "create") => "created",
                (false, "delete") => "deleted",
                _ => "updated",
            };
            let schema = row.schema.clone().unwrap_or_default();
            let mut out = json!({
                "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
                "kind": "change",
                "session": session,
                "operation": operation,
                "at": at,
                "result": result,
                "replayed": false,
                "target": {"kind": kind_of(&schema), "path": row.target_path, "concrete": schema},
                "diff": diff,
                "stale_rules": [],
                "violated_instances": [],
            });
            if let Some(i) = &row.intent {
                out["intent"] = json!(i);
            }
            out
        }
        OperationDetail::Validate {
            operation,
            at,
            session,
            row,
        } => {
            let mut m = Map::new();
            for v in Verdict::ALL {
                let n = row
                    .results
                    .iter()
                    .filter(|r| r.result == v.as_str())
                    .count();
                m.insert(v.as_str().to_string(), json!(n));
            }
            let counts = Value::Object(m);
            let schema = row.schema.clone().unwrap_or_default();
            json!({
                "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
                "kind": "validation",
                "session": session,
                "at": at,
                "operations": [operation],
                "targets": [{
                    "path": row.target_path,
                    "kind": kind_of(&schema),
                    "schema": schema,
                    "hash": row.hash,
                    "operation": operation,
                    "results": row.results.iter().map(rule_row).collect::<Vec<_>>(),
                    "counts": counts,
                }],
                "counts": counts,
                "untracked": [],
            })
        }
    };
    println!("{out}");
}

/// 検索の結果を1行で出す（構造で絞った一致か、確かめた意味の答えと出どころ）。
pub fn print_search(f: &Found) {
    let mut out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
        "kind": "search",
        "range": f.range.iter().map(|(p, h)| json!({"path": p, "hash": h})).collect::<Vec<_>>(),
    });
    if let Some(a) = &f.answer {
        out["answer"] = json!(a);
        out["sources"] = f
            .found
            .iter()
            .map(|m| json!({"path": m.path, "pointer": m.pointer, "value": m.value}))
            .collect();
    } else {
        out["matches"] = f
            .found
            .iter()
            .map(|m| json!({"path": m.path, "pointer": m.pointer, "value": m.value}))
            .collect();
    }
    println!("{out}");
}
