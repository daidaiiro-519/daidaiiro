//! agents コマンドの入口（文脈 agents）。引数を読み、adapter を組み立ててユースケースを呼び、結果を JSON 1つで返す。
//!
//! コマンドは skills と同じ形：create（semantic-agent-schema create を呼ぶ）・ update（semantic-agent-schema update を呼ぶ）・ validate（semantic-agent-schema validate と creator の検査と承認の材料）・
//! write-agent-md（agent.json から AGENT.md と codex.toml を書く）・ distribute（確かめてからツールの置き場所に定義を写して置く）・
//! withdraw（配布を取り下げる。人が打ったときだけ）。文脈 skills には、skills コマンドの結果（公開された言葉）でだけ従う。
//! 終了コード：0 = 正常 ／ 1 = 検出がある、semantic-agent-schema が断った、semantic-agent-schema ・ skills を呼べない、読めない ／ 2 = 使い方の誤り。

mod output;

use agents_application::use_cases::approve::approve;
use agents_application::use_cases::create::{Mode, create};
use agents_application::use_cases::distribute::{Scope, Tool, distribute};
use agents_application::use_cases::update::{self, update};
use agents_application::use_cases::validate::validate;
use agents_application::use_cases::withdraw::withdraw;
use agents_application::use_cases::write_agent_md::write_agent_md;
use agents_application::use_cases::{Outcome, Refused};
use outbound_filesystem::agents::WorkingTree;
use outbound_semantic_agent_schema::agents::Sas;
use outbound_skills::Skills;
use output::{APPROVE, CREATE, DISTRIBUTE, UPDATE, VALIDATE, WITHDRAW, WRITE_AGENT_MD};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "使い方：agents-skills-creator agents create <Agent のフォルダ> [--concrete <URN>] --semantic ／ \
agents-skills-creator agents create <Agent のフォルダ> [--concrete <URN>] --value <JSON か @<ファイル>> ／ \
agents-skills-creator agents update <Agent のフォルダ> --semantic --task <外れた具体> ／ \
agents-skills-creator agents update <Agent のフォルダ> --patch <JSON Patch か @<ファイル>> --if-match <ハッシュ> --reason 層：<層> ／ \
agents-skills-creator agents validate <Agent のフォルダ>... ／ \
agents-skills-creator agents write-agent-md <Agent のフォルダ> ／ \
agents-skills-creator approve <Agent のフォルダ>（人が打つ） ／ \
agents-skills-creator agents distribute <Agent のフォルダ> --tool <claude-code|codex> --scope <repository|user> ／ \
agents-skills-creator agents withdraw <Agent のフォルダ> --reason <理由>";

/// AI のツールのセッションを見分ける表（SAS の callers.json と同じ行）。
const CALLERS: &str = include_str!("../callers.json");

fn usage(message: String) -> Refused {
    Refused {
        code: "usage",
        message,
    }
}

fn refuse(schema: &str, r: &Refused) -> ExitCode {
    output::print_refused(schema, r);
    ExitCode::from(if r.code == "usage" { 2 } else { 1 })
}

fn sas() -> Result<Sas, Refused> {
    Sas::locate().ok_or_else(|| Refused {
        code: "semantic_agent_schema_unavailable",
        message: "semantic-agent-schema が PATH に無い（SAS を入れて、PATH に置いてください）"
            .to_string(),
    })
}

fn skills() -> Result<Skills, Refused> {
    Skills::locate().ok_or_else(|| Refused {
        code: "skills_unavailable",
        message: "agents-skills-creator を見つけられない（PATH に置いてください）".to_string(),
    })
}

/// 人が打ったか：表のどの環境変数にも値が無ければ人。
fn by_human() -> bool {
    let Ok(table) = serde_json::from_str::<serde_json::Value>(CALLERS) else {
        return false;
    };
    let names: Vec<&str> = table["tools"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|t| t["session_env"].as_str())
        .collect();
    !names.is_empty()
        && names
            .iter()
            .all(|n| std::env::var_os(n).is_none_or(|v| v.is_empty()))
}

/// 分けた引数：位置の引数（1つまで）・ 値を取る指定と値 ・ 値を取らない指定。
type Parsed<'a> = (Option<&'a str>, Vec<(&'a str, &'a str)>, Vec<&'a str>);

/// 引数を、位置の引数（1つまで）と、値を取る指定と、値を取らない指定に分ける。
fn parse<'a>(
    args: &'a [String],
    schema: &str,
    with_value: &[&str],
    flags: &[&str],
) -> Result<Parsed<'a>, ExitCode> {
    let mut folder = None;
    let mut values = Vec::new();
    let mut set = Vec::new();
    let mut rest = args.iter();
    while let Some(a) = rest.next() {
        let a = a.as_str();
        if with_value.contains(&a) {
            match rest.next() {
                Some(v) => values.push((a, v.as_str())),
                None => {
                    return Err(refuse(
                        schema,
                        &usage(format!("{a} に値を渡してください。{USAGE}")),
                    ));
                }
            }
        } else if flags.contains(&a) {
            set.push(a);
        } else if a.starts_with("--") {
            return Err(refuse(
                schema,
                &usage(format!("知らない指定です: {a}。{USAGE}")),
            ));
        } else if folder.is_none() {
            folder = Some(a);
        } else {
            return Err(refuse(
                schema,
                &usage(format!("フォルダは1つだけ渡してください。{USAGE}")),
            ));
        }
    }
    Ok((folder, values, set))
}

fn value<'a>(values: &[(&str, &'a str)], name: &str) -> Option<&'a str> {
    values.iter().find(|(k, _)| *k == name).map(|(_, v)| *v)
}

/// コマンドの使い方（USAGE のうち、そのコマンドの行だけ）。
fn help_for(command: &str) -> String {
    let lines: Vec<&str> = USAGE
        .trim_start_matches("使い方：")
        .split(" ／ ")
        .filter(|l| {
            let mut w = l.split_whitespace().skip(1);
            w.next() == Some(command) || w.next() == Some(command)
        })
        .collect();
    format!("使い方：{}", lines.join("\n使い方："))
}

/// 値の引数：@<ファイル> ならファイルの中身を読む（作業ディレクトリからの経路）。読めなければ使い方の誤り。
fn from_file(value: &str, schema: &str) -> Result<String, ExitCode> {
    match value.strip_prefix('@') {
        Some(path) => std::fs::read_to_string(path).map_err(|e| {
            refuse(
                schema,
                &usage(format!("{value} のファイルを読めない（{e}）。{USAGE}")),
            )
        }),
        None => Ok(value.to_string()),
    }
}

/// サブコマンド（agents-skills-creator agents のあとの引数）を読んで、ユースケースを呼び、結果を JSON 1つで出す。
#[must_use]
pub fn run(args: &[String]) -> ExitCode {
    let args: Vec<String> = args.to_vec();
    // コマンドの後ろの --help は、そのコマンドの使い方を返す
    if let Some(command) = args.first()
        && args.iter().skip(1).any(|a| a == "--help")
    {
        println!("{}", help_for(command));
        return ExitCode::SUCCESS;
    }
    let rest = args.get(1..).unwrap_or_default();
    match args.first().map(String::as_str) {
        Some("--help" | "help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some("create") => run_create(rest),
        Some("update") => run_update(rest),
        Some("approve") => run_approve(rest),
        Some("validate") => run_validate(rest),
        Some("write-agent-md") => run_write(rest),
        Some("distribute") => run_distribute(rest),
        Some("withdraw") => run_withdraw(rest),
        Some(other) => refuse(
            VALIDATE,
            &usage(format!("知らないコマンドです: {other}。{USAGE}")),
        ),
        None => refuse(
            VALIDATE,
            &usage(format!("コマンドを指定してください。{USAGE}")),
        ),
    }
}

fn run_create(args: &[String]) -> ExitCode {
    let (folder, values, set) =
        match parse(args, CREATE, &["--concrete", "--value"], &["--semantic"]) {
            Ok(p) => p,
            Err(c) => return c,
        };
    let mode = match (set.contains(&"--semantic"), value(&values, "--value")) {
        (true, None) => Some(Mode::Semantic),
        (false, Some(v)) => match from_file(v, CREATE) {
            Ok(text) => Some(Mode::Value(text)),
            Err(code) => return code,
        },
        _ => None,
    };
    if folder.is_none() || mode.is_none() {
        return refuse(
            CREATE,
            &usage(format!(
                "Agent のフォルダと、--semantic か --value <JSON> のどちらか一方を渡してください。{USAGE}"
            )),
        );
    }
    let sas = match sas() {
        Ok(s) => s,
        Err(r) => return refuse(CREATE, &r),
    };
    match create(folder, value(&values, "--concrete"), mode, &sas) {
        Ok(c) => {
            output::print_created(&c);
            if c.sas.exit == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(r) => refuse(CREATE, &r),
    }
}

fn run_update(args: &[String]) -> ExitCode {
    let (folder, values, set) = match parse(
        args,
        UPDATE,
        &["--task", "--patch", "--if-match", "--reason"],
        &["--semantic"],
    ) {
        Ok(p) => p,
        Err(c) => return c,
    };
    let mode = match (
        set.contains(&"--semantic"),
        value(&values, "--task"),
        value(&values, "--patch"),
    ) {
        (true, Some(t), None) => update::Mode::Semantic(t.to_string()),
        (false, None, Some(p)) => update::Mode::Patch {
            patch: match from_file(p, UPDATE) {
                Ok(text) => text,
                Err(code) => return code,
            },
            if_match: value(&values, "--if-match").map(ToString::to_string),
            reason: value(&values, "--reason").map(ToString::to_string),
        },
        _ => {
            return refuse(
                UPDATE,
                &usage(format!(
                    "Agent のフォルダと、--semantic --task <外れた具体> か --patch <JSON Patch> のどちらか一方を渡してください。{USAGE}"
                )),
            );
        }
    };
    if folder.is_none() {
        return refuse(
            UPDATE,
            &usage(format!("直す Agent のフォルダを渡してください。{USAGE}")),
        );
    }
    // 使い方の誤りは、semantic-agent-schema を探す前に返す
    if let Err(r) = update::check_mode(&mode) {
        return refuse(UPDATE, &r);
    }
    let sas = match sas() {
        Ok(s) => s,
        Err(r) => return refuse(UPDATE, &r),
    };
    match update(folder, Some(mode), &sas) {
        Ok(u) => {
            output::print_updated(&u);
            if u.sas.exit == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(r) => refuse(UPDATE, &r),
    }
}

fn run_approve(args: &[String]) -> ExitCode {
    if let Some(flag) = args.iter().find(|a| a.starts_with("--")) {
        return refuse(
            APPROVE,
            &usage(format!("知らない指定です: {flag}。{USAGE}")),
        );
    }
    if args.len() > 1 {
        return refuse(
            APPROVE,
            &usage(format!("フォルダは1つだけ渡してください。{USAGE}")),
        );
    }
    let sas = match sas() {
        Ok(s) => s,
        Err(r) => return refuse(APPROVE, &r),
    };
    match approve(
        args.first().map(String::as_str),
        &WorkingTree::new(PathBuf::from(".")),
        &sas,
        &sas,
    ) {
        Ok(a) => {
            output::print_approved(&a);
            if a.sas.exit == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(r) => refuse(APPROVE, &r),
    }
}

fn run_validate(args: &[String]) -> ExitCode {
    if let Some(flag) = args.iter().find(|a| a.starts_with("--")) {
        return refuse(
            VALIDATE,
            &usage(format!("知らない指定です: {flag}。{USAGE}")),
        );
    }
    if args.is_empty() {
        return refuse(
            VALIDATE,
            &usage(format!(
                "確かめる Agent のフォルダを1つ以上渡してください。{USAGE}"
            )),
        );
    }
    let (sas, cli) = match (sas(), skills()) {
        (Ok(s), Ok(k)) => (s, k),
        (Err(r), _) | (_, Err(r)) => return refuse(VALIDATE, &r),
    };
    match validate(
        args,
        &sas,
        &sas,
        &cli,
        &WorkingTree::new(PathBuf::from(".")),
    ) {
        Ok(c) => {
            output::print_validated(&c);
            if c.detected() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(r) => refuse(VALIDATE, &r),
    }
}

fn run_write(args: &[String]) -> ExitCode {
    let (folder, _, _) = match parse(args, WRITE_AGENT_MD, &[], &[]) {
        Ok(p) => p,
        Err(c) => return c,
    };
    let tree = WorkingTree::new(PathBuf::from("."));
    match write_agent_md(folder, &tree, &tree) {
        Ok(w) => {
            output::print_written(&w);
            ExitCode::SUCCESS
        }
        Err(r) => refuse(WRITE_AGENT_MD, &r),
    }
}

fn run_distribute(args: &[String]) -> ExitCode {
    let (folder, values, _) = match parse(args, DISTRIBUTE, &["--tool", "--scope"], &[]) {
        Ok(p) => p,
        Err(c) => return c,
    };
    let (Some(folder), Some(tool_name), Some(scope_name)) =
        (folder, value(&values, "--tool"), value(&values, "--scope"))
    else {
        return refuse(
            DISTRIBUTE,
            &usage(format!(
                "Agent のフォルダと、--tool と --scope を渡してください。{USAGE}"
            )),
        );
    };
    let Some(tool) = Tool::parse(tool_name) else {
        return refuse(
            DISTRIBUTE,
            &usage(format!(
                "知らないツールです: {tool_name}（claude-code か codex）。{USAGE}"
            )),
        );
    };
    let Some(scope) = Scope::parse(scope_name) else {
        return refuse(
            DISTRIBUTE,
            &usage(format!(
                "知らない範囲です: {scope_name}（repository か user）。{USAGE}"
            )),
        );
    };
    let (sas, cli) = match (sas(), skills()) {
        (Ok(s), Ok(k)) => (s, k),
        (Err(r), _) | (_, Err(r)) => return refuse(DISTRIBUTE, &r),
    };
    let tree = WorkingTree::new(PathBuf::from("."));
    match distribute(
        Some(folder),
        Some(tool),
        Some(scope),
        &sas,
        &sas,
        &cli,
        &tree,
        &tree,
    ) {
        Ok(d) => {
            output::print_distributed(&d, tool_name, scope_name);
            if d.detected() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(r) => refuse(DISTRIBUTE, &r),
    }
}

fn run_withdraw(args: &[String]) -> ExitCode {
    let (folder, values, _) = match parse(args, WITHDRAW, &["--reason"], &[]) {
        Ok(p) => p,
        Err(c) => return c,
    };
    let tree = WorkingTree::new(PathBuf::from("."));
    match withdraw(folder, value(&values, "--reason"), by_human(), &tree, &tree) {
        Ok(w) => {
            output::print_withdrawn(&w);
            if w.after.iter().all(|f| f.outcome == Outcome::Verified) {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(r) => refuse(WITHDRAW, &r),
    }
}
