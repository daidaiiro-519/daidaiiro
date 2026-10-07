//! ブレストボードの CLI。転写した schema-driven のツールを、ボードの Design で使う。

use serde_json::Value;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_brainstorming-board");
const REFERENCES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../references");

fn run(args: &[&str]) -> (i32, Value) {
    let out = Command::new(BIN)
        .current_dir(std::env::temp_dir())
        .args(args)
        .output()
        .unwrap();
    let value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    (out.status.code().unwrap_or(-1), value)
}

#[test]
fn json_without_a_tool_name_lists_the_tools() {
    let (code, out) = run(&["--json"]);
    assert_eq!(code, 0, "{out}");
    let names: Vec<&str> = out["data"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    for name in [
        "create", "update", "check", "approve", "render", "init", "inspect", "migrate", "serve",
    ] {
        assert!(names.contains(&name), "{name} が無い：{names:?}");
    }
}

#[test]
fn an_unknown_option_is_refused_with_exit_code_2() {
    let (code, _) = run(&["check", "--nope", "x"]);
    assert_eq!(code, 2);
}

#[test]
fn render_draws_a_board_with_the_board_design() {
    let dir = std::env::temp_dir().join(format!("bb-cli-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(
        format!("{REFERENCES}/board.schema.json"),
        dir.join("board.schema.json"),
    )
    .unwrap();
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../board/tests/fixtures/board.json"
        ),
        dir.join("board.json"),
    )
    .unwrap();
    let dir_text = dir.to_string_lossy().into_owned();
    let out_text = dir.join("out").to_string_lossy().into_owned();
    let pages = format!("{REFERENCES}/pages");
    let (code, out) = run(&[
        "render", "--dir", &dir_text, "--pages", &pages, "--out", &out_text,
    ]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out["pages"][0]["design"], "concrete");
    let page = std::fs::read_to_string(dir.join("out/board.html")).unwrap();
    assert!(page.contains("id=\"tabs\""));
}
