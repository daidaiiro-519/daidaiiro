//! CLI の受け口。`<操作> --引数 値 …` を読み、application を呼び、JSON と終了コードを返す。
//! 基盤の操作（create ・ get ・ update ・ delete ・ prompt ・ check ・ check-schemas ・ approve）は、
//! この Skill のフォルダを渡してエンジンへそのまま渡し、終了コードと出力を変えずに返す。
//! 終了コードは 0 正常 ／ 1 不合格 ・ 失敗 ／ 2 使い方の誤り。

use brainstorming_core::application::{advance, engine, inspect, lock, render, tokens, Env, Skill};
use brainstorming_core::domain::agent::RUN_VAR;
use brainstorming_core::ports::{Clock, Files, Programs};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// 操作が受け付ける旗の名前と、必須かどうか。
type Flags = &'static [(&'static str, bool)];

/// brainstorming 自身の操作。名前 ・ 説明 ・ 受け付ける旗。
pub const COMMANDS: [(&str, &str, Flags); 6] = [
    ("advance", "ボードを次の回へ進める。前の回と比べて変わったところを理由と一緒に書き、回の記録を残す", &[("board", true)]),
    ("inspect", "ボードの本文を検査する（1文の長さ ・ 前提の出どころ ・ 下流の行き先 ・ 完成イメージ ・ いま見る論点）", &[("board", true)]),
    ("render", ".brainstorming/ の全部のボードとダッシュボードを site/ へ描く", &[("dir", true)]),
    ("serve", "127.0.0.1 で画面を配り、回答と AI のチャットを受ける。止めるまで戻らない", &[("dir", true), ("port", false)]),
    ("tokens", "デザイントークンの正本から design-system.json を作る。--check 1 なら書かずに比べる", &[("check", false)]),
    ("unlock", "ターミナルから取ったボードのロックを外す。advance をせずに作業をやめるときに使う", &[("board", true)]),
];

/// serve が起動した AI の中なら、その依頼の ID。
fn run_id() -> Option<String> {
    std::env::var(RUN_VAR).ok().filter(|v| !v.is_empty())
}

/// ボードを書き換える前に、ボードのロックを取る。ボードでなければ何もしない。
fn enter<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    file: &std::path::Path,
) -> Result<(), String> {
    match lock::board_of(env, file) {
        Some((layout, id)) => lock::enter_cli(env, &layout, &id, run_id().as_deref()),
        None => Ok(()),
    }
}

/// 読んだ引数。
#[derive(Debug, PartialEq)]
pub enum Request {
    List,
    Version,
    Base(Vec<String>),
    Run {
        command: String,
        flags: BTreeMap<String, String>,
    },
}

/// 引数を読む。合わなければ、使い方の誤りの説明を返す。
pub fn parse(args: &[String]) -> Result<Request, String> {
    let words: Vec<&String> = args.iter().filter(|arg| *arg != "--json").collect();
    let Some((command, rest)) = words.split_first() else {
        return Ok(Request::List);
    };
    if command.as_str() == "--version" && rest.is_empty() {
        return Ok(Request::Version);
    }
    if engine::BASE.contains(&command.as_str()) {
        return Ok(Request::Base(args.to_vec()));
    }
    let (_, _, allowed) = COMMANDS
        .iter()
        .find(|(name, _, _)| name == command)
        .ok_or(format!("知らない操作: {command}"))?;
    let mut flags = BTreeMap::new();
    let mut iter = rest.iter();
    while let Some(key) = iter.next() {
        let name = key
            .strip_prefix("--")
            .ok_or(format!("知らない引数: {key}"))?;
        if !allowed.iter().any(|(known, _)| *known == name) {
            return Err(format!("{command} は --{name} を受け付けない"));
        }
        let value = iter.next().ok_or(format!("--{name} に値が無い"))?;
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
        flags,
    })
}

fn misuse(detail: &str) -> (i32, String) {
    let mut names: Vec<&str> = COMMANDS.iter().map(|(name, _, _)| *name).collect();
    names.extend(engine::BASE);
    (
        2,
        json!({"ok": false, "reason": "使い方が違う", "detail": detail, "commands": names})
            .to_string(),
    )
}

fn list() -> Value {
    let commands: Vec<Value> = COMMANDS
        .iter()
        .map(|(name, summary, flags)| {
            let args: Vec<Value> = flags
                .iter()
                .map(|(flag, required)| json!({"name": flag, "required": required}))
                .collect();
            json!({"name": name, "summary": summary, "args": args})
        })
        .collect();
    json!({"ok": true, "data": {"commands": commands, "base": engine::BASE}})
}

/// 読んだ操作。serve は戻らないので、受け口の外（compose）で扱う。
pub enum Action {
    Done(i32, String),
    Serve { dir: PathBuf, port: u16 },
}

/// 引数を読み、操作を行い、終了コードと出力を返す。skill は brainstorming 自身の Skill のフォルダ。
pub fn run<F: Files, P: Programs, C: Clock>(
    args: &[String],
    skill: &Skill,
    env: &Env<F, P, C>,
) -> Action {
    let request = match parse(args) {
        Ok(request) => request,
        Err(detail) => {
            let (code, out) = misuse(&detail);
            return Action::Done(code, out);
        }
    };
    let (command, flags) = match request {
        Request::List => return Action::Done(0, list().to_string()),
        Request::Version => {
            return Action::Done(0, json!({"ok": true, "data": {"name": "brainstorming", "version": env!("CARGO_PKG_VERSION")}}).to_string())
        }
        Request::Base(args) => {
            if matches!(args.first().map(String::as_str), Some("update" | "delete")) {
                if let Some(at) = args.iter().position(|a| a == "--path") {
                    let file = args.get(at + 1).map(PathBuf::from).unwrap_or_default();
                    let file = std::path::absolute(&file).unwrap_or(file);
                    if let Err(reason) = enter(env, &file) {
                        return Action::Done(1, json!({"ok": false, "reason": reason}).to_string());
                    }
                }
            }
            let args = engine::resolve_args(env, skill, &args);
            return match engine::call(env, skill, &args) {
                Ok(run) => Action::Done(run.code, format!("{}{}", run.stdout.trim_end(), if run.stderr.is_empty() { String::new() } else { format!("\n{}", run.stderr.trim_end()) })),
                Err(reason) => Action::Done(1, json!({"ok": false, "reason": reason}).to_string()),
            }
        }
        Request::Run { command, flags } => (command, flags),
    };
    let path =
        |key: &str| std::path::absolute(&flags[key]).unwrap_or_else(|_| PathBuf::from(&flags[key]));
    let done = match command.as_str() {
        "advance" => {
            let board = path("board");
            if let Err(reason) = enter(env, &board) {
                return Action::Done(1, json!({"ok": false, "reason": reason}).to_string());
            }
            let done = advance::run(env, skill, &board);
            if done.is_ok() && run_id().is_none() {
                if let Some((layout, id)) = lock::board_of(env, &board) {
                    lock::leave_cli(env, &layout, &id);
                }
            }
            done
        }
        "unlock" => match lock::board_of(env, &path("board")) {
            Some((layout, id)) => {
                lock::leave_cli(env, &layout, &id);
                Ok(json!({"unlocked": id}))
            }
            None => Err("ボードの本体でない".to_owned()),
        },
        "inspect" => {
            return match inspect::run(env, &path("board")) {
                Ok(out) => Action::Done(if out["ok"] == true { 0 } else { 1 }, out.to_string()),
                Err(reason) => Action::Done(1, json!({"ok": false, "reason": reason}).to_string()),
            }
        }
        "render" => render::run(env, skill, &path("dir")),
        "tokens" => tokens::run(env, skill, flags.get("check").is_some_and(|v| v != "0")),
        _ => {
            let port = match flags.get("port").map(|p| p.parse::<u16>()) {
                Some(Ok(port)) => port,
                Some(Err(_)) => return Action::Done(2, misuse("--port は数で書く").1),
                None => 0,
            };
            return Action::Serve {
                dir: path("dir"),
                port,
            };
        }
    };
    match done {
        Ok(data) => Action::Done(0, json!({"ok": true, "data": data}).to_string()),
        Err(reason) => Action::Done(1, json!({"ok": false, "reason": reason}).to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| (*arg).to_owned()).collect()
    }

    fn run_args(command: &str, flags: &[(&str, &str)]) -> Result<Request, String> {
        Ok(Request::Run {
            command: command.to_owned(),
            flags: flags
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
        })
    }

    #[test]
    fn parse_reads_each_command_and_passes_base_operations_through() {
        let cases = [
            (vec![], Ok(Request::List)),
            (vec!["--version"], Ok(Request::Version)),
            (
                vec!["advance", "--board", "x.json"],
                run_args("advance", &[("board", "x.json")]),
            ),
            (
                vec!["serve", "--dir", ".brainstorming", "--port", "8752"],
                run_args("serve", &[("dir", ".brainstorming"), ("port", "8752")]),
            ),
            (vec!["tokens"], run_args("tokens", &[])),
            (
                vec!["check", "--dir", "d"],
                Ok(Request::Base(words(&["check", "--dir", "d"]))),
            ),
            (
                vec!["advance"],
                Err("advance には --board が必要".to_owned()),
            ),
            (
                vec!["render", "--board", "x"],
                Err("render は --board を受け付けない".to_owned()),
            ),
            (vec!["build"], Err("知らない操作: build".to_owned())),
            (vec!["inspect", "x"], Err("知らない引数: x".to_owned())),
            (
                vec!["inspect", "--board"],
                Err("--board に値が無い".to_owned()),
            ),
        ];
        let got: Vec<Result<Request, String>> =
            cases.iter().map(|(args, _)| parse(&words(args))).collect();
        let want: Vec<Result<Request, String>> = cases.into_iter().map(|(_, want)| want).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn the_list_names_own_commands_and_base_operations() {
        let listed = list();
        assert_eq!(listed["data"]["commands"][0]["name"], "advance");
        assert_eq!(
            listed["data"]["commands"][3]["args"],
            json!([{"name": "dir", "required": true}, {"name": "port", "required": false}])
        );
        assert_eq!(listed["data"]["base"][0], "create");
    }
}
