//! MCP の実行ファイルを端から端まで確かめる。実行ファイルを起動し、rmcp のクライアントで一覧を取り、ツールを呼ぶ。

mod support;

use serde_json::json;
use support::{cli, eq, expect, mcp, Scratch};

/// スキーマ t を置いた作業用のフォルダ。
fn workdir(name: &str) -> Scratch {
    let scratch = Scratch::new(&format!("mcp-{name}"));
    scratch.put(
        "schema/t.schema.json",
        &json!({"$id": "example/t@v1", "type": "object", "required": ["name"], "properties": {"$schema": {"type": "string"},
               "name": {"type": "string", "x-prompt": {"read": "r", "write": "名前を書く"}}}}),
    );
    scratch
}

#[test]
fn mcp_lists_the_common_tools() {
    let scratch = workdir("list");
    let out = mcp(&scratch, &[]);
    expect(
        "MCP",
        &out,
        &[(
            "/tools",
            eq(json!([
                "create",
                "get",
                "update",
                "delete",
                "prompt",
                "check",
                "check-schemas",
                "approve",
                "render"
            ])),
        )],
    );
}

#[test]
fn mcp_creates_an_instance_at_a_path_relative_to_where_it_started() {
    let scratch = workdir("create");
    let out = mcp(
        &scratch,
        &[(
            "create",
            json!({"schema": "schema/t.schema.json", "path": "data/a.json"}),
        )],
    );
    expect(
        "MCP",
        &out,
        &[
            ("/calls/0/isError", eq(false)),
            ("/calls/0/body/path", eq("data/a.json")),
            ("/calls/0/body/unfilled/0/prompt/write", eq("名前を書く")),
        ],
    );
    assert!(scratch.exists("data/a.json"));
}

#[test]
fn mcp_reports_a_failed_tool_as_an_error_result() {
    let scratch = workdir("fail");
    let out = mcp(
        &scratch,
        &[("get", json!({"path": "data/none.json", "query": "name"}))],
    );
    expect(
        "MCP",
        &out,
        &[
            ("/calls/0/isError", eq(true)),
            ("/calls/0/body/ok", eq(false)),
            ("/calls/0/body/reason", eq("読めない")),
        ],
    );
}

#[test]
fn mcp_and_cli_answer_the_same_version() {
    let scratch = workdir("version");
    let (code, from_cli) = cli(&scratch, &["--version"]);
    assert_eq!(code, 0, "{from_cli}");
    let out = mcp(&scratch, &[]);
    expect(
        "MCP",
        &out,
        &[("/version", eq(from_cli["data"]["version"].clone()))],
    );
}
