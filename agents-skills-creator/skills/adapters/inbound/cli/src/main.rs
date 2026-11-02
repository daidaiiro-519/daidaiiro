//! agents-skills-creator コマンドの入口（文脈 skills）。引数を読み、adapter を組み立ててユースケースを呼び、結果を JSON 1つで返す。
//!
//! いまのコマンドは check（UC4 確かめる、Skill だけ）。
//! 終了コード：0 = 検出が無い ／ 1 = どれかの Skill で検出がある、または sas を呼べない ／ 2 = 使い方の誤り。

mod output;

use outbound_filesystem::WorkingTree;
use outbound_sas::Sas;
use skills_application::use_cases::check::{Refused, check};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "使い方：agents-skills-creator check <Skill のフォルダ>...";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") => run_check(&args[1..]),
        Some(other) => refuse(&Refused {
            code: "usage",
            message: format!("知らないコマンドです: {other}。{USAGE}"),
        }),
        None => refuse(&Refused {
            code: "usage",
            message: format!("コマンドを指定してください。{USAGE}"),
        }),
    }
}

fn run_check(args: &[String]) -> ExitCode {
    if let Some(flag) = args.iter().find(|a| a.starts_with("--")) {
        return refuse(&Refused {
            code: "usage",
            message: format!("知らない指定です: {flag}。{USAGE}"),
        });
    }
    if args.is_empty() {
        return refuse(&Refused {
            code: "usage",
            message: format!("確かめる Skill のフォルダを1つ以上渡してください。{USAGE}"),
        });
    }
    let Some(sas) = Sas::locate() else {
        return refuse(&Refused {
            code: "sas_unavailable",
            message: "sas が PATH に無い（SAS を入れて、PATH に置いてください）".to_string(),
        });
    };
    match check(args, &sas, &WorkingTree::new(PathBuf::from("."))) {
        Ok(c) => {
            output::print_checked(&c);
            if c.detected() {
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
    ExitCode::from(if r.code == "usage" { 2 } else { 1 })
}
