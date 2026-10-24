//! 依存の向きの検査ツール dependency-rules の入口。
//! 引数を読み、規則とグラフを公開の形から core の型へ直して判定し、出力と終了コードを決める。
//! 終了コード：0 違反なし ／ 1 違反あり ／ 2 規則かグラフが読めない ・ 使い方の誤り

mod cargo;
mod core;
mod graph;

use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

const HELP: &str = "依存の向きの検査ツール

使い方:
  dependency-rules check --rules <規則.json> (--cargo <workspace の根> | --graph <ファイル|->) [--json]
  dependency-rules graph --cargo <workspace の根>
  dependency-rules --help
  dependency-rules --version

終了コード: 0 違反なし ／ 1 違反あり ／ 2 規則かグラフが読めない ・ 使い方の誤り";

enum Source {
    Cargo(PathBuf),
    Graph(String),
}

struct Args {
    command: String,
    rules: Option<String>,
    source: Option<Source>,
    json: bool,
}

fn parse(argv: &[String]) -> Result<Args, String> {
    let mut it = argv.iter();
    let command = it.next().ok_or("操作（check か graph）が無い")?.clone();
    let mut a = Args {
        command,
        rules: None,
        source: None,
        json: false,
    };
    while let Some(flag) = it.next() {
        let mut value = || it.next().cloned().ok_or(format!("{flag} の値が無い"));
        match flag.as_str() {
            "--rules" => a.rules = Some(value()?),
            "--cargo" if a.source.is_none() => a.source = Some(Source::Cargo(value()?.into())),
            "--graph" if a.source.is_none() => a.source = Some(Source::Graph(value()?)),
            "--cargo" | "--graph" => return Err("--cargo と --graph はどちらか1つ".into()),
            "--json" => a.json = true,
            other => return Err(format!("知らない旗：{other}")),
        }
    }
    Ok(a)
}

fn read_text(path: &str) -> Result<String, String> {
    if path == "-" {
        let mut s = String::new();
        std::io::stdin()
            .read_to_string(&mut s)
            .map_err(|e| format!("標準入力を読めない：{e}"))?;
        Ok(s)
    } else {
        std::fs::read_to_string(path).map_err(|e| format!("{path} を読めない：{e}"))
    }
}

fn load_graph(source: &Source) -> Result<core::Graph, String> {
    match source {
        Source::Cargo(root) => cargo::to_graph(&cargo::metadata(root)?),
        Source::Graph(path) => graph::parse_graph(&read_text(path)?),
    }
}

fn print_text(vs: &[core::Violation]) {
    if vs.is_empty() {
        println!("違反なし");
        return;
    }
    println!("違反 {}件\n", vs.len());
    for v in vs {
        let to =
            v.to.as_deref()
                .map(|t| format!(" -> {t}"))
                .unwrap_or_default();
        let kind = v
            .kind
            .as_deref()
            .map(|k| format!("  ({k})"))
            .unwrap_or_default();
        println!(
            "{:<15} {}{}{}",
            graph::reason_name(v.reason),
            v.from,
            to,
            kind
        );
        let c = |x: &Option<String>| {
            x.clone()
                .unwrap_or_else(|| "（どの component にも入らない）".into())
        };
        match (v.reason, &v.to) {
            (core::Reason::NotInAllowed, _) => println!(
                "  component: {} -> {}   allow に無い",
                c(&v.from_component),
                c(&v.to_component)
            ),
            (core::Reason::Uncovered, Some(_)) => println!(
                "  component: {} -> {}",
                c(&v.from_component),
                c(&v.to_component)
            ),
            (core::Reason::Uncovered, None) => println!("  どの component にも入らない"),
        }
    }
}

fn run(argv: &[String]) -> Result<ExitCode, String> {
    if argv.first().map(String::as_str) == Some("--help") {
        println!("{HELP}");
        return Ok(ExitCode::SUCCESS);
    }
    if argv.first().map(String::as_str) == Some("--version") {
        println!("dependency-rules {}", env!("CARGO_PKG_VERSION"));
        return Ok(ExitCode::SUCCESS);
    }
    let a = parse(argv)?;
    match a.command.as_str() {
        "graph" => {
            let Some(src @ Source::Cargo(_)) = &a.source else {
                return Err("graph には --cargo が要る".into());
            };
            let g = load_graph(src)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&graph::graph_doc(&g)).map_err(|e| e.to_string())?
            );
            Ok(ExitCode::SUCCESS)
        }
        "check" => {
            let rules_path = a.rules.as_deref().ok_or("check には --rules が要る")?;
            let src = a
                .source
                .as_ref()
                .ok_or("check には --cargo か --graph が要る")?;
            let rules = graph::parse_rules(&read_text(rules_path)?)?;
            let g = load_graph(src)?;
            let vs = core::check(&g, &rules).map_err(|es| {
                es.iter()
                    .map(|e| format!("規則の誤り：{e}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            })?;
            if a.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&graph::result_doc(&vs))
                        .map_err(|e| e.to_string())?
                );
            } else {
                print_text(&vs);
            }
            Ok(if vs.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        other => Err(format!("知らない操作：{other}")),
    }
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    match run(&argv) {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("{msg}\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}
