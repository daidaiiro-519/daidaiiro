//! MCP の受け口のテスト。実行ファイルを起動し、標準入出力で JSON-RPC を送って確かめる。

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_schema-driven-mcp");

/// 試験の作業用のフォルダ。実行ごとに別の名前で作り、試験が終わると消す。
struct Scratch(std::path::PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }
}

impl std::ops::Deref for Scratch {
    type Target = std::path::Path;
    fn deref(&self) -> &std::path::Path {
        &self.0
    }
}

impl AsRef<std::path::Path> for Scratch {
    fn as_ref(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn call(lines: &[Value]) -> Vec<Value> {
    let dir = Scratch::new("schema-driven-mcp");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("schema")).unwrap();
    std::fs::write(
        dir.join("schema/t.schema.json"),
        json!({"type": "object", "required": ["name"], "properties": {"$schema": {"type": "string"},
               "name": {"type": "string", "x-prompt": {"read": "r", "write": "名前を書く"}}}})
        .to_string(),
    )
    .unwrap();
    let mut child = Command::new(BIN)
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut out = BufReader::new(child.stdout.take().unwrap());
    let mut answers = Vec::new();
    for line in lines {
        writeln!(stdin, "{line}").unwrap();
        stdin.flush().unwrap();
        if line.get("id").is_some() {
            let mut buf = String::new();
            out.read_line(&mut buf).unwrap();
            answers.push(serde_json::from_str(&buf).unwrap());
        }
    }
    drop(stdin);
    let _ = child.wait();
    answers
}

#[test]
fn mcp_lists_tools_and_creates_instance() {
    let answers = call(&[
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "test", "version": "0"}}}),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
        json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
            "name": "create", "arguments": {"schema": "schema/t.schema.json", "path": "data/a.json"}}}),
    ]);
    let names: Vec<&str> = answers[1]["result"]["tools"]
        .as_array()
        .unwrap()
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
            "render"
        ]
    );
    let result = &answers[2]["result"];
    assert_eq!(result["isError"], false, "{result}");
    let body: Value = serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(body["unfilled"][0]["prompt"]["write"], "名前を書く");
}

#[test]
fn mcp_lists_declared_tools_after_the_common_tools() {
    let dir = Scratch::new("schema-driven-mcp-skill");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("tools.json"),
        json!({"$schema": "schema-driven/tools@v1", "kind": "tools", "tools": [
            {"name": "aggregate", "description": "集計する", "args": [{"name": "dir", "description": "ディレクトリ", "required": true}],
             "command": ["bin/aggregate"], "sha256": "0".repeat(64)}]})
        .to_string(),
    )
    .unwrap();
    let mut child = Command::new(BIN)
        .arg("--skill")
        .arg(&*dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut out = BufReader::new(child.stdout.take().unwrap());
    let mut answers: Vec<Value> = Vec::new();
    for line in [
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "test", "version": "0"}}}),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
    ] {
        writeln!(stdin, "{line}").unwrap();
        stdin.flush().unwrap();
        if line.get("id").is_some() {
            let mut buf = String::new();
            out.read_line(&mut buf).unwrap();
            answers.push(serde_json::from_str(&buf).unwrap());
        }
    }
    drop(stdin);
    let _ = child.wait();
    let names: Vec<&str> = answers[1]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert_eq!(names.last(), Some(&"aggregate"));
    assert_eq!(names.len(), 10);
}
