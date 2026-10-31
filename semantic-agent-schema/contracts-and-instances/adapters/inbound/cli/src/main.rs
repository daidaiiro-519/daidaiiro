//! sas コマンドの入口。引数を読み、adapter を組み立ててユースケースを呼び、結果を JSON の1行で返す。
//!
//! 終了コード：0 = violated と error が無い ／ 1 = どちらかがある、または検証を始められなかった ／ 2 = 使い方の誤り。

mod caller;
mod output;

use caller::FromArguments;
use contracts_and_instances_adapters_outbound_files::WorkingTree;
use contracts_and_instances_adapters_outbound_jsonschema::Checker;
use contracts_and_instances_adapters_outbound_sqlite::SqliteLog;
use contracts_and_instances_conformance_application::use_cases::documents::{
    Refused, Scope, validate,
};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "使い方：sas validate [--session <名前>] (<経路>... | --all)";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("validate") => run_validate(&args[1..]),
        Some(other) => {
            eprintln!("sas: 知らないコマンドです: {other}。{USAGE}");
            ExitCode::from(2)
        }
        None => {
            eprintln!("sas: コマンドを指定してください。{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn usage(message: &str) -> ExitCode {
    output::print_refused(&Refused {
        code: "usage",
        path: None,
        message: format!("{message}。{USAGE}"),
    });
    ExitCode::from(2)
}

fn run_validate(args: &[String]) -> ExitCode {
    let mut session = None;
    let mut all = false;
    let mut paths = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--all" => all = true,
            "--session" => match it.next() {
                Some(s) if !s.is_empty() => session = Some(s.clone()),
                _ => return usage("--session に名前がありません"),
            },
            s if s.starts_with("--") => return usage(&format!("知らない指定です: {s}")),
            p => paths.push(p.to_string()),
        }
    }
    let scope = match (all, paths.is_empty()) {
        (true, true) => Scope::All,
        (false, false) => Scope::Paths(paths),
        (true, false) => return usage("経路と --all は一緒に渡せません"),
        (false, true) => return usage("検証する経路か --all を渡してください"),
    };
    let root = PathBuf::from(".");
    let store = WorkingTree::new(root.clone());
    let checker = match Checker::new() {
        Ok(c) => c,
        Err(message) => {
            return refuse(&Refused {
                code: "checker",
                path: None,
                message,
            });
        }
    };
    let mut log = match SqliteLog::open(&root.join(".sas").join("sas.db")) {
        Ok(l) => l,
        Err(message) => {
            return refuse(&Refused {
                code: "log",
                path: None,
                message,
            });
        }
    };
    match validate(
        &scope,
        &store,
        &checker,
        &FromArguments::new(session),
        &mut log,
    ) {
        Ok(v) => {
            output::print_validation(&v);
            if v.counts.fails() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(r) => refuse(&r),
    }
}

fn refuse(r: &Refused) -> ExitCode {
    output::print_refused(r);
    ExitCode::from(1)
}
