//! リポジトリに置いた JSON Schema が 2020-12 のメタスキーマに従い、
//! その例のインスタンスがスキーマに従うことを確かめる。

// 試験の補助の関数でも expect を使う（clippy.toml が #[test] の中で許すのと同じ考え）。
#![allow(clippy::expect_used)]

use serde_json::Value;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(rel: &str) -> Value {
    let text = std::fs::read_to_string(root().join(rel)).expect("ファイルを読めること");
    serde_json::from_str(&text).expect("JSON であること")
}

fn assert_meta_valid(rel: &str) {
    let schema = load(rel);
    assert!(
        jsonschema::meta::is_valid(&schema),
        "{rel} が 2020-12 のメタスキーマに従わない"
    );
}

#[test]
fn scenario_schema_follows_meta_schema() {
    assert_meta_valid("schemas-and-instances/system-tests/scenario.schema.json");
}

#[test]
fn exploration_schema_follows_meta_schema() {
    assert_meta_valid("docs/design/exploration/exploration.schema.json");
}

#[test]
fn exploration_records_follow_schema() {
    let schema = load("docs/design/exploration/exploration.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("探索の記録のスキーマを組めること");
    let dir = root().join("docs/design/exploration");
    let mut count = 0;
    for entry in std::fs::read_dir(&dir).expect("記録の置き場所を読めること") {
        let path = entry.expect("項目を読めること").path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if !(name.starts_with("step-") && name.ends_with(".json")) {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("記録を読めること");
        let record: Value = serde_json::from_str(&text).expect("記録が JSON であること");
        let errors: Vec<String> = validator
            .iter_errors(&record)
            .map(|e| e.to_string())
            .collect();
        assert!(errors.is_empty(), "{name} がスキーマに従わない: {errors:?}");
        count += 1;
    }
    assert!(count > 0, "探索の記録が1件も無い");
}

#[test]
fn broken_record_is_rejected() {
    let schema = load("docs/design/exploration/exploration.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("探索の記録のスキーマを組めること");
    let broken = serde_json::json!({"$schema": "urn:semantic-agent-schema:design:exploration:v1", "step": 1});
    assert!(!validator.is_valid(&broken), "欄の欠けた記録を通してしまう");
}

#[test]
fn shipped_result_schema_follows_meta_schema() {
    assert_meta_valid("schemas-and-instances/published-language/v1/operation-result.schema.json");
}

#[test]
fn shipped_procedure_schema_follows_meta_schema() {
    assert_meta_valid("schemas-and-instances/published-language/v1/procedure.schema.json");
}

#[test]
fn scenarios_follow_schema() {
    let schema = load("schemas-and-instances/system-tests/scenario.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("場面のスキーマを組めること");
    let mut count = 0;
    for entry in std::fs::read_dir(root().join("schemas-and-instances/system-tests/scenarios"))
        .expect("場面の置き場所を読めること")
    {
        let path = entry.expect("項目を読めること").path();
        let text = std::fs::read_to_string(&path).expect("場面を読めること");
        let scenario: Value = serde_json::from_str(&text).expect("場面が JSON であること");
        let errors: Vec<String> = validator
            .iter_errors(&scenario)
            .map(|e| e.to_string())
            .collect();
        assert!(
            errors.is_empty(),
            "{} がスキーマに従わない: {errors:?}",
            path.display()
        );
        count += 1;
    }
    assert!(count > 0, "場面が1つも無い");
}
