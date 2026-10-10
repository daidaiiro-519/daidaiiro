//! abstract スキーマの版ごとの試験データ（tool/contract/<版>/*.json）を実行する。
//! 扱う全部の版の試験データに合格することを、エンジンを公開する条件にする。

mod common;

use common::{sha256_of, stub};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn contract_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../contract")
}

/// 試験データ1件を一時ディレクトリで実行し、合わなかった点を返す。
fn run_case(version: &str, case: &Value) -> Vec<String> {
    let id = case["id"].as_str().unwrap_or("?");
    let dir = common::Scratch::new(&format!("sd-contract-{version}-{id}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    // Windows では実行ファイルに .exe を付ける
    let stub_at = case["stub"]
        .as_str()
        .map(|at| format!("{at}{}", std::env::consts::EXE_SUFFIX));
    let mut stub_sha = String::new();
    if let Some(at) = &stub_at {
        let target = dir.join(at);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(stub(), &target).unwrap();
        stub_sha = sha256_of(&target);
    }
    if let Some(files) = case["files"].as_object() {
        for (path, content) in files {
            let content: Value = serde_json::from_str(
                &content
                    .to_string()
                    .replace("$STUB_SHA256", &stub_sha)
                    .replace("$STUB", stub_at.as_deref().unwrap_or("")),
            )
            .unwrap();
            let target = dir.join(path);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(&target, serde_json::to_string_pretty(&content).unwrap()).unwrap();
        }
    }
    let mut command = Command::new(env!("CARGO_BIN_EXE_schema-driven"));
    command.current_dir(&dir);
    if case["skill"].as_bool() == Some(true) {
        command.args(["--skill", "."]);
    }
    command.arg(case["tool"].as_str().unwrap());
    if let Some(args) = case["args"].as_object() {
        for (name, value) in args {
            command
                .arg(format!("--{name}"))
                .arg(value.as_str().unwrap());
        }
    }
    let output = command.output().unwrap();
    let _ = fs::remove_dir_all(&dir);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut problems = Vec::new();
    let Ok(out) = serde_json::from_str::<Value>(&stdout) else {
        return vec![format!("{version}/{id}：出力が JSON でない：{stdout}")];
    };
    let code = output.status.code().unwrap_or(-1);
    if Some(code as i64) != case["expect"]["code"].as_i64() {
        problems.push(format!(
            "{version}/{id}：終了コードが {code}（期待 {}）：{out}",
            case["expect"]["code"]
        ));
    }
    for (pointer, expected) in case["expect"]["fields"].as_object().unwrap() {
        let actual = out.pointer(pointer);
        if actual != Some(expected) {
            problems.push(format!(
                "{version}/{id}：{pointer} が {}（期待 {expected}）",
                actual
                    .map(Value::to_string)
                    .unwrap_or_else(|| "無い".into())
            ));
        }
    }
    problems
}

#[test]
fn every_version_keeps_its_contract() {
    let mut versions: Vec<_> = fs::read_dir(contract_dir())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .collect();
    versions.sort_by_key(|entry| entry.file_name());
    assert!(!versions.is_empty(), "版の試験データが1つも無い");
    let (mut count, mut problems) = (0, Vec::new());
    for version in versions {
        let name = version.file_name().to_string_lossy().into_owned();
        let mut cases: Vec<_> = fs::read_dir(version.path())
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        cases.sort();
        for path in cases {
            let case: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
            problems.extend(run_case(&name, &case));
            count += 1;
        }
    }
    assert!(count > 0, "試験データが1件も無い");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
