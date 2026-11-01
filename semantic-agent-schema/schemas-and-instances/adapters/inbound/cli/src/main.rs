//! sas コマンドの入口。引数を読み、adapter を組み立ててユースケースを呼び、結果を JSON の1行で返す。
//!
//! 終了コード：0 = violated と error が無い（作成 ・ 更新は書けた）／ 1 = どちらかがある、または始められなかった ・
//! 書かなかった ／ 2 = 使い方の誤り。
//!
//! 操作の記録の置き場所：環境変数 SAS_DB_PATH（db_path の上書き）→ `<git-common-dir>/sas/sas.db`（ワークツリーで1つ）→
//! Git が無ければ `.sas/sas.db`。どれもコミットしない。無ければ空で作る（作り直せるデータ）。

mod caller;
mod output;

use caller::FromArguments;
use outbound_agents::Connector;
use outbound_filesystem::{OutDir, WorkingTree};
use outbound_git::GitCommits;
use outbound_javascript::Boa;
use outbound_jsonschema::Validator;
use outbound_sqlite::SqliteLog;
use outbound_templates::Mustache;
use rendering_application::use_cases::design::{DESIGN_SYSTEM, validate_design_systems};
use rendering_application::use_cases::render::{RenderRequest, render};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use validation_application::ports::SchemaValidator;
use validation_application::use_cases::meaning::MeaningCall;
use validation_application::use_cases::validate::{
    Refused, Scope, validate, validate_stale, validate_with_meaning,
};
use writing_application::use_cases::write::{
    AdoptAs, AdoptRequest, CreateRequest, DeleteRequest, UpdateRequest, adopt, create, delete,
    update,
};

const USAGE: &str = "使い方：sas validate [--session <名前>] [--stale] (<経路>... | --all) ／ \
sas validate (<経路>... | --all) --agent <接続の定義> [--continue <検証用の子セッション>] [--session <名前>] ／ \
sas create <経路> --concrete <URN> --value <JSON> [--intent <意図>] [--session <名前>] [--request-id <ID>] ／ \
sas create <経路> --adopt [--concrete <URN>] [--intent <意図>] [--session <名前>] [--request-id <ID>] ／ \
sas update <経路> --patch <JSON Patch> [--if-match <指紋>] [--intent <意図>] [--session <名前>] [--request-id <ID>] ／ \
sas update <経路> --adopt --intent <意図> [--session <名前>] [--request-id <ID>] ／ \
sas delete <経路> --intent <意図> [--if-match <指紋>] [--session <名前>] [--request-id <ID>] ／ \
sas render <経路>... --design-system <経路> --out <書き出す先>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("validate") => run_validate(&args[1..]),
        Some("create") => run_change(&args[1..], Command::Create),
        Some("update") => run_change(&args[1..], Command::Update),
        Some("delete") => run_delete(&args[1..]),
        Some("render") => run_render(&args[1..]),
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

/// 操作の記録の置き場所。
fn db_path(root: &Path) -> PathBuf {
    if let Some(p) = std::env::var_os("SAS_DB_PATH").filter(|p| !p.is_empty()) {
        return PathBuf::from(p);
    }
    GitCommits::new(root.to_path_buf())
        .common_dir()
        .map_or_else(
            || root.join(".sas").join("sas.db"),
            |dir| dir.join("sas").join("sas.db"),
        )
}

/// 呼び出し元：--session と、ツールごとの環境変数。
fn caller(session: Option<String>) -> Result<FromArguments, Refused> {
    FromArguments::new(session, |name| std::env::var(name).ok()).map_err(|tools| Refused {
        code: "usage",
        message: format!(
            "{} の中で打たれていて、どれが打ったかを決められません。--session <打った会話の ID> で名乗ってください",
            tools.join(" と ")
        ),
        ..Refused::default()
    })
}

/// 置き場所 ・ バリデーター ・ 記録を組み立てる。
fn adapters() -> Result<(WorkingTree, Validator, SqliteLog), Refused> {
    let root = PathBuf::from(".");
    let validator = Validator::new().map_err(|message| Refused {
        code: "validator",
        message,
        ..Refused::default()
    })?;
    let log = SqliteLog::open(&db_path(&root)).map_err(|message| Refused {
        code: "log",
        message,
        ..Refused::default()
    })?;
    Ok((WorkingTree::new(root), validator, log))
}

fn run_validate(args: &[String]) -> ExitCode {
    let mut session = None;
    let mut all = false;
    let mut stale = false;
    let mut agent = None;
    let mut continue_child = None;
    let mut paths = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--all" => all = true,
            "--stale" => stale = true,
            "--agent" => match it.next() {
                Some(s) if !s.is_empty() => agent = Some(s.clone()),
                _ => return usage("--agent に接続の定義の経路がありません"),
            },
            "--continue" => match it.next() {
                Some(s) if !s.is_empty() => continue_child = Some(s.clone()),
                _ => return usage("--continue に検証用の子セッションの ID がありません"),
            },
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
        // 入力が変わった rule だけなら、経路を渡さなければすべてを見る
        (false, true) if stale => Scope::All,
        (false, true) => return usage("検証する経路か --all を渡してください"),
    };
    // デザインシステムは、受け取りの型とテンプレートの書き方の契約に照らす（描画のモジュール）
    if let Scope::Paths(paths) = &scope {
        let design: Vec<bool> = paths.iter().map(|p| is_design_system(p)).collect();
        if design.iter().any(|d| *d) {
            if !design.iter().all(|d| *d) || stale {
                return usage(
                    "デザインシステムは、ほかの対象や --stale と一緒に渡さずに、それだけで検証してください",
                );
            }
            let (store, validator, mut log) = match adapters() {
                Ok(a) => a,
                Err(r) => return refuse(&r),
            };
            let who = match caller(session) {
                Ok(w) => w,
                Err(r) => return refuse(&r),
            };
            return match validate_design_systems(
                paths, &store, &validator, &Mustache, &Boa, &who, &mut log,
            ) {
                Ok(v) => {
                    output::print_design_validation(&v);
                    if v.counts.fails() {
                        ExitCode::from(1)
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Err(r) => refuse(&r),
            };
        }
    }
    if continue_child.is_some() && agent.is_none() {
        return usage("--continue は --agent と一緒に渡してください");
    }
    if agent.is_some() && stale {
        return usage("--agent と --stale は一緒に渡せません");
    }
    if let Some(agent) = agent {
        return run_validate_with_agent(&scope, &agent, continue_child, session);
    }
    let run = if stale { validate_stale } else { validate };
    let (store, validator, mut log) = match adapters() {
        Ok(a) => a,
        Err(r) => return refuse(&r),
    };
    let who = match caller(session) {
        Ok(w) => w,
        Err(r) => return refuse(&r),
    };
    match run(&scope, &store, &validator, &who, &mut log) {
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

/// by: ai の rule を、接続の定義のエージェントの検証用の子セッションに判定させる検証。
fn run_validate_with_agent(
    scope: &Scope,
    agent: &str,
    continue_child: Option<String>,
    session: Option<String>,
) -> ExitCode {
    let (store, validator, mut log) = match adapters() {
        Ok(a) => a,
        Err(r) => return refuse(&r),
    };
    let who = match caller(session) {
        Ok(w) => w,
        Err(r) => return refuse(&r),
    };
    let def: Value = match std::fs::read_to_string(agent)
        .map_err(|e| format!("{agent} を読めない：{e}"))
        .and_then(|t| serde_json::from_str(&t).map_err(|e| format!("{agent} が JSON でない：{e}")))
    {
        Ok(v) => v,
        Err(message) => {
            return refuse(&Refused {
                code: "invalid_input",
                path: Some(agent.into()),
                message,
                ..Refused::default()
            });
        }
    };
    match validator.check(AGENT_CONNECTOR, &def, &[]) {
        Ok(broken) if broken.is_empty() => {}
        Ok(broken) => {
            return refuse(&Refused {
                code: "invalid_input",
                path: Some(agent.into()),
                message: format!(
                    "接続の定義が型に合わない：{}",
                    broken
                        .iter()
                        .map(|(at, why)| format!("{at} {why}"))
                        .collect::<Vec<_>>()
                        .join(" ／ ")
                ),
                ..Refused::default()
            });
        }
        Err(message) => {
            return refuse(&Refused {
                code: "validator",
                message,
                ..Refused::default()
            });
        }
    }
    let root = PathBuf::from(".");
    let work = db_path(&root)
        .parent()
        .map_or_else(|| root.join(".sas"), Path::to_path_buf);
    let reader = Connector::new(def, caller::session_envs(), work);
    let call = MeaningCall {
        reader: &reader,
        continue_child,
    };
    match validate_with_meaning(scope, &store, &validator, &who, &mut log, &call) {
        Ok((v, meaning)) => {
            if let Some(m) = &meaning {
                output::print_procedures(m);
            }
            output::print_validation_with(&v, meaning.as_ref());
            if v.counts.fails() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(r) => refuse(&r),
    }
}

/// エージェントの接続の定義の型。
const AGENT_CONNECTOR: &str = "urn:semantic-agent-schema:published-language:agent-connector:v1";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Command {
    Create,
    Update,
}

fn run_change(args: &[String], command: Command) -> ExitCode {
    // --adopt は値を取らない旗なので、先に抜き出す
    let adopting = args.iter().any(|a| a == "--adopt");
    let args: Vec<String> = args.iter().filter(|a| *a != "--adopt").cloned().collect();
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
    if adopting {
        // 取り込みはファイルを書かないので、書く中身（--value ・ --patch）と前提の指紋（--if-match）を受けない
        if let Some(name) = ["--value", "--patch", "--if-match"]
            .into_iter()
            .find(|n| take(n).is_some())
        {
            return usage(&format!("--adopt と {name} は一緒に渡せません"));
        }
        if command == Command::Update && intent.is_none() {
            return usage(
                "--adopt で更新を取り込むときは、外で直した意図を --intent で渡してください",
            );
        }
        let request = AdoptRequest {
            path: path.clone(),
            as_kind: match command {
                Command::Create => AdoptAs::Create,
                Command::Update => AdoptAs::Update,
            },
            concrete: take("--concrete"),
            intent,
            request_id,
        };
        let committed = GitCommits::new(PathBuf::from("."));
        let result = run_with(
            |store, validator, who, log| adopt(&request, store, validator, &committed, who, log),
            session,
        );
        return match result {
            Ok(c) => {
                output::print_change(&c);
                ExitCode::SUCCESS
            }
            Err(r) => refuse(&r),
        };
    }
    let json = |name: &str| -> Result<Value, ExitCode> {
        let text = take(name).ok_or_else(|| usage(&format!("{name} を渡してください")))?;
        serde_json::from_str(&text).map_err(|e| {
            refuse(&Refused {
                code: "invalid_input",
                path: Some(path.as_str().into()),
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
                |store, validator, who, log| create(&request, store, validator, who, log),
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
                |store, validator, who, log| update(&request, store, validator, who, log),
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

fn run_delete(args: &[String]) -> ExitCode {
    let mut flags: Vec<(&str, Option<String>)> = vec![
        ("--intent", None),
        ("--if-match", None),
        ("--session", None),
        ("--request-id", None),
    ];
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
    // 消すと中身は記録の事前の状態にしか残らないので、なぜ消すかを必ず残す
    let Some(intent) = take("--intent") else {
        return usage("消すときは、なぜ消すかを --intent で渡してください");
    };
    let request = DeleteRequest {
        path: path.clone(),
        if_match: take("--if-match"),
        request_id: take("--request-id"),
        intent: Some(intent),
    };
    match run_with(
        |store, _, who, log| delete(&request, store, who, log),
        take("--session"),
    ) {
        Ok(d) => {
            output::print_deletion(&d);
            ExitCode::SUCCESS
        }
        Err(r) => refuse(&r),
    }
}

fn run_render(args: &[String]) -> ExitCode {
    let mut design_system = None;
    let mut out = None;
    let mut targets = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let slot = match a.as_str() {
            "--design-system" => &mut design_system,
            "--out" => &mut out,
            s if s.starts_with("--") => return usage(&format!("知らない指定です: {s}")),
            p => {
                targets.push(p.to_string());
                continue;
            }
        };
        match it.next() {
            Some(v) if !v.is_empty() => *slot = Some(v.clone()),
            _ => return usage(&format!("{a} に値がありません")),
        }
    }
    if targets.is_empty() {
        return usage("描く対象の経路を1つ以上渡してください");
    }
    let Some(design_system) = design_system else {
        return usage(
            "--design-system でデザインシステムのファイルを渡してください（SAS は既定を持たない）",
        );
    };
    let Some(out) = out else {
        return usage("--out で書き出す先を渡してください");
    };
    let root = PathBuf::from(".");
    let validator = match Validator::new() {
        Ok(v) => v,
        Err(message) => {
            return refuse(&Refused {
                code: "validator",
                message,
                ..Refused::default()
            });
        }
    };
    let request = RenderRequest {
        targets,
        design_system,
        out,
    };
    match render(
        &request,
        &WorkingTree::new(root.clone()),
        &validator,
        &Mustache,
        &Boa,
        &OutDir::new(root),
    ) {
        Ok(r) => {
            output::print_render(&r);
            ExitCode::SUCCESS
        }
        Err(r) => refuse(&r),
    }
}

/// 経路のファイルが、デザインシステムの受け取りの型を申告しているか。
fn is_design_system(path: &str) -> bool {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| {
            v.get("$schema")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .is_some_and(|s| s == DESIGN_SYSTEM)
}

fn run_with<T>(
    f: impl FnOnce(&WorkingTree, &Validator, &FromArguments, &mut SqliteLog) -> Result<T, Refused>,
    session: Option<String>,
) -> Result<T, Refused> {
    let (store, validator, mut log) = adapters()?;
    f(&store, &validator, &caller(session)?, &mut log)
}

fn refuse(r: &Refused) -> ExitCode {
    output::print_refused(r);
    ExitCode::from(if r.code == "usage" { 2 } else { 1 })
}
