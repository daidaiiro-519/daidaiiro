//! 1回目（インスタンスの読み書き）のテスト（テストレベル system）。
//! CLI のバイナリを一時ディレクトリで実行して確かめる。テストの名前は、テスト条件の ID を含む。

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_schema-driven");

/// テストごとの一時ディレクトリ。スキーマ（$ref で隣のスキーマを指す）を置いておく。
fn workdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("schema-driven-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("schema")).unwrap();
    fs::write(
        dir.join("schema/thing.schema.json"),
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["name", "items"],
            "properties": {
                "$schema": {"type": "string"},
                "name": {"type": "string", "minLength": 1,
                         "x-prompt": {"read": "名前として読む", "write": "名前を1語で書く"}},
                "items": {"type": "array", "items": {"$ref": "common.schema.json#/$defs/item"},
                          "x-prompt": {"read": "項目の一覧", "write": "項目を1件ずつ書く"}},
                "note": {"type": "string"}
            }
        })
        .to_string(),
    )
    .unwrap();
    fs::write(
        dir.join("schema/common.schema.json"),
        json!({"$defs": {"item": {"type": "string", "minLength": 1}}}).to_string(),
    )
    .unwrap();
    dir
}

/// CLI を実行し、終了コードと標準出力の JSON を返す。
fn run(dir: &Path, args: &[&str]) -> (i32, Value) {
    let out = Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    let value = serde_json::from_str(&text).unwrap_or(Value::Null);
    (out.status.code().unwrap_or(-1), value)
}

fn create(dir: &Path) -> (i32, Value) {
    run(
        dir,
        &[
            "create",
            "--schema",
            "schema/thing.schema.json",
            "--path",
            "data/a.json",
        ],
    )
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
    let mut p = fs::metadata(path).unwrap().permissions();
    p.set_readonly(readonly);
    fs::set_permissions(path, p).unwrap();
}

// ── 集約 インスタンス（AGG-1）

#[test]
fn agg_1_cmd_1_ok_1_create_records_path_and_schema() {
    let dir = workdir("agg1-cmd1-ok1");
    let (code, out) = create(&dir);
    assert_eq!(code, 0, "{out}");
    let saved: Value = serde_json::from_str(&read(&dir)).unwrap();
    assert_eq!(
        saved["$schema"], "../schema/thing.schema.json",
        "インスタンスのファイルからの相対パス"
    );
    assert_eq!(out["path"], "data/a.json");
}

#[test]
fn agg_1_cmd_1_br_1_create_rejects_existing_path() {
    let dir = workdir("agg1-cmd1-br1");
    create(&dir);
    let (code, out) = create(&dir);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "パスにインスタンスが既にある");
}

#[test]
fn agg_1_cmd_2_ok_1_update_applies_patch() {
    let dir = workdir("agg1-cmd2-ok1");
    create(&dir);
    let (code, out) = update(&dir, json!([{"op": "add", "path": "/name", "value": "a"}]));
    assert_eq!(code, 0, "{out}");
    let saved: Value = serde_json::from_str(&read(&dir)).unwrap();
    assert_eq!(saved["name"], "a");
}

#[test]
fn agg_1_cmd_2_br_1_update_rejects_stale_hash() {
    let dir = workdir("agg1-cmd2-br1");
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
fn agg_1_cmd_3_ok_1_delete_removes_file() {
    let dir = workdir("agg1-cmd3-ok1");
    create(&dir);
    let (code, _) = run(&dir, &["delete", "--path", "data/a.json"]);
    assert_eq!(code, 0);
    assert!(!dir.join("data/a.json").exists());
}

#[test]
fn agg_1_inv_1_hash_is_sha256_of_content() {
    let dir = workdir("agg1-inv1");
    let (_, out) = create(&dir);
    assert_eq!(out["hash"], sha256(&read(&dir)));
    let (_, out) = update(&dir, json!([{"op": "add", "path": "/name", "value": "a"}]));
    assert_eq!(out["hash"], sha256(&read(&dir)));
}

// ── UC-1 インスタンスを作成する

#[test]
fn uc_1_m_create_returns_unfilled_with_prompt() {
    let dir = workdir("uc1-m");
    let (code, out) = create(&dir);
    assert_eq!(code, 0, "{out}");
    assert!(dir.join("data/a.json").exists());
    let unfilled = out["unfilled"].as_array().unwrap();
    let props: Vec<&str> = unfilled
        .iter()
        .map(|u| u["property"].as_str().unwrap())
        .collect();
    assert_eq!(props, vec!["/items", "/name"]);
    assert_eq!(unfilled[1]["prompt"]["write"], "名前を1語で書く");
}

#[test]
fn uc_1_ext_1_unreadable_schema_fails_without_file() {
    let dir = workdir("uc1-ext1");
    let (code, out) = run(
        &dir,
        &[
            "create",
            "--schema",
            "schema/none.schema.json",
            "--path",
            "data/a.json",
        ],
    );
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "読めない");
    fs::write(dir.join("schema/broken.schema.json"), "{").unwrap();
    let (code, out) = run(
        &dir,
        &[
            "create",
            "--schema",
            "schema/broken.schema.json",
            "--path",
            "data/a.json",
        ],
    );
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "JSON として読めない");
    assert!(!dir.join("data/a.json").exists());
}

#[test]
fn uc_1_ext_2_existing_instance_is_kept() {
    let dir = workdir("uc1-ext2");
    create(&dir);
    fill(&dir);
    let before = read(&dir);
    let (code, out) = create(&dir);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "パスにインスタンスが既にある");
    assert_eq!(read(&dir), before);
}

#[test]
fn uc_1_ext_3_unwritable_directory_fails() {
    let dir = workdir("uc1-ext3");
    fs::create_dir_all(dir.join("data")).unwrap();
    set_readonly(&dir.join("data"), true);
    let (code, out) = create(&dir);
    set_readonly(&dir.join("data"), false);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "書けない");
}

// ── UC-2 値を取得する

#[test]
fn uc_2_m_get_returns_value() {
    let dir = workdir("uc2-m");
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
fn uc_2_ext_1_invalid_query_fails() {
    let dir = workdir("uc2-ext1");
    create(&dir);
    for q in ["", "items[0"] {
        let (code, out) = run(&dir, &["get", "--path", "data/a.json", "--query", q]);
        assert_eq!(code, 1, "{q}");
        assert_eq!(out["reason"], "JMESPath 式が読めない");
    }
}

#[test]
fn uc_2_ext_2_unreadable_instance_fails() {
    let dir = workdir("uc2-ext2");
    let (code, out) = run(
        &dir,
        &["get", "--path", "data/none.json", "--query", "name"],
    );
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "読めない");
}

#[test]
fn uc_2_ext_3_query_matching_nothing_returns_null() {
    let dir = workdir("uc2-ext3");
    create(&dir);
    let (code, out) = run(&dir, &["get", "--path", "data/a.json", "--query", "note"]);
    assert_eq!(code, 0);
    assert_eq!(out["value"], Value::Null);
    assert_eq!(out["found"], false);
}

// ── UC-3 インスタンスを更新する

#[test]
fn uc_3_m_update_returns_validation_result() {
    let dir = workdir("uc3-m");
    create(&dir);
    let (code, out) = update(&dir, json!([{"op": "add", "path": "/name", "value": "a"}]));
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["errors"], json!([]));
    assert_eq!(out["unfilled"][0]["property"], "/items");
}

#[test]
fn uc_3_ext_1_empty_patch_fails() {
    let dir = workdir("uc3-ext1");
    create(&dir);
    let before = read(&dir);
    let (code, out) = run(&dir, &["update", "--path", "data/a.json", "--patch", ""]);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "JSON Patch が空である");
    assert_eq!(read(&dir), before);
}

#[test]
fn uc_3_ext_2_unreadable_instance_fails() {
    let dir = workdir("uc3-ext2");
    fs::create_dir_all(dir.join("data")).unwrap();
    fs::write(dir.join("data/a.json"), "{").unwrap();
    let (code, out) = update(&dir, json!([{"op": "add", "path": "/name", "value": "a"}]));
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "JSON として読めない");
}

#[test]
fn uc_3_ext_3_patch_that_cannot_apply_is_rejected() {
    let dir = workdir("uc3-ext3");
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
fn uc_3_ext_4_invalid_result_is_rejected() {
    let dir = workdir("uc3-ext4");
    create(&dir);
    let before = read(&dir);
    let (code, out) = update(&dir, json!([{"op": "add", "path": "/name", "value": ""}]));
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "検証を通過しない");
    assert_eq!(out["errors"][0]["property"], "/name");
    assert_eq!(read(&dir), before);
}

#[test]
fn uc_3_ext_5_patch_without_change_ends() {
    let dir = workdir("uc3-ext5");
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
fn uc_3_ext_6_unwritable_file_fails() {
    let dir = workdir("uc3-ext6");
    create(&dir);
    let before = read(&dir);
    set_readonly(&dir.join("data/a.json"), true);
    let (code, out) = update(&dir, json!([{"op": "add", "path": "/name", "value": "a"}]));
    set_readonly(&dir.join("data/a.json"), false);
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "書けない");
    assert_eq!(read(&dir), before);
}

// ── UC-7 x-prompt を受け取る

#[test]
fn uc_7_m_prompt_returns_x_prompt() {
    let dir = workdir("uc7-m");
    let (code, out) = run(
        &dir,
        &[
            "prompt",
            "--schema",
            "schema/thing.schema.json",
            "--property",
            "/name",
        ],
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["prompt"]["write"], "名前を1語で書く");
}

#[test]
fn uc_7_ext_1_unreadable_schema_fails() {
    let dir = workdir("uc7-ext1");
    let (code, out) = run(
        &dir,
        &[
            "prompt",
            "--schema",
            "schema/none.schema.json",
            "--property",
            "/name",
        ],
    );
    assert_eq!(code, 1);
    assert_eq!(out["reason"], "読めない");
}

#[test]
fn uc_7_ext_2_property_without_prompt_fails() {
    let dir = workdir("uc7-ext2");
    for p in ["/note", "/missing"] {
        let (code, out) = run(
            &dir,
            &[
                "prompt",
                "--schema",
                "schema/thing.schema.json",
                "--property",
                p,
            ],
        );
        assert_eq!(code, 1, "{p}");
        assert_eq!(out["reason"], "x-prompt を持つプロパティがスキーマに無い");
    }
}

// インスタンスの $schema はファイルからの相対パスなので、別のディレクトリから実行しても更新できる
#[test]
fn schema_path_is_relative_to_instance_file() {
    let dir = workdir("schema-relative");
    create(&dir);
    let patch = json!([{"op": "add", "path": "/name", "value": "a"}]).to_string();
    let (code, out) = run(
        &dir.join("data"),
        &["update", "--path", "a.json", "--patch", &patch],
    );
    assert_eq!(code, 0, "{out}");
}
