//! sas コマンドの入口。引数を読み、adapter を組み立ててユースケースを呼び、結果を JSON の1行で返す。
//!
//! 終了コード：0 = violated と error が無い（作成 ・ 更新は書けた）／ 1 = どちらかがある、または始められなかった ・
//! 書かなかった ／ 2 = 使い方の誤り。

mod caller;
mod output;

use caller::FromArguments;
use schemas_and_instances_adapters_outbound_files::WorkingTree;
use schemas_and_instances_adapters_outbound_jsonschema::Checker;
use schemas_and_instances_adapters_outbound_sqlite::SqliteLog;
use schemas_and_instances_conformance_application::use_cases::documents::{
    ChangeResult, CreateRequest, Refused, Scope, UpdateRequest, create, update, validate,
};
use serde_json::Value;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "使い方：sas validate [--session <名前>] (<経路>... | --all) ／ \
sas create <経路> --concrete <URN> --value <JSON> [--intent <意図>] [--session <名前>] [--request-id <ID>] ／ \
sas update <経路> --patch <JSON Patch> [--if-match <指紋>] [--intent <意図>] [--session <名前>] [--request-id <ID>]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("validate") => run_validate(&args[1..]),
        Some("create") => run_change(&args[1..], Command::Create),
        Some("update") => run_change(&args[1..], Command::Update),
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
        message: format!("{message}。{USAGE}"),
        ..Refused::default()
    });
    ExitCode::from(2)
}

/// 置き場所 ・ 検査器 ・ 記録を組み立てる。
fn adapters() -> Result<(WorkingTree, Checker, SqliteLog), Refused> {
    let root = PathBuf::from(".");
    let checker = Checker::new().map_err(|message| Refused {
        code: "checker",
        message,
        ..Refused::default()
    })?;
    let log = SqliteLog::open(&root.join(".sas").join("sas.db")).map_err(|message| Refused {
        code: "log",
        message,
        ..Refused::default()
    })?;
    Ok((WorkingTree::new(root), checker, log))
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
    let (store, checker, mut log) = match adapters() {
        Ok(a) => a,
        Err(r) => return refuse(&r),
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Command {
    Create,
    Update,
}

fn run_change(args: &[String], command: Command) -> ExitCode {
    let mut flags: Vec<(&str, Option<String>)> = match command {
        Command::Create => vec![("--concrete", None), ("--value", None)],
        Command::Update => vec![("--patch", None), ("--if-match", None)],
    };
    flags.extend([
        ("--intent", None),
        ("--session", None),
        ("--request-id", None),
    ]);
    let mut paths = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if let Some(slot) = flags.iter_mut().find(|(name, _)| *name == a.as_str()) {
            match it.next() {
                Some(v) if !v.is_empty() => slot.1 = Some(v.clone()),
                _ => return usage(&format!("{a} に値がありません")),
            }
        } else if a.starts_with("--") {
            return usage(&format!("知らない指定です: {a}"));
        } else {
            paths.push(a.clone());
        }
    }
    let [path] = paths.as_slice() else {
        return usage("経路を1つ渡してください");
    };
    let take = |name: &str| {
        flags
            .iter()
            .find(|(n, _)| *n == name)
            .and_then(|(_, v)| v.clone())
    };
    let session = take("--session");
    let request_id = take("--request-id");
    let intent = take("--intent");
    let json = |name: &str| -> Result<Value, ExitCode> {
        let text = take(name).ok_or_else(|| usage(&format!("{name} を渡してください")))?;
        serde_json::from_str(&text).map_err(|e| {
            refuse(&Refused {
                code: "invalid_input",
                path: Some(path.clone()),
                message: format!("{name} が JSON として読めない：{e}"),
                ..Refused::default()
            })
        })
    };
    let result = match command {
        Command::Create => {
            let Some(concrete) = take("--concrete") else {
                return usage("--concrete を渡してください");
            };
            let value = match json("--value") {
                Ok(v) => v,
                Err(code) => return code,
            };
            let request = CreateRequest {
                path: path.clone(),
                concrete,
                value,
                request_id,
                intent,
            };
            run_with(
                |store, checker, who, log| create(&request, store, checker, who, log),
                session,
            )
        }
        Command::Update => {
            let patch = match json("--patch") {
                Ok(v) => v,
                Err(code) => return code,
            };
            let request = UpdateRequest {
                path: path.clone(),
                patch,
                if_match: take("--if-match"),
                request_id,
                intent,
            };
            run_with(
                |store, checker, who, log| update(&request, store, checker, who, log),
                session,
            )
        }
    };
    match result {
        Ok(c) => {
            output::print_change(&c);
            ExitCode::SUCCESS
        }
        Err(r) => refuse(&r),
    }
}

type Change = Result<ChangeResult, Refused>;

fn run_with(
    f: impl FnOnce(&WorkingTree, &Checker, &FromArguments, &mut SqliteLog) -> Change,
    session: Option<String>,
) -> Change {
    let (store, checker, mut log) = adapters()?;
    f(&store, &checker, &FromArguments::new(session), &mut log)
}

fn refuse(r: &Refused) -> ExitCode {
    output::print_refused(r);
    ExitCode::from(1)
}
