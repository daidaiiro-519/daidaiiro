//! 固有のツールの宣言（Skill のフォルダの tools.json）と、別の実行ファイルを呼ぶ境界。
//! 見本の実行ファイル（schema-driven-stub）で、境界の違反を1つずつ起こして確かめる。

use serde_json::{json, Value};
mod common;

use common::{sha256_of, stub};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

const COMMON: [&str; 9] = [
    "create",
    "get",
    "update",
    "delete",
    "prompt",
    "check",
    "check-schemas",
    "approve",
    "render",
];

/// 見本の実行ファイルを bin/ に置いた Skill のフォルダを作る。
fn skill(name: &str, tools: Value) -> common::Scratch {
    let dir = common::Scratch::new(&format!("sd-declared-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("bin")).unwrap();
    let exe = dir
        .join("bin")
        .join(format!("stub{}", std::env::consts::EXE_SUFFIX));
    fs::copy(stub(), &exe).unwrap();
    let digest = sha256_of(&exe);
    let tools: Value = serde_json::from_str(
        &tools
            .to_string()
            .replace("$EXE", &format!("bin/stub{}", std::env::consts::EXE_SUFFIX))
            .replace("$SHA", &digest),
    )
    .unwrap();
    fs::write(
        dir.join("tools.json"),
        serde_json::to_string_pretty(&json!({
            "$schema": "schema-driven/tools@v1", "kind": "tools", "tools": tools
        }))
        .unwrap(),
    )
    .unwrap();
    dir
}

fn tool(name: &str, mode: &str) -> Value {
    json!({"name": name, "description": "見本", "args": [{"name": "word", "description": "語", "required": true}],
           "command": ["$EXE", mode], "sha256": "$SHA"})
}

fn run(skill: &Path, args: &[&str]) -> (i32, Value) {
    run_with_env(skill, args, &[])
}

fn run_with_env(skill: &Path, args: &[&str], env: &[(&str, &str)]) -> (i32, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_schema-driven"))
        .arg("--skill")
        .arg(skill)
        .args(args)
        .envs(env.iter().copied())
        .output()
        .unwrap();
    let out = serde_json::from_slice(&output.stdout).unwrap_or_else(
        |_| json!({"unparsed": String::from_utf8_lossy(&output.stdout).into_owned()}),
    );
    (output.status.code().unwrap_or(-1), out)
}

#[test]
fn declared_tools_follow_the_common_tools_in_the_catalog() {
    let dir = skill("catalog", json!([tool("echo", "echo")]));
    let (code, out) = run(&dir, &["--json"]);
    assert_eq!(code, 0, "{out}");
    let names: Vec<&str> = out["data"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    let mut expected: Vec<&str> = COMMON.to_vec();
    expected.push("echo");
    assert_eq!(names, expected);
}

#[test]
fn a_declared_tool_gets_its_args_on_stdin() {
    let dir = skill("echo", json!([tool("echo", "echo")]));
    let (code, out) = run(&dir, &["echo", "--word", "語"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(
        out["request"],
        json!({"tool": "echo", "args": {"word": "語"}})
    );
}

#[test]
fn a_missing_required_arg_is_refused_before_running() {
    let dir = skill("missing", json!([tool("echo", "echo")]));
    let (code, out) = run(&dir, &["echo"]);
    assert_eq!(code, 2, "{out}");
}

#[test]
fn a_changed_executable_is_not_run() {
    let mut declared = tool("echo", "echo");
    declared["sha256"] = json!("0".repeat(64));
    let dir = skill("sha", json!([declared]));
    let (code, out) = run(&dir, &["echo", "--word", "語"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "実行ファイルの SHA-256 が宣言と合わない");
}

#[test]
fn a_command_outside_the_skill_is_refused() {
    let mut declared = tool("echo", "echo");
    declared["command"] = json!(["../outside", "echo"]);
    let dir = skill("outside", json!([declared]));
    let (code, out) = run(&dir, &["echo", "--word", "語"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "宣言が契約に合わない");
}

#[test]
fn a_name_that_clashes_with_a_common_tool_is_refused() {
    let dir = skill("clash", json!([tool("check", "echo")]));
    let (code, out) = run(&dir, &["--json"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "宣言が契約に合わない");
}

#[test]
fn a_slow_tool_is_stopped_at_its_time_limit() {
    let mut declared = tool("slow", "slow");
    declared["timeout_seconds"] = json!(1);
    let dir = skill("slow", json!([declared]));
    let started = Instant::now();
    let (code, out) = run(&dir, &["slow", "--word", "語"]);
    assert!(started.elapsed() < Duration::from_secs(4), "止まらなかった");
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "制限時間を超えた");
}

#[test]
fn output_that_is_not_json_is_a_failure() {
    let dir = skill("not-json", json!([tool("noisy", "not-json")]));
    let (code, out) = run(&dir, &["noisy", "--word", "語"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "出力が契約に合わない");
}

#[test]
fn output_without_ok_is_a_failure() {
    let dir = skill("no-ok", json!([tool("bare", "no-ok")]));
    let (code, out) = run(&dir, &["bare", "--word", "語"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "出力が契約に合わない");
}

#[test]
fn output_over_one_mebibyte_is_a_failure() {
    let dir = skill("big", json!([tool("big", "big")]));
    let (code, out) = run(&dir, &["big", "--word", "語"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "出力が大きすぎる");
}

#[test]
fn a_failing_tool_passes_its_own_result_through() {
    let dir = skill("fail", json!([tool("fail", "fail")]));
    let (code, out) = run(&dir, &["fail", "--word", "語"]);
    assert_eq!(code, 1);
    assert_eq!(out, json!({"ok": false, "reason": "見本の失敗"}));
}

#[test]
fn an_unknown_exit_code_becomes_a_failure() {
    let dir = skill("exit-7", json!([tool("seven", "exit-7")]));
    let (code, out) = run(&dir, &["seven", "--word", "語"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["reason"], "終了コードが契約に合わない");
}

#[test]
fn only_declared_environment_variables_reach_the_tool() {
    let mut declared = tool("echo", "echo");
    declared["env"] = json!(["SD_ALLOWED"]);
    let dir = skill("env", json!([declared]));
    let (code, out) = run_with_env(
        &dir,
        &["echo", "--word", "語"],
        &[("SD_ALLOWED", "見える"), ("SD_SECRET", "見えない")],
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["allowed"], "見える");
    assert_eq!(out["secret"], Value::Null);
}

#[test]
fn without_a_declaration_file_only_common_tools_are_listed() {
    let dir = common::Scratch::new("sd-declared-none");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let (code, out) = run(&dir, &["--json"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["data"]["tools"].as_array().unwrap().len(), COMMON.len());
}
