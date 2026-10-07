// SPDX-License-Identifier: MIT
//! MCP サーバーを起動して、道具の一覧と resolve の3つの結果を確かめる。
//!
//! CLI の終了コード 0 ・ 1 ・ 2 に当たるものは、MCP では結果の欄で表す ── 0 は検出が空、
//! 1 は検出あり（誤りではない）、2 は `isError: true`。
//!
//!     cargo test -p ds_mcp

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

fn file(name: &str, body: &str) -> String {
    let dir = std::env::temp_dir().join(format!("ds_mcp_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("作れる");
    let p = dir.join(name);
    std::fs::write(&p, body).expect("書ける");
    p.display().to_string()
}

/// サーバーへ要求を順に送り、id の付いた応答を並べて返す。
fn exchange(requests: &[Value]) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_design-svg-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("起動できる");
    let mut stdin = child.stdin.take().expect("標準入力");
    let mut lines = BufReader::new(child.stdout.take().expect("標準出力")).lines();
    let mut send = |v: &Value| {
        writeln!(stdin, "{v}").expect("送れる");
        stdin.flush().expect("送れる");
    };
    send(
        &json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {},
        "clientInfo": {"name": "test", "version": "0"}}}),
    );
    let _ = lines.next();
    send(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
    let mut out = Vec::new();
    for r in requests {
        send(r);
        let line = lines.next().expect("応答が在る").expect("読める");
        out.push(serde_json::from_str(&line).expect("JSON"));
    }
    drop(stdin);
    let _ = child.wait();
    out
}

fn call(id: i64, args: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "method": "tools/call",
           "params": {"name": "resolve", "arguments": args}})
}

const AUTHORED: &str = r#"<svg viewBox="0 0 300 100"><rect class="box" x="10" y="20" width="130" height="40"/><text class="label" x="75" y="44" text-anchor="middle">注文</text><path class="flow" d="M140,40 H200"/></svg>"#;

#[test]
fn the_tools_include_resolve_and_not_canvas() {
    let got = exchange(&[json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})]);
    let names: Vec<&str> = got[0]["result"]["tools"]
        .as_array()
        .expect("並び")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(names.contains(&"resolve"), "{names:?}");
    assert!(!names.contains(&"canvas"), "{names:?}");
}

#[test]
fn resolve_answers_the_three_outcomes() {
    let ok = file("ok.svg", AUTHORED);
    let found = file(
        "found.svg",
        &AUTHORED.replace(r#"<rect class="box""#, "<rect"),
    );
    let broken = file("broken.svg", "<svg><rect></svg>");
    let got = exchange(&[
        call(1, json!({"svg": ok})),
        call(2, json!({"svg": found})),
        call(3, json!({"svg": broken})),
    ]);
    let r = |i: usize| &got[i]["result"];
    // 0 ── 検出が空
    assert_eq!(r(0)["isError"], json!(false), "{}", got[0]);
    assert_eq!(r(0)["structuredContent"]["findings"], json!([]));
    // 1 ── 検出あり（誤りではない）
    assert_eq!(r(1)["isError"], json!(false), "{}", got[1]);
    assert!(r(1)["structuredContent"]["findings"]
        .as_array()
        .is_some_and(|f| !f.is_empty()));
    // 2 ── 誤用
    assert_eq!(r(2)["isError"], json!(true), "{}", got[2]);
}
