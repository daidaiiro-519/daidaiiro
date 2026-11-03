//! skills コマンドの入口（文脈 skills）。引数を読み、adapter を組み立ててユースケースを呼び、結果を JSON 1つで返す。
//!
//! コマンドは文脈 skills とユースケースの動詞で、Semantic Agent Schema の言葉にそろえる：create（UC1 作る。semantic-agent-schema create を呼ぶ）・
//! update（UC3 直す。semantic-agent-schema update を呼ぶ）・ distribute（UC5 配布する。確かめてからツールの置き場所にリンクを置く）・
//! withdraw（UC6 使わなくする。配布を取り下げ、置いたリンクを外す。人が打ったときだけ）・
//! validate（UC4 確かめる。semantic-agent-schema validate と creator の検査と承認の材料）・ write-skill-md（skill.json から SKILL.md を書く）。
//! creator は AI を起動せず、skill.json を書かない（Semantic Agent Schema が書く）。
//! 終了コード：0 = 正常（validate は検出が無い）／ 1 = validate で検出がある、semantic-agent-schema が断った、semantic-agent-schema を呼べない、skill.json を
//! 読めない ／ 2 = 使い方の誤り。

mod output;

use outbound_filesystem::skills::WorkingTree;
use outbound_jsonschema::JsonSchema;
use outbound_semantic_agent_schema::skills::Sas;
use output::{APPROVE, CREATE, DISTRIBUTE, UPDATE, VALIDATE, WITHDRAW, WRITE_SKILL_MD};
use skills_application::use_cases::Refused;
use skills_application::use_cases::approve::approve;
use skills_application::use_cases::create::{Mode, create};
use skills_application::use_cases::distribute::{Scope, Tool, distribute};
use skills_application::use_cases::update::{self, update};
use skills_application::use_cases::validate::{Outcome, validate};
use skills_application::use_cases::withdraw::withdraw;
use skills_application::use_cases::write_skill_md::write_skill_md;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "使い方：agents-skills-creator skills create <Skill のフォルダ> [--concrete <URN>] --semantic ／ \
agents-skills-creator skills create <Skill のフォルダ> [--concrete <URN>] --value <JSON か @<ファイル>> ／ \
agents-skills-creator skills update <Skill のフォルダ> --semantic --task <外れた具体> ／ \
agents-skills-creator skills update <Skill のフォルダ> --patch <JSON Patch か @<ファイル>> --if-match <ハッシュ> --reason 層：<層> ／ \
agents-skills-creator skills validate <Skill のフォルダ>... ／ \
agents-skills-creator skills distribute <Skill のフォルダ> --tool <claude-code|codex> --scope <repository|user> ／ \
agents-skills-creator skills withdraw <Skill のフォルダ> --reason <理由> ／ \
agents-skills-creator skills write-skill-md <Skill のフォルダ> ／ \
agents-skills-creator approve <Skill のフォルダ>（人が打つ）";

fn usage(message: String) -> Refused {
    Refused {
        code: "usage",
        message,
    }
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

/// サブコマンド（agents-skills-creator skills のあとの引数）を読んで、ユースケースを呼び、結果を JSON 1つで出す。
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
    match args.first().map(String::as_str) {
        Some("--help" | "help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some("create") => run_create(&args[1..]),
        Some("update") => run_update(&args[1..]),
        Some("approve") => run_approve(&args[1..]),
        Some("distribute") => run_distribute(&args[1..]),
        Some("withdraw") => run_withdraw(&args[1..]),
        Some("validate") => run_validate(&args[1..]),
        Some("write-skill-md") => run_write_skill_md(&args[1..]),
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

fn locate() -> Result<Sas, Refused> {
    Sas::locate().ok_or_else(|| Refused {
        code: "semantic_agent_schema_unavailable",
        message: "semantic-agent-schema が PATH に無い（Semantic Agent Schema を入れて、PATH に置いてください）"
            .to_string(),
    })
}

fn run_create(args: &[String]) -> ExitCode {
    let mut target = None;
    let mut concrete = None;
    let mut mode = None;
    let mut rest = args.iter();
    while let Some(a) = rest.next() {
        match a.as_str() {
            "--concrete" => match rest.next() {
                Some(v) => concrete = Some(v.as_str()),
                None => {
                    return refuse(
                        CREATE,
                        &usage(format!("--concrete に URN を渡してください。{USAGE}")),
                    );
                }
            },
            "--semantic" => mode = Some(Mode::Semantic),
            "--value" => match rest.next() {
                Some(v) => match from_file(v, CREATE) {
                    Ok(text) => mode = Some(Mode::Value(text)),
                    Err(code) => return code,
                },
                None => {
                    return refuse(
                        CREATE,
                        &usage(format!("--value に JSON を渡してください。{USAGE}")),
                    );
                }
            },
            flag if flag.starts_with("--") => {
                return refuse(CREATE, &usage(format!("知らない指定です: {flag}。{USAGE}")));
            }
            _ if target.is_none() => target = Some(a.as_str()),
            _ => {
                return refuse(
                    CREATE,
                    &usage(format!("フォルダは1つだけ渡してください。{USAGE}")),
                );
            }
        }
    }
    if target.is_none() || mode.is_none() {
        return refuse(
            CREATE,
            &usage(format!(
                "Skill のフォルダと、--semantic か --value <JSON> のどちらか一方を渡してください。{USAGE}"
            )),
        );
    }
    let sas = match locate() {
        Ok(s) => s,
        Err(r) => return refuse(CREATE, &r),
    };
    match create(target, concrete, mode, &sas) {
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
    let mut folder = None;
    let mut semantic = false;
    let mut task = None;
    let mut patch = None;
    let mut if_match = None;
    let mut reason = None;
    let mut rest = args.iter();
    while let Some(a) = rest.next() {
        let slot = match a.as_str() {
            "--semantic" => {
                semantic = true;
                continue;
            }
            "--task" => &mut task,
            "--patch" => &mut patch,
            "--if-match" => &mut if_match,
            "--reason" => &mut reason,
            flag if flag.starts_with("--") => {
                return refuse(UPDATE, &usage(format!("知らない指定です: {flag}。{USAGE}")));
            }
            _ if folder.is_none() => {
                folder = Some(a.as_str());
                continue;
            }
            _ => {
                return refuse(
                    UPDATE,
                    &usage(format!("フォルダは1つだけ渡してください。{USAGE}")),
                );
            }
        };
        match rest.next() {
            Some(v) => *slot = Some(v.clone()),
            None => {
                return refuse(UPDATE, &usage(format!("{a} に値を渡してください。{USAGE}")));
            }
        }
    }
    let mode = match (semantic, task, patch) {
        (true, Some(t), None) => update::Mode::Semantic(t),
        (false, None, Some(p)) => match from_file(&p, UPDATE) {
            Ok(patch) => update::Mode::Patch {
                patch,
                if_match,
                reason,
            },
            Err(code) => return code,
        },
        _ => {
            return refuse(
                UPDATE,
                &usage(format!(
                    "Skill のフォルダと、--semantic --task <外れた具体> か --patch <JSON Patch> のどちらか一方を渡してください。{USAGE}"
                )),
            );
        }
    };
    if folder.is_none() {
        return refuse(
            UPDATE,
            &usage(format!("直す Skill のフォルダを渡してください。{USAGE}")),
        );
    }
    // 使い方の誤りは、semantic-agent-schema を探す前に返す（--if-match ・ 層の理由が無い --patch では semantic-agent-schema を打たない）
    if let Err(r) = update::check_mode(&mode) {
        return refuse(UPDATE, &r);
    }
    let sas = match locate() {
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

fn run_distribute(args: &[String]) -> ExitCode {
    let mut folder = None;
    let mut tool = None;
    let mut scope = None;
    let mut rest = args.iter();
    while let Some(a) = rest.next() {
        let slot = match a.as_str() {
            "--tool" => &mut tool,
            "--scope" => &mut scope,
            flag if flag.starts_with("--") => {
                return refuse(
                    DISTRIBUTE,
                    &usage(format!("知らない指定です: {flag}。{USAGE}")),
                );
            }
            _ if folder.is_none() => {
                folder = Some(a.as_str());
                continue;
            }
            _ => {
                return refuse(
                    DISTRIBUTE,
                    &usage(format!("フォルダは1つだけ渡してください。{USAGE}")),
                );
            }
        };
        match rest.next() {
            Some(v) => *slot = Some(v.as_str()),
            None => {
                return refuse(
                    DISTRIBUTE,
                    &usage(format!("{a} に値を渡してください。{USAGE}")),
                );
            }
        }
    }
    let (Some(folder), Some(tool_name), Some(scope_name)) = (folder, tool, scope) else {
        return refuse(
            DISTRIBUTE,
            &usage(format!(
                "Skill のフォルダと、--tool と --scope を渡してください。{USAGE}"
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
    let sas = match locate() {
        Ok(s) => s,
        Err(r) => return refuse(DISTRIBUTE, &r),
    };
    let tree = WorkingTree::new(PathBuf::from("."));
    match distribute(
        Some(folder),
        Some(tool),
        Some(scope),
        &sas,
        &sas,
        &tree,
        &JsonSchema,
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

/// AI のツールのセッションを見分ける表（Semantic Agent Schema の callers.json と同じ行）。
const CALLERS: &str = include_str!("../callers.json");

/// 人が打ったか：表のどの環境変数にも値が無ければ人。表を読めなければ、人とみなさない。
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

fn run_withdraw(args: &[String]) -> ExitCode {
    let mut folder = None;
    let mut reason = None;
    let mut rest = args.iter();
    while let Some(a) = rest.next() {
        match a.as_str() {
            "--reason" => match rest.next() {
                Some(v) => reason = Some(v.as_str()),
                None => {
                    return refuse(
                        WITHDRAW,
                        &usage(format!("--reason に理由を渡してください。{USAGE}")),
                    );
                }
            },
            flag if flag.starts_with("--") => {
                return refuse(
                    WITHDRAW,
                    &usage(format!("知らない指定です: {flag}。{USAGE}")),
                );
            }
            _ if folder.is_none() => folder = Some(a.as_str()),
            _ => {
                return refuse(
                    WITHDRAW,
                    &usage(format!("フォルダは1つだけ渡してください。{USAGE}")),
                );
            }
        }
    }
    let tree = WorkingTree::new(PathBuf::from("."));
    match withdraw(folder, reason, by_human(), &tree) {
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
    let sas = match locate() {
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
                "確かめる Skill のフォルダを1つ以上渡してください。{USAGE}"
            )),
        );
    }
    let sas = match locate() {
        Ok(s) => s,
        Err(r) => return refuse(VALIDATE, &r),
    };
    let tree = WorkingTree::new(PathBuf::from("."));
    match validate(args, &sas, &sas, &tree, &JsonSchema) {
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

fn run_write_skill_md(args: &[String]) -> ExitCode {
    if let Some(flag) = args.iter().find(|a| a.starts_with("--")) {
        return refuse(
            WRITE_SKILL_MD,
            &usage(format!("知らない指定です: {flag}。{USAGE}")),
        );
    }
    let folder = match args {
        [] => None,
        [one] => Some(one.as_str()),
        _ => {
            return refuse(
                WRITE_SKILL_MD,
                &usage(format!("フォルダは1つだけ渡してください。{USAGE}")),
            );
        }
    };
    let tree = WorkingTree::new(PathBuf::from("."));
    match write_skill_md(folder, &tree, &tree) {
        Ok(b) => {
            output::print_written(&b);
            ExitCode::SUCCESS
        }
        Err(r) => refuse(WRITE_SKILL_MD, &r),
    }
}

fn refuse(schema: &str, r: &Refused) -> ExitCode {
    output::print_refused(schema, r);
    ExitCode::from(if r.code == "usage" { 2 } else { 1 })
}
