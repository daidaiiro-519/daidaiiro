//! ステップ2 の表の決まり：作成 ・ 更新の行と変わった欄、断った操作は対象を変えない、1つの依頼に操作は1回、
//! seq は前の次の番号、変わった欄は書いた作成 ・ 更新にだけ付く、判定は検証にだけ付く。
//! 対象を書いた操作は、断ったものと検証を除いて記録した順に読める。
//! あわせて、ステップ1 の DB を開くと行を残したまま（版3 まで）移すこと。

#![allow(clippy::unwrap_used)]

use schemas_and_instances_adapters_outbound_sqlite::SqliteLog;
use schemas_and_instances_operation_recording_application::ports::{
    Caller, ChangeRow, FieldRow, OperationLog, RuleRow, ValidateRow,
};

fn update(request_id: Option<&str>) -> ChangeRow {
    ChangeRow {
        kind: "update".into(),
        source: "sas".into(),
        target_kind: "instance".into(),
        target_path: "u1.json".into(),
        concrete: "urn:x:v1".into(),
        request_id: request_id.map(Into::into),
        before: Some("sha256:a".into()),
        after: Some("sha256:b".into()),
        refusal: None,
        intent: Some("目標を直す".into()),
        cause: None,
        fields: vec![FieldRow {
            pointer: "/goal".into(),
            change: "replace".into(),
            before: Some("\"x\"".into()),
            after: Some("null".into()),
            before_unavailable: false,
        }],
    }
}

fn validate_row() -> ValidateRow {
    ValidateRow {
        target_kind: "instance".into(),
        target_path: "u1.json".into(),
        before: "sha256:a".into(),
        after: "sha256:a".into(),
        results: vec![RuleRow {
            rule: "goal-is-outcome".into(),
            pointer: "/goal".into(),
            by: "human".into(),
            result: "unverified".into(),
            reason: "人が決める".into(),
            reads: Some("sha256:r".into()),
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
fn change_and_fields_are_kept_and_found_by_request() {
    let mut log = SqliteLog::in_memory().unwrap();
    let s = log.caller_session(&Caller::Human).unwrap();
    let o = log.append_change(&s, &update(Some("r-1"))).unwrap();
    let found = log.find_request("r-1").unwrap().unwrap();
    assert_eq!(
        (found.operation.as_str(), found.session.as_str()),
        (o.as_str(), s.as_str())
    );
    assert_eq!(found.row, update(Some("r-1")));
    assert!(log.find_request("r-2").unwrap().is_none());
    // 1つの依頼に操作は1回
    assert!(log.append_change(&s, &update(Some("r-1"))).is_err());
    rejects(&log, "UPDATE field_changes SET after = '1'");
    rejects(&log, "DELETE FROM field_changes");
}

#[test]
fn writes_are_written_changes_in_order() {
    let mut log = SqliteLog::in_memory().unwrap();
    let s = log.caller_session(&Caller::Human).unwrap();
    let first = log.append_change(&s, &update(None)).unwrap();
    log.append_validate(&s, &validate_row()).unwrap();
    let mut refused = update(None);
    refused.refusal = Some(("conflict".into(), "指紋が違う".into()));
    refused.after = refused.before.clone();
    refused.fields.clear();
    log.append_change(&s, &refused).unwrap();
    let mut second = update(None);
    second.before = Some("sha256:b".into());
    second.after = Some("sha256:c".into());
    second.intent = None;
    let second_id = log.append_change(&s, &second).unwrap();
    let writes = log.writes("u1.json").unwrap();
    let ids: Vec<&str> = writes.iter().map(|w| w.operation.as_str()).collect();
    assert_eq!(ids, [first.as_str(), second_id.as_str()]);
    assert_eq!(writes[0].row, update(None));
    assert_eq!(writes[0].session, s);
    assert!(!writes[0].at.is_empty());
    assert!(log.writes("u9.json").unwrap().is_empty());
}

#[test]
fn refused_change_keeps_target_and_has_no_fields() {
    let mut log = SqliteLog::in_memory().unwrap();
    let s = log.caller_session(&Caller::Human).unwrap();
    let mut refused = update(None);
    refused.refusal = Some(("conflict".into(), "指紋が違う".into()));
    assert!(
        log.append_change(&s, &refused).is_err(),
        "前後の指紋が違う断り"
    );
    refused.after = refused.before.clone();
    assert!(
        log.append_change(&s, &refused).is_err(),
        "断った操作に変わった欄"
    );
    refused.fields.clear();
    let o = log.append_change(&s, &refused).unwrap();
    let found: (String, Option<String>) = log
        .connection()
        .query_row(
            "SELECT refusal_code, after_hash FROM operations WHERE id = ?1",
            [&o],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(found, ("conflict".into(), Some("sha256:a".into())));
    let mut create = update(None);
    create.kind = "create".into();
    assert!(log.append_change(&s, &create).is_err(), "作成に事前の指紋");
    create.before = None;
    create.refusal = Some(("invalid_value".into(), "形".into()));
    create.after = None;
    create.fields.clear();
    assert!(
        log.append_change(&s, &create).is_ok(),
        "断った作成は事前も事後も無い"
    );
}

#[test]
fn seq_follows_and_rows_go_to_the_right_kind() {
    let mut log = SqliteLog::in_memory().unwrap();
    let s = log.caller_session(&Caller::Human).unwrap();
    let v = log.append_validate(&s, &validate_row()).unwrap();
    let c = log.append_change(&s, &update(None)).unwrap();
    rejects(
        &log,
        "INSERT INTO operations (id, session_id, seq, kind, target_kind, target_path, before_hash, after_hash, at) \
         VALUES ('o-9', 's-0001', 5, 'validate', 'instance', 'a', 'h', 'h', 't')",
    );
    rejects(
        &log,
        &format!(
            "INSERT INTO field_changes (operation_id, pointer, change, after) VALUES ('{v}', '/a', 'add', '1')"
        ),
    );
    rejects(
        &log,
        &format!(
            "INSERT INTO rule_results (operation_id, ord, rule_id, pointer, by, result, reason) \
             VALUES ('{c}', 0, 'r', '', 'machine', 'verified', 'x')"
        ),
    );
    let last = log.last_judgments("u1.json").unwrap();
    assert_eq!(last.len(), 1);
    assert_eq!(
        (last[0].operation.as_str(), last[0].rule.rule.as_str()),
        (v.as_str(), "goal-is-outcome")
    );
    assert!(log.last_judgments("u9.json").unwrap().is_empty());
}

#[test]
fn step1_database_is_moved_with_its_rows() {
    let dir = std::env::temp_dir().join(format!("sas-migrate-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("sas.db");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch(&format!(
            "BEGIN; {} PRAGMA user_version = 1; COMMIT;",
            include_str!("../migrations/0001-validate.sql")
        ))
        .unwrap();
        conn.execute_batch(
            "INSERT INTO sessions (id, actor, started_at) VALUES ('s-0001', 'human', 't'); \
             INSERT INTO operations (id, session_id, seq, kind, target_kind, target_path, before_hash, after_hash, at) \
             VALUES ('o-0001', 's-0001', 1, 'validate', 'instance', 'u1.json', 'h', 'h', 't'); \
             INSERT INTO rule_results (operation_id, ord, rule_id, pointer, by, result, reason) \
             VALUES ('o-0001', 0, 'r', '/goal', 'machine', 'verified', 'x');",
        )
        .unwrap();
    }
    let mut log = SqliteLog::open(&path).unwrap();
    let version: i64 = log
        .connection()
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 4);
    let last = log.last_judgments("u1.json").unwrap();
    assert_eq!(last.len(), 1);
    assert_eq!(
        last[0].rule.reads, None,
        "ステップ1 の判定は読んだ所の指紋を持たない"
    );
    let s = log.caller_session(&Caller::Human).unwrap();
    assert_eq!(s, "s-0001");
    assert_eq!(log.append_change(&s, &update(None)).unwrap(), "o-0002");
    rejects(&log, "DELETE FROM operations");
    drop(log);
    std::fs::remove_dir_all(&dir).unwrap();
}
