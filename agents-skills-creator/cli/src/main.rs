//! 実行ファイル agents-skills-creator（sas と同じく PATH に置く1つのコマンド）。
//!
//! サブコマンドを振り分けるだけで、ユースケースは持たない。skills と agents の2つの文脈は分けたまま、それぞれの入口
//! （skills-inbound-cli ・ agents-inbound-cli）へ渡す。approve は、渡されたフォルダに skill.json があれば skills へ、
//! agent.json があれば agents へ渡す。
//!
//! 終了コードと出力は、渡した先の入口が決める。

use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "使い方：agents-skills-creator skills <create ・ update ・ write-skill-md ・ validate ・ distribute ・ withdraw> … ／ \
agents-skills-creator agents <create ・ update ・ write-agent-md ・ validate ・ distribute ・ withdraw> … ／ \
agents-skills-creator approve <Skill か Agent のフォルダ>（人が打つ） ／ \
agents-skills-creator <サブコマンド> --help";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest = args.get(1..).unwrap_or_default();
    match args.first().map(String::as_str) {
        Some("skills") => skills_inbound_cli::run(rest),
        Some("agents") => agents_inbound_cli::run(rest),
        Some("approve") => approve(rest),
        Some("--help" | "help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(other) => {
            println!(
                "{}",
                error(&format!("知らないサブコマンドです: {other}。{USAGE}"))
            );
            ExitCode::from(2)
        }
        None => {
            println!(
                "{}",
                error(&format!("サブコマンドを指定してください。{USAGE}"))
            );
            ExitCode::from(2)
        }
    }
}

/// approve を、フォルダの中身（skill.json か agent.json か）で振り分ける。
fn approve(rest: &[String]) -> ExitCode {
    let mut args = vec!["approve".to_string()];
    args.extend(rest.iter().cloned());
    let folder = rest.iter().find(|a| !a.starts_with("--"));
    match folder {
        Some(f)
            if rest.iter().any(|a| a == "--help") || Path::new(f).join("skill.json").is_file() =>
        {
            skills_inbound_cli::run(&args)
        }
        Some(f) if Path::new(f).join("agent.json").is_file() => agents_inbound_cli::run(&args),
        _ if rest.iter().any(|a| a == "--help") => skills_inbound_cli::run(&args),
        Some(f) => {
            println!(
                "{}",
                error(&format!(
                    "{f} に skill.json も agent.json も無い。承認する Skill か Agent のフォルダを渡してください。{USAGE}"
                ))
            );
            ExitCode::from(2)
        }
        None => {
            println!(
                "{}",
                error(&format!(
                    "承認する Skill か Agent のフォルダを渡してください。{USAGE}"
                ))
            );
            ExitCode::from(2)
        }
    }
}

/// 振り分ける前の使い方の誤り（形は skills の validate-result の error と同じ）。
fn error(message: &str) -> serde_json::Value {
    serde_json::json!({
        "$schema": "urn:agents-skills-creator:skills:validate-result:v1",
        "kind": "error",
        "code": "usage",
        "message": message,
    })
}
