//! CLI の実行ファイルを端から端まで確かめる。作業用のフォルダをいまのフォルダにして実行ファイルを起動する。

mod support;

use serde_json::json;
use support::{cli, eq, expect, Scratch};

#[test]
fn a_bare_relative_path_is_resolved_from_where_the_cli_started() {
    let scratch = Scratch::new("cli-bare-path");
    scratch.put(
        "t.schema.json",
        &json!({"$id": "example/t@v1", "type": "object", "properties": {"$schema": {"type": "string"}}}),
    );
    let (code, created) = cli(
        &scratch,
        &["create", "--schema", "t.schema.json", "--path", "w.json"],
    );
    expect(
        "create",
        &created,
        &[("/ok", eq(true)), ("/path", eq("w.json"))],
    );
    assert_eq!(code, 0, "{created}");
    assert!(scratch.exists("w.json"));
    // ディレクトリの無いパスは、いまのディレクトリにある
    let (code, deleted) = cli(&scratch, &["delete", "--path", "w.json"]);
    assert_eq!(code, 0, "{deleted}");
    expect(
        "delete",
        &deleted,
        &[("/ok", eq(true)), ("/remaining", eq(json!([])))],
    );
    assert!(!scratch.exists("w.json"));
}

#[test]
fn an_unknown_tool_ends_with_exit_code_2() {
    let scratch = Scratch::new("cli-unknown");
    let (code, out) = cli(&scratch, &["nothing"]);
    assert_eq!(code, 2, "{out}");
    expect(
        "nothing",
        &out,
        &[("/ok", eq(false)), ("/reason", eq("使い方が違う"))],
    );
}

#[test]
fn json_without_a_tool_name_lists_the_tools() {
    let scratch = Scratch::new("cli-list");
    let (code, out) = cli(&scratch, &["--json"]);
    assert_eq!(code, 0, "{out}");
    expect(
        "--json",
        &out,
        &[
            ("/ok", eq(true)),
            ("/data/tools/0/name", eq("create")),
            ("/data/tools/8/name", eq("render")),
            ("/data/tools/9", support::Is::Absent),
        ],
    );
}
