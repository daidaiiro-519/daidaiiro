//! 2回目（検査と承認）のテスト（テストレベル system）。CLI を一時ディレクトリで実行して確かめる。
//! テストの名前は、テスト条件の ID を含む。

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_schema-driven");

/// スキーマ（thing）と、インスタンスを置くディレクトリ decls を用意する。
fn workdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("schema-driven-r2-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("schema")).unwrap();
    fs::create_dir_all(dir.join("decls")).unwrap();
    fs::write(
        dir.join("schema/thing.schema.json"),
        json!({
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
    let mut v =
        json!({"$schema": "../schema/thing.schema.json", "id": id, "kind": "thing", "name": id});
    for (k, x) in extra.as_object().unwrap() {
        v[k] = x.clone();
    }
    if v["name"].is_null() {
        v.as_object_mut().unwrap().remove("name");
    }
    fs::write(dir.join(format!("decls/{id}.json")), v.to_string()).unwrap();
}

fn run(dir: &Path, args: &[&str]) -> (i32, Value) {
    let out = Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    let value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    (out.status.code().unwrap_or(-1), value)
}

fn check(dir: &Path) -> (i32, Value) {
    run(dir, &["check", "--dir", "decls"])
}

fn approve(dir: &Path) -> (i32, Value) {
    run(dir, &["approve", "--dir", "decls"])
}

fn approval(dir: &Path) -> Option<String> {
    fs::read_to_string(dir.join("decls/approval.json")).ok()
}

fn set_readonly(path: &Path, readonly: bool) {
    let mut p = fs::metadata(path).unwrap().permissions();
    p.set_readonly(readonly);
    fs::set_permissions(path, p).unwrap();
}

fn drifts(out: &Value) -> Vec<Value> {
    out["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["status"] == "ずれ")
        .cloned()
        .collect()
}

// ── UC-5 ディレクトリのインスタンスを検査する

#[test]
fn uc_5_m_check_returns_validation_and_findings() {
    let dir = workdir("uc5-m");
    put(&dir, "A", json!({"refs": ["B"]}));
    put(&dir, "B", json!({}));
    let before = fs::read_to_string(dir.join("decls/A.json")).unwrap();
    let (code, out) = check(&dir);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["instances"].as_array().unwrap().len(), 2);
    assert!(drifts(&out).is_empty(), "{out}");
    assert!(out["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["check"] == "指す先がある" && f["to"] == "B"));
    assert_eq!(
        fs::read_to_string(dir.join("decls/A.json")).unwrap(),
        before,
        "インスタンスを書き換えない"
    );
}

#[test]
fn uc_5_ext_1_unreadable_instance_fails() {
    let dir = workdir("uc5-ext1");
    put(&dir, "A", json!({}));
    fs::write(dir.join("decls/broken.json"), "{").unwrap();
    let (code, out) = check(&dir);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "JSON として読めない");
}

#[test]
fn uc_5_ext_2_empty_directory_ends() {
    let dir = workdir("uc5-ext2");
    let (code, out) = check(&dir);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["instances"], json!([]));
    assert_eq!(out["findings"], json!([]));
}

// ── UC-8 承認を記録する

#[test]
fn uc_8_m_approve_records_paths_and_hashes() {
    let dir = workdir("uc8-m");
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
        .any(|f| f["check"] == "承認のあとの変化" && f["status"] == "合格"));
}

#[test]
fn uc_8_ext_1_validation_error_is_rejected() {
    let dir = workdir("uc8-ext1");
    put(&dir, "A", json!({"name": ""}));
    let (code, out) = approve(&dir);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "検証を通過しないインスタンスがある");
    assert!(approval(&dir).is_none(), "拒んだら承認記録を書かない");
}

#[test]
fn uc_8_ext_2_no_change_since_last_approval_ends() {
    let dir = workdir("uc8-ext2");
    put(&dir, "A", json!({}));
    approve(&dir);
    let before = approval(&dir);
    let (code, out) = approve(&dir);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["changed"], false);
    assert_eq!(approval(&dir), before);
}

#[test]
fn uc_8_ext_3_unwritable_directory_fails() {
    let dir = workdir("uc8-ext3");
    put(&dir, "A", json!({}));
    set_readonly(&dir.join("decls"), true);
    let (code, out) = approve(&dir);
    set_readonly(&dir.join("decls"), false);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "書けない");
}

#[test]
fn uc_8_ext_4_drift_is_rejected() {
    let dir = workdir("uc8-ext4");
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
fn uc_8_ext_5_unfilled_is_rejected() {
    let dir = workdir("uc8-ext5");
    put(&dir, "A", json!({"name": null}));
    let (code, out) = approve(&dir);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "未記入のプロパティがある");
}

// ── UC-4 インスタンスを削除する

#[test]
fn uc_4_m_delete_reports_remaining_references() {
    let dir = workdir("uc4-m");
    put(&dir, "A", json!({"refs": ["B"]}));
    put(&dir, "B", json!({}));
    let (code, out) = run(&dir, &["delete", "--path", "decls/B.json"]);
    assert_eq!(code, 0, "{out}");
    assert!(!dir.join("decls/B.json").exists());
    let remaining = out["remaining"].as_array().unwrap();
    assert_eq!(remaining.len(), 1, "{out}");
    assert_eq!(remaining[0]["from"], "A/refs");
}

#[test]
fn uc_4_ext_1_missing_instance_fails() {
    let dir = workdir("uc4-ext1");
    let (code, out) = run(&dir, &["delete", "--path", "decls/none.json"]);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "読めない");
}

#[test]
fn uc_4_ext_2_unwritable_directory_fails() {
    let dir = workdir("uc4-ext2");
    put(&dir, "B", json!({}));
    set_readonly(&dir.join("decls"), true);
    let (code, out) = run(&dir, &["delete", "--path", "decls/B.json"]);
    set_readonly(&dir.join("decls"), false);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "書けない");
    assert!(dir.join("decls/B.json").exists());
}

#[test]
fn uc_4_ext_3_unreadable_directory_after_delete_fails() {
    let dir = workdir("uc4-ext3");
    put(&dir, "B", json!({}));
    fs::write(dir.join("decls/broken.json"), "{").unwrap();
    let (code, out) = run(&dir, &["delete", "--path", "decls/B.json"]);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "JSON として読めない");
}
