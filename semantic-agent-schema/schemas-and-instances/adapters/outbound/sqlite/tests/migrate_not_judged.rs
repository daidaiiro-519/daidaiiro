//! 移行 0007：判定の区分の値 unsupported を not-judged にする（rule_results を作り直して値を移す）。
//! 版6 までの DB を開くと、行と決まり（CHECK ・ 追記だけの trigger）を保ったまま版7 になる。

#![allow(clippy::unwrap_used)]

use outbound_sqlite::SqliteLog;
use recording_application::ports::{OperationFilter, OperationReader};
use rusqlite::Connection;

/// 版6 までの表（移行 0001〜0006 だけを当てた DB）。
const UP_TO_6: [&str; 6] = [
    include_str!("../migrations/0001-validate.sql"),
    include_str!("../migrations/0002-change.sql"),
    include_str!("../migrations/0003-adopt.sql"),
    include_str!("../migrations/0004-delete.sql"),
    include_str!("../migrations/0005-read.sql"),
    include_str!("../migrations/0006-reason.sql"),
];

#[test]
fn unsupported_rows_are_read_as_not_judged_and_the_rules_of_the_table_remain() {
    let dir = std::env::temp_dir().join(format!("sas-migrate-not-judged-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("sas.db");
    {
        let conn = Connection::open(&path).unwrap();
        for (i, sql) in UP_TO_6.iter().enumerate() {
            conn.execute_batch(&format!(
                "BEGIN; {sql} PRAGMA user_version = {}; COMMIT;",
                i + 1
            ))
            .unwrap();
        }
        conn.execute_batch(
            "INSERT INTO validations (id, at, target_path, hash) \
             VALUES ('o-0001', '2026-11-01T00:00:00Z', 'u1.json', 'sha256:a'); \
             INSERT INTO rule_results (validation_id, ord, rule_id, pointer, by, result, reason, reads_hash) \
             VALUES ('o-0001', 0, 'json-schema', '', 'machine', 'verified', '形が従う', 'sha256:r'), \
                    ('o-0001', 1, 'goal-is-outcome', '/goal', 'ai', 'unsupported', '判定を渡されていない', 'sha256:g');",
        )
        .unwrap();
    }
    let log = SqliteLog::open(&path).unwrap();
    let version: i64 = log
        .connection()
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 8); // いまの版まで進む
    // 行は2つとも残り、値は not-judged で読める
    let judgments = log.last_judgments("u1.json").unwrap();
    let results: Vec<(&str, &str)> = judgments
        .iter()
        .map(|j| (j.rule.rule.as_str(), j.rule.result.as_str()))
        .collect();
    assert_eq!(
        results,
        [
            ("json-schema", "verified"),
            ("goal-is-outcome", "not-judged")
        ]
    );
    let all = log.operations(&OperationFilter::default()).unwrap();
    assert_eq!(
        all[0].counts,
        [("not-judged".to_string(), 1), ("verified".to_string(), 1)]
    );
    // CHECK は新しい値の一覧で効き、追記だけの決まりも付け直してある
    let insert = |result: &str| {
        log.connection().execute(
            "INSERT INTO rule_results (validation_id, ord, rule_id, pointer, by, result, reason, reads_hash) \
             VALUES ('o-0001', 2, 'r', '', 'ai', ?1, 'x', 'sha256:x')",
            [result],
        )
    };
    assert!(insert("unsupported").is_err());
    assert!(
        log.connection()
            .execute("UPDATE rule_results SET reason = 'x'", [])
            .is_err()
    );
    assert!(
        log.connection()
            .execute("DELETE FROM rule_results", [])
            .is_err()
    );
    assert!(insert("not-judged").is_ok());
    let _ = std::fs::remove_dir_all(&dir);
}
