//! 結合テスト：使う場面を、ポートの代役で動かす

mod doubles;

use doubles::contracts::FakeContracts;
use semantic_agent_schema_application::use_cases::{create, patch};
use serde_json::json;

const PATH: &str = "spec/record.schema.json";

fn created() -> FakeContracts {
    let contracts = FakeContracts::default();
    create::create(&contracts, PATH, "o", "record");
    contracts
}

/// traces: U1:失敗時
#[test]
fn creating_over_an_existing_file_is_refused() {
    let contracts = created();
    let before = contracts.files.borrow().clone();
    let outcome = create::create(&contracts, PATH, "o", "record");
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
fn patch_breaking_the_abstract_is_saved_and_violated() {
    let contracts = created();
    let outcome = patch::patch(
        &contracts,
        PATH,
        &json!([{"op": "add", "path": "/x-extensions", "value": {"x-prompt": {"schema": true, "description": "d"}}}]),
    );
    assert_eq!(
        (
            outcome.findings[0].result.as_str(),
            contracts.files.borrow()[PATH].get("x-extensions").is_some()
        ),
        ("violated", true)
    );
}

/// traces: U1:失敗時
#[test]
fn unreadable_glossary_after_saving_is_an_error_not_a_refusal() {
    let mut contracts = created();
    contracts.find_error = Some("読めない".to_string());
    let outcome = patch::patch(
        &contracts,
        PATH,
        &json!([{"op": "add", "path": "/properties/p", "value": {"$ref": "urn:semantic-agent-schema:o:glossary:v1#Price"}}]),
    );
    assert_eq!(
        (
            outcome.findings[0].result.as_str(),
            outcome.findings[0].pointer.as_str()
        ),
        ("error", "/properties/p/$ref")
    );
}
