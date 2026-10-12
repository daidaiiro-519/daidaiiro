//! ディレクトリのインスタンスの検査 ・ 承認と、参照が残るインスタンスの削除のテスト。
//! CLI を一時ディレクトリで実行して確かめる。スキーマは Skill example の assets に置き、--skill で渡す。

mod common;

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_schema-driven");

/// スキーマを置く Skill の、作業用のフォルダからのパス。
const SKILL: &str = "skills/example";

/// Skill example の assets のスキーマ（example/thing@v1）と、インスタンスを置くディレクトリ declarations を用意する。
fn workdir(name: &str) -> common::Scratch {
    let dir = common::Scratch::new(&format!("schema-driven-check-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join(SKILL).join("assets")).unwrap();
    fs::create_dir_all(dir.join("declarations")).unwrap();
    fs::write(
        dir.join(SKILL).join("assets/thing.schema.json"),
        json!({
            "$id": "example/thing@v1",
            "type": "object", "additionalProperties": false, "required": ["id", "kind", "name"],
            "properties": {
                "$schema": {"type": "string"}, "id": {"type": "string"}, "kind": {"const": "thing"},
                "name": {"type": "string", "minLength": 1},
                "refs": {"type": "array", "items": {"type": "string"}, "x-ref": {"to": "thing"}}
            }
        })
        .to_string(),
    )
    .unwrap();
    dir
}

fn put(dir: &Path, id: &str, extra: Value) {
    let mut instance =
        json!({"$schema": "example/thing@v1", "id": id, "kind": "thing", "name": id});
    for (key, value) in extra.as_object().unwrap() {
        instance[key] = value.clone();
    }
    if instance["name"].is_null() {
        instance.as_object_mut().unwrap().remove("name");
    }
    fs::write(
        dir.join(format!("declarations/{id}.json")),
        instance.to_string(),
    )
    .unwrap();
}

/// CLI を --skill に Skill example を渡して実行し、終了コードと標準出力の JSON を返す。
fn run(dir: &Path, args: &[&str]) -> (i32, Value) {
    let out = Command::new(BIN)
        .current_dir(dir)
        .arg("--skill")
        .arg(dir.join(SKILL))
        .args(args)
        .output()
        .unwrap();
    let value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    (out.status.code().unwrap_or(-1), value)
}

fn check(dir: &Path) -> (i32, Value) {
    run(dir, &["check", "--dir", "declarations"])
}

fn approve(dir: &Path) -> (i32, Value) {
    run(dir, &["approve", "--dir", "declarations"])
}

fn approval(dir: &Path) -> Option<String> {
    fs::read_to_string(dir.join("declarations/approval.json")).ok()
}

fn set_readonly(path: &Path, readonly: bool) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_readonly(readonly);
    fs::set_permissions(path, permissions).unwrap();
}

fn drifts(out: &Value) -> Vec<Value> {
    out["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["status"] == "ずれ")
        .cloned()
        .collect()
}

// ── ディレクトリのインスタンスを検査する

#[test]
fn check_returns_validation_and_findings() {
    let dir = workdir("check-returns-validation-and-findings");
    put(&dir, "A", json!({"refs": ["B"]}));
    put(&dir, "B", json!({}));
    let before = fs::read_to_string(dir.join("declarations/A.json")).unwrap();
    let (code, out) = check(&dir);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["instances"].as_array().unwrap().len(), 2);
    assert!(drifts(&out).is_empty(), "{out}");
    assert!(out["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["check"] == "指す先がある" && finding["to"] == "B"));
    assert_eq!(
        fs::read_to_string(dir.join("declarations/A.json")).unwrap(),
        before,
        "インスタンスを書き換えない"
    );
}

#[test]
fn check_of_an_unreadable_instance_fails() {
    let dir = workdir("check-of-an-unreadable-instance-fails");
    put(&dir, "A", json!({}));
    fs::write(dir.join("declarations/broken.json"), "{").unwrap();
    let (code, out) = check(&dir);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "JSON として読めない");
}

#[test]
fn check_of_an_empty_directory_ends() {
    let dir = workdir("check-of-an-empty-directory-ends");
    let (code, out) = check(&dir);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["instances"], json!([]));
    assert_eq!(out["findings"], json!([]));
}

// ── 承認を記録する

#[test]
fn approve_records_paths_and_hashes() {
    let dir = workdir("approve-records-paths-and-hashes");
    put(&dir, "A", json!({"refs": ["B"]}));
    put(&dir, "B", json!({}));
    let (code, out) = approve(&dir);
    assert_eq!(code, 0, "{out}");
    let record: Value = serde_json::from_str(&approval(&dir).unwrap()).unwrap();
    assert_eq!(record["instances"].as_array().unwrap().len(), 2);
    let (_, after) = check(&dir);
    assert!(after["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["check"] == "承認のあとの変化" && finding["status"] == "合格"));
}

#[test]
fn approve_refuses_a_validation_error() {
    let dir = workdir("approve-refuses-a-validation-error");
    put(&dir, "A", json!({"name": ""}));
    let (code, out) = approve(&dir);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "検証を通過しないインスタンスがある");
    assert!(approval(&dir).is_none(), "拒んだら承認記録を書かない");
}

#[test]
fn approve_without_a_change_since_the_last_approval_ends() {
    let dir = workdir("approve-without-a-change-since-the-last-approval-ends");
    put(&dir, "A", json!({}));
    approve(&dir);
    let before = approval(&dir);
    let (code, out) = approve(&dir);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["changed"], false);
    assert_eq!(approval(&dir), before);
}

#[test]
fn approve_in_an_unwritable_directory_fails() {
    let dir = workdir("approve-in-an-unwritable-directory-fails");
    put(&dir, "A", json!({}));
    set_readonly(&dir.join("declarations"), true);
    let (code, out) = approve(&dir);
    set_readonly(&dir.join("declarations"), false);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "書けない");
}

#[test]
fn approve_refuses_a_drift() {
    let dir = workdir("approve-refuses-a-drift");
    put(&dir, "A", json!({}));
    approve(&dir);
    let before = approval(&dir);
    put(&dir, "A", json!({"refs": ["Z"]}));
    let (code, out) = approve(&dir);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "参照と導出値のずれがある");
    assert_eq!(approval(&dir), before, "拒んだら前の承認記録のまま");
}

#[test]
fn approve_refuses_an_unfilled_property() {
    let dir = workdir("approve-refuses-an-unfilled-property");
    put(&dir, "A", json!({"name": null}));
    let (code, out) = approve(&dir);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "未記入のプロパティがある");
}

// ── インスタンスを削除する

#[test]
fn delete_reports_remaining_references() {
    let dir = workdir("delete-reports-remaining-references");
    put(&dir, "A", json!({"refs": ["B"]}));
    put(&dir, "B", json!({}));
    let (code, out) = run(&dir, &["delete", "--path", "declarations/B.json"]);
    assert_eq!(code, 0, "{out}");
    assert!(!dir.join("declarations/B.json").exists());
    let remaining = out["remaining"].as_array().unwrap();
    assert_eq!(remaining.len(), 1, "{out}");
    assert_eq!(remaining[0]["from"], "A/refs");
}

#[test]
fn delete_of_a_missing_instance_fails() {
    let dir = workdir("delete-of-a-missing-instance-fails");
    let (code, out) = run(&dir, &["delete", "--path", "declarations/none.json"]);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "読めない");
}

#[test]
fn delete_in_an_unwritable_directory_fails() {
    let dir = workdir("delete-in-an-unwritable-directory-fails");
    put(&dir, "B", json!({}));
    set_readonly(&dir.join("declarations"), true);
    let (code, out) = run(&dir, &["delete", "--path", "declarations/B.json"]);
    set_readonly(&dir.join("declarations"), false);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "書けない");
    assert!(dir.join("declarations/B.json").exists());
}

#[test]
fn delete_fails_when_the_directory_is_unreadable_afterwards() {
    let dir = workdir("delete-fails-when-the-directory-is-unreadable-afterwards");
    put(&dir, "B", json!({}));
    fs::write(dir.join("declarations/broken.json"), "{").unwrap();
    let (code, out) = run(&dir, &["delete", "--path", "declarations/B.json"]);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "JSON として読めない");
}
