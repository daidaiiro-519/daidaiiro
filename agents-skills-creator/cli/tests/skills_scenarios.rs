//! システムテスト（文脈 skills）：skills/system-tests/scenarios/ の場面を1つずつ、空の作業ディレクトリで実行ファイル
//! agents-skills-creator を動かして確かめる。場面の args は agents-skills-creator skills のあとに渡す。sas は本物を外の
//! 実行ファイルとして呼ぶ。ネットワークと AI は使わない。

#![allow(clippy::expect_used, clippy::panic)]

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 試験を動かす環境から持ち込まない環境変数（AI のツールのセッション ID と SAS の記録の置き場所）。
/// 持ち込むと、sas の出力のセッションが試験を動かす環境で変わる。
const NOT_INHERITED: [&str; 5] = [
    "CLAUDE_CODE_SESSION_ID",
    "CODEX_SESSION_ID",
    "COPILOT_AGENT_SESSION_ID",
    "KIRO_SESSION_ID",
    "SAS_DB_PATH",
];

fn suite() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../skills/system-tests")
}

fn published_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../skills/published-language/v1")
}

/// creator が返す結果の形（出力のたびに、出力の $schema の形に照らす）。公開された言葉の $id → 形。
fn result_schema(id: &str) -> jsonschema::Validator {
    for entry in std::fs::read_dir(published_dir()).expect("公開された言葉を読めること")
    {
        let path = entry.expect("項目を読めること").path();
        let schema: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("形を読めること"))
                .expect("形が JSON であること");
        if schema.get("$id").and_then(Value::as_str) == Some(id) {
            return jsonschema::validator_for(&schema).expect("形を組めること");
        }
    }
    panic!("出力の $schema {id} の形が公開された言葉に無い")
}

/// 場面に書いた中身（@fixtures/… ・ @published/… はファイルを読む）。
fn content_of(content: &str) -> String {
    let from = |base: PathBuf, name: &str| {
        std::fs::read_to_string(base.join(name))
            .unwrap_or_else(|e| panic!("{name} を読めない: {e}"))
    };
    if let Some(name) = content.strip_prefix("@fixtures/") {
        from(suite().join("fixtures"), name)
    } else if let Some(name) = content.strip_prefix("@published/") {
        from(published_dir(), name)
    } else {
        content.to_string()
    }
}

fn workdir(id: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("asc-scenario-{}-{id}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("前の作業ディレクトリを消せること");
    }
    std::fs::create_dir_all(&dir).expect("作業ディレクトリを作れること");
    dir
}

fn place(dir: &Path, files: &serde_json::Map<String, Value>) {
    for (rel, content) in files {
        let text = content_of(content.as_str().expect("中身は文字列"));
        let to = dir.join(rel);
        std::fs::create_dir_all(to.parent().expect("親のフォルダがある"))
            .expect("フォルダを作れること");
        std::fs::write(to, text).expect("ファイルを置けること");
    }
}

#[cfg(unix)]
fn link(dir: &Path, links: &serde_json::Map<String, Value>) {
    for (rel, target) in links {
        let at = dir.join(rel);
        std::fs::create_dir_all(at.parent().expect("親のフォルダがある"))
            .expect("フォルダを作れること");
        let target = target.as_str().expect("先は文字列");
        // @bin/<名前> は、この workspace で作った実行ファイル（agents-skills-creator と同じ置き場所）を指す
        let target = match target.strip_prefix("@bin/") {
            Some(name) => PathBuf::from(env!("CARGO_BIN_EXE_agents-skills-creator"))
                .parent()
                .expect("実行ファイルの置き場所がある")
                .join(name),
            None => PathBuf::from(target),
        };
        std::os::unix::fs::symlink(target, at).expect("リンクを置けること");
    }
}

/// 試験で使う sas の置き場所：このリポジトリの隣の SAS で作った sas があれば、それを PATH の前に足す。
/// 無ければ、試験を動かす環境の PATH のまま（そこにも無ければ sas_unavailable で止まる）。
fn path_with_sas() -> std::ffi::OsString {
    let built =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../semantic-agent-schema/target/debug");
    let mut dirs: Vec<PathBuf> = Vec::new();
    if built.join("sas").is_file() {
        dirs.push(built);
    }
    if let Some(path) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&path));
    }
    std::env::join_paths(dirs).expect("PATH を組めること")
}

fn run(dir: &Path, cmd: &Value) -> Output {
    // args は agents-skills-creator skills のあと、creator は agents-skills-creator のあと（approve ・ --help など）、
    // sas は SAS（人が承認を書くときなど）に渡す
    let (program, key) = match (cmd.get("sas"), cmd.get("creator")) {
        (Some(_), _) => ("sas".to_string(), "sas"),
        (None, Some(_)) => (
            env!("CARGO_BIN_EXE_agents-skills-creator").to_string(),
            "creator",
        ),
        (None, None) => (
            env!("CARGO_BIN_EXE_agents-skills-creator").to_string(),
            "args",
        ),
    };
    let mut args: Vec<String> = cmd[key]
        .as_array()
        .expect("args か creator か sas がある")
        .iter()
        .map(|s| s.as_str().expect("引数は文字列").to_string())
        .collect();
    if key == "args" {
        args.insert(0, "skills".to_string());
    }
    let path = path_with_sas();
    let program = if key == "sas" {
        std::env::split_paths(&path)
            .map(|d| d.join("sas"))
            .find(|p| p.is_file())
            .expect("試験に使う sas がある")
    } else {
        PathBuf::from(program)
    };
    let mut c = Command::new(program);
    c.args(args).current_dir(dir);
    c.env("PATH", path);
    for name in NOT_INHERITED {
        c.env_remove(name);
    }
    if let Some(env) = cmd.get("env").and_then(Value::as_object) {
        for (k, v) in env {
            let v = v.as_str().expect("環境変数の値は文字列");
            // 値の頭の @workdir は、場面の作業ディレクトリの経路にする
            match v.strip_prefix("@workdir") {
                Some(rest) => c.env(k, format!("{}{rest}", dir.display())),
                None => c.env(k, v),
            };
        }
    }
    c.output().expect("コマンドを起動できること")
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
    // 出力は JSON が1つ。どの出力も結果の形に従う
    let json: Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("{id} {label}: 出力が JSON 1つでない（{e}）: {stdout}"));
    // creator の出力は公開された言葉の形に照らす。sas の出力の形は SAS の試験が確かめる
    let schema_id = json["$schema"].as_str().unwrap_or_default().to_string();
    if schema_id.starts_with("urn:agents-skills-creator:") {
        let errors: Vec<String> = result_schema(&schema_id)
            .iter_errors(&json)
            .map(|e| e.to_string())
            .collect();
        assert!(
            errors.is_empty(),
            "{id} {label}: 出力が {schema_id} の形に従わない {errors:?}"
        );
    }
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
    if let Some(no) = expect.get("lacks").and_then(Value::as_str) {
        let text = got.map(Value::to_string).unwrap_or_default();
        assert!(
            !text.contains(no),
            "{id} {label}: {pointer} に {no} がある。{json}"
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

/// 作業ディレクトリのファイルの有無と中身。すべてのコマンドのあとで見る。
fn check_file(id: &str, dir: &Path, file: &str, expect: &Value) {
    let text = std::fs::read_to_string(dir.join(file)).ok();
    if let Some(exists) = expect.get("exists").and_then(Value::as_bool) {
        assert_eq!(text.is_some(), exists, "{id}: {file} の有無");
    }
    if let Some(want) = expect.get("contains").and_then(Value::as_str) {
        let text = text
            .as_deref()
            .unwrap_or_else(|| panic!("{id}: {file} が無い"));
        assert!(text.contains(want), "{id}: {file} に {want} が無い。{text}");
    }
    if let Some(no) = expect.get("lacks").and_then(Value::as_str) {
        let text = text
            .as_deref()
            .unwrap_or_else(|| panic!("{id}: {file} が無い"));
        assert!(!text.contains(no), "{id}: {file} に {no} がある。{text}");
    }
    if let Some(want) = expect.get("same_as").and_then(Value::as_str) {
        let text = text
            .as_deref()
            .unwrap_or_else(|| panic!("{id}: {file} が無い"));
        assert_eq!(text, content_of(want), "{id}: {file} が {want} と違う");
    }
}

fn play(scenario: &Value) {
    let id = scenario["id"].as_str().expect("id がある");
    let dir = workdir(id);
    let given = scenario.get("given").cloned().unwrap_or_default();
    if let Some(files) = given.get("files").and_then(Value::as_object) {
        place(&dir, files);
    }
    if let Some(links) = given.get("links").and_then(Value::as_object) {
        link(&dir, links);
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

/// 場面のファイルが、場面の形（製品の根の system-tests/scenario.schema.json。文脈で同じ形）に従う。
#[test]
fn scenarios_follow_their_shape() {
    let schema: Value = serde_json::from_str(
        &std::fs::read_to_string(suite().join("../../system-tests/scenario.schema.json"))
            .expect("場面の形を読めること"),
    )
    .expect("場面の形が JSON であること");
    let validator = jsonschema::validator_for(&schema).expect("場面の形を組めること");
    for entry in std::fs::read_dir(suite().join("scenarios")).expect("場面の置き場所を読めること")
    {
        let path = entry.expect("項目を読めること").path();
        let scenario: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("場面を読めること"))
                .expect("場面が JSON であること");
        let errors: Vec<String> = validator
            .iter_errors(&scenario)
            .map(|e| e.to_string())
            .collect();
        assert!(errors.is_empty(), "{}: {errors:?}", path.display());
    }
}
