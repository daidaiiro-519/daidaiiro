//! 変更のまとめ（記録から機械的に作る）

use crate::aggregates::change_record::ChangeRecord;
use crate::aggregates::judgment::IntentJudgment;
use crate::values::change_id::ChangeId;
use crate::values::file_state::FileState;
use crate::values::verdict::Verdict;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

/// 基盤が書くファイル（まとめと判定の記録）は突き合わせから外す（不変条件 I31）
const BASE_WRITTEN: [&str; 2] = [
    ".semantic-agent-schema/changes/",
    ".semantic-agent-schema/judgments/",
];

fn escape(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

/// 前後で値が変わった場所（JSON Pointer）。オブジェクトは中へたどり、ほかは値ごと比べる
fn changed_pointers(before: &Value, after: &Value, at: &str, out: &mut Vec<String>) {
    match (before, after) {
        (Value::Object(b), Value::Object(a)) => {
            let mut keys: Vec<&String> = b.keys().chain(a.keys()).collect();
            keys.sort();
            keys.dedup();
            for k in keys {
                let here = format!("{at}/{}", escape(k));
                match (b.get(k), a.get(k)) {
                    (Some(x), Some(y)) => changed_pointers(x, y, &here, out),
                    _ => out.push(here),
                }
            }
        }
        (x, y) if x != y => out.push(at.to_string()),
        _ => {}
    }
}

/// 場所 p が、記録した欄 field に入るか（field が空ならファイル全体）
fn covers(field: Option<&str>, p: &str) -> bool {
    match field {
        None | Some("") => true,
        Some(f) => p == f || p.starts_with(&format!("{f}/")) || f.starts_with(&format!("{p}/")),
    }
}

/// 1つのファイルの、変わった場所（JSON Pointer）・ 前の値 ・ 後の値。JSON として読めなければ None
type Diff = Option<(Vec<String>, Value, Value)>;

/// JSON でないファイルの行の差（前後の同じ行を除いた真ん中を、- と + で並べる）
fn line_changes(before: &str, after: &str) -> Vec<Value> {
    let b: Vec<&str> = before.lines().collect();
    let a: Vec<&str> = after.lines().collect();
    let head = b.iter().zip(&a).take_while(|(x, y)| x == y).count();
    let tail = b[head..]
        .iter()
        .rev()
        .zip(a[head..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let mut out: Vec<Value> = Vec::new();
    out.extend(
        b[head..b.len() - tail]
            .iter()
            .map(|t| json!({"mark": "-", "text": t})),
    );
    out.extend(
        a[head..a.len() - tail]
            .iter()
            .map(|t| json!({"mark": "+", "text": t})),
    );
    out
}

/// 1つのファイルの前後から、変わった場所を出す。JSON として読めなければ None
fn diff_of(f: &FileState) -> Diff {
    let parse = |t: Option<&str>| match t {
        None => Some(Value::Null),
        Some(s) => serde_json::from_str::<Value>(s).ok(),
    };
    let (b, a) = (parse(f.before())?, parse(f.after())?);
    let mut out = Vec::new();
    changed_pointers(&b, &a, "", &mut out);
    if out == [""] {
        out = vec![String::new()];
    }
    Some((out, b, a))
}

/// 変更のまとめ：まだまとめに入っていない変更の記録と、前の姿からの変わったファイルを両方向で突き合わせて作る（AI は書かない）。
/// 例外は種類ごとの件数と中身で先頭に出す。書いたブランチがもう無い記録（moved：記録の id と元のブランチ）は「ブランチが無くなった記録」として出す。
/// 不変条件 I31 ・ I35。返すのは、まとめと、入った記録の id
pub fn change_summary(
    id: &ChangeId,
    range: (&str, &str),
    records: &[(ChangeRecord, Option<IntentJudgment>)],
    files: &[FileState],
    moved: &[(ChangeId, String)],
) -> (Value, Vec<ChangeId>) {
    let files: Vec<&FileState> = files
        .iter()
        .filter(|f| !BASE_WRITTEN.iter().any(|b| f.path().starts_with(b)))
        .collect();
    let mut exceptions: BTreeMap<&str, Vec<Value>> = BTreeMap::new();
    let mut unreadable: Vec<String> = Vec::new();
    for (change_id, branch) in moved {
        exceptions
            .entry("branch-gone")
            .or_default()
            .push(json!({"change_id": change_id.as_str(), "branch": branch}));
    }
    let diffs: Vec<(&FileState, Diff)> = files.iter().map(|f| (*f, diff_of(f))).collect();
    // 記録に無い書き換え ・ 人が決める項目の書き換え
    for (f, d) in &diffs {
        let mine: Vec<&(ChangeRecord, Option<IntentJudgment>)> = records
            .iter()
            .filter(|(r, _)| r.items().iter().any(|i| i.target() == f.target()))
            .collect();
        let fields = |r: &ChangeRecord| -> Vec<Option<String>> {
            r.items()
                .iter()
                .filter(|i| i.target() == f.target())
                .map(|i| i.field().map(str::to_string))
                .collect()
        };
        let Some((pointers, _, _)) = d else {
            if f.after().is_some_and(|t| t.contains('\u{fffd}')) {
                unreadable.push(f.path().to_string());
            }
            if mine.is_empty() {
                exceptions
                    .entry("unrecorded")
                    .or_default()
                    .push(json!({"target": f.target().to_json(), "path": f.path()}));
            }
            continue;
        };
        for p in pointers {
            if !mine
                .iter()
                .any(|(r, _)| fields(r).iter().any(|fl| covers(fl.as_deref(), p)))
            {
                exceptions
                    .entry("unrecorded")
                    .or_default()
                    .push(json!({"target": f.target().to_json(), "path": f.path(), "at": p}));
            }
            for decided in f.decided() {
                if covers(Some(decided), p) {
                    let approved = mine.iter().any(|(r, j)| {
                        j.as_ref().is_some_and(IntentJudgment::approved_by_human)
                            && fields(r).iter().any(|fl| covers(fl.as_deref(), decided))
                    });
                    if !approved {
                        exceptions
                            .entry("human-field-changed")
                            .or_default()
                            .push(json!({"target": f.target().to_json(), "at": decided}));
                    }
                }
            }
        }
    }
    // 書き換えの無い記録 ・ 意図が合わない ・ 目的 → 意図 → 対象
    let mut purposes: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for (r, j) in records {
        if j.as_ref().is_some_and(|j| j.result() == Verdict::Violated) {
            exceptions
                .entry("intent-mismatch")
                .or_default()
                .push(json!({"change_id": r.id().as_str(), "intent": r.intent()}));
        }
        let mut targets: Vec<Value> = Vec::new();
        for item in r.items() {
            let found = diffs.iter().find(|(f, _)| f.target() == item.target());
            let changed_here = match found {
                Some((_, Some((ps, _, _)))) => ps.iter().any(|p| covers(item.field(), p)),
                Some((_, None)) => true,
                None => false,
            };
            if !changed_here {
                exceptions.entry("no-change").or_default().push(json!({"change_id": r.id().as_str(), "target": item.target().to_json(), "at": item.field()}));
                continue;
            }
            let shown = match found {
                Some((_, Some((ps, b, a)))) => {
                    let fields: Vec<Value> = ps
                        .iter()
                        .filter(|p| covers(item.field(), p))
                        .map(|p| json!({"at": p, "before": b.pointer(p).cloned().unwrap_or(Value::Null), "after": a.pointer(p).cloned().unwrap_or(Value::Null)}))
                        .collect();
                    json!({"target": item.target().to_json(), "kind": "fields", "fields": fields})
                }
                Some((f, None)) => {
                    json!({"target": item.target().to_json(), "kind": "lines", "lines": line_changes(f.before().unwrap_or_default(), f.after().unwrap_or_default())})
                }
                None => Value::Null,
            };
            targets.push(shown);
        }
        let judgment = j.as_ref().map_or("none", |j| j.result().as_str());
        let mut intent = json!({"change_id": r.id().as_str(), "intent": r.intent(), "judgment": judgment, "targets": targets});
        if let Some(j) = j {
            intent["meaning"] =
                json!({"text": j.meaning(), "reason": j.reason(), "by": j.by().as_str()});
        }
        purposes
            .entry(r.purpose_ref().to_string())
            .or_default()
            .push(intent);
    }
    let exceptions: Vec<Value> = exceptions
        .into_iter()
        .map(|(t, items)| json!({"type": t, "count": items.len(), "items": items}))
        .collect();
    let purposes: Vec<Value> = purposes
        .into_iter()
        .map(|(p, intents)| json!({"purpose_ref": p, "intents": intents}))
        .collect();
    let mut doc = Map::new();
    doc.insert(
        "$schema".to_string(),
        json!("urn:semantic-agent-schema:base:change-summary:v1"),
    );
    doc.insert("kind".to_string(), json!("change-summary"));
    doc.insert("id".to_string(), json!(id.as_str()));
    doc.insert(
        "range".to_string(),
        json!({"before": range.0, "after": range.1}),
    );
    doc.insert("exceptions".to_string(), json!(exceptions));
    doc.insert("purposes".to_string(), json!(purposes));
    doc.insert("unreadable".to_string(), json!(unreadable));
    (
        Value::Object(doc),
        records.iter().map(|(r, _)| r.id().clone()).collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::change_summary;
    use crate::aggregates::change_record::{ChangeRecord, Item, Writer};
    use crate::aggregates::judgment::IntentJudgment;
    use crate::values::change_id::ChangeId;
    use crate::values::file_state::FileState;
    use crate::values::verdict::Verdict;
    use semantic_agent_schema_declaration::values::by::By;
    use semantic_agent_schema_declaration::values::target::Target;
    use serde_json::{Value, json};

    fn target() -> Target {
        Target::Instance {
            id: "r-1".into(),
            kind: "urn:semantic-agent-schema:t:record:v1".into(),
        }
    }

    fn file(before: Value, after: Value) -> FileState {
        FileState::new(
            "t/r-1.json",
            target(),
            Some(before.to_string()),
            Some(after.to_string()),
            vec!["/status".into()],
        )
    }

    fn record(n: u128, field: &str) -> ChangeRecord {
        ChangeRecord::new(
            ChangeId::from_parts(1, n),
            "0199",
            "意図",
            Writer::AiAgent,
            vec![Item::new(target(), Some(field.into())).unwrap()],
            None,
        )
        .unwrap()
    }

    fn types(v: &Value) -> Vec<String> {
        v["exceptions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["type"].as_str().unwrap().to_string())
            .collect()
    }

    fn summarize(records: &[(ChangeRecord, Option<IntentJudgment>)], files: &[FileState]) -> Value {
        change_summary(&ChangeId::from_parts(9, 9), ("a", "b"), records, files, &[]).0
    }

    /// traces: U25:事後
    #[test]
    fn recorded_change_has_no_exception_and_shows_before_and_after() {
        let s = summarize(
            &[(record(1, "/why"), None)],
            &[file(json!({"why": "a"}), json!({"why": "b"}))],
        );
        assert_eq!(
            (
                types(&s).len(),
                s["purposes"][0]["intents"][0]["targets"][0]["fields"][0]["after"].clone()
            ),
            (0, json!("b"))
        );
    }

    /// traces: I31
    #[test]
    fn change_without_record_is_unrecorded() {
        assert_eq!(
            types(&summarize(
                &[],
                &[file(json!({"why": "a"}), json!({"why": "b"}))]
            )),
            vec!["unrecorded"]
        );
    }

    /// traces: I31
    #[test]
    fn record_without_change_is_no_change() {
        assert_eq!(
            types(&summarize(&[(record(1, "/why"), None)], &[])),
            vec!["no-change"]
        );
    }

    /// traces: U25:失敗時
    #[test]
    fn violated_intent_is_intent_mismatch() {
        let j = IntentJudgment::new(
            ChangeId::from_parts(1, 1),
            By::Human,
            Verdict::Violated,
            "意味",
            "違う",
        )
        .unwrap();
        let s = summarize(
            &[(record(1, "/why"), Some(j))],
            &[file(json!({"why": "a"}), json!({"why": "b"}))],
        );
        assert_eq!(types(&s), vec!["intent-mismatch"]);
    }

    /// traces: I21
    #[test]
    fn human_field_changed_without_approval_is_an_exception() {
        let s = summarize(
            &[(record(1, "/status"), None)],
            &[file(
                json!({"status": "proposed"}),
                json!({"status": "accepted"}),
            )],
        );
        assert_eq!(types(&s), vec!["human-field-changed"]);
    }

    /// traces: I21
    #[test]
    fn human_field_changed_with_approval_is_not_an_exception() {
        let j = IntentJudgment::new(
            ChangeId::from_parts(1, 1),
            By::Human,
            Verdict::Verified,
            "意味",
            "確かめた",
        )
        .unwrap();
        let s = summarize(
            &[(record(1, "/status"), Some(j))],
            &[file(
                json!({"status": "proposed"}),
                json!({"status": "accepted"}),
            )],
        );
        assert!(types(&s).is_empty());
    }

    /// traces: I35
    #[test]
    fn record_of_a_gone_branch_is_an_exception() {
        let moved = [(ChangeId::from_parts(1, 1), "old".to_string())];
        let s = change_summary(
            &ChangeId::from_parts(9, 9),
            ("a", "b"),
            &[(record(1, "/why"), None)],
            &[file(json!({"why": "a"}), json!({"why": "b"}))],
            &moved,
        )
        .0;
        assert_eq!(types(&s), vec!["branch-gone"]);
    }

    /// traces: I31
    #[test]
    fn base_written_files_are_left_out() {
        let f = FileState::new(
            ".semantic-agent-schema/changes/x.json",
            Target::Path(".semantic-agent-schema/changes/x.json".into()),
            None,
            Some("{}".into()),
            vec![],
        );
        assert!(types(&summarize(&[], &[f])).is_empty());
    }
}
