//! ステップ2 の表の決まり：書いた作成 ・ 更新の行と変わった欄、1つの依頼に変更は1回、作成は事前を持たない、
//! 変わった欄は変更にだけ付き、判定は検証にだけ付く（外部キーの先の表で決まる）。
//! 対象を書いた操作は、検証を除いて記録した順に読める。changes と validations は同じ採番から ID を振る。

#![allow(clippy::unwrap_used)]

use outbound_sqlite::SqliteLog;
use validation_domain::values::{RuleRow, ValidateRow};
use writing_domain::values::{Caller, ChangeRow, FieldRow, Via};

fn update(request_id: Option<&str>) -> ChangeRow {
    ChangeRow {
        kind: "update".into(),
        source: "sas".into(),
        target_path: "u1.json".into(),
        request_id: request_id.map(Into::into),
        before: Some("sha256:a".into()),
        after: Some("sha256:b".into()),
        reason: Some("目標を直す".into()),
        schema: None,
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
        target_path: "u1.json".into(),
        hash: "sha256:a".into(),
        schema: None,
        results: vec![RuleRow {
            rule: "goal-is-outcome".into(),
            pointer: "/goal".into(),
            by: "ai".into(),
            result: "unverified".into(),
            reason: "判定できない".into(),
            reads: "sha256:r".into(),
        }],
    }
}

fn rejects(log: &SqliteLog, sql: &str) {
    assert!(
        log.connection().execute(sql, []).is_err(),
        "通ってしまった: {sql}"
    );
}

fn session(log: &mut SqliteLog) -> Option<String> {
    log.session(&Caller::Session {
        id: "c5c5".into(),
        tool: "claude-code".into(),
        folder: "/w".into(),
        via: Via::Env,
    })
    .unwrap()
}

#[test]
fn change_and_fields_are_kept_and_found_by_request() {
    let mut log = SqliteLog::in_memory().unwrap();
    let p = session(&mut log);
    let o = log
        .append_change(p.as_deref(), &update(Some("r-1")))
        .unwrap();
    let found = log.find_request("r-1").unwrap().unwrap();
    assert_eq!(
        (found.operation.as_str(), found.session.clone()),
        (o.as_str(), p.clone())
    );
    assert_eq!(found.row, update(Some("r-1")));
    assert!(log.find_request("r-2").unwrap().is_none());
    // 1つの依頼に変更は1回
    assert!(
        log.append_change(p.as_deref(), &update(Some("r-1")))
            .is_err()
    );
    rejects(&log, "UPDATE changes SET reason = 'x'");
    rejects(&log, "DELETE FROM changes");
    rejects(&log, "UPDATE field_changes SET after = '1'");
    rejects(&log, "DELETE FROM field_changes");
}

#[test]
fn writes_are_written_changes_in_order() {
    let mut log = SqliteLog::in_memory().unwrap();
    let first = log.append_change(None, &update(None)).unwrap();
    log.append_validate(None, &validate_row()).unwrap();
    let mut second = update(None);
    second.before = Some("sha256:b".into());
    second.after = Some("sha256:c".into());
    second.reason = None;
    let second_id = log.append_change(None, &second).unwrap();
    assert_eq!(
        (first.as_str(), second_id.as_str()),
        ("o-0001", "o-0003"),
        "changes と validations は同じ採番"
    );
    let writes = log.writes("u1.json").unwrap();
    let ids: Vec<&str> = writes.iter().map(|w| w.operation.as_str()).collect();
    assert_eq!(ids, [first.as_str(), second_id.as_str()]);
    assert_eq!(writes[0].row, update(None));
    assert_eq!(writes[0].session, None);
    assert!(!writes[0].at.is_empty());
    assert!(log.writes("u9.json").unwrap().is_empty());
}

#[test]
fn a_create_has_no_before_and_an_update_has_one() {
    let mut log = SqliteLog::in_memory().unwrap();
    let mut create = update(None);
    create.kind = "create".into();
    assert!(
        log.append_change(None, &create).is_err(),
        "作成に事前のハッシュ"
    );
    create.before = None;
    create.fields[0].change = "add".into();
    create.fields[0].before = None;
    assert!(log.append_change(None, &create).is_ok());
    let mut no_before = update(None);
    no_before.before = None;
    assert!(
        log.append_change(None, &no_before).is_err(),
        "更新に事前のハッシュが無い"
    );
    let mut no_after = update(None);
    no_after.after = None;
    assert!(
        log.append_change(None, &no_after).is_err(),
        "更新に事後のハッシュが無い"
    );
}

#[test]
fn rows_go_to_the_right_table() {
    let mut log = SqliteLog::in_memory().unwrap();
    let v = log.append_validate(None, &validate_row()).unwrap();
    let c = log.append_change(None, &update(None)).unwrap();
    rejects(
        &log,
        &format!(
            "INSERT INTO field_changes (change_id, pointer, change, after) VALUES ('{v}', '/a', 'add', '1')"
        ),
    );
    rejects(
        &log,
        &format!(
            "INSERT INTO rule_results (validation_id, ord, rule_id, pointer, by, result, reason, reads_hash) \
             VALUES ('{c}', 0, 'r', '', 'machine', 'verified', 'x', 'h')"
        ),
    );
    let last = log.last_judgments("u1.json").unwrap();
    assert_eq!(last.len(), 1);
    assert_eq!(
        (last[0].operation.as_str(), last[0].rule.rule.as_str()),
        (v.as_str(), "goal-is-outcome")
    );
    assert!(log.last_judgments("u9.json").unwrap().is_empty());
    // UC7：変更と検証を記録した順に1つの並びで読める
    let all: Vec<(String, String)> = log
        .connection()
        .prepare("SELECT id, kind FROM operations ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        all,
        [(v, "validate".to_string()), (c, "update".to_string())]
    );
}

#[test]
fn a_database_in_a_new_place_is_made_empty() {
    let dir = std::env::temp_dir().join(format!("sas-open-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("sas").join("sas.db");
    let mut log = SqliteLog::open(&path).unwrap();
    let version: i64 = log
        .connection()
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 7);
    assert_eq!(log.append_change(None, &update(None)).unwrap(), "o-0001");
    drop(log);
    // 開き直しても行は残り、版は進まない
    let log = SqliteLog::open(&path).unwrap();
    assert_eq!(log.writes("u1.json").unwrap().len(), 1);
    drop(log);
    std::fs::remove_dir_all(&dir).unwrap();
}
