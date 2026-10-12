//! インスタンスの作成 ・ 取得 ・ 更新 ・ 削除と、x-prompt の受け取りのテスト。
//! CLI のバイナリを一時ディレクトリで実行して確かめる。スキーマは Skill example の assets に置き、--skill で渡す。

mod common;

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_schema-driven");

/// スキーマを置く Skill の、作業用のフォルダからのパス。
const SKILL: &str = "skills/example";
/// thing のスキーマの、作業用のフォルダからのパス。
const THING: &str = "skills/example/assets/thing.schema.json";

/// テストごとの一時ディレクトリ。Skill example の assets にスキーマ（$ref で同じ Skill のスキーマを名前で指す）を置いておく。
fn workdir(name: &str) -> common::Scratch {
    let dir = common::Scratch::new(&format!("schema-driven-{name}"));
    let _ = fs::remove_dir_all(&dir);
    let assets = dir.join(SKILL).join("assets");
    fs::create_dir_all(&assets).unwrap();
    fs::write(
        assets.join("thing.schema.json"),
        json!({
            "$id": "example/thing@v1",
            "type": "object",
            "additionalProperties": false,
            "required": ["name", "items"],
            "properties": {
                "$schema": {"type": "string"},
                "name": {"type": "string", "minLength": 1,
                         "x-prompt": {"read": "名前として読む", "write": "名前を1語で書く"}},
                "items": {"type": "array", "items": {"$ref": "common@v1#/$defs/item"},
                          "x-prompt": {"read": "項目の一覧", "write": "項目を1件ずつ書く"}},
                "note": {"type": "string"}
            }
        })
        .to_string(),
    )
    .unwrap();
    fs::write(
        assets.join("common.schema.json"),
        json!({"$id": "example/common@v1", "$defs": {"item": {"type": "string", "minLength": 1}}})
            .to_string(),
    )
    .unwrap();
    dir
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
    let text = String::from_utf8_lossy(&out.stdout);
    let value = serde_json::from_str(&text).unwrap_or(Value::Null);
    (out.status.code().unwrap_or(-1), value)
}

fn create(dir: &Path) -> (i32, Value) {
    run(dir, &["create", "--schema", THING, "--path", "data/a.json"])
}

fn update(dir: &Path, patch: Value) -> (i32, Value) {
    run(
        dir,
        &[
            "update",
            "--path",
            "data/a.json",
            "--patch",
            &patch.to_string(),
        ],
    )
}

fn read(dir: &Path) -> String {
    fs::read_to_string(dir.join("data/a.json")).unwrap()
}

fn sha256(text: &str) -> String {
    schema_driven_core::domain::values::JsonValue::new(text)
        .hash()
        .as_str()
        .to_owned()
}

fn fill(dir: &Path) {
    let (code, _) = update(
        dir,
        json!([{"op": "add", "path": "/name", "value": "a"}, {"op": "add", "path": "/items", "value": ["x"]}]),
    );
    assert_eq!(code, 0);
}

fn set_readonly(path: &Path, readonly: bool) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_readonly(readonly);
    fs::set_permissions(path, permissions).unwrap();
}

// ── インスタンスの作成 ・ 更新 ・ 削除と、ハッシュ

#[test]
fn create_writes_the_schema_name_and_returns_the_path() {
    let dir = workdir("create-writes-the-schema-name-and-returns-the-path");
    let (code, out) = create(&dir);
    assert_eq!(code, 0, "{out}");
    let saved: Value = serde_json::from_str(&read(&dir)).unwrap();
    assert_eq!(saved["$schema"], "example/thing@v1", "スキーマの名前を書く");
    assert_eq!(out["path"], "data/a.json");
}

#[test]
fn create_refuses_an_existing_path() {
    let dir = workdir("create-refuses-an-existing-path");
    create(&dir);
    let (code, out) = create(&dir);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "パスにインスタンスが既にある");
}

#[test]
fn update_applies_the_patch() {
    let dir = workdir("update-applies-the-patch");
    create(&dir);
    let (code, out) = update(&dir, json!([{"op": "add", "path": "/name", "value": "a"}]));
    assert_eq!(code, 0, "{out}");
    let saved: Value = serde_json::from_str(&read(&dir)).unwrap();
    assert_eq!(saved["name"], "a");
}

#[test]
fn update_refuses_a_stale_hash() {
    let dir = workdir("update-refuses-a-stale-hash");
    create(&dir);
    let stale = "0".repeat(64);
    let patch = json!([{"op": "add", "path": "/name", "value": "a"}]).to_string();
    let (code, out) = run(
        &dir,
        &[
            "update",
            "--path",
            "data/a.json",
            "--patch",
            &patch,
            "--hash",
            &stale,
        ],
    );
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "ほかの更新と競合した");
}

#[test]
fn delete_removes_the_file() {
    let dir = workdir("delete-removes-the-file");
    create(&dir);
    let (code, _) = run(&dir, &["delete", "--path", "data/a.json"]);
    assert_eq!(code, 0);
    assert!(!dir.join("data/a.json").exists());
}

#[test]
fn hash_is_the_sha256_of_the_content() {
    let dir = workdir("hash-is-the-sha256-of-the-content");
    let (_, out) = create(&dir);
    assert_eq!(out["hash"], sha256(&read(&dir)));
    let (_, out) = update(&dir, json!([{"op": "add", "path": "/name", "value": "a"}]));
    assert_eq!(out["hash"], sha256(&read(&dir)));
}

// ── インスタンスを作成する

#[test]
fn create_returns_unfilled_properties_with_their_prompt() {
    let dir = workdir("create-returns-unfilled-properties-with-their-prompt");
    let (code, out) = create(&dir);
    assert_eq!(code, 0, "{out}");
    assert!(dir.join("data/a.json").exists());
    let unfilled = out["unfilled"].as_array().unwrap();
    let properties: Vec<&str> = unfilled
        .iter()
        .map(|unfilled| unfilled["property"].as_str().unwrap())
        .collect();
    assert_eq!(properties, vec!["/items", "/name"]);
    assert_eq!(unfilled[1]["prompt"]["write"], "名前を1語で書く");
}

#[test]
fn create_from_an_unreadable_schema_fails_without_a_file() {
    let dir = workdir("create-from-an-unreadable-schema-fails-without-a-file");
    let (code, out) = run(
        &dir,
        &[
            "create",
            "--schema",
            "skills/example/assets/none.schema.json",
            "--path",
            "data/a.json",
        ],
    );
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "読めない");
    fs::write(dir.join("skills/example/assets/broken.schema.json"), "{").unwrap();
    let (code, out) = run(
        &dir,
        &[
            "create",
            "--schema",
            "skills/example/assets/broken.schema.json",
            "--path",
            "data/a.json",
        ],
    );
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "JSON として読めない");
    assert!(!dir.join("data/a.json").exists());
}

#[test]
fn create_keeps_an_existing_instance() {
    let dir = workdir("create-keeps-an-existing-instance");
    create(&dir);
    fill(&dir);
    let before = read(&dir);
    let (code, out) = create(&dir);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "パスにインスタンスが既にある");
    assert_eq!(read(&dir), before);
}

#[test]
fn create_in_an_unwritable_directory_fails() {
    let dir = workdir("create-in-an-unwritable-directory-fails");
    fs::create_dir_all(dir.join("data")).unwrap();
    set_readonly(&dir.join("data"), true);
    let (code, out) = create(&dir);
    set_readonly(&dir.join("data"), false);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "書けない");
}

// ── 値を取得する

#[test]
fn get_returns_the_value() {
    let dir = workdir("get-returns-the-value");
    create(&dir);
    fill(&dir);
    let (code, out) = run(
        &dir,
        &["get", "--path", "data/a.json", "--query", "items[0]"],
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["value"], "x");
}

#[test]
fn get_with_an_invalid_query_fails() {
    let dir = workdir("get-with-an-invalid-query-fails");
    create(&dir);
    for query in ["", "items[0"] {
        let (code, out) = run(&dir, &["get", "--path", "data/a.json", "--query", query]);
        assert_eq!(code, 1, "{query}");
        assert_eq!(out["reason"], "JMESPath 式が読めない");
    }
}

#[test]
fn get_from_an_unreadable_instance_fails() {
    let dir = workdir("get-from-an-unreadable-instance-fails");
    let (code, out) = run(
        &dir,
        &["get", "--path", "data/none.json", "--query", "name"],
    );
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "読めない");
}

#[test]
fn get_matching_nothing_returns_null() {
    let dir = workdir("get-matching-nothing-returns-null");
    create(&dir);
    let (code, out) = run(&dir, &["get", "--path", "data/a.json", "--query", "note"]);
    assert_eq!(code, 0);
    assert_eq!(out["value"], Value::Null);
    assert_eq!(out["found"], false);
}

// ── インスタンスを更新する

#[test]
fn update_returns_the_validation_result() {
    let dir = workdir("update-returns-the-validation-result");
    create(&dir);
    let (code, out) = update(&dir, json!([{"op": "add", "path": "/name", "value": "a"}]));
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["errors"], json!([]));
    assert_eq!(out["unfilled"][0]["property"], "/items");
}

#[test]
fn update_with_an_empty_patch_fails() {
    let dir = workdir("update-with-an-empty-patch-fails");
    create(&dir);
    let before = read(&dir);
    let (code, out) = run(&dir, &["update", "--path", "data/a.json", "--patch", ""]);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "JSON Patch が空である");
    assert_eq!(read(&dir), before);
}

#[test]
fn update_of_an_unreadable_instance_fails() {
    let dir = workdir("update-of-an-unreadable-instance-fails");
    fs::create_dir_all(dir.join("data")).unwrap();
    fs::write(dir.join("data/a.json"), "{").unwrap();
    let (code, out) = update(&dir, json!([{"op": "add", "path": "/name", "value": "a"}]));
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "JSON として読めない");
}

#[test]
fn update_refuses_a_patch_that_cannot_apply() {
    let dir = workdir("update-refuses-a-patch-that-cannot-apply");
    create(&dir);
    let before = read(&dir);
    let (code, out) = update(
        &dir,
        json!([{"op": "replace", "path": "/missing/x", "value": 1}]),
    );
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "JSON Patch を適用できない");
    assert_eq!(read(&dir), before);
}

#[test]
fn update_refuses_an_invalid_result() {
    let dir = workdir("update-refuses-an-invalid-result");
    create(&dir);
    let before = read(&dir);
    let (code, out) = update(&dir, json!([{"op": "add", "path": "/name", "value": ""}]));
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "検証を通過しない");
    assert_eq!(out["errors"][0]["property"], "/name");
    assert_eq!(read(&dir), before);
}

#[test]
fn update_without_a_change_ends() {
    let dir = workdir("update-without-a-change-ends");
    create(&dir);
    fill(&dir);
    let before = read(&dir);
    let (code, out) = update(
        &dir,
        json!([{"op": "replace", "path": "/name", "value": "a"}]),
    );
    assert_eq!(code, 0);
    assert_eq!(out["changed"], false);
    assert_eq!(read(&dir), before);
}

#[test]
fn update_of_an_unwritable_file_fails() {
    let dir = workdir("update-of-an-unwritable-file-fails");
    create(&dir);
    let before = read(&dir);
    set_readonly(&dir.join("data/a.json"), true);
    let (code, out) = update(&dir, json!([{"op": "add", "path": "/name", "value": "a"}]));
    set_readonly(&dir.join("data/a.json"), false);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "書けない");
    assert_eq!(read(&dir), before);
}

// ── x-prompt を受け取る

#[test]
fn prompt_returns_the_x_prompt() {
    let dir = workdir("prompt-returns-the-x-prompt");
    let (code, out) = run(&dir, &["prompt", "--schema", THING, "--property", "/name"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["prompt"]["write"], "名前を1語で書く");
}

#[test]
fn prompt_from_an_unreadable_schema_fails() {
    let dir = workdir("prompt-from-an-unreadable-schema-fails");
    let (code, out) = run(
        &dir,
        &[
            "prompt",
            "--schema",
            "skills/example/assets/none.schema.json",
            "--property",
            "/name",
        ],
    );
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "読めない");
}

#[test]
fn prompt_for_a_property_without_one_fails() {
    let dir = workdir("prompt-for-a-property-without-one-fails");
    for property in ["/note", "/missing"] {
        let (code, out) = run(&dir, &["prompt", "--schema", THING, "--property", property]);
        assert_eq!(code, 1, "{property}");
        assert_eq!(out["reason"], "x-prompt を持つプロパティがスキーマに無い");
    }
}

#[test]
fn create_refuses_a_schema_without_a_name() {
    let dir = workdir("create-refuses-a-schema-without-a-name");
    let unnamed = "skills/example/assets/unnamed.schema.json";
    fs::write(
        dir.join(unnamed),
        json!({"type": "object", "properties": {"$schema": {"type": "string"}}}).to_string(),
    )
    .unwrap();
    let (code, out) = run(
        &dir,
        &["create", "--schema", unnamed, "--path", "data/a.json"],
    );
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "スキーマの名前が無い");
    assert!(
        !dir.join("data/a.json").exists(),
        "断ったらファイルを作らない"
    );
}
