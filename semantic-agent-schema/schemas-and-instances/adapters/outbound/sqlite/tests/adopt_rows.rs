//! ステップ3 の表の決まり：取り込み（source = outside）は作成か更新だけ、更新の取り込みは意図を持ち、
//! 事前は対象を最後に書いた操作の事後、作成の取り込みは記録の無い対象だけ。前の値が取れなかった欄は取り込んだ更新だけ。
//! 判定は rule ごとに、その rule を最後に判定した検証の行を引く。

#![allow(clippy::unwrap_used)]

use schemas_and_instances_adapters_outbound_sqlite::SqliteLog;
use schemas_and_instances_operation_recording_application::ports::{
    Caller, ChangeRow, FieldRow, OperationLog, RuleRow, ValidateRow,
};

fn change(kind: &str, source: &str, before: Option<&str>, after: &str) -> ChangeRow {
    ChangeRow {
        kind: kind.into(),
        source: source.into(),
        target_kind: "instance".into(),
        target_path: "u1.json".into(),
        concrete: "urn:x:v1".into(),
        request_id: None,
        before: before.map(Into::into),
        after: Some(after.into()),
        refusal: None,
        intent: Some("エディターで直した".into()),
        fields: Vec::new(),
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
        target_kind: "instance".into(),
        target_path: "u1.json".into(),
        before: "sha256:a".into(),
        after: "sha256:a".into(),
        results: rules
            .iter()
            .map(|(rule, reads)| RuleRow {
                rule: (*rule).into(),
                pointer: "/goal".into(),
                by: "machine".into(),
                result: "verified".into(),
                reason: "x".into(),
                reads: Some((*reads).into()),
            })
            .collect(),
    }
}

#[test]
fn adopted_update_follows_the_last_write_and_keeps_the_source() {
    let mut log = SqliteLog::in_memory().unwrap();
    let s = log.caller_session(&Caller::Human).unwrap();
    log.append_change(&s, &change("create", "outside", None, "sha256:a"))
        .unwrap();
    // 事前が最後の事後と違う取り込みは断る
    assert!(
        log.append_change(
            &s,
            &change("update", "outside", Some("sha256:x"), "sha256:b")
        )
        .is_err()
    );
    let mut ok = change("update", "outside", Some("sha256:a"), "sha256:b");
    ok.fields = vec![unavailable()];
    log.append_change(&s, &ok).unwrap();
    let w = log.writes("u1.json").unwrap();
    assert_eq!(
        w.iter().map(|w| w.row.source.as_str()).collect::<Vec<_>>(),
        ["outside", "outside"]
    );
    assert!(w[1].row.fields[0].before_unavailable);
    // 記録がある対象を作成として取り込まない
    assert!(
        log.append_change(&s, &change("create", "outside", None, "sha256:c"))
            .is_err()
    );
}

#[test]
fn adopted_update_needs_an_intent_and_only_changes_adopt() {
    let mut log = SqliteLog::in_memory().unwrap();
    let s = log.caller_session(&Caller::Human).unwrap();
    log.append_change(&s, &change("create", "sas", None, "sha256:a"))
        .unwrap();
    let mut no_intent = change("update", "outside", Some("sha256:a"), "sha256:b");
    no_intent.intent = None;
    assert!(log.append_change(&s, &no_intent).is_err());
    assert!(
        log.connection()
            .execute(
                "INSERT INTO operations (id, session_id, seq, kind, target_kind, target_path, before_hash, after_hash, at, source) \
                 VALUES ('o-9', 's-0001', 2, 'validate', 'instance', 'u1.json', 'h', 'h', 't', 'outside')",
                [],
            )
            .is_err()
    );
}

#[test]
fn only_an_adopted_update_lacks_previous_values() {
    let mut log = SqliteLog::in_memory().unwrap();
    let s = log.caller_session(&Caller::Human).unwrap();
    log.append_change(&s, &change("create", "sas", None, "sha256:a"))
        .unwrap();
    let mut by_sas = change("update", "sas", Some("sha256:a"), "sha256:b");
    by_sas.fields = vec![unavailable()];
    assert!(log.append_change(&s, &by_sas).is_err());
    let mut with_before = unavailable();
    with_before.before = Some("\"x\"".into());
    let mut adopted = change("update", "outside", Some("sha256:a"), "sha256:b");
    adopted.fields = vec![with_before];
    assert!(
        log.append_change(&s, &adopted).is_err(),
        "前の値が無いのに before を持つ"
    );
}

#[test]
fn judgments_are_the_latest_per_rule() {
    let mut log = SqliteLog::in_memory().unwrap();
    let s = log.caller_session(&Caller::Human).unwrap();
    let first = log
        .append_validate(&s, &judged(&[("a", "sha256:1"), ("b", "sha256:2")]))
        .unwrap();
    let second = log
        .append_validate(&s, &judged(&[("a", "sha256:3")]))
        .unwrap();
    let last = log.last_judgments("u1.json").unwrap();
    let got: Vec<(&str, &str, Option<&str>)> = last
        .iter()
        .map(|j| {
            (
                j.operation.as_str(),
                j.rule.rule.as_str(),
                j.rule.reads.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        got,
        [
            (first.as_str(), "b", Some("sha256:2")),
            (second.as_str(), "a", Some("sha256:3"))
        ]
    );
}
