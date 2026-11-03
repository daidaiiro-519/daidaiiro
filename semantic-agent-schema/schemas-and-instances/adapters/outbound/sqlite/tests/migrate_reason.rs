//! 移行 0006：変更の記録の intent を reason に名前を変える。版5 までの DB を開くと、行と決まり（CHECK）を保ったまま新しい版になる。

#![allow(clippy::unwrap_used)]

use outbound_sqlite::SqliteLog;
use recording_application::ports::{OperationDetail, OperationFilter, OperationReader};
use rusqlite::Connection;
use writing_domain::values::ChangeRow;

/// 版5 までの表（移行 0001〜0005 だけを当てた DB）。
const UP_TO_5: [&str; 5] = [
    include_str!("../migrations/0001-validate.sql"),
    include_str!("../migrations/0002-change.sql"),
    include_str!("../migrations/0003-adopt.sql"),
    include_str!("../migrations/0004-delete.sql"),
    include_str!("../migrations/0005-read.sql"),
];

#[test]
fn rows_written_as_intent_are_read_as_reason() {
    let dir = std::env::temp_dir().join(format!("sas-migrate-reason-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("sas.db");
    {
        let conn = Connection::open(&path).unwrap();
        for (i, sql) in UP_TO_5.iter().enumerate() {
            conn.execute_batch(&format!(
                "BEGIN; {sql} PRAGMA user_version = {}; COMMIT;",
                i + 1
            ))
            .unwrap();
        }
        conn.execute(
            "INSERT INTO changes (id, at, kind, target_path, before_hash, after_hash, intent, source) \
             VALUES ('o-0001', '2026-11-01T00:00:00Z', 'update', 'u1.json', 'sha256:a', 'sha256:b', '目標を直す', 'sas')",
            [],
        )
        .unwrap();
    }
    let mut log = SqliteLog::open(&path).unwrap();
    let version: i64 = log
        .connection()
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 8); // いまの版まで進む
    // 版5 で書いた理由が、名前を変えた欄から読める
    let all = log.operations(&OperationFilter::default()).unwrap();
    assert_eq!(all[0].reason.as_deref(), Some("目標を直す"));
    let Some(OperationDetail::Change { row, .. }) = log.operation("o-0001").unwrap() else {
        unreachable!("変更の行が読めること")
    };
    assert_eq!(row.reason.as_deref(), Some("目標を直す"));
    // 欄の決まりも移る：削除は理由を持つ
    let no_reason = ChangeRow {
        kind: "delete".into(),
        source: "sas".into(),
        target_path: "u1.json".into(),
        request_id: None,
        before: Some("sha256:b".into()),
        after: None,
        reason: None,
        schema: None,
        fields: Vec::new(),
    };
    assert!(log.append_change(None, &no_reason).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}
