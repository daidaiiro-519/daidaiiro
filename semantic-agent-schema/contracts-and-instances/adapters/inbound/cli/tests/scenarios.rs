//! システムテスト：test-suite/scenarios/ の場面を1つずつ、空の作業ディレクトリで sas を動かして確かめる。
//! ネットワークと本物のエージェントは使わない。given の git だけは準備として本物の git を使う。

#![allow(clippy::expect_used, clippy::panic)]

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn suite() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../test-suite")
}

/// sas が返す結果の形（出力のたびに照らす）。
fn result_schema() -> jsonschema::Validator {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../schemas/v1/operation-result.schema.json");
    let schema: Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("結果の形を読めること"))
            .expect("結果の形が JSON であること");
    jsonschema::validator_for(&schema).expect("結果の形を組めること")
}

fn workdir(id: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sas-scenario-{}-{id}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("前の作業ディレクトリを消せること");
    }
    std::fs::create_dir_all(&dir).expect("作業ディレクトリを作れること");
    dir
}

fn place(dir: &Path, files: &serde_json::Map<String, Value>) {
    for (rel, content) in files {
        let content = content.as_str().expect("中身は文字列");
        let text = match content.strip_prefix("@fixtures/") {
            Some(name) => std::fs::read_to_string(suite().join("fixtures").join(name))
                .unwrap_or_else(|e| panic!("{name} を読めない: {e}")),
            None => content.to_string(),
        };
        let to = dir.join(rel);
        std::fs::create_dir_all(to.parent().expect("親がある")).expect("フォルダを作れること");
        std::fs::write(to, text).expect("ファイルを置けること");
    }
}

fn run(dir: &Path, cmd: &Value) -> Output {
    let cwd = cmd
        .get("cwd")
        .and_then(Value::as_str)
        .map_or_else(|| dir.to_path_buf(), |c| dir.join(c));
    let strings = |k: &str| -> Option<Vec<String>> {
        cmd.get(k).and_then(Value::as_array).map(|a| {
            a.iter()
                .map(|s| s.as_str().expect("引数は文字列").to_string())
                .collect()
        })
    };
    let mut c = if let Some(args) = strings("args") {
        let mut c = Command::new(env!("CARGO_BIN_EXE_sas"));
        c.args(args);
        c
    } else {
        let mut c = Command::new("git");
        c.args(strings("git").expect("args か git がある"));
        c
    };
    c.current_dir(cwd)
        .output()
        .expect("コマンドを起動できること")
}

fn check(id: &str, label: &str, out: &Output, expect: &Value) {
    let stdout = String::from_utf8_lossy(&out.stdout);
    if let Some(code) = expect.get("exit").and_then(Value::as_i64) {
        assert_eq!(
            i64::from(out.status.code().unwrap_or(-1)),
            code,
            "{id} {label}: 終了コード。stdout={stdout} stderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let Some(pointer) = expect.get("pointer").and_then(Value::as_str) else {
        return;
    };
    let json: Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("{id} {label}: 出力が JSON でない（{e}）: {stdout}"));
    let errors: Vec<String> = result_schema()
        .iter_errors(&json)
        .map(|e| e.to_string())
        .collect();
    assert!(
        errors.is_empty(),
        "{id} {label}: 出力が operation-result.schema.json に従わない {errors:?}"
    );
    let got = json.pointer(pointer);
    if let Some(exists) = expect.get("exists").and_then(Value::as_bool) {
        assert_eq!(
            got.is_some(),
            exists,
            "{id} {label}: {pointer} の有無。{json}"
        );
    }
    if let Some(want) = expect.get("equals") {
        assert_eq!(got, Some(want), "{id} {label}: {pointer}。{json}");
    }
    if let Some(want) = expect.get("contains").and_then(Value::as_str) {
        let text = got.map(Value::to_string).unwrap_or_default();
        assert!(
            text.contains(want),
            "{id} {label}: {pointer} に {want} が無い。{json}"
        );
    }
    if let Some(want) = expect.get("count").and_then(Value::as_u64) {
        let n = got.and_then(Value::as_array).map(Vec::len);
        assert_eq!(
            n,
            usize::try_from(want).ok(),
            "{id} {label}: {pointer} の件数。{json}"
        );
    }
}

fn play(scenario: &Value) {
    let id = scenario["id"].as_str().expect("id がある");
    let dir = workdir(id);
    let given = scenario.get("given").cloned().unwrap_or_default();
    if given.get("repo").and_then(Value::as_bool) == Some(true) {
        run(&dir, &serde_json::json!({"git": ["init", "-q"]}));
    }
    if let Some(files) = given.get("files").and_then(Value::as_object) {
        place(&dir, files);
    }
    for step in given
        .get("steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let out = run(&dir, step);
        assert!(
            out.status.success(),
            "{id}: 前提のコマンドが失敗した {step}"
        );
    }
    let unchanged: Vec<(String, Vec<u8>)> = scenario
        .get("unchanged")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|p| {
            (
                p.to_string(),
                std::fs::read(dir.join(p)).expect("ファイルがある"),
            )
        })
        .collect();
    let when = run(&dir, &scenario["when"]);
    let after: Vec<Output> = scenario
        .get("after")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|c| run(&dir, c))
        .collect();
    for expect in scenario["then"].as_array().expect("then がある") {
        let of = expect["of"].as_str().expect("of がある");
        let out = if of == "when" {
            &when
        } else {
            let n: usize = of
                .trim_start_matches("after[")
                .trim_end_matches(']')
                .parse()
                .expect("after の番号");
            &after[n]
        };
        check(id, of, out, expect);
    }
    for (p, before) in unchanged {
        assert_eq!(
            std::fs::read(dir.join(&p)).expect("ファイルがある"),
            before,
            "{id}: {p} が変わった"
        );
    }
    std::fs::remove_dir_all(&dir).expect("作業ディレクトリを消せること");
}

#[test]
fn scenarios_pass() {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(suite().join("scenarios"))
        .expect("場面の置き場所を読めること")
        .map(|e| e.expect("項目を読めること").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "場面が1つも無い");
    let mut failed = Vec::new();
    for path in paths {
        let scenario: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("場面を読めること"))
                .expect("場面が JSON であること");
        let id = scenario["id"].as_str().unwrap_or_default().to_string();
        if std::panic::catch_unwind(|| play(&scenario)).is_err() {
            failed.push(id);
        }
    }
    assert!(failed.is_empty(), "通らなかった場面: {failed:?}");
}
