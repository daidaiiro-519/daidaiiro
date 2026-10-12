//! 公開した版のスキーマが書き換えられていないことを、公開の記録（published.json）の指紋で確かめる。
//! check-schemas が、記録した版の中身の変化と消失を指摘し、まだ記録していない名前を指紋つきで返す。

mod common;

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::Command;

const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

fn schema(id: &str, max: u64) -> Value {
    json!({"$schema": DIALECT, "x-abstract": "schema-driven/abstract@v1", "$id": id,
           "title": "語の決まり", "description": "見本",
           "$defs": {"short": {"type": "string", "maxLength": max}}})
}

fn run(dir: &Path) -> (i32, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_schema-driven"))
        .args(["check-schemas", "--dir"])
        .arg(dir)
        .output()
        .unwrap();
    let out = serde_json::from_slice(&output.stdout).unwrap_or_else(
        |_| json!({"unparsed": String::from_utf8_lossy(&output.stdout).into_owned()}),
    );
    (output.status.code().unwrap_or(-1), out)
}

fn rules(out: &Value) -> Vec<String> {
    out["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["rule"].as_str().unwrap().to_owned())
        .collect()
}

/// いまのスキーマの指紋を check-schemas から受け取り、そのまま公開の記録にする。
fn publish(dir: &Path) {
    let (_, out) = run(dir);
    let records: Vec<Value> = out["unpublished"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| json!({"name": item["name"], "sha256": item["sha256"]}))
        .collect();
    fs::write(
        dir.join("published.json"),
        json!({"$schema": "schema-driven/published@v1", "kind": "published", "schemas": records})
            .to_string(),
    )
    .unwrap();
}

#[test]
fn a_new_name_is_listed_with_its_fingerprint() {
    let dir = common::Scratch::new("published-new");
    fs::write(
        dir.join("term.schema.json"),
        schema("lexicon/term@v1", 3).to_string(),
    )
    .unwrap();
    let (code, out) = run(&dir);
    assert_eq!(code, 0, "{out}");
    assert_eq!(rules(&out), Vec::<String>::new(), "{out}");
    assert_eq!(out["unpublished"][0]["name"], "lexicon/term@v1", "{out}");
    assert_eq!(out["unpublished"][0]["sha256"].as_str().unwrap().len(), 64);
}

#[test]
fn a_recorded_version_that_did_not_change_passes() {
    let dir = common::Scratch::new("published-same");
    fs::write(
        dir.join("term.schema.json"),
        schema("lexicon/term@v1", 3).to_string(),
    )
    .unwrap();
    publish(&dir);
    let (_, out) = run(&dir);
    assert_eq!(rules(&out), Vec::<String>::new(), "{out}");
    assert_eq!(out["unpublished"], json!([]), "{out}");
}

#[test]
fn key_order_and_spacing_do_not_change_the_fingerprint() {
    let dir = common::Scratch::new("published-order");
    fs::write(
        dir.join("term.schema.json"),
        schema("lexicon/term@v1", 3).to_string(),
    )
    .unwrap();
    publish(&dir);
    // 同じ中身を、キーの順を変えて字下げして書き直す
    let text = r#"{
      "$defs": {"short": {"maxLength": 3, "type": "string"}},
      "description": "見本", "title": "語の決まり",
      "$id": "lexicon/term@v1", "x-abstract": "schema-driven/abstract@v1",
      "$schema": "https://json-schema.org/draft/2020-12/schema"
    }"#;
    fs::write(dir.join("term.schema.json"), text).unwrap();
    let (_, out) = run(&dir);
    assert_eq!(rules(&out), Vec::<String>::new(), "{out}");
}

#[test]
fn a_rewritten_published_version_is_a_finding() {
    let dir = common::Scratch::new("published-rewritten");
    fs::write(
        dir.join("term.schema.json"),
        schema("lexicon/term@v1", 3).to_string(),
    )
    .unwrap();
    publish(&dir);
    fs::write(
        dir.join("term.schema.json"),
        schema("lexicon/term@v1", 4).to_string(),
    )
    .unwrap();
    let (code, out) = run(&dir);
    assert_eq!(code, 0, "{out}");
    assert_eq!(rules(&out), vec!["公開した版が書き換えられている"], "{out}");
    assert!(
        out["findings"][0]["message"]
            .as_str()
            .unwrap()
            .contains("@v2"),
        "{out}"
    );
}

#[test]
fn a_removed_published_version_is_a_finding() {
    let dir = common::Scratch::new("published-removed");
    fs::write(
        dir.join("term.schema.json"),
        schema("lexicon/term@v1", 3).to_string(),
    )
    .unwrap();
    publish(&dir);
    fs::remove_file(dir.join("term.schema.json")).unwrap();
    let (_, out) = run(&dir);
    assert_eq!(rules(&out), vec!["公開した版が無い"], "{out}");
}

#[test]
fn a_new_version_beside_the_old_one_is_only_unpublished() {
    let dir = common::Scratch::new("published-v2");
    fs::write(
        dir.join("term.schema.json"),
        schema("lexicon/term@v1", 3).to_string(),
    )
    .unwrap();
    publish(&dir);
    fs::write(
        dir.join("term.v2.schema.json"),
        schema("lexicon/term@v2", 4).to_string(),
    )
    .unwrap();
    let (_, out) = run(&dir);
    assert_eq!(rules(&out), Vec::<String>::new(), "{out}");
    assert_eq!(out["unpublished"][0]["name"], "lexicon/term@v2", "{out}");
}

#[test]
fn a_named_schema_whose_file_is_not_dot_schema_json_is_also_fingerprinted() {
    let dir = common::Scratch::new("published-meta");
    fs::write(
        dir.join("meta-schema.json"),
        json!({"$schema": DIALECT, "$id": "lexicon/abstract@v1", "type": "object"}).to_string(),
    )
    .unwrap();
    let (_, out) = run(&dir);
    assert_eq!(
        out["unpublished"][0]["name"], "lexicon/abstract@v1",
        "{out}"
    );
    publish(&dir);
    fs::write(
        dir.join("meta-schema.json"),
        json!({"$schema": DIALECT, "$id": "lexicon/abstract@v1", "type": "array"}).to_string(),
    )
    .unwrap();
    let (_, out) = run(&dir);
    assert_eq!(rules(&out), vec!["公開した版が書き換えられている"], "{out}");
}
