//! 消したあとの判定の引き方：消して作り直した文書に、消す前の判定を当てない（validate --stale が読む）。

#![allow(clippy::unwrap_used)]

use outbound_sqlite::SqliteLog;
use recording_application::ports::{ChangeRow, FieldRow, OperationLog, RuleRow, ValidateRow};

fn validate_row() -> ValidateRow {
    ValidateRow {
        target_path: "u1.json".into(),
        hash: "sha256:a".into(),
        results: vec![RuleRow {
            rule: "json-schema".into(),
            pointer: String::new(),
            by: "machine".into(),
            result: "verified".into(),
            reason: "形が従う".into(),
            reads: "sha256:r".into(),
        }],
    }
}

fn delete_row() -> ChangeRow {
    ChangeRow {
        kind: "delete".into(),
        source: "sas".into(),
        target_path: "u1.json".into(),
        request_id: None,
        before: Some("sha256:a".into()),
        after: None,
        intent: Some("消す".into()),
        fields: vec![FieldRow {
            pointer: "/id".into(),
            change: "remove".into(),
            before: Some("\"U1\"".into()),
            after: None,
            before_unavailable: false,
        }],
    }
}

#[test]
fn judgments_before_a_delete_are_not_returned() {
    let mut log = SqliteLog::in_memory().unwrap();
    log.append_validate(None, &validate_row()).unwrap();
    log.append_change(None, &delete_row()).unwrap();
    assert!(
        log.last_judgments("u1.json").unwrap().is_empty(),
        "消す前の判定を、作り直した文書に当ててしまう"
    );
    log.append_validate(None, &validate_row()).unwrap();
    assert_eq!(log.last_judgments("u1.json").unwrap().len(), 1);
}
