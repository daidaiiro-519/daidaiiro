//! CLI の実行ファイルを端から端まで確かめる。基盤のエンジンと AI の CLI を使わないものだけを置く。

mod support;

use serde_json::json;
use support::{cli, Scratch};

#[test]
fn version_answers_the_name_and_the_version() {
    let (code, out) = cli(&["--version"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(
        out["data"],
        json!({"name": "brainstorming", "version": env!("CARGO_PKG_VERSION")})
    );
}

#[test]
fn no_arguments_list_own_commands_and_base_operations() {
    let (code, out) = cli(&["--json"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["data"]["commands"].as_array().map(Vec::len), Some(6));
    assert_eq!(out["data"]["base"][2], "update");
}

#[test]
fn an_unknown_command_ends_with_exit_code_2() {
    let (code, out) = cli(&["build"]);
    assert_eq!(code, 2, "{out}");
    assert_eq!(out["detail"], "知らない操作: build");
}

#[test]
fn inspect_reads_a_board_from_disk_and_exits_1_on_findings() {
    let scratch = Scratch::new("inspect");
    scratch.put(".brainstorming/b.json", r#"{"id":"b","status":"answer","queue":[],"topics":[{"id":"Q1","status":"open","answer":{"text":"a"}}]}"#);
    let board = scratch.path(".brainstorming/b.json");
    let (code, out) = cli(&["inspect", "--board", board.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(
        out["findings"][0],
        "回答待ちの論点があるのに、queue に いま見る論点が無い"
    );
}

#[test]
fn a_base_operation_without_the_engine_says_so() {
    let (code, out) = cli(&["check", "--dir", "x"]);
    assert_eq!(code, 1, "{out}");
    assert!(out["reason"]
        .as_str()
        .unwrap_or("")
        .starts_with("基盤のエンジンが無い"));
}
