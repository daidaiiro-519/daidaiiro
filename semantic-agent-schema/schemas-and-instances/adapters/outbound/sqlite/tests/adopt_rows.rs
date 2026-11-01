//! ステップ3 の表の決まり：取り込み（source = outside）は作成か更新だけで、更新の取り込みは意図を持つ。
//! 前の値が取れなかった欄は取り込んだ更新だけ。判定は rule ごとに、その rule を最後に判定した検証の行を引く。
//! 「取り込みの事前は記録のどれかの事後」「作成の取り込みは記録の無い対象だけ」は application が確かめる（DB はワークツリーで1つ）。

#![allow(clippy::unwrap_used)]

use outbound_sqlite::SqliteLog;
use recording_application::ports::{ChangeRow, FieldRow, OperationLog, RuleRow, ValidateRow};

fn change(kind: &str, source: &str, before: Option<&str>, after: Option<&str>) -> ChangeRow {
    ChangeRow {
        kind: kind.into(),
        source: source.into(),
        target_path: "u1.json".into(),
        request_id: None,
        before: before.map(Into::into),
        after: after.map(Into::into),
        intent: Some("エディターで直した".into()),
        fields: Vec::new(),
        child: None,
    }
}

fn unavailable() -> FieldRow {
    FieldRow {
        pointer: "/goal".into(),
        change: "replace".into(),
        before: None,
        after: Some("\"y\"".into()),
        before_unavailable: true,
    }
}

fn judged(rules: &[(&str, &str)]) -> ValidateRow {
    ValidateRow {
        child_session: None,
        target_path: "u1.json".into(),
        hash: "sha256:a".into(),
        results: rules
            .iter()
            .map(|(rule, reads)| RuleRow {
                rule: (*rule).into(),
                pointer: "/goal".into(),
                by: "machine".into(),
                result: "verified".into(),
                reason: "x".into(),
                reads: (*reads).into(),
            })
            .collect(),
    }
}

#[test]
fn adopted_changes_keep_the_source() {
    let mut log = SqliteLog::in_memory().unwrap();
    log.append_change(None, &change("create", "outside", None, Some("sha256:a")))
        .unwrap();
    let mut ok = change("update", "outside", Some("sha256:a"), Some("sha256:b"));
    ok.fields = vec![unavailable()];
    log.append_change(None, &ok).unwrap();
    let w = log.writes("u1.json").unwrap();
    assert_eq!(
        w.iter().map(|w| w.row.source.as_str()).collect::<Vec<_>>(),
        ["outside", "outside"]
    );
    assert!(w[1].row.fields[0].before_unavailable);
}

#[test]
fn adopted_update_needs_an_intent_and_a_delete_is_not_adopted() {
    let mut log = SqliteLog::in_memory().unwrap();
    log.append_change(None, &change("create", "sas", None, Some("sha256:a")))
        .unwrap();
    let mut no_intent = change("update", "outside", Some("sha256:a"), Some("sha256:b"));
    no_intent.intent = None;
    assert!(log.append_change(None, &no_intent).is_err());
    assert!(
        log.append_change(None, &change("delete", "outside", Some("sha256:a"), None))
            .is_err()
    );
}

#[test]
fn only_an_adopted_update_lacks_previous_values() {
    let mut log = SqliteLog::in_memory().unwrap();
    log.append_change(None, &change("create", "sas", None, Some("sha256:a")))
        .unwrap();
    let mut by_sas = change("update", "sas", Some("sha256:a"), Some("sha256:b"));
    by_sas.fields = vec![unavailable()];
    assert!(log.append_change(None, &by_sas).is_err());
    let mut with_before = unavailable();
    with_before.before = Some("\"x\"".into());
    let mut adopted = change("update", "outside", Some("sha256:a"), Some("sha256:b"));
    adopted.fields = vec![with_before];
    assert!(
        log.append_change(None, &adopted).is_err(),
        "前の値が無いのに before を持つ"
    );
}

#[test]
fn judgments_are_the_latest_per_rule() {
    let mut log = SqliteLog::in_memory().unwrap();
    let first = log
        .append_validate(None, &judged(&[("a", "sha256:1"), ("b", "sha256:2")]))
        .unwrap();
    let second = log
        .append_validate(None, &judged(&[("a", "sha256:3")]))
        .unwrap();
    let last = log.last_judgments("u1.json").unwrap();
    let got: Vec<(&str, &str, &str)> = last
        .iter()
        .map(|j| {
            (
                j.operation.as_str(),
                j.rule.rule.as_str(),
                j.rule.reads.as_str(),
            )
        })
        .collect();
    assert_eq!(
        got,
        [
            (first.as_str(), "b", "sha256:2"),
            (second.as_str(), "a", "sha256:3")
        ]
    );
}
