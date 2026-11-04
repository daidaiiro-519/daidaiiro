//! 結果を JSON の1行にする（command-result.schema.json の validation ・ change ・ render ・ session ・ operation ・ search ・
//! error と、procedure.schema.json の手順）。

use recording_application::ports::{OperationDetail, OperationSummary, SessionRow};
use rendering_application::use_cases::design::DESIGN_SYSTEM;
use rendering_application::use_cases::design::DesignValidation;
use rendering_application::use_cases::render::Rendered;
use search_application::use_cases::search::Found;
use serde_json::{Map, Value, json};
use validation_application::use_cases::validate::Validation;
use validation_domain::values::{Counts, RuleResult, RuleRow, Verdict};
use writing_application::use_cases::write::{
    Blocked, ChangeResult, Checked, Deletion, Interleaved, Written,
};
use writing_domain::aggregates::ABSTRACT_VERSIONS;
use writing_domain::values::{Diff, TargetKind, WriteRecord};

/// 操作を始められなかった ・ 書かなかった理由（出力の kind = error の形）。断る理由の型は各モジュールが持ち、
/// 入口が出力の形へ移す。入口が自分で断るとき（使い方の誤り ・ 置き場所を開けない）もこの形で返す。
// 各モジュールの Refused と同じく、まれな欄は Box にして Result の Err を小さく保つ（clippy の result_large_err）
#[allow(clippy::box_collection)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Refused {
    /// 決まった名前（各モジュールの Refused の code と、入口の usage ・ invalid_input ・ store ・ validator ・ log ・ not_found など）
    pub code: &'static str,
    /// 対象の経路（あれば）
    pub path: Option<Box<str>>,
    /// 理由
    pub message: String,
    /// ハッシュが違って断ったとき（conflict）の、間に入った変更（書くモジュール）
    pub interleaved: Option<Box<Interleaved>>,
    /// 次にすることの案内
    pub hint: Option<Box<String>>,
    /// 削除を止めた理由になったもの（書くモジュール）
    pub blocked: Option<Box<Vec<Blocked>>>,
    /// デザインシステムに無かった x-view の種類（描画のモジュール）
    pub missing: Option<Box<Vec<String>>>,
    /// 同じ $id を持つファイルの経路（duplicate_schema）
    pub paths: Option<Box<Vec<String>>>,
}

impl From<validation_application::use_cases::validate::Refused> for Refused {
    fn from(r: validation_application::use_cases::validate::Refused) -> Self {
        Self {
            code: r.code,
            path: r.path,
            message: r.message,
            hint: r.hint,
            paths: r.paths,
            ..Self::default()
        }
    }
}

impl From<writing_application::use_cases::write::Refused> for Refused {
    fn from(r: writing_application::use_cases::write::Refused) -> Self {
        Self {
            code: r.code,
            path: r.path,
            message: r.message,
            interleaved: r.interleaved,
            hint: r.hint,
            blocked: r.blocked,
            missing: None,
            paths: r.paths,
        }
    }
}

impl From<rendering_application::use_cases::render::Refused> for Refused {
    fn from(r: rendering_application::use_cases::render::Refused) -> Self {
        Self {
            code: r.code,
            path: r.path,
            message: r.message,
            hint: r.hint,
            missing: r.missing,
            paths: r.paths,
            ..Self::default()
        }
    }
}

impl From<search_application::use_cases::search::Refused> for Refused {
    fn from(r: search_application::use_cases::search::Refused) -> Self {
        Self {
            code: r.code,
            path: r.path,
            message: r.message,
            hint: r.hint,
            paths: r.paths,
            ..Self::default()
        }
    }
}

/// 記録を読み書きできなかったことを、断った理由にする。
pub fn log_refused(e: String) -> Refused {
    Refused {
        code: "log",
        message: e,
        ..Refused::default()
    }
}

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

/// semantic-agent-schema init で登録したセッションを出す。
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
            json!({
                "path": t.path,
                "kind": t.kind.as_str(),
                "schema": t.schema,
                "hash": t.hash.as_str(),
                "operation": t.operation,
                "results": t.results.iter().map(rule_result).collect::<Vec<_>>(),
                "counts": counts(&t.counts),
            })
        })
        .collect();
    let mut out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
        "kind": "validation",
        "session": v.session,
        "operations": v.targets.iter().map(|t| t.operation.clone()).collect::<Vec<_>>(),
        "targets": targets,
        "counts": counts(&v.counts),
        "unrecorded": v.unrecorded.iter().map(|u| json!({
            "path": u.path,
            "last_operation": u.last_operation,
            "expected": u.expected,
            "actual": u.actual,
        })).collect::<Vec<_>>(),
    });
    if let Some(h) = &v.hint {
        out["hint"] = json!(h);
    }
    println!("{out}");
}

/// デザインシステムの検証の結果を出す（検証と同じ形。種類は design-system）。記録しないので、操作の ID と打ったセッションを持たない。
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
                "results": t.results.iter().map(rule_result).collect::<Vec<_>>(),
                "counts": counts(&t.counts),
            })
        })
        .collect();
    let out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
        "kind": "validation",
        "operations": [],
        "targets": targets,
        "counts": counts(&v.counts),
        "unrecorded": [],
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
    if let Some(i) = &w.row.reason {
        v["reason"] = json!(i);
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
        "unrecorded": i.unrecorded,
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
    if let Some(paths) = &r.paths {
        out["paths"] = json!(paths.as_slice());
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

/// Concrete を直す ・ 消すときに、従うインスタンスかを見た文書。
fn checked(c: &[Checked]) -> Value {
    c.iter()
        .map(|c| {
            json!({"path": c.path, "hash": c.hash.as_str(), "from": c.from, "follows": c.follows})
        })
        .collect()
}

/// 削除の結果を出す。diff は消す前の状態（根の欄ごとの remove）で、事後を持たない。
pub fn print_deletion(d: &Deletion) {
    let mut out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
        "kind": "change",
        "session": d.session,
        "operation": d.operation,
        "result": "deleted",
        "target": {"kind": d.kind.as_str(), "path": d.path, "schema": d.schema},
        "diff": {
            "source": "semantic-agent-schema",
            "before": {"hash": d.before.as_str()},
            "changes": changes(&d.diff),
        },
        "violated_instances": [],
    });
    if d.kind == TargetKind::Concrete {
        out["checked"] = checked(&d.checked);
    }
    println!("{out}");
}

/// 作成か更新の結果を出す。
pub fn print_change(c: &ChangeResult) {
    println!("{}", change_json(c));
}

fn change_json(c: &ChangeResult) -> Value {
    let mut diff = json!({
        "source": if c.outside { "outside" } else { "semantic-agent-schema" },
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
    let mut out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
        "kind": "change",
        "session": c.session,
        "operation": c.operation,
        "result": if c.outside { "adopted" } else { c.written.as_str() },
        "target": {"kind": c.kind.as_str(), "path": c.path, "schema": c.schema},
        "diff": diff,
        "violated_instances": c.violated.iter().map(|v| json!({
            "path": v.path,
            "results": v.results.iter().map(rule_result).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    });
    if c.kind == TargetKind::Concrete && c.written == Written::Updated {
        out["checked"] = checked(&c.checked);
    }
    out
}

/// 描画の結果を1行で出す。
pub fn print_render(r: &Rendered) {
    let pages: Vec<Value> = r
        .pages
        .iter()
        .map(|p| json!({"target": p.target, "page_path": p.page_path, "view": p.view}))
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

/// 記録にある操作1件の要約（semantic-agent-schema log の一覧の1行）。
pub fn print_operation(o: &OperationSummary) {
    let mut out = json!({
        "$schema": "urn:semantic-agent-schema:published-language:command-result:v1",
        "kind": "operation",
        "operation": o.operation,
        "at": o.at,
        "op": o.op,
        "path": o.path,
        "session": o.session.as_ref().map(|s| json!({"id": s.id, "tool": s.tool, "folder": s.folder, "via": s.via})),
    });
    if let Some(i) = &o.reason {
        out["reason"] = json!(i);
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

/// 記録にある操作1件の中身（semantic-agent-schema log --operation）。書いたとき ・ 検証したときと同じ change ・ validation の形で出し、
/// 記録した日時 at と理由 reason を足す。記録にあるのは差分と判定だけなので、violated_instances は空。
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
                "target": {"kind": kind_of(&schema), "path": row.target_path, "schema": schema},
                "diff": diff,
                "violated_instances": [],
            });
            if let Some(i) = &row.reason {
                out["reason"] = json!(i);
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
                "unrecorded": [],
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
