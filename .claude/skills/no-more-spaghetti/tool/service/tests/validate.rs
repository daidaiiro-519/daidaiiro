// SPDX-License-Identifier: MIT
//! `validate` は、規則ファイルの検査と references の検査（契約の版2）を1つの名前で受ける。
//! **rules も root も渡さなければ references を検査し、どちらかを渡せば規則ファイルを検査する。**
//!
//!     cargo test -p nms_service --test validate

use std::path::PathBuf;

use nms_service::{tools, Given, Outcome};

fn skill_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn call(name: &str, pairs: &[(&str, String)]) -> Outcome {
    let tool = tools()
        .into_iter()
        .find(|t| t.name == name)
        .expect("道具が在る");
    let mut given = Given::default();
    for (k, v) in pairs {
        given.push(k, v.clone());
    }
    (tool.run)(&given)
}

#[test]
fn the_catalog_holds_each_references_tool_once() {
    let names: Vec<&str> = tools().iter().map(|t| t.name).collect();
    for want in ["get", "validate", "view", "import"] {
        assert_eq!(
            names.iter().filter(|n| **n == want).count(),
            1,
            "{want} が1件だけ在る ── {names:?}"
        );
    }
}

#[test]
fn validate_without_rules_checks_the_references() {
    let root = skill_root().display().to_string();
    let out = call("validate", &[("skill_root", root)]);
    assert!(out.ok, "誤用でない ── {:?}", out.findings);
    assert!(out.findings.is_empty(), "{:?}", out.findings);
    let kinds = out.data.get("kinds").and_then(|x| x.as_array());
    assert!(
        kinds.is_some_and(|k| !k.is_empty()),
        "references の種類を返す ── {}",
        out.data
    );
}

#[test]
fn validate_with_rules_checks_the_rules_file() {
    let root = skill_root();
    let rules = root
        .join("references/self-rules.json")
        .display()
        .to_string();
    let out = call(
        "validate",
        &[("rules", rules), ("skill_root", root.display().to_string())],
    );
    assert!(out.ok, "誤用でない ── {:?}", out.findings);
    assert!(out.findings.is_empty(), "{:?}", out.findings);
    assert_eq!(
        out.data.get("kind").and_then(|x| x.as_str()),
        Some("rules"),
        "規則ファイルとして検査する ── {}",
        out.data
    );
}
