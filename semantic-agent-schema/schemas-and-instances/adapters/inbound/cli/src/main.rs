//! sas コマンドの入口。引数を読み、adapter を組み立ててユースケースを呼び、結果を JSON の1行で返す。
//! 意味の部分（--semantic）は、結果の代わりに手順（procedure.schema.json）を返す。SAS はエージェントを起動しない。
//!
//! 終了コード：0 = violated と error が無い（作成 ・ 更新は書けた）／ 1 = どちらかがある、または始められなかった ・
//! 書かなかった ／ 2 = 使い方の誤り。
//!
//! 操作の記録の置き場所：環境変数 SAS_DB_PATH（db_path の上書き）→ `<git-common-dir>/sas/sas.db`（ワークツリーで1つ）→
//! Git が無ければ `.sas/sas.db`。どれもコミットしない。無ければ空で作る（作り直せるデータ）。

mod caller;
mod output;

use caller::FromArguments;
use outbound_filesystem::{OutDir, WorkingTree};
use outbound_git::GitCommits;
use outbound_javascript::Boa;
use outbound_jsonschema::Validator;
use outbound_sqlite::SqliteLog;
use outbound_templates::Mustache;
use recording_application::ports::{IdentifyCaller, OperationLog};
use recording_application::use_cases::history::register_session;
use rendering_application::use_cases::design::{DESIGN_SYSTEM, validate_design_systems};
use rendering_application::use_cases::render::{RenderRequest, render};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use validation_application::use_cases::validate::{
    Refused, Scope, log_refused, semantic_procedures, validate, validate_stale,
    validate_with_answer,
};
use writing_application::use_cases::write::{
    AdoptAs, AdoptRequest, CreateRequest, DeleteRequest, SemanticRequest, UpdateRequest, adopt,
    create, delete, semantic_procedure, update,
};

const USAGE: &str = "使い方：sas init [--session <ID>] ／ \
sas validate [--session <名前>] [--stale] (<経路>... | --all) ／ \
sas validate [--session <名前>] [--stale] (<経路>... | --all) --semantic ／ \
sas validate <経路> [--stale] --answer <判定の JSON のファイル | -> [--session <名前>] ／ \
sas create <経路> --concrete <URN> --value <JSON> [--intent <意図>] [--session <名前>] [--request-id <ID>] ／ \
sas create <経路> --concrete <URN> --semantic [--brief <依頼>] [--ref <経路>]... [--intent <意図>] [--session <名前>] ／ \
sas create <経路> --adopt [--concrete <URN>] [--intent <意図>] [--session <名前>] [--request-id <ID>] ／ \
sas update <経路> --patch <JSON Patch> [--if-match <ハッシュ>] [--intent <意図>] [--session <名前>] [--request-id <ID>] ／ \
sas update <経路> --semantic --brief <依頼> [--ref <経路>]... [--intent <意図>] [--session <名前>] ／ \
sas update <経路> --adopt --intent <意図> [--session <名前>] [--request-id <ID>] ／ \
sas delete <経路> --intent <意図> [--if-match <ハッシュ>] [--session <名前>] [--request-id <ID>] ／ \
sas render <経路>... --design-system <経路> --out <書き出す先>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("init") => run_init(&args[1..]),
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

/// sas を打ったフォルダ：Git の中なら作業ツリーの根、外なら作業フォルダ（絶対の経路）。
fn folder() -> String {
    GitCommits::new(PathBuf::from("."))
        .toplevel()
        .or_else(|| std::env::current_dir().ok())
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| ".".to_string())
}

/// 呼び出し元：--session ・ ツールごとの環境変数 ・ このフォルダで sas init が登録したセッション。
/// init は sas init のとき（登録の前なので、前の登録を使わない）。
fn caller(session: Option<String>, log: &SqliteLog, init: bool) -> Result<FromArguments, Refused> {
    let folder = folder();
    let initialized = if init {
        None
    } else {
        log.initialized(&folder).map_err(|message| Refused {
            code: "log",
            message,
            ..Refused::default()
        })?
    };
    FromArguments::new(session, |name| std::env::var(name).ok(), &folder, initialized).map_err(
        |tools| Refused {
            code: "usage",
            message: format!(
                "{} の中で打たれていて、どれが打ったかを決められません。--session <打った会話の ID> で名乗ってください",
                tools.join(" と ")
            ),
            ..Refused::default()
        },
    )
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

/// sas init：いまのセッションを、このフォルダの記録の相手として登録する。
fn run_init(args: &[String]) -> ExitCode {
    let mut session = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--session" => match it.next() {
                Some(s) if !s.is_empty() => session = Some(s.clone()),
                _ => return usage("--session に ID がありません"),
            },
            s => return usage(&format!("知らない指定です: {s}")),
        }
    }
    let (_, _, mut log) = match adapters() {
        Ok(a) => a,
        Err(r) => return refuse(&r),
    };
    let who = match caller(session, &log, true) {
        Ok(w) => w,
        Err(r) => return refuse(&r),
    };
    match register_session(&who, &folder(), &mut log) {
        Ok(row) => {
            output::print_session(&row);
            ExitCode::SUCCESS
        }
        Err(message) => refuse(&Refused {
            code: "log",
            message,
            ..Refused::default()
        }),
    }
}

fn run_validate(args: &[String]) -> ExitCode {
    let mut session = None;
    let mut all = false;
    let mut stale = false;
    let mut semantic = false;
    let mut answer = None;
    let mut paths = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--all" => all = true,
            "--stale" => stale = true,
            "--semantic" => semantic = true,
            "--answer" => match it.next() {
                Some(s) if !s.is_empty() => answer = Some(s.clone()),
                _ => return usage("--answer に判定の JSON のファイルか - がありません"),
            },
            "--session" => match it.next() {
                Some(s) if !s.is_empty() => session = Some(s.clone()),
                _ => return usage("--session に名前がありません"),
            },
            s if s.starts_with("--") => return usage(&format!("知らない指定です: {s}")),
            p => paths.push(p.to_string()),
        }
    }
    if semantic && answer.is_some() {
        return usage(
            "--semantic と --answer は一緒に渡せません（--semantic で手順を受け取り、判定を --answer で渡す）",
        );
    }
    let scope = match (all, paths.is_empty()) {
        (true, true) => Scope::All,
        (false, false) => Scope::Paths(paths),
        (true, false) => return usage("経路と --all は一緒に渡せません"),
        // 入力が変わった rule だけなら、経路を渡さなければすべてを見る
        (false, true) if stale && answer.is_none() => Scope::All,
        (false, true) => return usage("検証する経路か --all を渡してください"),
    };
    // デザインシステムは、受け取りの型とテンプレートの書き方の契約に照らす（描画のモジュール）
    if let Scope::Paths(paths) = &scope {
        let design: Vec<bool> = paths.iter().map(|p| is_design_system(p)).collect();
        if design.iter().any(|d| *d) {
            if !design.iter().all(|d| *d) || stale || semantic || answer.is_some() {
                return usage(
                    "デザインシステムは、ほかの対象や --stale ・ --semantic ・ --answer と一緒に渡さずに、それだけで検証してください",
                );
            }
            let (store, validator, mut log) = match adapters() {
                Ok(a) => a,
                Err(r) => return refuse(&r),
            };
            let who = match caller(session, &log, false) {
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
    let answer_text = match &answer {
        Some(from) => {
            let Scope::Paths(paths) = &scope else {
                return usage("--answer は --all と一緒に渡せません。経路を1つ渡してください");
            };
            if paths.len() != 1 {
                return usage("--answer は経路を1つだけ渡してください");
            }
            match read_answer(from) {
                Ok(t) => Some(t),
                Err(r) => return refuse(&r),
            }
        }
        None => None,
    };
    let (store, validator, mut log) = match adapters() {
        Ok(a) => a,
        Err(r) => return refuse(&r),
    };
    let who = match caller(session, &log, false) {
        Ok(w) => w,
        Err(r) => return refuse(&r),
    };
    if let Err(message) = log.session(&who.identify()) {
        return refuse(&log_refused(message));
    }
    if semantic {
        // 判定の手順を出すだけで、記録しない。判定する rule がどの対象にも無ければ、機械の検証をそのまま記録する
        match semantic_procedures(&scope, stale, &store, &validator, &who, &log) {
            Ok(procedures) if !procedures.is_empty() => {
                for p in &procedures {
                    output::print_procedure(p);
                }
                return ExitCode::SUCCESS;
            }
            Ok(_) => {}
            Err(r) => return refuse(&r),
        }
    }
    let result = match (&scope, answer_text) {
        (Scope::Paths(paths), Some(text)) => {
            validate_with_answer(&paths[0], stale, &text, &store, &validator, &who, &mut log)
        }
        _ if stale => validate_stale(&scope, &store, &validator, &who, &mut log),
        _ => validate(&scope, &store, &validator, &who, &mut log),
    };
    match result {
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

/// 判定の JSON を、ファイルか標準入力（-）から読む。
fn read_answer(from: &str) -> Result<String, Refused> {
    let read = if from == "-" {
        let mut text = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut text)
            .map(|_| text)
            .map_err(|e| format!("標準入力を読めない：{e}"))
    } else {
        std::fs::read_to_string(from).map_err(|e| format!("{from} を読めない：{e}"))
    };
    read.map_err(|message| Refused {
        code: "invalid_input",
        message,
        ..Refused::default()
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Command {
    Create,
    Update,
}

fn run_change(args: &[String], command: Command) -> ExitCode {
    // --adopt ・ --semantic は値を取らない旗、--ref は何度でも渡せるので、先に抜き出す
    let adopting = args.iter().any(|a| a == "--adopt");
    let semantic = args.iter().any(|a| a == "--semantic");
    let mut refs = Vec::new();
    let mut rest = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--adopt" | "--semantic" => {}
            "--ref" => match it.next() {
                Some(v) if !v.is_empty() => refs.push(v.clone()),
                _ => return usage("--ref に値がありません"),
            },
            _ => rest.push(a.clone()),
        }
    }
    let args = rest;
    let mut flags: Vec<(&str, Option<String>)> = match command {
        Command::Create => vec![("--concrete", None), ("--value", None)],
        Command::Update => vec![("--patch", None), ("--if-match", None)],
    };
    flags.extend([
        ("--intent", None),
        ("--session", None),
        ("--request-id", None),
        ("--brief", None),
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
    if !semantic && take("--brief").is_some() {
        return usage("--brief は --semantic と一緒に渡してください");
    }
    if !semantic && !refs.is_empty() {
        return usage("--ref は --semantic と一緒に渡してください");
    }
    if semantic {
        // 意味で書く：手順を返すだけ。値と JSON Patch は、手順を受け取ったセッションが決めて、手順の command で書く
        if adopting {
            return usage("--semantic と --adopt は一緒に渡せません");
        }
        if let Some(name) = ["--value", "--patch", "--if-match"]
            .into_iter()
            .find(|n| take(n).is_some())
        {
            return usage(&format!("--semantic と {name} は一緒に渡せません"));
        }
        let concrete = match command {
            Command::Create => match take("--concrete") {
                Some(c) => Some(c),
                None => return usage("--concrete を渡してください"),
            },
            Command::Update => None,
        };
        let request = SemanticRequest {
            path: path.clone(),
            concrete,
            brief: take("--brief"),
            refs,
            intent,
            request_id,
        };
        let result = run_with(
            |store, validator, who, _| semantic_procedure(&request, store, validator, who),
            session,
        );
        return match result {
            Ok(p) => {
                output::print_procedure(&p);
                ExitCode::SUCCESS
            }
            Err(r) => refuse(&r),
        };
    }
    if adopting {
        // 取り込みはファイルを書かないので、書く中身（--value ・ --patch）と前提のハッシュ（--if-match）を受けない
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

/// 置き場所と呼び出し元を組み立て、打ったセッションを記録してから f を呼ぶ（最初に sas を打ったときに sessions に1行）。
fn run_with<T>(
    f: impl FnOnce(&WorkingTree, &Validator, &FromArguments, &mut SqliteLog) -> Result<T, Refused>,
    session: Option<String>,
) -> Result<T, Refused> {
    let (store, validator, mut log) = adapters()?;
    let who = caller(session, &log, false)?;
    log.session(&who.identify()).map_err(log_refused)?;
    f(&store, &validator, &who, &mut log)
}

fn refuse(r: &Refused) -> ExitCode {
    output::print_refused(r);
    ExitCode::from(if r.code == "usage" { 2 } else { 1 })
}
