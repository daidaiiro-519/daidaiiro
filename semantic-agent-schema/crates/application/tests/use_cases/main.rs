//! 結合テスト：使う場面を、ポートの代役で動かす

mod doubles;

use doubles::contracts::FakeContracts;
use semantic_agent_schema_application::use_cases::{create, patch, validate};
use serde_json::json;

const PATH: &str = "o/record.schema.json";

fn created() -> FakeContracts {
    let contracts = FakeContracts::default();
    create::create(
        &contracts,
        PATH,
        "o",
        "record",
        &json!({"title": "記録", "description": "決定の記録"}),
    );
    contracts
}

/// traces: U1:失敗時
#[test]
fn creating_over_an_existing_file_is_refused() {
    let contracts = created();
    let before = contracts.files.borrow().clone();
    let outcome = create::create(
        &contracts,
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
    let contracts = created();
    let before = contracts.files.borrow().clone();
    let outcome = patch::patch(
        &contracts,
        &contracts,
        PATH,
        &json!([{"op": "replace", "path": "/$id", "value": "x"}]),
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
    let contracts = created();
    let before = contracts.files.borrow().clone();
    let outcome = patch::patch(
        &contracts,
        &contracts,
        PATH,
        &json!([{"op": "add", "path": "/x-extensions", "value": {"x-prompt": {"schema": true, "description": "d", "prompt": {"read": {"means": "m"}}}}}]),
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
    let contracts = created();
    contracts.files.borrow_mut().get_mut(PATH).unwrap()["x-color"] = json!("red");
    let outcome = patch::patch(
        &contracts,
        &contracts,
        PATH,
        &json!([{"op": "replace", "path": "/title", "value": "記録"}]),
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
    let mut contracts = created();
    contracts.find_error = Some("読めない".to_string());
    let outcome = patch::patch(
        &contracts,
        &contracts,
        PATH,
        &json!([{"op": "add", "path": "/properties", "value": {"p": {"description": "値段", "x-prompt": {"read": {"means": "値段"}}, "$ref": "urn:semantic-agent-schema:o:glossary:v1#Price"}}}]),
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
    let contracts = FakeContracts::default();
    contracts.files.borrow_mut().insert(
        "x/i.json".to_string(),
        json!({"$schema": "urn:semantic-agent-schema:x:none:v1", "kind": "k", "id": "1"}),
    );
    let outcome = validate::validate(&contracts, &contracts, "x/i.json");
    assert_eq!(
        (
            outcome.findings[0].rule.as_str(),
            outcome.data["state"]["label"].clone()
        ),
        ("concrete-resolves", json!("下書き"))
    );
}
