//! システムテスト：追跡の検査を通したあと、test-suite/scenarios/ のシナリオを1件ずつ、
//! 一時ディレクトリで実行ファイル semantic-agent-schema を動かして確かめる。
//! シナリオの形は test-suite/scenario.schema.json。

mod trace;

use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

/// ワークスペースの根（semantic-agent-schema/）
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// 1つのコマンドを実行した結果
struct Ran {
    exit: i32,
    stdout: String,
}

fn main() -> ExitCode {
    let root = root();
    let mut failed = 0usize;

    let problems = trace::check(&root);
    if problems.is_empty() {
        println!("ok   追跡の検査");
    } else {
        failed += 1;
        println!("NG   追跡の検査（{}件）", problems.len());
        for p in &problems {
            println!("     - {p}");
        }
    }

    let scenarios = match list_scenarios(&root) {
        Ok(s) => s,
        Err(e) => {
            println!("NG   シナリオを読めない：{e}");
            return ExitCode::FAILURE;
        }
    };
    for path in &scenarios {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        match run_scenario(&root, path) {
            Ok(()) => println!("ok   {name}"),
            Err(e) => {
                failed += 1;
                println!("NG   {name}：{e}");
            }
        }
    }
    println!("シナリオ {} 件 ・ 不合格 {} 件", scenarios.len(), failed);
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn list_scenarios(root: &Path) -> Result<Vec<PathBuf>, String> {
    let dir = root.join("test-suite/scenarios");
    let mut out = Vec::new();
    for e in fs::read_dir(&dir).map_err(|e| format!("{}：{e}", dir.display()))? {
        let p = e.map_err(|e| e.to_string())?.path();
        if p.extension().is_some_and(|x| x == "json") {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}

/// シナリオの形を確かめる（scenario.schema.json の必須の欄と、知らない欄）
fn check_shape(s: &Value) -> Result<(), String> {
    let obj = s.as_object().ok_or("シナリオがオブジェクトでない")?;
    for k in ["id", "name", "title", "why", "traces", "when", "then"] {
        if !obj.contains_key(k) {
            return Err(format!("必須の欄 {k} が無い"));
        }
    }
    let known = [
        "$schema",
        "id",
        "name",
        "title",
        "why",
        "traces",
        "given",
        "when",
        "after",
        "then",
        "unchanged",
    ];
    if let Some(k) = obj.keys().find(|k| !known.contains(&k.as_str())) {
        return Err(format!("知らない欄 {k}"));
    }
    Ok(())
}

fn args_of(cmd: &Value) -> Result<Vec<String>, String> {
    cmd.get("args")
        .and_then(Value::as_array)
        .ok_or("コマンドに args が無い")?
        .iter()
        .map(|a| {
            a.as_str()
                .map(str::to_string)
                .ok_or_else(|| "args が文字列でない".to_string())
        })
        .collect()
}

/// コマンドを実行する。args は semantic-agent-schema に、git は Git に渡す（前提のリポジトリを作る ・ 取り込む）
fn run(work: &Path, cmd: &Value) -> Result<Ran, String> {
    let (program, args) = match cmd.get("git") {
        Some(g) => ("git", args_of(&serde_json::json!({ "args": g }))?),
        None => (env!("CARGO_BIN_EXE_semantic-agent-schema"), args_of(cmd)?),
    };
    let dir = match cmd.get("cwd").and_then(Value::as_str) {
        Some(c) => work.join(c),
        None => work.to_path_buf(),
    };
    let out = Command::new(program)
        .args(&args)
        .current_dir(&dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .env("HOME", work.join(".home"))
        .env("XDG_CONFIG_HOME", work.join(".home/config"))
        .env("XDG_DATA_HOME", work.join(".home/data"))
        .env("XDG_STATE_HOME", work.join(".home/state"))
        .env("XDG_CACHE_HOME", work.join(".home/cache"))
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("起動できない：{e}"))?;
    Ok(Ran {
        exit: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
    })
}

fn run_scenario(root: &Path, path: &Path) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let s: Value = serde_json::from_str(&text).map_err(|e| format!("JSON でない：{e}"))?;
    check_shape(&s)?;
    let id = s["id"].as_str().unwrap_or("S-?");
    let work = std::env::temp_dir().join(format!("sas-system-{}-{}", id, std::process::id()));
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(work.join(".home")).map_err(|e| e.to_string())?;
    let result = run_in(root, &work, &s);
    let _ = fs::remove_dir_all(&work);
    result
}

fn run_in(root: &Path, work: &Path, s: &Value) -> Result<(), String> {
    if s.pointer("/given/repo").and_then(Value::as_bool) == Some(true) {
        let r = run(
            work,
            &serde_json::json!({"git": ["init", "-q", "-b", "main"]}),
        )?;
        if r.exit != 0 {
            return Err("準備の git init が失敗した".into());
        }
    }
    if let Some(files) = s.pointer("/given/files").and_then(Value::as_object) {
        for (rel, body) in files {
            let body = body.as_str().ok_or("given.files の中身が文字列でない")?;
            let text = match body.strip_prefix("@fixtures/") {
                Some(f) => fs::read_to_string(root.join("test-suite/fixtures").join(f))
                    .map_err(|e| format!("{f}：{e}"))?,
                None => body.to_string(),
            };
            let dest = work.join(rel);
            if let Some(dir) = dest.parent() {
                fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            fs::write(&dest, text).map_err(|e| e.to_string())?;
        }
    }
    for step in s
        .pointer("/given/steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let r = run(work, step)?;
        if r.exit != 0 {
            return Err(format!("前提のコマンド {step} が終了コード {}", r.exit));
        }
    }
    let watched: Vec<String> = s
        .get("unchanged")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    let before: BTreeMap<String, Option<Vec<u8>>> = watched
        .iter()
        .map(|f| (f.clone(), fs::read(work.join(f)).ok()))
        .collect();
    let when = run(work, &s["when"])?;
    let mut after = Vec::new();
    for cmd in s
        .get("after")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        after.push(run(work, cmd)?);
    }
    for f in &watched {
        if before.get(f).cloned().flatten() != fs::read(work.join(f)).ok() {
            return Err(format!("変わってはいけない {f} が変わった"));
        }
    }
    for (n, e) in s["then"]
        .as_array()
        .ok_or("then が配列でない")?
        .iter()
        .enumerate()
    {
        check_expect(n, e, &when, &after)?;
    }
    Ok(())
}

fn check_expect(n: usize, e: &Value, when: &Ran, after: &[Ran]) -> Result<(), String> {
    let of = e["of"].as_str().ok_or("then の of が無い")?;
    let r = if of == "when" {
        when
    } else {
        let i: usize = of
            .trim_start_matches("after[")
            .trim_end_matches(']')
            .parse()
            .map_err(|_| format!("of が読めない {of}"))?;
        after.get(i).ok_or(format!("after[{i}] が無い"))?
    };
    if let Some(x) = e.get("exit").and_then(Value::as_i64)
        && i64::from(r.exit) != x
    {
        return Err(format!("then[{n}]：終了コードが {} （期待 {x}）", r.exit));
    }
    if let Some(ptr) = e.get("pointer").and_then(Value::as_str) {
        let doc: Value = serde_json::from_str(&r.stdout)
            .map_err(|_| format!("then[{n}]：出力が JSON でない"))?;
        let got = doc.pointer(ptr);
        if let Some(x) = e.get("exists").and_then(Value::as_bool)
            && got.is_some() != x
        {
            return Err(format!(
                "then[{n}]：{ptr} が在るか {}（期待 {x}）",
                got.is_some()
            ));
        }
        if let Some(x) = e.get("equals")
            && got != Some(x)
        {
            return Err(format!("then[{n}]：{ptr} が {:?}（期待 {x}）", got));
        }
        if let Some(x) = e.get("contains").and_then(Value::as_str)
            && !got.map(Value::to_string).unwrap_or_default().contains(x)
        {
            return Err(format!("then[{n}]：{ptr} に「{x}」が無い"));
        }
        if let Some(x) = e.get("count").and_then(Value::as_u64)
            && got.and_then(Value::as_array).map(Vec::len) != usize::try_from(x).ok()
        {
            return Err(format!("then[{n}]：{ptr} の数が違う（期待 {x}）"));
        }
    }
    Ok(())
}
