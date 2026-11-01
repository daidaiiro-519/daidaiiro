//! ステップ4 の表の決まり：削除は事後を持たず、意図を持ち、変わった欄は事前の remove だけ。参照元の書き換えは、
//! 先に記録した削除を原因（cause_id）とする SAS の update で、依頼の ID を持たない。
//! 消すと記録のつながりが切れ、書いた操作の並び（writes）は、最後の削除より後だけを見る。

#![allow(clippy::unwrap_used)]

use outbound_sqlite::SqliteLog;
use recording_application::ports::{ChangeRow, FieldRow, OperationLog};

fn row(kind: &str, path: &str, before: Option<&str>, after: Option<&str>) -> ChangeRow {
    ChangeRow {
        kind: kind.into(),
        source: "sas".into(),
        target_path: path.into(),
        request_id: None,
        before: before.map(Into::into),
        after: after.map(Into::into),
        intent: Some("消す".into()),
        cause: None,
        fields: Vec::new(),
    }
}

fn removed(pointer: &str) -> FieldRow {
    FieldRow {
        pointer: pointer.into(),
        change: "remove".into(),
        before: Some("\"U1\"".into()),
        after: None,
        before_unavailable: false,
    }
}

#[test]
fn a_delete_and_the_rewrite_it_caused_are_recorded_together() {
    let mut log = SqliteLog::in_memory().unwrap();
    let mut d = row("delete", "u1.json", Some("sha256:a"), None);
    d.fields = vec![removed("/id")];
    let del = log.append_change(None, &d).unwrap();
    let mut u = row("update", "u2.json", Some("sha256:b"), Some("sha256:c"));
    u.cause = Some(del.clone());
    u.fields = vec![removed("/calls/0")];
    let rewrite = log.append_change(None, &u).unwrap();
    let caused = log.caused_by(&del).unwrap();
    assert_eq!(caused.len(), 1);
    assert_eq!(caused[0].operation, rewrite);
    assert_eq!(caused[0].row.cause.as_deref(), Some(del.as_str()));
}

#[test]
fn a_delete_has_no_after_and_records_only_removals() {
    let mut log = SqliteLog::in_memory().unwrap();
    assert!(
        log.append_change(
            None,
            &row("delete", "u1.json", Some("sha256:a"), Some("sha256:a"))
        )
        .is_err(),
        "消した操作は事後を持たない"
    );
    assert!(
        log.append_change(None, &row("delete", "u1.json", None, None))
            .is_err(),
        "消した操作は事前を持つ"
    );
    let mut d = row("delete", "u1.json", Some("sha256:a"), None);
    d.fields = vec![FieldRow {
        pointer: "/goal".into(),
        change: "replace".into(),
        before: Some("\"x\"".into()),
        after: Some("\"y\"".into()),
        before_unavailable: false,
    }];
    assert!(
        log.append_change(None, &d).is_err(),
        "削除の欄は remove だけ"
    );
}

#[test]
fn a_cause_must_be_a_delete() {
    let mut log = SqliteLog::in_memory().unwrap();
    let del = log
        .append_change(None, &row("delete", "u1.json", Some("sha256:a"), None))
        .unwrap();
    let created = log
        .append_change(None, &row("create", "u3.json", None, Some("sha256:d")))
        .unwrap();
    let mut not_delete = row("update", "u3.json", Some("sha256:d"), Some("sha256:e"));
    not_delete.cause = Some(created);
    assert!(
        log.append_change(None, &not_delete).is_err(),
        "原因は削除だけ"
    );
    let mut with_request = row("update", "u2.json", Some("sha256:b"), Some("sha256:c"));
    with_request.cause = Some(del.clone());
    with_request.request_id = Some("r-1".into());
    assert!(
        log.append_change(None, &with_request).is_err(),
        "原因を持つ書き換えは依頼の ID を持たない"
    );
    let mut adopted = row("update", "u2.json", Some("sha256:b"), Some("sha256:c"));
    adopted.cause = Some(del);
    adopted.source = "outside".into();
    assert!(
        log.append_change(None, &adopted).is_err(),
        "原因を持つのは SAS が書いた update だけ"
    );
    let mut no_intent = row("delete", "u4.json", Some("sha256:f"), None);
    no_intent.intent = None;
    assert!(
        log.append_change(None, &no_intent).is_err(),
        "削除は意図を持つ"
    );
}

#[test]
fn writes_start_again_after_a_delete() {
    let mut log = SqliteLog::in_memory().unwrap();
    log.append_change(None, &row("create", "u1.json", None, Some("sha256:a")))
        .unwrap();
    assert_eq!(log.writes("u1.json").unwrap().len(), 1);
    log.append_change(None, &row("delete", "u1.json", Some("sha256:a"), None))
        .unwrap();
    assert!(
        log.writes("u1.json").unwrap().is_empty(),
        "消したあとは書いた記録が無い対象になる"
    );
    let mut adopted = row("create", "u1.json", None, Some("sha256:b"));
    adopted.source = "outside".into();
    log.append_change(None, &adopted).unwrap();
    assert_eq!(log.writes("u1.json").unwrap().len(), 1);
}
