//! sas コマンドの入口。引数を読み、adapter を組み立ててユースケースを呼び、結果を JSON の1行で返す。
//! 意味の部分（--semantic）は、結果の代わりに手順（procedure.schema.json）を返す。SAS はエージェントを起動しない。
//! sas log（UC7）は記録を読むだけで、何も足さない（記録が無ければ作らない）。sas search（UC6）は記録しない。
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
use output::{Refused, log_refused};
use recording_application::ports::OperationFilter;
use recording_application::use_cases::history::{
    Unidentified, identify, read_operation, read_operations, register, register_session,
};
use rendering_application::use_cases::design::{
    DESIGN_SYSTEM, check_design_systems, record_design_systems,
};
use rendering_application::use_cases::render::{RenderRequest, render};
use search_application::use_cases::search::{
    Range, search_procedure, search_where, search_with_answer,
};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use validation_application::use_cases::validate::{
    Scope, semantic_procedures, validate, validate_stale, validate_with_answer,
};
use writing_application::use_cases::write::{
    AdoptAs, AdoptRequest, CreateRequest, DeleteRequest, SemanticRequest, UpdateRequest, adopt,
    create, delete, semantic_procedure, update,
};
use writing_domain::values::Caller;

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
sas render <経路>... --design-system <経路> --out <書き出す先> ／ \
sas log [<経路>...] [--by <セッション ID>] [--folder <フォルダ>] [--since <時刻>] [--limit <件数>] ／ \
sas log --operation <ID> ／ \
sas search (<経路>... | --concrete <URN> | --all) --where <JSON Pointer>=<値> [--where …]... [--session <名前>] ／ \
sas search (<経路>... | --concrete <URN> | --all) --semantic --question <問い> [--session <名前>] ／ \
sas search (<経路>... | --concrete <URN> | --all) --answer <答えの JSON のファイル | -> [--session <名前>]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("init") => run_init(&args[1..]),
        Some("validate") => run_validate(&args[1..]),
        Some("create") => run_change(&args[1..], Command::Create),
        Some("update") => run_change(&args[1..], Command::Update),
        Some("delete") => run_delete(&args[1..]),
        Some("render") => run_render(&args[1..]),
        Some("log") => run_log(&args[1..]),
        Some("search") => run_search(&args[1..]),
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

/// 誰が打ったかを決められなかった理由を、出力の形にする。
fn unidentified(u: Unidentified) -> Refused {
    match u {
        Unidentified::Ambiguous(tools) => Refused {
            code: "usage",
            message: format!(
                "{} の中で打たれていて、どれが打ったかを決められません。--session <打った会話の ID> で名乗ってください",
                tools.join(" と ")
            ),
            ..Refused::default()
        },
        Unidentified::Log(message) => log_refused(message),
    }
}

/// 呼び出し元：--session ・ ツールごとの環境変数 ・ このフォルダで sas init が登録したセッション ・ 人（記録のモジュールの手続きで決める）。
/// sessions にはまだ足さない（足すのは register）。
fn caller(session: Option<String>, log: &SqliteLog) -> Result<Caller, Refused> {
    let folder = folder();
    let who = FromArguments::new(session, |name| std::env::var(name).ok(), &folder);
    identify(&who, &folder, log).map_err(unidentified)
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
        Err(r) => return refuse(r),
    };
    // 登録の前なので、前の登録を使わない
    let folder = folder();
    let who = FromArguments::new(session, |name| std::env::var(name).ok(), &folder);
    match register_session(&who, &folder, &mut log) {
        Ok(row) => {
            output::print_session(&row);
            ExitCode::SUCCESS
        }
        Err(u) => refuse(unidentified(u)),
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
                Err(r) => return refuse(r),
            };
            let caller = match caller(session, &log) {
                Ok(c) => c,
                Err(r) => return refuse(r),
            };
            // 照らしてから、記録する直前に打ったセッションを sessions に足す
            let checked = match check_design_systems(paths, &store, &validator, &Mustache, &Boa) {
                Ok(c) => c,
                Err(r) => return refuse(r),
            };
            if let Err(message) = register(&caller, &mut log) {
                return refuse(log_refused(message));
            }
            return match record_design_systems(checked, &caller, &mut log) {
                Ok(v) => {
                    output::print_design_validation(&v);
                    if v.counts.fails() {
                        ExitCode::from(1)
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Err(r) => refuse(r),
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
                Err(r) => return refuse(r),
            }
        }
        None => None,
    };
    let (store, validator, mut log) = match adapters() {
        Ok(a) => a,
        Err(r) => return refuse(r),
    };
    let caller = match caller(session, &log) {
        Ok(c) => c,
        Err(r) => return refuse(r),
    };
    if let Err(message) = register(&caller, &mut log) {
        return refuse(log_refused(message));
    }
    if semantic {
        // 判定の手順を出すだけで、記録しない。判定する rule がどの対象にも無ければ、機械の検証をそのまま記録する
        match semantic_procedures(&scope, stale, &store, &validator, &caller, &log) {
            Ok(procedures) if !procedures.is_empty() => {
                for p in &procedures {
                    output::print_procedure(p);
                }
                return ExitCode::SUCCESS;
            }
            Ok(_) => {}
            Err(r) => return refuse(r),
        }
    }
    let result = match (&scope, answer_text) {
        (Scope::Paths(paths), Some(text)) => validate_with_answer(
            &paths[0], stale, &text, &store, &validator, &caller, &mut log,
        ),
        _ if stale => validate_stale(&scope, &store, &validator, &caller, &mut log),
        _ => validate(&scope, &store, &validator, &caller, &mut log),
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
        Err(r) => refuse(r),
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
            |store, validator, caller, _| semantic_procedure(&request, store, validator, caller),
            session,
        );
        return match result {
            Ok(p) => {
                output::print_procedure(&p);
                ExitCode::SUCCESS
            }
            Err(r) => refuse(r),
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
            |store, validator, caller, log| {
                adopt(&request, store, validator, &committed, caller, log)
            },
            session,
        );
        return match result {
            Ok(c) => {
                output::print_change(&c);
                ExitCode::SUCCESS
            }
            Err(r) => refuse(r),
        };
    }
    let json = |name: &str| -> Result<Value, ExitCode> {
        let text = take(name).ok_or_else(|| usage(&format!("{name} を渡してください")))?;
        serde_json::from_str(&text).map_err(|e| {
            refuse(Refused {
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
                |store, validator, caller, log| create(&request, store, validator, caller, log),
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
                |store, validator, caller, log| update(&request, store, validator, caller, log),
                session,
            )
        }
    };
    match result {
        Ok(c) => {
            output::print_change(&c);
            ExitCode::SUCCESS
        }
        Err(r) => refuse(r),
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
        |store, _, caller, log| delete(&request, store, caller, log),
        take("--session"),
    ) {
        Ok(d) => {
            output::print_deletion(&d);
            ExitCode::SUCCESS
        }
        Err(r) => refuse(r),
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
            return refuse(Refused {
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
        Err(r) => refuse(r),
    }
}

/// sas log：操作の記録を読む（UC7）。読むだけで、セッションの行も足さない。記録が無ければ作らずに、無いものとして返す。
fn run_log(args: &[String]) -> ExitCode {
    let mut filter = OperationFilter::default();
    let mut operation = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let name = a.as_str();
        if !name.starts_with("--") {
            filter.paths.push(a.clone());
            continue;
        }
        let Some(v) = it.next().filter(|v| !v.is_empty()) else {
            return if ["--by", "--folder", "--since", "--limit", "--operation"].contains(&name) {
                usage(&format!("{a} に値がありません"))
            } else {
                usage(&format!("知らない指定です: {a}"))
            };
        };
        match name {
            "--by" => filter.by = Some(v.clone()),
            "--folder" => filter.folder = Some(normalized_folder(v)),
            "--since" => {
                if !is_time(v) {
                    return usage(&format!(
                        "--since は UTC の RFC 3339 の時刻（2026-11-01 ・ 2026-11-01T09:30:00Z など）で渡してください: {v}"
                    ));
                }
                filter.since = Some(v.clone());
            }
            "--limit" => match v.parse::<usize>() {
                Ok(n) if n > 0 => filter.limit = Some(n),
                _ => return usage(&format!("--limit は1以上の整数で渡してください: {v}")),
            },
            "--operation" => operation = Some(v.clone()),
            _ => return usage(&format!("知らない指定です: {a}")),
        }
    }
    if operation.is_some() && filter != OperationFilter::default() {
        return usage("--operation は1件を読むので、経路やほかの絞り方と一緒に渡せません");
    }
    let path = db_path(&PathBuf::from("."));
    // 記録が無ければ、作らずに「無い」として返す（読んでも何も変えない）
    let log = if path.exists() {
        match SqliteLog::open(&path) {
            Ok(l) => Some(l),
            Err(message) => {
                return refuse(Refused {
                    code: "log",
                    message,
                    ..Refused::default()
                });
            }
        }
    } else {
        None
    };
    if let Some(id) = operation {
        let found = match &log {
            Some(l) => read_operation(l, &id),
            None => Ok(None),
        };
        return match found {
            Ok(Some(d)) => {
                output::print_operation_detail(&d);
                ExitCode::SUCCESS
            }
            Ok(None) => refuse(Refused {
                code: "not_found",
                message: format!("操作 {id} は記録に無い"),
                ..Refused::default()
            }),
            Err(message) => refuse(log_refused(message)),
        };
    }
    let found = match &log {
        Some(l) => read_operations(l, &filter),
        None => Ok(Vec::new()),
    };
    match found {
        Ok(rows) => {
            for o in &rows {
                output::print_operation(o);
            }
            ExitCode::SUCCESS
        }
        Err(message) => refuse(log_refused(message)),
    }
}

/// --folder の値を、sessions の folder と比べられる形（実体の絶対の経路）にする。無いフォルダは、作業フォルダから引いた絶対の経路。
fn normalized_folder(given: &str) -> String {
    let p = PathBuf::from(given);
    if let Ok(real) = std::fs::canonicalize(&p) {
        return real.display().to_string();
    }
    let absolute = if p.is_absolute() {
        p
    } else {
        std::env::current_dir().unwrap_or_default().join(p)
    };
    let mut out = PathBuf::new();
    for part in absolute.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out.display().to_string()
}

/// UC7 の --since に渡せる時刻か（UTC の RFC 3339。日付だけ ・ 分まで ・ 秒まででもよい）。記録の時刻と文字の順で比べる。
fn is_time(t: &str) -> bool {
    let shape = "dddd-dd-ddTdd:dd:ddZ";
    let digits = |s: &str, at: &str| {
        s.len() == at.len()
            && s.chars()
                .zip(at.chars())
                .all(|(c, a)| if a == 'd' { c.is_ascii_digit() } else { c == a })
    };
    [10, 16, 19, 20].into_iter().any(|n| digits(t, &shape[..n]))
}

/// sas search：呼ぶ側が渡した範囲を、構造で絞るか、意味で探す手順を返すか、渡された答えの出どころを確かめる（UC6）。記録しない。
fn run_search(args: &[String]) -> ExitCode {
    let mut session = None;
    let mut concrete = None;
    let mut all = false;
    let mut semantic = false;
    let mut question = None;
    let mut answer = None;
    let mut conditions: Vec<(String, Value)> = Vec::new();
    let mut paths = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let slot = match a.as_str() {
            "--all" => {
                all = true;
                continue;
            }
            "--semantic" => {
                semantic = true;
                continue;
            }
            "--session" => &mut session,
            "--concrete" => &mut concrete,
            "--question" => &mut question,
            "--answer" => &mut answer,
            "--where" => {
                let Some(w) = it.next().filter(|w| !w.is_empty()) else {
                    return usage("--where に <JSON Pointer>=<値> がありません");
                };
                // 値は JSON として読み、読めなければ文字列として扱う（/level=summary と書ける）
                let Some((at, raw)) = w.split_once('=') else {
                    return usage(&format!(
                        "--where は <JSON Pointer>=<値> の形で渡してください: {w}"
                    ));
                };
                if !at.is_empty() && !at.starts_with('/') {
                    return usage(&format!(
                        "--where の所は JSON Pointer（/ で始まる）で渡してください: {at}"
                    ));
                }
                let value = serde_json::from_str::<Value>(raw)
                    .unwrap_or_else(|_| Value::String(raw.to_string()));
                conditions.push((at.to_string(), value));
                continue;
            }
            s if s.starts_with("--") => return usage(&format!("知らない指定です: {s}")),
            p => {
                paths.push(p.to_string());
                continue;
            }
        };
        match it.next() {
            Some(v) if !v.is_empty() => *slot = Some(v.clone()),
            _ => return usage(&format!("{a} に値がありません")),
        }
    }
    // 範囲は必ず渡す（経路 ・ --concrete ・ --all のどれか1つ）
    let range = match (paths.is_empty(), concrete, all) {
        (false, None, false) => Range::Paths(paths),
        (true, Some(c), false) => Range::Concrete(c),
        (true, None, true) => Range::All,
        (true, None, false) => {
            return usage("探す範囲を、経路 ・ --concrete <URN> ・ --all のどれかで渡してください");
        }
        _ => {
            return usage(
                "探す範囲は、経路 ・ --concrete ・ --all のどれか1つだけを渡してください",
            );
        }
    };
    let ways = [!conditions.is_empty(), semantic, answer.is_some()];
    match ways.iter().filter(|w| **w).count() {
        0 => {
            return usage(
                "探し方を、--where ・ --semantic --question ・ --answer のどれかで渡してください",
            );
        }
        1 => {}
        _ => return usage("--where ・ --semantic ・ --answer は一緒に渡せません"),
    }
    if semantic != question.is_some() {
        return usage("意味で探すときは、--semantic と --question <問い> を一緒に渡してください");
    }
    let answer_text = match &answer {
        Some(from) => match read_answer(from) {
            Ok(t) => Some(t),
            Err(r) => return refuse(r),
        },
        None => None,
    };
    if let Some(q) = question {
        return match run_with(
            |store, _, caller, _| search_procedure(&range, &q, store, caller),
            session,
        ) {
            Ok(p) => {
                output::print_procedure(&p);
                ExitCode::SUCCESS
            }
            Err(r) => refuse(r),
        };
    }
    let result = match answer_text {
        Some(text) => run_with(
            |store, validator, _, _| search_with_answer(&range, &text, store, validator),
            session,
        ),
        None => run_with(
            |store, _, _, _| search_where(&range, &conditions, store),
            session,
        ),
    };
    match result {
        Ok(f) => {
            output::print_search(&f);
            ExitCode::SUCCESS
        }
        Err(r) => refuse(r),
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

/// 置き場所を組み立て、誰が打ったかを決めて sessions に足してから（最初に sas を打ったときに1行）、その値で f を呼ぶ。
/// f の断った理由（各モジュールの Refused）は、出力の形（output::Refused）へ移して返す。
fn run_with<T, E>(
    f: impl FnOnce(&WorkingTree, &Validator, &Caller, &mut SqliteLog) -> Result<T, E>,
    session: Option<String>,
) -> Result<T, Refused>
where
    Refused: From<E>,
{
    let (store, validator, mut log) = adapters()?;
    let caller = caller(session, &log)?;
    register(&caller, &mut log).map_err(log_refused)?;
    f(&store, &validator, &caller, &mut log).map_err(Refused::from)
}

fn refuse(r: impl Into<Refused>) -> ExitCode {
    let r = r.into();
    output::print_refused(&r);
    ExitCode::from(if r.code == "usage" { 2 } else { 1 })
}
