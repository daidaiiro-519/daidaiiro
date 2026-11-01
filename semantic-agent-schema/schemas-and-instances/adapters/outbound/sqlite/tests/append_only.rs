//! 表の決まり：追記だけ ・ セッションの行は初めて見たときだけ ・ via は env か flag か init ・ by は machine か ai。
//! sas init が登録したセッションをフォルダから引け、SAS が決めるセッション ID は重ならない。
//! あわせて、bundled の SQLite の版が 3.38.0 以上であること（ステップ1 の照合の実地の証拠）。

#![allow(clippy::unwrap_used)]

use outbound_sqlite::SqliteLog;
use validation_domain::values::{RuleRow, ValidateRow};
use writing_domain::values::{Caller, Via};

fn row(by: &str, result: &str) -> ValidateRow {
    ValidateRow {
        target_path: "u1.json".into(),
        hash: "sha256:a".into(),
        schema: None,
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

fn session(id: &str, folder: &str, via: Via) -> Caller {
    Caller::Session {
        id: id.into(),
        tool: "claude-code".into(),
        folder: folder.into(),
        via,
    }
}

fn parent(id: &str) -> Caller {
    session(id, "/w", Via::Env)
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
fn a_session_is_written_once_and_a_human_has_no_row() {
    let mut log = SqliteLog::in_memory().unwrap();
    assert_eq!(log.session(&Caller::Human).unwrap(), None);
    let p = log.session(&parent("c5c5")).unwrap();
    assert_eq!(p.as_deref(), Some("c5c5"));
    assert_eq!(log.session(&parent("c5c5")).unwrap(), p);
    assert_eq!(count(&log, "sessions"), 1);
    // 同じ ID を別の入り方で見ても、初めの行のまま
    log.session(&session("c5c5", "/other", Via::Init)).unwrap();
    let seen = log.session_row("c5c5").unwrap().unwrap();
    assert_eq!(
        (seen.tool.as_str(), seen.folder.as_str(), seen.via.as_str()),
        ("claude-code", "/w", "env")
    );
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
    let p = log.session(&parent("c5c5")).unwrap();
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
    // via は env ・ flag ・ init だけ ・ 検証の session_id は sessions の行だけを指す
    rejects(
        &log,
        "INSERT INTO sessions (id, tool, folder, via, first_seen) VALUES ('q', 'claude-code', '/w', 'child', 't')",
    );
    rejects(
        &log,
        "INSERT INTO validations (id, at, target_path, hash, session_id) VALUES ('o-9', 't', 'a', 'h', 'nobody')",
    );
}

#[test]
fn the_last_session_registered_by_init_is_found_by_folder() {
    let mut log = SqliteLog::in_memory().unwrap();
    assert_eq!(log.initialized("/w").unwrap(), None);
    log.session(&session("a", "/w", Via::Init)).unwrap();
    log.session(&session("e", "/w", Via::Env)).unwrap();
    log.session(&session("b", "/w", Via::Init)).unwrap();
    log.session(&session("c", "/other", Via::Init)).unwrap();
    assert_eq!(
        log.initialized("/w").unwrap().map(|r| r.id),
        Some("b".to_string())
    );
}

#[test]
fn new_session_ids_do_not_collide() {
    let log = SqliteLog::in_memory().unwrap();
    let ids: std::collections::BTreeSet<String> = (0..100).map(|_| log.new_session_id()).collect();
    assert_eq!(ids.len(), 100);
    assert!(ids.iter().all(|i| i.starts_with("sas-") && i.len() == 36));
}
