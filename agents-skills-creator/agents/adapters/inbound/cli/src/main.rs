//! agents コマンドの入口（文脈 agents）。引数を読み、adapter を組み立ててユースケースを呼び、結果を JSON 1つで返す。
//!
//! コマンドは skills と同じ形：create（sas create を呼ぶ）・ validate（sas validate と creator の検査と承認の材料）・
//! write-agent-md（agent.json から AGENT.md と codex.toml を書く）・ distribute（確かめてからツールの置き場所に定義を写して置く）・
//! withdraw（配布を取り下げる。人が打ったときだけ）。文脈 skills には、skills コマンドの結果（公開された言葉）でだけ従う。
//! 終了コード：0 = 正常 ／ 1 = 検出がある、sas が断った、sas ・ skills を呼べない、読めない ／ 2 = 使い方の誤り。

mod output;

use agents_application::use_cases::create::{Mode, create};
use agents_application::use_cases::distribute::{Scope, Tool, distribute};
use agents_application::use_cases::validate::validate;
use agents_application::use_cases::withdraw::withdraw;
use agents_application::use_cases::write_agent_md::write_agent_md;
use agents_application::use_cases::{Outcome, Refused};
use agents_outbound_filesystem::WorkingTree;
use agents_outbound_sas::Sas;
use agents_outbound_skills::Skills;
use output::{CREATE, DISTRIBUTE, VALIDATE, WITHDRAW, WRITE_AGENT_MD};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "使い方：agents create <Agent のフォルダ> --concrete <URN> --semantic ／ \
agents create <Agent のフォルダ> --concrete <URN> --value <JSON> ／ \
agents validate <Agent のフォルダ>... ／ \
agents write-agent-md <Agent のフォルダ> ／ \
agents distribute <Agent のフォルダ> --tool <claude-code|codex> --scope <repository|user> ／ \
agents withdraw <Agent のフォルダ> --reason <理由>";

/// AI のツールのセッションを見分ける表（SAS の callers.json と同じ行）。
const CALLERS: &str = include_str!("callers.json");

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
        code: "sas_unavailable",
        message: "sas が PATH に無い（SAS を入れて、PATH に置いてください）".to_string(),
    })
}

fn skills() -> Result<Skills, Refused> {
    Skills::locate().ok_or_else(|| Refused {
        code: "skills_unavailable",
        message: "skills が PATH に無い（agents-skills-creator の skills を入れて、PATH に置いてください）".to_string(),
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

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest = args.get(1..).unwrap_or_default();
    match args.first().map(String::as_str) {
        Some("create") => run_create(rest),
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
        (false, Some(v)) => Some(Mode::Value(v.to_string())),
        _ => None,
    };
    if folder.is_none() || value(&values, "--concrete").is_none() || mode.is_none() {
        return refuse(
            CREATE,
            &usage(format!(
                "Agent のフォルダと、--concrete <URN> と、--semantic か --value <JSON> のどちらか一方を渡してください。{USAGE}"
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
    match validate(args, &sas, &cli, &WorkingTree::new(PathBuf::from("."))) {
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
