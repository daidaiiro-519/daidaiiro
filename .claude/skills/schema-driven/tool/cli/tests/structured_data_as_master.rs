//! UC-0 構造化データを正本として持つ（要約レベル。テストレベル system）。
//! 基盤の CLI だけで、作成（UC-1）→ x-prompt を受け取る（UC-7）→ 更新（UC-3）→ 描画（UC-6）→ 承認（UC-8）を通す。
//! 具体のデザインは無いので、基盤の文書の契約（references/document.schema.json）で書く。

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_schema-driven");
const SCHEMA: &str = "declarations/document.schema.json";
const INSTANCE: &str = "declarations/D-1.json";

fn workdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("schema-driven-uc0-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("declarations")).unwrap();
    fs::write(
        dir.join(SCHEMA),
        include_str!("../../../references/document.schema.json"),
    )
    .unwrap();
    dir
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

fn properties(list: &Value) -> Vec<String> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|unfilled| unfilled["property"].as_str().unwrap().to_owned())
        .collect()
}

/// 未記入の題と節を埋める JSON Patch。
fn fill_patch() -> String {
    json!([
        {"op": "add", "path": "/kind", "value": "document"},
        {"op": "add", "path": "/title", "value": "注文 API の応答遅延の調査報告"},
        {"op": "add", "path": "/sections", "value": [
            {"heading": "起きたこと", "blocks": [{"type": "paragraph", "text": "応答が最大 **8.2 秒**まで遅れた。"}]}
        ]}
    ])
    .to_string()
}

#[test]
fn uc_0_sg_1_the_owner_approves_a_master_written_from_prompts() {
    let dir = workdir("success");
    // STEP-1 UC-1 作成する：必須のプロパティは未記入として返る
    let (code, created) = run(&dir, &["create", "--schema", SCHEMA, "--path", INSTANCE]);
    assert_eq!(code, 0, "{created}");
    let unfilled = properties(&created["unfilled"]);
    assert!(
        unfilled.contains(&"/title".to_owned()) && unfilled.contains(&"/sections".to_owned()),
        "{created}"
    );
    // STEP-2 UC-7 未記入のプロパティの x-prompt を受け取る
    let (code, prompted) = run(
        &dir,
        &["prompt", "--schema", SCHEMA, "--property", "/sections"],
    );
    assert_eq!(code, 0, "{prompted}");
    assert!(!prompted["prompt"].is_null(), "{prompted}");
    // STEP-3 UC-3 更新する：書いたものをその場で確かめ、未記入が無くなる
    let (code, updated) = run(
        &dir,
        &["update", "--path", INSTANCE, "--patch", &fill_patch()],
    );
    assert_eq!(code, 0, "{updated}");
    assert_eq!(updated["errors"], json!([]), "{updated}");
    assert_eq!(updated["unfilled"], json!([]), "{updated}");
    // STEP-4 UC-6 描画して、データの持ち主が読む
    let (code, rendered) = run(&dir, &["render", "--dir", "declarations", "--out", "out"]);
    assert_eq!(code, 0, "{rendered}");
    let page = fs::read_to_string(dir.join("out/D-1.html")).unwrap();
    assert!(
        page.contains("注文 API の応答遅延の調査報告") && page.contains("<strong>8.2 秒</strong>"),
        "{page}"
    );
    // STEP-5 UC-8 承認する：承認記録が空でない（SG-1）
    let (code, approved) = run(&dir, &["approve", "--dir", "declarations"]);
    assert_eq!(code, 0, "{approved}");
    let paths: Vec<&str> = approved["instances"]
        .as_array()
        .unwrap()
        .iter()
        .map(|instance| instance["path"].as_str().unwrap())
        .collect();
    assert_eq!(paths, vec![INSTANCE], "{approved}");
    assert!(dir.join("declarations/approval.json").exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn uc_0_mg_1_a_master_with_validation_errors_is_neither_rendered_nor_approved() {
    let dir = workdir("minimal");
    let (code, _) = run(&dir, &["create", "--schema", SCHEMA, "--path", INSTANCE]);
    assert_eq!(code, 0);
    run(
        &dir,
        &["update", "--path", INSTANCE, "--patch", &fill_patch()],
    );
    // 更新の道を通らずにファイルを書き換え、形の違う値を入れる
    let mut instance: Value =
        serde_json::from_str(&fs::read_to_string(dir.join(INSTANCE)).unwrap()).unwrap();
    instance["title"] = json!(1);
    fs::write(dir.join(INSTANCE), instance.to_string()).unwrap();
    let (code, rendered) = run(&dir, &["render", "--dir", "declarations", "--out", "out"]);
    assert_ne!(code, 0, "{rendered}");
    assert!(!dir.join("out/D-1.html").exists());
    // 承認した正本は検証を通過している（MG-1）：検証エラーがあれば承認記録を書かない
    let (code, approved) = run(&dir, &["approve", "--dir", "declarations"]);
    assert_ne!(code, 0, "{approved}");
    assert!(!dir.join("declarations/approval.json").exists());
    fs::remove_dir_all(&dir).unwrap();
}
