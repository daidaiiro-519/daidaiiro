// SPDX-License-Identifier: MIT
//! acdr の道具の宣言。**能力の正本はここである。**
//!
//! 入口（CLI ・ MCP）はこの宣言から組む ── 能力を2回書くと、片方だけが古くなる。
//! 許可辺は `Cargo.toml` が宣言する ── この crate は部品だけを参照する。

pub mod contract;

use std::path::PathBuf;

use acd_parts::{record, tokens, validate};
use serde_json::{json, Value};

pub use contract::{Arg, Given, Outcome, Tool};

/// 題を書いていないことが、出来上がりから分かる文字列。
/// この Skill の部品が呼ぶ外部の道具。**OS によって無いコマンド（date ・ timeout など）を
/// 書かない** ── 日付の計算と時間の制限は Rust の中で行う。名前を実行時に決める道具は
/// `"*"`（利用者が指定する道具）と書く。skills-creator の check が、部品の呼び出しと照合する。
/// git ── 差分の変更前を git show で取得する
pub const REQUIRES: &[&str] = &["git"];

const DEFAULT_TITLE: &str = "題を記入する";

/// この Skill の references/。**置き場所は Given::skill_root が求める。**
/// Skill の置き場所が見つからなければ、誤用として返す。**黙って「.」へ寄せない** ──
/// 実行した場所で結果が変わり、契約を読めずに止まる（ACDR 0019 ・ 0029）。
macro_rules! or_misuse {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(why) => return Outcome::misuse(why),
        }
    };
}

fn references(given: &Given) -> Result<PathBuf, String> {
    Ok(given.skill_root()?.join("references"))
}

/// 検出を、人が読む行へ組む。
fn crosses(findings: &[String]) -> String {
    findings
        .iter()
        .map(|x| format!("  × {x}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn lines_of(out: &Outcome) -> Vec<String> {
    out.data
        .get("lines")
        .and_then(|x| x.as_array())
        .map(|rows| {
            rows.iter()
                .filter_map(|x| x.as_str())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn run_new(given: &Given) -> Outcome {
    let record = given.one("record", "");
    if record.is_empty() {
        return Outcome::misuse("記録のフォルダを渡していない".to_owned());
    }
    let folder = PathBuf::from(record);
    match record::new(
        &or_misuse!(references(given)),
        &folder,
        given.one("title", DEFAULT_TITLE),
    ) {
        Ok(lines) => Outcome::found(
            Vec::new(),
            json!({
                "record": record,
                "path": folder.join("acdr.json").display().to_string(),
                "lines": lines,
            }),
        ),
        Err(why) => Outcome::found(vec![why], json!({ "record": record })),
    }
}

fn human_new(out: &Outcome) -> String {
    let lines = lines_of(out);
    if lines.is_empty() {
        out.findings.join("\n")
    } else {
        lines.join("\n")
    }
}

fn run_validate(given: &Given) -> Outcome {
    let record = given.one("record", "");
    if record.is_empty() {
        return Outcome::misuse("記録のフォルダを渡していない".to_owned());
    }
    let folder = match std::fs::canonicalize(record) {
        Ok(folder) => folder,
        Err(e) => return Outcome::misuse(format!("{record}: 開けない ── {e}")),
    };
    let root = record::repo_root(&folder);
    match validate::check(&or_misuse!(references(given)), &folder, Some(&root)) {
        Ok(bad) => Outcome::found(bad, json!({ "record": record })),
        Err(why) => Outcome::misuse(why),
    }
}

fn human_validate(out: &Outcome) -> String {
    let tail = if out.findings.is_empty() {
        "入力の検査　通った".to_owned()
    } else {
        format!("入力の検査　通っていない（{} 件）", out.findings.len())
    };
    if out.findings.is_empty() {
        tail
    } else {
        format!("{}\n{tail}", crosses(&out.findings))
    }
}

fn run_render(given: &Given) -> Outcome {
    let record = given.one("record", "");
    if record.is_empty() {
        return Outcome::misuse("記録のフォルダを渡していない".to_owned());
    }
    let folder = PathBuf::from(record);
    let check_only = !given.one("check", "").is_empty();
    let force = !given.one("force", "").is_empty();
    let report =
        match record::build_record(&or_misuse!(references(given)), &folder, check_only, force) {
            Ok(report) => report,
            Err(why) => {
                // **入力の検査は、検出であって誤用ではない** ── 呼び方は正しい。
                // **見出しの行は検出ではない** ── 印字する行には残し、検出からは外す
                let lines: Vec<String> = why.lines().map(str::to_owned).collect();
                let findings = if lines.len() > 1 && lines[0].ends_with(':') {
                    lines[1..].to_vec()
                } else {
                    lines.clone()
                };
                return Outcome::found(
                    findings,
                    json!({ "record": record, "code": 1, "lines": lines }),
                );
            }
        };
    // **通過の行は検出ではない。** 検出は、検査に落ちた行 ・ 差が在る行 ・ 拒否した行である
    let mut bad: Vec<String> = report
        .lines
        .iter()
        .filter(|x| x.trim_start().starts_with("NG"))
        .map(|x| x.trim_start()[2..].trim().to_owned())
        .collect();
    bad.extend(
        report
            .lines
            .iter()
            .filter(|x| x.contains("差が在る") || x.contains("拒否"))
            .map(|x| x.trim().to_owned()),
    );
    Outcome::found(
        bad,
        json!({
            "record": record, "code": report.code, "lines": report.lines,
            "path": folder.join("index.html").display().to_string(),
        }),
    )
}

fn human_render(out: &Outcome) -> String {
    lines_of(out).join("\n")
}

fn run_tokens(given: &Given) -> Outcome {
    let refs = or_misuse!(references(given));
    let value = match tokens::load(&tokens::path(&refs)) {
        Ok(value) => value,
        Err(why) => return Outcome::misuse(why),
    };
    let bad = tokens::validate(&refs, &value);
    let ok = bad.is_empty();
    if !given.one("check", "").is_empty() {
        let base = tokens::raw(&value).map_or(0, |rows| rows.len());
        let meaning = value
            .pointer("/semantic/light")
            .and_then(|x| x.as_object())
            .map_or(0, serde_json::Map::len);
        let parts = value
            .get("component")
            .and_then(|x| x.as_object())
            .map_or(0, serde_json::Map::len);
        return with_ok(
            ok,
            bad,
            json!({ "checked": true, "base": base, "meaning": meaning, "parts": parts }),
        );
    }
    let css = if ok {
        tokens::css(&value, false).unwrap_or_default()
    } else {
        String::new()
    };
    with_ok(ok, bad, json!({ "css": css }))
}

/// `ok` を、検査の結果で決める。**検出が在るなら、正常に終わっていない。**
fn with_ok(ok: bool, findings: Vec<String>, data: Value) -> Outcome {
    let mut out = Outcome::found(findings, data);
    out.ok = ok;
    out
}

fn human_tokens(out: &Outcome) -> String {
    if !out.findings.is_empty() {
        return format!(
            "{}\nトークンの検査　通っていない（{} 件）",
            crosses(&out.findings),
            out.findings.len()
        );
    }
    let count = |key: &str| {
        out.data
            .get(key)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
    };
    if out.data.get("checked").and_then(Value::as_bool) == Some(true) {
        return format!(
            "トークンの検査　通った　／　基礎 {} ・ 意味 {} ・ 部品 {}",
            count("base"),
            count("meaning"),
            count("parts")
        );
    }
    out.data
        .get("css")
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_owned()
}

/// この Skill が持つ道具の一覧。**能力の正本である。**
#[must_use]
pub fn tools() -> Vec<Tool> {
    let root = Arg::opt(
        "skill_root",
        "この Skill の置き場所（既定は、実行ファイルの1つ上）",
        None,
    );
    vec![
        Tool {
            name: "new",
            summary: "雛形から記録のフォルダを起こす",
            args: vec![
                Arg::need("record", "記録のフォルダ（.acdr/<番号>-<短い名詞句>）"),
                Arg::opt("title", "題。短い名詞句", Some(DEFAULT_TITLE)),
                root.clone(),
            ],
            run: run_new,
            human: human_new,
        },
        Tool {
            name: "validate",
            summary: "入力（acdr.json）を検査する",
            args: vec![Arg::need("record", "記録のフォルダ"), root.clone()],
            run: run_validate,
            human: human_validate,
        },
        Tool {
            name: "render",
            summary: "acdr.json から1枚（index.html）を組む",
            args: vec![
                Arg::need("record", "記録のフォルダ"),
                Arg::opt("check", "組み直さず、差が無いかだけを検査する", None),
                Arg::opt("force", "承認済みの記録でも組み直す", None),
                root.clone(),
            ],
            run: run_render,
            human: human_render,
        },
        Tool {
            name: "tokens",
            summary: "トークンを検査し、CSS を出す",
            args: vec![Arg::opt("check", "検査だけを実行する", None), root],
            run: run_tokens,
            human: human_tokens,
        },
    ]
}
