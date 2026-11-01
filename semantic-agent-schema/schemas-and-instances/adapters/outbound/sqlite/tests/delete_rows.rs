//! ステップ4 の表の決まり：削除は事後を持たず、意図を持ち、変わった欄は事前の remove だけ。
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
        fields: Vec::new(),
        child: None,
    }
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
fn a_delete_has_an_intent() {
    let mut log = SqliteLog::in_memory().unwrap();
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
