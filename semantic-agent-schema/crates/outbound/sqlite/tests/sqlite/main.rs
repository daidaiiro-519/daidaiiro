//! 結合テスト：手元の DB に変更の記録と意図の判定を書いて読み戻す

use semantic_agent_schema_application::ports::change_records::ChangeRecords;
use semantic_agent_schema_application::ports::judgments::Judgments;
use semantic_agent_schema_application::semantic_agent_schema_change::aggregates::change_record::{
    ChangeRecord, Item, Writer,
};
use semantic_agent_schema_application::semantic_agent_schema_change::aggregates::judgment::IntentJudgment;
use semantic_agent_schema_application::semantic_agent_schema_change::values::change_id::ChangeId;
use semantic_agent_schema_application::semantic_agent_schema_change::values::verdict::Verdict;
use semantic_agent_schema_application::semantic_agent_schema_declaration::values::by::By;
use semantic_agent_schema_application::semantic_agent_schema_declaration::values::target::Target;
use semantic_agent_schema_sqlite::change_store::LocalDb;

fn db(name: &str) -> Result<LocalDb, String> {
    let p = std::env::temp_dir().join(format!("sas-sqlite-{name}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&p);
    LocalDb::open(&p)
}

fn target() -> Target {
    Target::Instance {
        id: "r-1".into(),
        kind: "urn:semantic-agent-schema:t:record:v1".into(),
    }
}

fn record(n: u128) -> Result<ChangeRecord, String> {
    let item = Item::new(target(), Some("/why".into()))?;
    ChangeRecord::new(
        ChangeId::from_parts(1, n),
        "0199",
        "理由を書き直す",
        Writer::AiAgent,
        vec![item],
        None,
    )
}

/// traces: U13:事後
#[test]
fn record_is_found_by_its_target() {
    let db = db("touching").unwrap();
    db.add(&record(1).unwrap(), None, None).unwrap();
    assert_eq!(db.touching(&target()).unwrap(), vec![record(1).unwrap()]);
}

/// traces: U5:失敗時
#[test]
fn same_id_twice_is_refused() {
    let db = db("twice").unwrap();
    db.add(&record(2).unwrap(), None, None).unwrap();
    assert!(db.add(&record(2).unwrap(), None, None).is_err());
}

/// traces: U16:事後
#[test]
fn later_intent_judgment_replaces_the_earlier() {
    let db = db("judge").unwrap();
    db.add(&record(3).unwrap(), None, None).unwrap();
    let id = record(3).unwrap().id().clone();
    db.put_intent(
        &IntentJudgment::new(id.clone(), By::Human, Verdict::Violated, "意味", "違う").unwrap(),
    )
    .unwrap();
    db.put_intent(
        &IntentJudgment::new(id.clone(), By::Human, Verdict::Verified, "意味", "合う").unwrap(),
    )
    .unwrap();
    assert!(db.intent_of(&id).unwrap().unwrap().approved_by_human());
}

/// traces: I35
#[test]
fn record_goes_into_only_one_summary() {
    let db = db("summary").unwrap();
    db.add(&record(4).unwrap(), None, None).unwrap();
    let id = record(4).unwrap().id().clone();
    db.summarized(std::slice::from_ref(&id), "s-1").unwrap();
    assert_eq!(
        (
            db.pending(None).unwrap().len(),
            db.summarized(&[id], "s-2").is_err()
        ),
        (0, true)
    );
}

/// traces: I32
#[test]
fn written_record_is_not_rewritten() {
    let p = std::env::temp_dir().join(format!("sas-sqlite-rewrite-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&p);
    let db = LocalDb::open(&p).unwrap();
    db.add(&record(5).unwrap(), None, None).unwrap();
    let conn = rusqlite::Connection::open(&p).unwrap();
    let id = record(5).unwrap().id().as_str().to_string();
    assert!(
        conn.execute(
            "UPDATE changes SET intent = '別の意図' WHERE id = ?1",
            [&id]
        )
        .is_err()
            && conn
                .execute(
                    "UPDATE change_items SET pointer = '/x' WHERE change_id = ?1",
                    [&id]
                )
                .is_err()
    );
}

/// traces: I35
#[test]
fn pending_is_only_of_the_same_workspace() {
    let db = db("workspace").unwrap();
    db.add(&record(6).unwrap(), Some("worktrees/a"), None)
        .unwrap();
    db.add(&record(7).unwrap(), Some("worktrees/b"), None)
        .unwrap();
    let ids: Vec<_> = db
        .pending(Some("worktrees/a"))
        .unwrap()
        .iter()
        .map(|r| r.id().clone())
        .collect();
    assert_eq!(ids, vec![record(6).unwrap().id().clone()]);
}
