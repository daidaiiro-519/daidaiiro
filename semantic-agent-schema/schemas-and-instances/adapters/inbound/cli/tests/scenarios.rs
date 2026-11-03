//! システムテスト：system-tests/scenarios/ の場面を1つずつ、空の作業ディレクトリで semantic-agent-schema を動かして確かめる。
//! ネットワークと本物のエージェントは使わない。given の git だけは準備として本物の git を使う。
//! 作業ディレクトリは、given の repo が false でなければ Git のリポジトリにしてから始める（記録は Git の中だけで作る）。
//! given の write は、SAS を通らずにファイルを書く（エディターで直したことの代わり）。

#![allow(clippy::expect_used, clippy::panic)]

use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn suite() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../system-tests")
}

/// semantic-agent-schema が返す結果の形（出力のたびに照らす）。
fn result_schema() -> jsonschema::Validator {
    published("command-result.schema.json")
}

/// 手順の形（意味の部分を含む操作で、結果の行の代わりに出る）。
fn procedure_schema() -> jsonschema::Validator {
    published("procedure.schema.json")
}

fn published(name: &str) -> jsonschema::Validator {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../published-language/v1")
        .join(name);
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
        std::fs::create_dir_all(to.parent().expect("親のフォルダがある"))
            .expect("フォルダを作れること");
        std::fs::write(to, text).expect("ファイルを置けること");
    }
}

/// セッション ID が入る環境変数の名前（入口の表 callers.json から）。
fn session_envs() -> Vec<String> {
    let table: Value = serde_json::from_str(
        &std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("callers.json"))
            .expect("callers.json を読めること"),
    )
    .expect("callers.json が JSON であること");
    table["tools"]
        .as_array()
        .expect("tools がある")
        .iter()
        .map(|t| {
            t["session_env"]
                .as_str()
                .expect("session_env は文字列")
                .to_string()
        })
        .collect()
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
        let mut c = Command::new(env!("CARGO_BIN_EXE_semantic-agent-schema"));
        c.args(args);
        // 試験を動かす環境（AI のツールの中など）のセッション ID と記録の置き場所を持ち込まない。
        // 場面が env で渡したものだけを使う
        for name in session_envs() {
            c.env_remove(name);
        }
        c.env_remove("SEMANTIC_AGENT_SCHEMA_DATABASE_FILE");
        if let Some(env) = cmd.get("env").and_then(Value::as_object) {
            for (k, v) in env {
                c.env(k, v.as_str().expect("環境変数の値は文字列"));
            }
        }
        c
    } else {
        let mut c = Command::new("git");
        c.args(strings("git").expect("args か git がある"));
        c
    };
    c.current_dir(cwd);
    let Some(input) = cmd.get("stdin").and_then(Value::as_str) else {
        return c.output().expect("コマンドを起動できること");
    };
    // 標準入力に渡す（--answer - など）
    let mut child = c
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("コマンドを起動できること");
    child
        .stdin
        .take()
        .expect("標準入力がある")
        .write_all(input.as_bytes())
        .expect("標準入力に書けること");
    child.wait_with_output().expect("終わりを待てること")
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
    // 出力は1行1つの JSON。どの行も結果の形に従い、見るのは line の行（無ければ最後の行）
    let lines: Vec<Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str(l.trim())
                .unwrap_or_else(|e| panic!("{id} {label}: 出力が JSON でない（{e}）: {stdout}"))
        })
        .collect();
    for line in &lines {
        let (schema, name) = if line.get("kind").and_then(Value::as_str) == Some("procedure") {
            (procedure_schema(), "procedure.schema.json")
        } else {
            (result_schema(), "command-result.schema.json")
        };
        let errors: Vec<String> = schema.iter_errors(line).map(|e| e.to_string()).collect();
        assert!(
            errors.is_empty(),
            "{id} {label}: 出力が {name} に従わない {errors:?}"
        );
    }
    if let Some(n) = expect.get("lines").and_then(Value::as_u64) {
        assert_eq!(
            lines.len(),
            usize::try_from(n).unwrap_or_default(),
            "{id} {label}: 出力の行の数。{stdout}"
        );
    }
    let json = match expect.get("line").and_then(Value::as_u64) {
        Some(n) => lines
            .get(usize::try_from(n).unwrap_or_default())
            .cloned()
            .unwrap_or_else(|| panic!("{id} {label}: {n} 行目が無い。{stdout}")),
        None => lines
            .last()
            .cloned()
            .unwrap_or_else(|| panic!("{id} {label}: 出力が無い")),
    };
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

/// 書き出したファイル（作業ディレクトリからの経路）の有無と中身。すべてのコマンドのあとで見る。
fn check_file(id: &str, dir: &Path, file: &str, expect: &Value) {
    let text = std::fs::read_to_string(dir.join(file)).ok();
    if let Some(exists) = expect.get("exists").and_then(Value::as_bool) {
        // 記録のデータベースのような、文字列で読めないファイルもあるので、有無は経路で見る
        assert_eq!(dir.join(file).is_file(), exists, "{id}: {file} の有無");
    }
    if let Some(want) = expect.get("contains").and_then(Value::as_str) {
        let text = text
            .as_deref()
            .unwrap_or_else(|| panic!("{id}: {file} が無い"));
        assert!(text.contains(want), "{id}: {file} に {want} が無い。{text}");
    }
    if let Some(other) = expect.get("same_as").and_then(Value::as_str) {
        assert_eq!(
            std::fs::read(dir.join(file)).ok(),
            std::fs::read(dir.join(other)).ok(),
            "{id}: {file} と {other} のバイト列が違う"
        );
    }
    if let Some(no) = expect.get("lacks").and_then(Value::as_str) {
        let text = text
            .as_deref()
            .unwrap_or_else(|| panic!("{id}: {file} が無い"));
        assert!(!text.contains(no), "{id}: {file} に {no} がある。{text}");
    }
}

fn play(scenario: &Value) {
    let id = scenario["id"].as_str().expect("id がある");
    let dir = workdir(id);
    let given = scenario.get("given").cloned().unwrap_or_default();
    if given.get("repo").and_then(Value::as_bool) != Some(false) {
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
        // SAS を通らない書き込み（エディターで直したことの代わり）
        if let Some(w) = step.get("write") {
            let to = dir.join(w["path"].as_str().expect("path は文字列"));
            if let Some(parent) = to.parent() {
                std::fs::create_dir_all(parent).expect("フォルダを作れること");
            }
            std::fs::write(to, w["content"].as_str().expect("content は文字列"))
                .expect("ファイルを書けること");
            continue;
        }
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
        if let Some(file) = expect.get("file").and_then(Value::as_str) {
            check_file(id, &dir, file, expect);
            continue;
        }
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
