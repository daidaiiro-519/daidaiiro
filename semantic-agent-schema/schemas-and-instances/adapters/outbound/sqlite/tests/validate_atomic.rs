//! UC4 の最低保証「判定は1回の記録で入り、途中で止まっても半端な判定の行は残らない」：
//! 検証1件の判定を書く途中で置き場所が行を拒んだら、検証の行（validations）も rule ごとの結果（rule_results）も1行も残らない。

#![allow(clippy::unwrap_used)]

use outbound_sqlite::SqliteLog;
use validation_domain::values::{RuleRow, ValidateRow};

fn rule(id: &str, by: &str) -> RuleRow {
    RuleRow {
        rule: id.into(),
        pointer: "/goal".into(),
        by: by.into(),
        result: "verified".into(),
        reason: "理由".into(),
        reads: "sha256:r".into(),
    }
}

fn count(log: &SqliteLog, table: &str) -> i64 {
    log.connection()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

#[test]
fn a_judgment_that_fails_midway_leaves_no_rows() {
    let mut log = SqliteLog::in_memory().unwrap();
    // 1件目の判定は書ける。2件目は by が machine でも ai でもないので、表が拒む
    let row = ValidateRow {
        target_path: "u1.json".into(),
        hash: "sha256:a".into(),
        schema: None,
        results: vec![
            rule("goal-is-outcome", "ai"),
            rule("summary-matches", "someone"),
        ],
    };
    assert!(log.append_validate(None, &row).is_err());
    assert_eq!(count(&log, "validations"), 0);
    assert_eq!(count(&log, "rule_results"), 0);
    assert!(log.last_judgments("u1.json").unwrap().is_empty());

    // 拒まれたあとも、次の検証はふつうに入り、ID は飛ばない
    let ok = ValidateRow {
        results: vec![rule("goal-is-outcome", "ai")],
        ..row
    };
    assert_eq!(log.append_validate(None, &ok).unwrap(), "o-0001");
    assert_eq!(count(&log, "rule_results"), 1);
}
