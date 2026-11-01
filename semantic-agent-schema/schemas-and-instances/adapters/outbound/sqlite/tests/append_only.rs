//! 表の決まり：追記だけ ・ 親セッションの行は初めて見たときだけ ・ 親セッションの欄は親セッションの行だけ ・ 子セッションの役割 ・ by = human は unverified だけ。
//! あわせて、bundled の SQLite の版が 3.38.0 以上であること（ステップ1 の照合の実地の証拠）。

#![allow(clippy::unwrap_used)]

use outbound_sqlite::SqliteLog;
use recording_application::ports::{Caller, OperationLog, RuleRow, ValidateRow};

fn row(by: &str, result: &str) -> ValidateRow {
    ValidateRow {
        child_session: None,
        target_path: "u1.json".into(),
        hash: "sha256:a".into(),
        results: vec![RuleRow {
            rule: "r".into(),
            pointer: "/goal".into(),
            by: by.into(),
            result: result.into(),
            reason: "理由".into(),
            reads: "sha256:r".into(),
        }],
    }
}

fn parent(id: &str) -> Caller {
    Caller::Parent {
        id: id.into(),
        tool: "claude-code".into(),
    }
}

fn rejects(log: &SqliteLog, sql: &str) {
    assert!(
        log.connection().execute(sql, []).is_err(),
        "通ってしまった: {sql}"
    );
}

fn count(log: &SqliteLog, table: &str) -> i64 {
    log.connection()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
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
fn a_parent_is_written_once_and_a_human_has_no_row() {
    let mut log = SqliteLog::in_memory().unwrap();
    assert_eq!(log.parent_session(&Caller::Human).unwrap(), None);
    let p = log.parent_session(&parent("c5c5")).unwrap();
    assert_eq!(p.as_deref(), Some("c5c5"));
    assert_eq!(log.parent_session(&parent("c5c5")).unwrap(), p);
    assert_eq!(count(&log, "sessions"), 1);
    let (role, tool): (String, String) = log
        .connection()
        .query_row(
            "SELECT role, tool FROM sessions WHERE id = 'c5c5'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((role.as_str(), tool.as_str()), ("caller", "claude-code"));
    assert_eq!(
        log.append_validate(p.as_deref(), &row("machine", "verified"))
            .unwrap(),
        "o-0001"
    );
    assert_eq!(
        log.append_validate(None, &row("ai", "unverified")).unwrap(),
        "o-0002"
    );
}

#[test]
fn records_cannot_be_changed_or_removed() {
    let mut log = SqliteLog::in_memory().unwrap();
    let p = log.parent_session(&parent("c5c5")).unwrap();
    log.append_validate(p.as_deref(), &row("machine", "verified"))
        .unwrap();
    rejects(&log, "UPDATE validations SET target_path = 'x'");
    rejects(&log, "DELETE FROM validations");
    rejects(&log, "UPDATE rule_results SET result = 'violated'");
    rejects(&log, "DELETE FROM rule_results");
    rejects(&log, "UPDATE sessions SET tool = 'codex'");
    rejects(&log, "DELETE FROM sessions");
}

#[test]
fn rows_against_the_rules_are_refused() {
    let mut log = SqliteLog::in_memory().unwrap();
    assert!(
        log.append_validate(None, &row("person", "verified"))
            .is_err()
    );
    // 拒んだ検証の行は残らない（rule の結果と一緒に1つのトランザクションで足す）
    assert_eq!(count(&log, "validations"), 0);
    // 親セッションの欄は親セッションの行だけを指し、検証は検証用の子セッションだけがする
    log.connection()
        .execute_batch(
            "INSERT INTO sessions (id, role, tool, first_seen) VALUES ('p', 'caller', 'claude-code', 't'); \
             INSERT INTO sessions (id, role, parent_id, tool, launch, first_seen) VALUES ('w', 'writer', 'p', 'claude-code', '{}', 't');",
        )
        .unwrap();
    rejects(
        &log,
        "INSERT INTO validations (id, at, target_path, hash, parent_session_id) VALUES ('o-9', 't', 'a', 'h', 'w')",
    );
    rejects(
        &log,
        "INSERT INTO validations (id, at, target_path, hash, parent_session_id, child_session_id) VALUES ('o-9', 't', 'a', 'h', 'p', 'w')",
    );
    // 子セッションの親セッションは親セッションの行だけ ・ 親セッションの行は起動の設定を持たない ・ 子セッションは起動の設定を持つ
    rejects(
        &log,
        "INSERT INTO sessions (id, role, parent_id, tool, launch, first_seen) VALUES ('c', 'checker', 'w', 'claude-code', '{}', 't')",
    );
    rejects(
        &log,
        "INSERT INTO sessions (id, role, tool, launch, first_seen) VALUES ('q', 'caller', 'claude-code', '{}', 't')",
    );
    rejects(
        &log,
        "INSERT INTO sessions (id, role, parent_id, tool, first_seen) VALUES ('c', 'checker', 'p', 'claude-code', 't')",
    );
    // 子セッションの ID を親セッションとして名乗れない
    assert!(log.parent_session(&parent("w")).is_err());
}
