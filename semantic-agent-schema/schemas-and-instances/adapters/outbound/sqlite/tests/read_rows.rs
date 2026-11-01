//! ステップ9 の記録を読む（UC7）：変更と検証を新しい順に並べ、経路 ・ セッション ・ フォルダ ・ 時刻 ・ 件数で絞る（かつ）。
//! 1件の中身は、変更なら変わった欄、検証なら rule ごとの結果と一緒に読める。無い ID は None。読んでも行は増えない。

#![allow(clippy::unwrap_used, clippy::panic)]

use outbound_sqlite::SqliteLog;
use recording_application::ports::{OperationDetail, OperationFilter, OperationReader};
use validation_domain::values::{RuleRow, ValidateRow};
use writing_domain::values::{Caller, ChangeRow, FieldRow, Via};

fn change(path: &str) -> ChangeRow {
    ChangeRow {
        kind: "create".into(),
        source: "sas".into(),
        target_path: path.into(),
        request_id: None,
        before: None,
        after: Some("sha256:b".into()),
        intent: Some("足す".into()),
        schema: Some("urn:example:u:v1".into()),
        fields: vec![FieldRow {
            pointer: "/goal".into(),
            change: "add".into(),
            before: None,
            after: Some("\"結果\"".into()),
            before_unavailable: false,
        }],
    }
}

fn validated(path: &str) -> ValidateRow {
    ValidateRow {
        target_path: path.into(),
        hash: "sha256:b".into(),
        schema: Some("urn:example:u:v1".into()),
        results: vec![
            RuleRow {
                rule: "json-schema".into(),
                pointer: String::new(),
                by: "machine".into(),
                result: "verified".into(),
                reason: "形が従う".into(),
                reads: "sha256:r".into(),
            },
            RuleRow {
                rule: "goal-is-outcome".into(),
                pointer: "/goal".into(),
                by: "ai".into(),
                result: "unsupported".into(),
                reason: "判定を渡されていない".into(),
                reads: "sha256:r".into(),
            },
        ],
    }
}

fn session(log: &mut SqliteLog, id: &str, folder: &str) -> Option<String> {
    log.session(&Caller::Session {
        id: id.into(),
        tool: "claude-code".into(),
        folder: folder.into(),
        via: Via::Env,
    })
    .unwrap()
}

/// a.json を A（/w/a）が作り、b.json を B（/w/b）が作って、人が a.json を検証する。
fn filled() -> SqliteLog {
    let mut log = SqliteLog::in_memory().unwrap();
    let a = session(&mut log, "A", "/w/a");
    let b = session(&mut log, "B", "/w/b");
    log.append_change(a.as_deref(), &change("a.json")).unwrap();
    log.append_change(b.as_deref(), &change("b.json")).unwrap();
    log.append_validate(None, &validated("a.json")).unwrap();
    log
}

fn ids(log: &SqliteLog, filter: &OperationFilter) -> Vec<String> {
    log.operations(filter)
        .unwrap()
        .into_iter()
        .map(|o| o.operation)
        .collect()
}

#[test]
fn operations_are_listed_newest_first_with_their_session() {
    let log = filled();
    let all = log.operations(&OperationFilter::default()).unwrap();
    let ops: Vec<(&str, &str)> = all
        .iter()
        .map(|o| (o.operation.as_str(), o.op.as_str()))
        .collect();
    assert_eq!(
        ops,
        [
            ("o-0003", "validate"),
            ("o-0002", "create"),
            ("o-0001", "create")
        ]
    );
    assert_eq!(all[0].session, None);
    assert_eq!(
        all[0].counts,
        [("unsupported".to_string(), 1), ("verified".to_string(), 1)]
    );
    assert_eq!(
        all[1].session.as_ref().map(|s| s.folder.as_str()),
        Some("/w/b")
    );
    assert_eq!(all[2].intent.as_deref(), Some("足す"));
}

#[test]
fn filters_are_combined() {
    let log = filled();
    let by = |f: OperationFilter| ids(&log, &f);
    assert_eq!(
        by(OperationFilter {
            paths: vec!["a.json".into()],
            ..OperationFilter::default()
        }),
        ["o-0003", "o-0001"]
    );
    assert_eq!(
        by(OperationFilter {
            by: Some("B".into()),
            ..OperationFilter::default()
        }),
        ["o-0002"]
    );
    assert_eq!(
        by(OperationFilter {
            folder: Some("/w/a".into()),
            ..OperationFilter::default()
        }),
        ["o-0001"]
    );
    assert_eq!(
        by(OperationFilter {
            paths: vec!["a.json".into()],
            by: Some("B".into()),
            ..OperationFilter::default()
        }),
        Vec::<String>::new()
    );
    assert_eq!(
        by(OperationFilter {
            limit: Some(1),
            ..OperationFilter::default()
        }),
        ["o-0003"]
    );
    assert_eq!(
        by(OperationFilter {
            since: Some("9999-01-01".into()),
            ..OperationFilter::default()
        }),
        Vec::<String>::new()
    );
    assert_eq!(
        by(OperationFilter {
            since: Some("2000-01-01".into()),
            ..OperationFilter::default()
        })
        .len(),
        3
    );
}

#[test]
fn one_operation_is_read_with_its_fields_or_rule_results() {
    let log = filled();
    let Some(OperationDetail::Change { session, row, .. }) = log.operation("o-0001").unwrap()
    else {
        panic!("変更として読めること");
    };
    assert_eq!(session.as_deref(), Some("A"));
    assert_eq!(row.schema.as_deref(), Some("urn:example:u:v1"));
    assert_eq!(row.fields[0].pointer, "/goal");
    let Some(OperationDetail::Validate { session, row, .. }) = log.operation("o-0003").unwrap()
    else {
        panic!("検証として読めること");
    };
    assert_eq!(session, None);
    assert_eq!(row.results.len(), 2);
    assert_eq!(row.results[1].rule, "goal-is-outcome");
    assert_eq!(log.operation("o-0099").unwrap(), None);
}

#[test]
fn reading_adds_no_rows() {
    let log = filled();
    let count = |log: &SqliteLog| -> i64 {
        log.connection()
            .query_row(
                "SELECT (SELECT count(*) FROM sessions) + (SELECT count(*) FROM changes) \
                 + (SELECT count(*) FROM validations)",
                [],
                |r| r.get(0),
            )
            .unwrap()
    };
    let before = count(&log);
    log.operations(&OperationFilter::default()).unwrap();
    log.operation("o-0001").unwrap();
    log.operation("o-0099").unwrap();
    assert_eq!(count(&log), before);
}
