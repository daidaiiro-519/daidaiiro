//! 結合テスト：使う場面を、ポートの代役で動かす

mod doubles;

use doubles::change_records::FakeChangeRecords;
use doubles::contracts::FakeContracts;
use doubles::judgments::FakeJudgments;
use semantic_agent_schema_application::semantic_agent_schema_declaration::values::etag::ETag;
use semantic_agent_schema_application::use_cases::{
    Ports, create, history, judge, patch, record, validate,
};
use serde_json::json;

const PATH: &str = "o/record.schema.json";

/// 代役の組（契約とインスタンスは同じ置き場所を共有する）
#[derive(Default)]
struct World {
    contracts: FakeContracts,
    records: FakeChangeRecords,
    judgments: FakeJudgments,
}

impl World {
    fn ports(&self) -> Ports<'_> {
        Ports {
            contracts: &self.contracts,
            instances: &self.contracts,
            records: &self.records,
            judgments: &self.judgments,
        }
    }
}

fn created() -> World {
    let w = World::default();
    create::create(
        &w.ports(),
        PATH,
        "o",
        "record",
        &json!({"title": "記録", "description": "決定の記録"}),
    );
    w
}

/// traces: U1:失敗時
#[test]
fn creating_over_an_existing_file_is_refused() {
    let w = created();
    let contracts = &w.contracts;
    let before = contracts.files.borrow().clone();
    let outcome = create::create(
        &w.ports(),
        PATH,
        "o",
        "record",
        &json!({"title": "記録", "description": "決定の記録"}),
    );
    assert_eq!(
        (
            outcome.findings[0].result.as_str(),
            contracts.files.borrow().clone()
        ),
        ("refused", before)
    );
}

/// traces: U1:失敗時
#[test]
fn patch_changing_the_id_is_refused_and_leaves_the_file() {
    let w = created();
    let contracts = &w.contracts;
    let before = contracts.files.borrow().clone();
    let outcome = patch::patch(
        &w.ports(),
        PATH,
        &json!([{"op": "replace", "path": "/$id", "value": "x"}]),
        None,
    );
    assert_eq!(
        (
            outcome.findings[0].result.as_str(),
            contracts.files.borrow().clone()
        ),
        ("refused", before)
    );
}

/// traces: U23:失敗時
#[test]
fn patch_introducing_a_violation_is_returned_and_leaves_the_file() {
    let w = created();
    let contracts = &w.contracts;
    let before = contracts.files.borrow().clone();
    let outcome = patch::patch(
        &w.ports(),
        PATH,
        &json!([{"op": "add", "path": "/x-extensions", "value": {"x-prompt": {"schema": true, "description": "d", "prompt": {"read": {"means": "m"}}}}}]),
        None,
    );
    assert_eq!(
        (
            outcome.findings[0].result.as_str(),
            contracts.files.borrow().clone()
        ),
        ("violated", before)
    );
}

/// traces: U1:事後
#[test]
fn patch_keeping_an_earlier_violation_is_written() {
    let w = created();
    let contracts = &w.contracts;
    contracts.files.borrow_mut().get_mut(PATH).unwrap()["x-color"] = json!("red");
    let outcome = patch::patch(
        &w.ports(),
        PATH,
        &json!([{"op": "replace", "path": "/title", "value": "記録"}]),
        None,
    );
    assert_eq!(
        (
            outcome.data["saved"].clone(),
            contracts.files.borrow()[PATH]["title"].clone()
        ),
        (json!(true), json!("記録"))
    );
}

/// traces: U1:失敗時
#[test]
fn unreadable_glossary_is_an_error_not_a_refusal() {
    let mut w = created();
    w.contracts.find_error = Some("読めない".to_string());
    let outcome = patch::patch(
        &w.ports(),
        PATH,
        &json!([{"op": "add", "path": "/properties", "value": {"p": {"description": "値段", "x-prompt": {"read": {"means": "値段"}}, "$ref": "urn:semantic-agent-schema:o:glossary:v1#Price"}}}]),
        None,
    );
    assert_eq!(
        (
            outcome.findings[0].result.as_str(),
            outcome.findings[0].pointer.as_str()
        ),
        ("error", "/properties/p/$ref")
    );
}

/// traces: I8
#[test]
fn instance_whose_concrete_cannot_be_found_is_a_draft() {
    let w = World::default();
    w.contracts.files.borrow_mut().insert(
        "x/i.json".to_string(),
        json!({"$schema": "urn:semantic-agent-schema:x:none:v1", "kind": "k", "id": "1"}),
    );
    let outcome = validate::validate(&w.ports(), "x/i.json");
    assert_eq!(
        (
            outcome.findings[0].rule.as_str(),
            outcome.data["state"]["label"].clone()
        ),
        ("concrete-resolves", json!("下書き"))
    );
}

const INSTANCE: &str = "o/r-1.json";

/// 人が決める項目 status を持つ Concrete と、その記録を1件置いた代役
fn with_instance(status: &str) -> World {
    let w = World::default();
    w.contracts.files.borrow_mut().insert(PATH.to_string(), json!({
        "$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1",
        "$id": "urn:semantic-agent-schema:o:record:v1",
        "title": "記録", "description": "決定の記録", "type": "object",
        "required": ["$schema", "kind", "id"],
        "properties": {
            "$schema": {"const": "urn:semantic-agent-schema:o:record:v1"},
            "kind": {"const": "record", "description": "種類", "x-prompt": {"read": {"means": "種類"}}},
            "id": {"type": "string", "description": "id", "x-prompt": {"read": {"means": "id"}}},
            "status": {"enum": ["proposed", "accepted"], "default": "proposed", "x-decided-by": "human", "description": "状態", "x-prompt": {"read": {"means": "状態"}}}
        }
    }));
    w.contracts.files.borrow_mut().insert(INSTANCE.to_string(), json!({"$schema": "urn:semantic-agent-schema:o:record:v1", "kind": "record", "id": "r-1", "status": status}));
    w
}

fn human_entry(w: &World) -> serde_json::Value {
    let out = validate::validate(&w.ports(), INSTANCE);
    out.data["results"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|r| r["rule"] == "human-entry-only")
        .map(|r| r["result"].clone())
        .unwrap_or_default()
}

fn record_status(w: &World, id: &str) {
    record::record(
        &w.ports(),
        &record::RecordRequest {
            path: INSTANCE,
            pointers: &["/status".to_string()],
            intent: "承認済みにする",
            purpose: "0199",
            writer: "ai-agent",
            replaces: None,
            id,
            session: None,
            workspace: None,
        },
    );
}

const ID: &str = "01JB7Z0000000000000000000A";

/// traces: U16:事後
#[test]
fn human_field_changed_waits_until_a_human_approves_the_record() {
    let w = with_instance("accepted");
    record_status(&w, ID);
    let before = human_entry(&w);
    judge::judge_intent(
        &w.ports(),
        ID,
        "verified",
        "状態を承認済みにした",
        "決定を確かめた",
    );
    assert_eq!(
        (before, human_entry(&w)),
        (json!("unverified"), json!("verified"))
    );
}

/// traces: U16:失敗時
#[test]
fn judging_an_unknown_record_is_refused() {
    let w = with_instance("proposed");
    assert_eq!(
        judge::judge_intent(&w.ports(), ID, "verified", "m", "r").findings[0].result,
        "refused"
    );
}

/// traces: U13:事後
#[test]
fn history_lists_the_record_with_its_judgment() {
    let w = with_instance("accepted");
    record_status(&w, ID);
    judge::judge_intent(
        &w.ports(),
        ID,
        "verified",
        "状態を承認済みにした",
        "決定を確かめた",
    );
    let out = history::history(&w.ports(), INSTANCE);
    assert_eq!(
        (
            out.data["changes"][0]["intent"].clone(),
            out.data["changes"][0]["intent_judgment"]["result"].clone()
        ),
        (json!("承認済みにする"), json!("verified"))
    );
}

/// traces: U6:事後
#[test]
fn patch_with_the_current_etag_is_written() {
    let w = created();
    let etag = ETag::of(&w.contracts.files.borrow()[PATH]).to_string();
    let out = patch::patch(
        &w.ports(),
        PATH,
        &json!([{"op": "replace", "path": "/title", "value": "記録"}]),
        Some(&etag),
    );
    assert_eq!(out.data["saved"], json!(true));
}

/// traces: U6:失敗時
#[test]
fn patch_with_an_old_etag_is_refused_and_leaves_the_file() {
    let w = created();
    let before = w.contracts.files.borrow().clone();
    let out = patch::patch(
        &w.ports(),
        PATH,
        &json!([{"op": "replace", "path": "/title", "value": "記録"}]),
        Some("00000000000000000000"),
    );
    assert_eq!(
        (
            out.findings[0].result.as_str(),
            w.contracts.files.borrow().clone()
        ),
        ("refused", before)
    );
}
