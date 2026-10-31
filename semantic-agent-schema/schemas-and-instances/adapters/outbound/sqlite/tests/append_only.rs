//! 表の決まり：追記だけ ・ 閉じたセッションに足さない ・ by = human は unverified だけ ・ 検証の前後の指紋は同じ。
//! あわせて、bundled の SQLite の版が 3.38.0 以上であること（ステップ1 の照合の実地の証拠）。

#![allow(clippy::unwrap_used)]

use schemas_and_instances_adapters_outbound_sqlite::SqliteLog;
use schemas_and_instances_operation_recording_application::ports::{
    Caller, OperationLog, RuleRow, ValidateRow,
};

fn row(by: &str, result: &str) -> ValidateRow {
    ValidateRow {
        target_kind: "instance".into(),
        target_path: "u1.json".into(),
        before: "sha256:a".into(),
        after: "sha256:a".into(),
        results: vec![RuleRow {
            rule: "r".into(),
            pointer: "/goal".into(),
            by: by.into(),
            result: result.into(),
            reason: "理由".into(),
        }],
    }
}

fn rejects(log: &SqliteLog, sql: &str) {
    assert!(
        log.connection().execute(sql, []).is_err(),
        "通ってしまった: {sql}"
    );
}

#[test]
fn bundled_sqlite_is_new_enough() {
    let log = SqliteLog::in_memory().unwrap();
    let text: String = log
        .connection()
        .query_row("SELECT sqlite_version()", [], |r| r.get(0))
        .unwrap();
    let parts: Vec<u32> = text.split('.').map(|p| p.parse().unwrap()).collect();
    assert!(parts[0] > 3 || (parts[0] == 3 && parts[1] >= 38), "{text}");
}

#[test]
fn ids_are_sequential_and_sessions_are_reused() {
    let mut log = SqliteLog::in_memory().unwrap();
    let h = log.caller_session(&Caller::Human).unwrap();
    let t = log
        .caller_session(&Caller::Tool {
            tool_ref: "x".into(),
        })
        .unwrap();
    assert_eq!((h.as_str(), t.as_str()), ("s-0001", "s-0002"));
    assert_eq!(log.caller_session(&Caller::Human).unwrap(), "s-0001");
    assert_eq!(
        log.append_validate(&t, &row("machine", "verified"))
            .unwrap(),
        "o-0001"
    );
    assert_eq!(
        log.append_validate(&h, &row("human", "unverified"))
            .unwrap(),
        "o-0002"
    );
}

#[test]
fn records_cannot_be_changed_or_removed() {
    let mut log = SqliteLog::in_memory().unwrap();
    let s = log.caller_session(&Caller::Human).unwrap();
    log.append_validate(&s, &row("machine", "verified"))
        .unwrap();
    rejects(&log, "UPDATE operations SET target_path = 'x'");
    rejects(&log, "DELETE FROM operations");
    rejects(&log, "UPDATE rule_results SET result = 'violated'");
    rejects(&log, "DELETE FROM rule_results");
    rejects(&log, "DELETE FROM sessions");
}

#[test]
fn rows_against_the_rules_are_refused() {
    let mut log = SqliteLog::in_memory().unwrap();
    let s = log.caller_session(&Caller::Human).unwrap();
    assert!(log.append_validate(&s, &row("human", "verified")).is_err());
    let mut changed = row("machine", "verified");
    changed.after = "sha256:b".into();
    assert!(log.append_validate(&s, &changed).is_err());
    // 拒んだ操作の行は残らない（rule の結果と一緒に1つのトランザクションで足す）
    let n: i64 = log
        .connection()
        .query_row("SELECT count(*) FROM operations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 0);
}

#[test]
fn closed_session_takes_no_more_operations() {
    let mut log = SqliteLog::in_memory().unwrap();
    let s = log
        .caller_session(&Caller::Tool {
            tool_ref: "x".into(),
        })
        .unwrap();
    log.connection()
        .execute("UPDATE sessions SET ended_at = '2026-10-31T00:00:00Z'", [])
        .unwrap();
    rejects(&log, "UPDATE sessions SET ended_at = NULL");
    assert!(
        log.append_validate(&s, &row("machine", "verified"))
            .is_err()
    );
    assert!(
        log.caller_session(&Caller::Tool {
            tool_ref: "x".into()
        })
        .is_err()
    );
}
