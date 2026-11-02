//! agents-skills-creator コマンドの入口（文脈 skills）。引数を読み、adapter を組み立ててユースケースを呼び、結果を JSON 1つで返す。
//!
//! コマンド：new（UC1 作る手順を返す。--like で既にある Skill と同じ形で作る手順）・ build（skill.json から SKILL.md を作る）・ confirm（確認の1画面の材料を返す）・
//! check（UC4 確かめる、Skill だけ）。creator は AI を起動せず、skill.json を書かない（SAS が書く）。
//! 終了コード：0 = 正常（check は検出が無い）／ 1 = check で検出がある、sas を呼べない、skill.json を読めない ／ 2 = 使い方の誤り。

mod output;

use outbound_filesystem::WorkingTree;
use outbound_jsonschema::JsonSchema;
use outbound_sas::Sas;
use output::{BUILD, CHECK, CONFIRM};
use skills_application::use_cases::Refused;
use skills_application::use_cases::build::build;
use skills_application::use_cases::check::check;
use skills_application::use_cases::confirm::confirm;
use skills_application::use_cases::new::{PROCEDURE, new_like_procedure, new_procedure};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "使い方：agents-skills-creator new [<Skill の置き場所>] ／ \
agents-skills-creator new --like <同じ形にする Skill のフォルダ> [<Skill の置き場所>] ／ \
agents-skills-creator build <Skill のフォルダ> ／ \
agents-skills-creator confirm <Skill のフォルダ> ／ \
agents-skills-creator check <Skill のフォルダ>...";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") => run_check(&args[1..]),
        Some("new") => run_new(&args[1..]),
        Some("build") => run_build(&args[1..]),
        Some("confirm") => run_confirm(&args[1..]),
        Some(other) => refuse(
            CHECK,
            &Refused {
                code: "usage",
                message: format!("知らないコマンドです: {other}。{USAGE}"),
            },
        ),
        None => refuse(
            CHECK,
            &Refused {
                code: "usage",
                message: format!("コマンドを指定してください。{USAGE}"),
            },
        ),
    }
}

fn run_check(args: &[String]) -> ExitCode {
    if let Some(flag) = args.iter().find(|a| a.starts_with("--")) {
        return refuse(
            CHECK,
            &Refused {
                code: "usage",
                message: format!("知らない指定です: {flag}。{USAGE}"),
            },
        );
    }
    if args.is_empty() {
        return refuse(
            CHECK,
            &Refused {
                code: "usage",
                message: format!("確かめる Skill のフォルダを1つ以上渡してください。{USAGE}"),
            },
        );
    }
    let Some(sas) = Sas::locate() else {
        return refuse(
            CHECK,
            &Refused {
                code: "sas_unavailable",
                message: "sas が PATH に無い（SAS を入れて、PATH に置いてください）".to_string(),
            },
        );
    };
    match check(
        args,
        &sas,
        &WorkingTree::new(PathBuf::from(".")),
        &JsonSchema,
    ) {
        Ok(c) => {
            output::print_checked(&c);
            if c.detected() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(r) => refuse(CHECK, &r),
    }
}

/// 引数がちょうど1つ（フォルダか置き場所）で、旗が無いか。無ければ None、多ければ使い方の誤り。
fn one(args: &[String], schema: &str) -> Result<Option<String>, ExitCode> {
    if let Some(flag) = args.iter().find(|a| a.starts_with("--")) {
        return Err(refuse(
            schema,
            &Refused {
                code: "usage",
                message: format!("知らない指定です: {flag}。{USAGE}"),
            },
        ));
    }
    match args {
        [] => Ok(None),
        [one] => Ok(Some(one.clone())),
        _ => Err(refuse(
            schema,
            &Refused {
                code: "usage",
                message: format!("フォルダは1つだけ渡してください。{USAGE}"),
            },
        )),
    }
}

fn run_new(args: &[String]) -> ExitCode {
    let tree = WorkingTree::new(PathBuf::from("."));
    if args.first().map(String::as_str) == Some("--like") {
        // --like <フォルダ> [<置き場所>]。フォルダが無ければ使い方の誤り
        let (like, rest) = match args.get(1) {
            Some(f) if !f.starts_with("--") => (Some(f.as_str()), &args[2..]),
            _ => (None, &args[1..]),
        };
        let place = match one(rest, PROCEDURE) {
            Ok(p) => p,
            Err(code) => return code,
        };
        return match new_like_procedure(like, place.as_deref(), &tree) {
            Ok(p) => {
                output::print_procedure(&p);
                ExitCode::SUCCESS
            }
            Err(r) => refuse(PROCEDURE, &r),
        };
    }
    let place = match one(args, PROCEDURE) {
        Ok(p) => p,
        Err(code) => return code,
    };
    output::print_procedure(&new_procedure(place.as_deref(), &tree));
    ExitCode::SUCCESS
}

fn run_build(args: &[String]) -> ExitCode {
    let folder = match one(args, BUILD) {
        Ok(f) => f,
        Err(code) => return code,
    };
    let tree = WorkingTree::new(PathBuf::from("."));
    match build(folder.as_deref(), &tree, &tree) {
        Ok(b) => {
            output::print_built(&b);
            ExitCode::SUCCESS
        }
        Err(r) => refuse(BUILD, &r),
    }
}

fn run_confirm(args: &[String]) -> ExitCode {
    let folder = match one(args, CONFIRM) {
        Ok(f) => f,
        Err(code) => return code,
    };
    match confirm(folder.as_deref(), &WorkingTree::new(PathBuf::from("."))) {
        Ok(c) => {
            output::print_confirmation(&c);
            ExitCode::SUCCESS
        }
        Err(r) => refuse(CONFIRM, &r),
    }
}

fn refuse(schema: &str, r: &Refused) -> ExitCode {
    output::print_refused(schema, r);
    ExitCode::from(if r.code == "usage" { 2 } else { 1 })
}
