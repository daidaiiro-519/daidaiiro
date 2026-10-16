//! CLI の実行ファイルを端から端まで確かめる。基盤のエンジンと gh を使わないものだけを置く。

mod support;

use serde_json::json;
use support::{cli, results, Scratch};

#[test]
fn version_answers_the_name_and_the_version() {
    let (code, out) = cli(&["--version"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(
        out["data"],
        json!({"name": "skills-creator", "version": env!("CARGO_PKG_VERSION")})
    );
}

#[test]
fn no_arguments_list_the_three_commands() {
    let (code, out) = cli(&["--json"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["data"]["commands"][0]["name"], "scaffold");
    assert_eq!(out["data"]["commands"][1]["name"], "engine");
    assert_eq!(out["data"]["commands"][2]["name"], "check");
}

#[test]
fn an_unknown_command_ends_with_exit_code_2() {
    let (code, out) = cli(&["build", "x"]);
    assert_eq!(code, 2, "{out}");
    assert_eq!(out["detail"], "知らない操作: build");
}

#[test]
fn scaffold_refuses_a_folder_that_already_exists() {
    let scratch = Scratch::new("refuse");
    scratch.put("practice/SKILL.md", "mine");
    let dir = scratch.path("practice");
    let (code, out) = cli(&["scaffold", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(out["ok"], false);
    assert_eq!(scratch.read("practice/SKILL.md"), "mine");
}

#[test]
fn check_of_an_empty_folder_fails_and_says_why() {
    let scratch = Scratch::new("empty");
    scratch.put("practice/.keep", "");
    let dir = scratch.path("practice");
    let (code, out) = cli(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    let got = results(&out);
    assert_eq!(
        &got[..3],
        &[
            ("skill.frontmatter".to_owned(), "fail".to_owned()),
            ("skill.type".to_owned(), "fail".to_owned()),
            ("skill.placeholders".to_owned(), "fail".to_owned()),
        ]
    );
    assert_eq!(got[5], ("engine.runs".to_owned(), "fail".to_owned()));
}
