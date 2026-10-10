//! skills-creator の実行ファイル。scaffold と check の2つを持つ。
//!
//! 使い方：
//!   skills-creator scaffold <Skill のフォルダ> --kind <種類> [--pages] [--tools] [--refs] [--json]
//!   skills-creator check <Skill のフォルダ> [--level source|dist] [--json]
//! どちらも --engine <基盤のエンジン> と --self <skills-creator の Skill のフォルダ> を受ける。
//! 渡さなければ、この実行ファイルと同じフォルダの schema-driven と、その1つ上のフォルダを使う。
//! 引数が無く、標準入力に {"tool": …, "args": {…}} があれば、基盤から呼ばれたものとして読む。
//! 終了コードは 0 合格 ・ 1 不合格か失敗 ・ 2 使い方の誤り。

mod check;
mod engine;
mod read;
mod scaffold;

use serde_json::{json, Value};
use std::io::{IsTerminal, Read as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() && !std::io::stdin().is_terminal() {
        match from_stdin() {
            Ok(a) => args = a,
            Err(why) => return misuse(&why),
        }
    }
    let Some(tool) = args.first().cloned() else {
        return misuse("ツールの名前が無い（scaffold か check）");
    };
    let mut positional = Vec::new();
    let mut flags: Vec<(String, Option<String>)> = Vec::new();
    let mut rest = args.into_iter().skip(1).peekable();
    while let Some(arg) = rest.next() {
        if let Some(flag) = arg.strip_prefix("--") {
            let takes_value = matches!(flag, "kind" | "level" | "engine" | "self");
            let value = if takes_value { rest.next() } else { None };
            flags.push((flag.to_owned(), value));
        } else {
            positional.push(arg);
        }
    }
    let flag = |name: &str| {
        flags
            .iter()
            .find(|(f, _)| f == name)
            .map(|(_, v)| v.clone())
    };
    let allowed: &[&str] = match tool.as_str() {
        "check" | "check-skill" => &["level", "json", "engine", "self"],
        "scaffold" => &["kind", "pages", "tools", "refs", "json", "engine", "self"],
        other => return misuse(&format!("知らないツール：{other}")),
    };
    if let Some((unknown, _)) = flags.iter().find(|(f, _)| !allowed.contains(&f.as_str())) {
        return misuse(&format!("{tool} は --{unknown} を受けない"));
    }
    let [dir] = positional.as_slice() else {
        return misuse("Skill のフォルダを1つ渡す");
    };
    let here = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));
    let engine = flag("engine")
        .flatten()
        .map(PathBuf::from)
        .unwrap_or_else(|| here.join("schema-driven"));
    let own_skill = flag("self")
        .flatten()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            here.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        });
    // エンジンは Skill のフォルダの名前で型のスキーマを引くので、.. を含まない形にする
    let own_skill = own_skill.canonicalize().unwrap_or(own_skill);
    let as_json = flag("json").is_some();

    let (code, value) = match tool.as_str() {
        "scaffold" => {
            let Some(Some(kind)) = flag("kind") else {
                return misuse("scaffold には --kind <種類> を渡す");
            };
            let request = scaffold::Request {
                dir: Path::new(dir),
                kind: &kind,
                pages: flag("pages").is_some(),
                tools: flag("tools").is_some(),
                refs: flag("refs").is_some(),
            };
            match scaffold::scaffold(&own_skill, &request) {
                Ok(value) => (0, value),
                Err(why) => (1, json!({"ok": false, "reason": "置けない", "detail": why})),
            }
        }
        _ => {
            let dist = match flag("level").flatten().as_deref() {
                None | Some("source") => false,
                Some("dist") => true,
                Some(other) => return misuse(&format!("--level は source か dist：{other}")),
            };
            let dir = Path::new(dir)
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(dir));
            let value = check::check(
                &dir,
                &check::Options {
                    engine,
                    own_skill,
                    dist,
                },
            );
            (if value["ok"] == true { 0 } else { 1 }, value)
        }
    };
    if as_json {
        println!("{value}");
    } else {
        print_text(&value);
    }
    ExitCode::from(code)
}

/// 基盤から呼ばれたとき：標準入力の {"tool", "args"} を、引数の並びに直す
fn from_stdin() -> Result<Vec<String>, String> {
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    if text.trim().is_empty() {
        return Err("ツールの名前が無い（scaffold か check）".into());
    }
    let value: Value =
        serde_json::from_str(&text).map_err(|e| format!("標準入力が JSON でない：{e}"))?;
    let mut args = vec![value["tool"].as_str().unwrap_or("").to_owned()];
    for (key, v) in value["args"].as_object().into_iter().flatten() {
        match key.as_str() {
            "dir" => args.push(v.as_str().unwrap_or("").to_owned()),
            _ => match v {
                Value::Bool(true) => args.push(format!("--{key}")),
                Value::String(s) => {
                    args.push(format!("--{key}"));
                    args.push(s.clone());
                }
                _ => {}
            },
        }
    }
    args.push("--json".into());
    Ok(args)
}

fn print_text(value: &Value) {
    if let Some(results) = value["results"].as_array() {
        println!(
            "{}（{}）",
            value["skill"].as_str().unwrap_or(""),
            value["level"].as_str().unwrap_or("")
        );
        for r in results {
            let mark = match r["status"].as_str().unwrap_or("") {
                "pass" => "合格  ",
                "fail" => "不合格",
                "warn" => "警告  ",
                "unknown" => "判定不能",
                _ => "対象外",
            };
            let detail = r["detail"]
                .as_str()
                .map(|d| format!("　{d}"))
                .unwrap_or_default();
            println!(
                "  {mark}  {}  {}{detail}",
                r["id"].as_str().unwrap_or(""),
                r["label"].as_str().unwrap_or("")
            );
        }
        let c = &value["counts"];
        println!(
            "→ {}（不合格 {} ・ 判定不能 {} ・ 警告 {} ・ 対象外 {}）",
            if value["ok"] == true {
                "合格"
            } else {
                "不合格"
            },
            c["fail"],
            c["unknown"],
            c["warn"],
            c["skip"]
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(value).unwrap_or_default()
        );
    }
}

fn misuse(why: &str) -> ExitCode {
    println!(
        "{}",
        json!({"ok": false, "reason": "使い方が違う", "detail": why, "tools": ["scaffold", "check"]})
    );
    ExitCode::from(2)
}
