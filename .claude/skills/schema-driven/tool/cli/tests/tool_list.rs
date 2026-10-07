//! 動詞なしの `--json` で、ツールの一覧を返す（ほかの Skill の CLI と同じ形。AI エージェントはこれでツールを調べる）。
//! 出力はもともとすべて JSON なので、動詞のあとに付けた `--json` は受け付けて、何も変えない。

use serde_json::Value;
use std::path::Path;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_schema-driven");

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
fn json_without_a_tool_name_lists_every_tool_with_its_arguments() {
    let (code, out) = run(&["--json"]);
    assert_eq!(code, 0, "{out}");
    let tools = out["data"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        vec![
            "create",
            "get",
            "update",
            "delete",
            "prompt",
            "check",
            "check-schemas",
            "approve",
            "render",
            "transcribe",
            "check-copy"
        ]
    );
    let update = &tools[2];
    assert!(!update["summary"].as_str().unwrap().is_empty());
    let arguments: Vec<(&str, bool)> = update["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|argument| {
            (
                argument["name"].as_str().unwrap(),
                argument["required"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        arguments,
        vec![("path", true), ("patch", true), ("hash", false)]
    );
}

#[test]
fn json_without_a_tool_name_tells_the_skill_directory() {
    let (_, out) = run(&["--json"]);
    // 実行ファイルは <Skill>/…/<実行ファイル> にあり、テストではビルドの出力の場所になる。空でなく、実在する場所を返す
    let skill_root = out["data"]["skill_root"].as_str().unwrap();
    assert!(Path::new(skill_root).is_dir(), "{out}");
}

#[test]
fn json_after_a_tool_name_is_accepted_and_changes_nothing() {
    let (code, out) = run(&["get", "--path", "missing.json", "--query", "id", "--json"]);
    let (code_without, out_without) = run(&["get", "--path", "missing.json", "--query", "id"]);
    assert_eq!((code, out), (code_without, out_without));
}

#[test]
fn no_arguments_at_all_is_still_a_misuse() {
    let (code, out) = run(&[]);
    assert_eq!(code, 2, "{out}");
}
