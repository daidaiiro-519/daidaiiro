//! concrete スキーマは abstract スキーマのインスタンスとして、エンティティのインスタンスと同じ共通ツールで作り、書き進め、検査する。

mod common;

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::Command;

const ABSTRACT: &str = "schema-driven/abstract@v1";
const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

fn run(dir: &Path, args: &[&str]) -> (i32, Value) {
    let out = Command::new(env!("CARGO_BIN_EXE_schema-driven"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    let value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    (out.status.code().unwrap_or(-1), value)
}

fn read(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

/// abstract から demo/assets/term.schema.json を作る。
fn create_schema(dir: &Path) -> (i32, Value) {
    fs::create_dir_all(dir.join("demo/assets")).unwrap();
    run(
        dir,
        &[
            "create",
            "--schema",
            ABSTRACT,
            "--path",
            "demo/assets/term.schema.json",
        ],
    )
}

fn prompted(out: &Value) -> Vec<&str> {
    out["unfilled"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["prompt"]["write"].is_array())
        .map(|item| item["property"].as_str().unwrap())
        .collect()
}

#[test]
fn create_from_the_abstract_writes_the_dialect_and_the_version() {
    let dir = common::Scratch::new("concrete-create");
    let (code, out) = create_schema(&dir);
    assert_eq!(code, 0, "{out}");
    let written = read(&dir.join("demo/assets/term.schema.json"));
    assert_eq!(written, json!({"$schema": DIALECT, "x-abstract": ABSTRACT}));
    assert_eq!(prompted(&out), vec!["/description", "/title"], "{out}");
}

#[test]
fn create_from_the_abstract_refuses_an_existing_path() {
    let dir = common::Scratch::new("concrete-exists");
    assert_eq!(create_schema(&dir).0, 0);
    let (code, out) = create_schema(&dir);
    assert_eq!(code, 1, "{out}");
}

#[test]
fn prompt_reads_the_abstract_by_name() {
    let dir = common::Scratch::new("concrete-prompt");
    let (code, out) = run(
        &dir,
        &["prompt", "--schema", ABSTRACT, "--property", "/title"],
    );
    assert_eq!(code, 0, "{out}");
    assert!(
        out["prompt"]["write"]
            .as_array()
            .is_some_and(|rules| !rules.is_empty()),
        "{out}"
    );
}

#[test]
fn a_concrete_schema_is_written_with_update_and_then_makes_entities() {
    let dir = common::Scratch::new("concrete-flow");
    assert_eq!(create_schema(&dir).0, 0);
    let prompt = json!({"read": {"means": "種類である。"},
                        "write": [{"id": "const", "rule": "term と書く。", "why": "種類を判定する。", "test": "値が term かを見る。"}]});
    let patch = json!([
        {"op": "add", "path": "/$id", "value": "demo/term@v1"},
        {"op": "add", "path": "/title", "value": "用語"},
        {"op": "add", "path": "/description", "value": "用語1つ。説明の文書は持たない。"},
        {"op": "add", "path": "/x-generates", "value": "terms/<名前>.json"},
        {"op": "add", "path": "/type", "value": "object"},
        {"op": "add", "path": "/required", "value": ["$schema", "kind"]},
        {"op": "add", "path": "/properties", "value": {
            "$schema": {"const": "demo/term@v1", "description": "従うスキーマの名前", "x-prompt": prompt},
            "kind": {"const": "term", "description": "種類", "x-prompt": prompt}}}
    ]);
    let (code, out) = run(
        &dir,
        &[
            "update",
            "--path",
            "demo/assets/term.schema.json",
            "--patch",
            &patch.to_string(),
        ],
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["errors"], json!([]), "{out}");

    let (code, out) = run(&dir, &["check-schemas", "--dir", "demo/assets"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["findings"], json!([]), "{out}");

    let skill = dir.join("demo");
    let (code, out) = run(
        &dir,
        &[
            "--skill",
            skill.to_str().unwrap(),
            "create",
            "--schema",
            "demo/assets/term.schema.json",
            "--path",
            "terms/a.json",
        ],
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(
        read(&dir.join("terms/a.json")),
        json!({"$schema": "demo/term@v1"})
    );
}

#[test]
fn check_schemas_returns_how_to_write_a_missing_field() {
    let dir = common::Scratch::new("concrete-findings");
    assert_eq!(create_schema(&dir).0, 0);
    let (code, out) = run(&dir, &["check-schemas", "--dir", "demo/assets"]);
    assert_eq!(code, 0, "{out}");
    let title = out["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["at"] == "/title")
        .unwrap_or_else(|| panic!("/title の指摘が無い：{out}"));
    assert!(
        title["prompt"]["write"]
            .as_array()
            .is_some_and(|rules| !rules.is_empty()),
        "{title}"
    );
}
