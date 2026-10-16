//! CLI の受け口。`<操作> <Skill のフォルダ> --引数 値 …` を読み、application を呼び、JSON と終了コードを返す。
//! 終了コードは 0 正常 ／ 1 不合格 ・ 判定不能 ・ 止まった ／ 2 使い方の誤り。

use serde_json::{json, Value};
use skills_creator_core::application::{check, engine, scaffold, Env};
use skills_creator_core::domain::release::host_target;
use skills_creator_core::ports::{Files, Programs, Releases};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// 操作が受け付ける旗の名前と、必須かどうか。
type Flags = &'static [(&'static str, bool)];

/// 操作の一覧。名前 ・ 説明 ・ 受け付ける旗。
const COMMANDS: [(&str, &str, Flags); 3] = [
    (
        "scaffold",
        "まだ無いフォルダに、新しい Skill の雛形（SKILL.md ・ 入口 ・ 照合した基盤のエンジン ・ スキーマの雛形）を置く",
        &[("engine", false), ("target", false)],
    ),
    (
        "engine",
        "Skill のフォルダの scripts/bin/ に、基盤のエンジンを版を指して入れる（入れ替える）",
        &[("version", true), ("target", false)],
    ),
    (
        "check",
        "Skill のフォルダを項目ごとに検査する。不合格か判定不能が1件でもあれば終了コード 1",
        &[],
    ),
];

/// 読んだ引数。
#[derive(Debug, PartialEq)]
pub enum Request {
    List,
    Version,
    Run {
        command: String,
        dir: PathBuf,
        flags: BTreeMap<String, String>,
    },
}

/// 引数を読む。合わなければ、使い方の誤りの説明を返す。
pub fn parse(args: &[String]) -> Result<Request, String> {
    let args: Vec<&String> = args.iter().filter(|arg| *arg != "--json").collect();
    match args.as_slice() {
        [] => Ok(Request::List),
        [only] if only.as_str() == "--version" => Ok(Request::Version),
        [command, dir, rest @ ..] => {
            let (_, _, allowed) = COMMANDS
                .iter()
                .find(|(name, _, _)| name == command)
                .ok_or(format!("知らない操作: {command}"))?;
            let mut flags = BTreeMap::new();
            let mut words = rest.iter();
            while let Some(key) = words.next() {
                let name = key
                    .strip_prefix("--")
                    .ok_or(format!("知らない引数: {key}"))?;
                if !allowed.iter().any(|(known, _)| *known == name) {
                    return Err(format!("{command} は --{name} を受け付けない"));
                }
                let value = words.next().ok_or(format!("--{name} に値が無い"))?;
                flags.insert(name.to_owned(), (*value).clone());
            }
            if let Some((missing, _)) = allowed
                .iter()
                .find(|(name, required)| *required && !flags.contains_key(*name))
            {
                return Err(format!("{command} には --{missing} が必要"));
            }
            Ok(Request::Run {
                command: (*command).clone(),
                dir: PathBuf::from(dir.as_str()),
                flags,
            })
        }
        [command] => Err(format!("{command} には Skill のフォルダが必要")),
    }
}

fn misuse(detail: &str) -> (i32, Value) {
    let names: Vec<&str> = COMMANDS.iter().map(|(name, _, _)| *name).collect();
    (
        2,
        json!({"ok": false, "reason": "使い方が違う", "detail": detail, "commands": names}),
    )
}

fn list() -> Value {
    let commands: Vec<Value> = COMMANDS
        .iter()
        .map(|(name, summary, flags)| {
            let flags: Vec<Value> = flags
                .iter()
                .map(|(flag, required)| json!({"name": flag, "required": required}))
                .collect();
            json!({"name": name, "summary": summary, "args": flags})
        })
        .collect();
    json!({"ok": true, "data": {"commands": commands}})
}

/// 引数を読み、操作を行い、終了コードと出力を返す。own は skills-creator 自身の Skill のフォルダ。
pub fn run<F: Files, P: Programs, R: Releases>(
    args: &[String],
    own: &Path,
    env: &Env<F, P, R>,
) -> (i32, Value) {
    let request = match parse(args) {
        Ok(request) => request,
        Err(detail) => return misuse(&detail),
    };
    let (command, dir, flags) = match request {
        Request::List => return (0, list()),
        Request::Version => {
            return (
                0,
                json!({"ok": true, "data": {"name": "skills-creator", "version": env!("CARGO_PKG_VERSION")}}),
            )
        }
        Request::Run {
            command,
            dir,
            flags,
        } => (command, dir, flags),
    };
    let target = flags
        .get("target")
        .map(String::as_str)
        .unwrap_or(host_target());
    let done = match command.as_str() {
        "scaffold" => scaffold::scaffold(
            env,
            own,
            &dir,
            flags.get("engine").map(String::as_str),
            target,
        ),
        "engine" => engine::install(env, &dir, &flags["version"], target),
        _ => {
            let report = check::check(env, own, &dir);
            return (report.code(), report.to_json());
        }
    };
    match done {
        Ok(data) => (0, json!({"ok": true, "data": data})),
        Err(reason) => (1, json!({"ok": false, "reason": reason})),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| (*arg).to_owned()).collect()
    }

    fn run_args(command: &str, dir: &str, flags: &[(&str, &str)]) -> Result<Request, String> {
        Ok(Request::Run {
            command: command.to_owned(),
            dir: PathBuf::from(dir),
            flags: flags
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
        })
    }

    #[test]
    fn parse_reads_each_command_and_rejects_the_rest() {
        let cases = [
            (vec![], Ok(Request::List)),
            (vec!["--json"], Ok(Request::List)),
            (vec!["--version"], Ok(Request::Version)),
            (vec!["check", "s/a"], run_args("check", "s/a", &[])),
            (
                vec!["check", "s/a", "--json"],
                run_args("check", "s/a", &[]),
            ),
            (
                vec!["scaffold", "s/a", "--engine", "0.1.0"],
                run_args("scaffold", "s/a", &[("engine", "0.1.0")]),
            ),
            (
                vec!["engine", "s/a", "--version", "0.1.0"],
                run_args("engine", "s/a", &[("version", "0.1.0")]),
            ),
            (
                vec!["engine", "s/a"],
                Err("engine には --version が必要".to_owned()),
            ),
            (
                vec!["check", "s/a", "--engine", "1"],
                Err("check は --engine を受け付けない".to_owned()),
            ),
            (
                vec!["check"],
                Err("check には Skill のフォルダが必要".to_owned()),
            ),
            (vec!["build", "s/a"], Err("知らない操作: build".to_owned())),
            (vec!["check", "s/a", "x"], Err("知らない引数: x".to_owned())),
            (
                vec!["engine", "s/a", "--version"],
                Err("--version に値が無い".to_owned()),
            ),
        ];
        let got: Vec<Result<Request, String>> =
            cases.iter().map(|(args, _)| parse(&words(args))).collect();
        let want: Vec<Result<Request, String>> = cases.into_iter().map(|(_, want)| want).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn the_list_names_every_command_with_its_arguments() {
        let listed = list();
        assert_eq!(listed["data"]["commands"][0]["name"], "scaffold");
        assert_eq!(
            listed["data"]["commands"][1]["args"],
            json!([{"name": "version", "required": true}, {"name": "target", "required": false}])
        );
        assert_eq!(listed["data"]["commands"][2]["name"], "check");
    }
}
