//! 起動時の結合（outbound を application へ渡す1か所）

use crate::commands::mechanical::{Mechanical, parse};
use semantic_agent_schema_application::use_cases::{Outcome, create, patch, search, validate};
use semantic_agent_schema_fs::files::Files;
use serde_json::json;
use std::process::ExitCode;

/// 引数を受け取って1つのコマンドを動かし、終了コードを返す（0 正常 ・ 1 検出あり ・ 2 使い方の誤り）
pub fn run(args: &[String]) -> ExitCode {
    let parsed = match parse(args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("使い方の誤り：{e}");
            return ExitCode::from(2);
        }
    };
    let root = std::env::current_dir().unwrap_or_default();
    let files = Files::new(root);
    let outcome = match &parsed.command {
        Mechanical::Create {
            path,
            owner,
            kind,
            value,
        } => create::create(&files, path, owner, kind, value),
        Mechanical::Get { path, pointer } => search::get(&files, path, pointer),
        Mechanical::CreateInstance {
            path,
            schema,
            id,
            value,
        } => {
            let id = id.clone().unwrap_or_else(new_id);
            create::create_instance(&files, &files, path, schema, &id, value)
        }
        Mechanical::Patch { path, ops } => patch::patch(&files, &files, path, ops),
        Mechanical::Validate { path } => validate::validate(&files, &files, path),
    };
    print(&outcome, parsed.json);
    if outcome.ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

/// インスタンスの id を決める（時刻とプロセスの番号から。時刻を読むのは inbound だけ）
fn new_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    format!("i{nanos:x}{:x}", std::process::id())
}

/// MCP の面を起動する（段5で書く。いまは使い方の誤りを返す）
pub fn run_mcp() -> ExitCode {
    eprintln!("MCP の面はまだ無い");
    ExitCode::from(2)
}

fn print(outcome: &Outcome, as_json: bool) {
    if as_json {
        let findings: Vec<_> = outcome.findings.iter().map(|l| l.to_json()).collect();
        println!(
            "{}",
            json!({"ok": outcome.ok, "findings": findings, "data": outcome.data})
        );
        return;
    }
    for l in &outcome.findings {
        println!("{}\t{}\t{}\t{}", l.result, l.rule, l.pointer, l.reason);
    }
    println!("{}", if outcome.ok { "ok" } else { "ng" });
}
