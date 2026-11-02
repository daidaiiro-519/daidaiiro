//! skills コマンドの入口（文脈 skills）。引数を読み、adapter を組み立ててユースケースを呼び、結果を JSON 1つで返す。
//!
//! コマンドは文脈 skills とユースケースの動詞で、SAS の言葉にそろえる：create（UC1 作る。sas create を呼ぶ）・
//! validate（UC4 確かめる。sas validate と creator の検査と承認の材料）・ write-skill-md（skill.json から SKILL.md を書く）。
//! creator は AI を起動せず、skill.json を書かない（SAS が書く）。
//! 終了コード：0 = 正常（validate は検出が無い）／ 1 = validate で検出がある、sas が断った、sas を呼べない、skill.json を
//! 読めない ／ 2 = 使い方の誤り。

mod output;

use outbound_filesystem::WorkingTree;
use outbound_jsonschema::JsonSchema;
use outbound_sas::Sas;
use output::{CREATE, VALIDATE, WRITE_SKILL_MD};
use skills_application::use_cases::Refused;
use skills_application::use_cases::create::{Mode, create};
use skills_application::use_cases::validate::validate;
use skills_application::use_cases::write_skill_md::write_skill_md;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "使い方：skills create <置き場所> --concrete <URN> --semantic ／ \
skills create <Skill のフォルダ> --concrete <URN> --value <JSON> ／ \
skills validate <Skill のフォルダ>... ／ \
skills write-skill-md <Skill のフォルダ>";

fn usage(message: String) -> Refused {
    Refused {
        code: "usage",
        message,
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("create") => run_create(&args[1..]),
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
        code: "sas_unavailable",
        message: "sas が PATH に無い（SAS を入れて、PATH に置いてください）".to_string(),
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
                Some(v) => mode = Some(Mode::Value(v.clone())),
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
                    &usage(format!(
                        "置き場所かフォルダは1つだけ渡してください。{USAGE}"
                    )),
                );
            }
        }
    }
    if target.is_none() || concrete.is_none() || mode.is_none() {
        return refuse(
            CREATE,
            &usage(format!(
                "置き場所かフォルダと、--concrete <URN> と、--semantic か --value <JSON> を渡してください。{USAGE}"
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
    match validate(args, &sas, &tree, &JsonSchema) {
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
