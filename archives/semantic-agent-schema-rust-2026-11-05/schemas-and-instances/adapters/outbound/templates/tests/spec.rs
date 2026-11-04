//! mustache/spec の必須の部分の試験を、そのまま通す（tests/mustache-spec/）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use outbound_templates::mustache::{parse, render};
use serde_json::Value;
use std::collections::BTreeMap;

fn run(file: &str) {
    let path = format!(
        "{}/tests/mustache-spec/{file}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let spec: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut failed = Vec::new();
    let tests = spec["tests"].as_array().unwrap();
    assert!(!tests.is_empty(), "{file} に試験が無い");
    for t in tests {
        let name = t["name"].as_str().unwrap();
        let partials: BTreeMap<String, String> = t
            .get("partials")
            .and_then(Value::as_object)
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let got = parse(t["template"].as_str().unwrap())
            .and_then(|nodes| render(&nodes, &t["data"], &partials));
        let want = t["expected"].as_str().unwrap();
        if got.as_deref() != Ok(want) {
            failed.push(format!("{name}: want {want:?} got {got:?}"));
        }
    }
    assert!(failed.is_empty(), "{file}：\n{}", failed.join("\n"));
}

#[test]
fn comments() {
    run("comments");
}

#[test]
fn delimiters() {
    run("delimiters");
}

#[test]
fn interpolation() {
    run("interpolation");
}

#[test]
fn inverted() {
    run("inverted");
}

#[test]
fn partials() {
    run("partials");
}

#[test]
fn sections() {
    run("sections");
}
