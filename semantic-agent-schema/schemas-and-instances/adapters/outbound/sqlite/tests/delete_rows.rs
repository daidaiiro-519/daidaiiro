//! ステップ4 の表の決まり：削除は事後を持たず、理由を持ち、変わった欄は事前の remove だけ。
//! 消すと記録のつながりが切れ、書いた操作の並び（writes）は、最後の削除より後だけを見る。

#![allow(clippy::unwrap_used)]

mod support;

use writing_domain::values::{ChangeRow, FieldRow};

fn row(kind: &str, path: &str, before: Option<&str>, after: Option<&str>) -> ChangeRow {
    ChangeRow {
        kind: kind.into(),
        source: "semantic-agent-schema".into(),
        target_path: path.into(),
        before: before.map(Into::into),
        after: after.map(Into::into),
        reason: Some("消す".into()),
        schema: None,
        fields: Vec::new(),
    }
}

#[test]
fn a_delete_has_no_after_and_records_only_removals() {
    let mut log = support::log();
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
fn a_delete_has_a_reason() {
    let mut log = support::log();
    let mut no_reason = row("delete", "u4.json", Some("sha256:f"), None);
    no_reason.reason = None;
    assert!(
        log.append_change(None, &no_reason).is_err(),
        "削除は理由を持つ"
    );
}

#[test]
fn writes_start_again_after_a_delete() {
    let mut log = support::log();
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
