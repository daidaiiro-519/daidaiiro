// SPDX-License-Identifier: MIT
//! brainstorming-board の道具の宣言。**能力の正本はここである。**
//!
//! 入口（CLI ・ MCP）はこの宣言から組む ── 能力を2回書くと、片方だけが古くなる。
//! 許可辺は `Cargo.toml` が宣言する ── この crate は部品だけを参照する。

pub mod contract;

use std::path::PathBuf;

use bb_parts::{figcheck, init, render, serve, tokens};
use serde_json::{json, Value};

pub use contract::{catalog, Arg, Given, Outcome, Tool};

/// ブレストボードの置き場所の親の既定。
const BOARDS: &str = ".brainstorming-board";
/// 回答の蓄積先の既定。
const ANSWERS: &str = "answers";
/// 待ち受ける番号の既定。
const PORT: &str = "8731";

/// この Skill の置き場所。**呼ぶ側が決める** ── どこから呼ばれるかを、この側で推測しない。
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

fn crosses(findings: &[String]) -> String {
    findings
        .iter()
        .map(|x| format!("  × {x}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn lines_of(out: &Outcome, key: &str) -> Vec<String> {
    out.data
        .get(key)
        .and_then(|x| x.as_array())
        .map(|rows| {
            rows.iter()
                .filter_map(|x| x.as_str())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn text(out: &Outcome, key: &str) -> String {
    out.data
        .get(key)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_owned()
}

/// `ok` を、検査の結果で決める。**検出が在るなら、正常に終わっていない。**
fn with_ok(ok: bool, findings: Vec<String>, data: Value) -> Outcome {
    let mut out = Outcome::found(findings, data);
    out.ok = ok;
    out
}

fn run_init(given: &Given) -> Outcome {
    let name = given.one("name", "");
    if name.is_empty() {
        return Outcome::misuse("ブレストボードの名前を渡していない".to_owned());
    }
    let dir = given.one("dir", BOARDS);
    match init::create(
        &or_misuse!(references(given)),
        name,
        dir,
        given.one("title", ""),
    ) {
        Ok(lines) => Outcome::found(
            Vec::new(),
            json!({
                "name": name,
                "path": PathBuf::from(dir).join(name).display().to_string(),
                "lines": lines,
            }),
        ),
        // **検出であって、誤用ではない** ── 既に在ることは、呼び方の誤りではない
        Err(why) => Outcome::found(vec![why], json!({ "name": name })),
    }
}

fn human_init(out: &Outcome) -> String {
    let lines = lines_of(out, "lines");
    if lines.is_empty() {
        out.findings.join("\n")
    } else {
        lines.join("\n")
    }
}

fn run_validate(given: &Given) -> Outcome {
    let board = given.one("board", "");
    if board.is_empty() {
        return Outcome::misuse("ブレストボードのディレクトリを渡していない".to_owned());
    }
    let dir = match render::board_dir(board) {
        Ok(dir) => dir,
        Err(why) => return Outcome::misuse(why),
    };
    match bb_parts::validate::check(&or_misuse!(references(given)), &dir) {
        Ok(bad) => Outcome::found(bad, json!({ "board": board })),
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
    let board = given.one("board", "");
    if board.is_empty() {
        return Outcome::misuse("ブレストボードのディレクトリを渡していない".to_owned());
    }
    let dir = match render::board_dir(board) {
        Ok(dir) => dir,
        Err(why) => return Outcome::misuse(why),
    };
    let refs = or_misuse!(references(given));
    let made = match render::render(&refs, &dir, true) {
        Ok(made) => made,
        Err(why) => {
            let lines: Vec<String> = why.lines().map(str::to_owned).collect();
            let findings = if lines.len() > 1 && lines[0].ends_with(':') {
                lines[1..].to_vec()
            } else {
                lines.clone()
            };
            return Outcome::found(findings, json!({ "board": board, "lines": lines }));
        }
    };
    if !given.one("check", "").is_empty() {
        return match render::check_idempotent(&refs, &dir, &made.body) {
            Ok(same) => Outcome::found(
                same.findings,
                json!({ "board": board, "checked": true, "message": same.message }),
            ),
            Err(why) => Outcome::misuse(why),
        };
    }
    match render::write(&refs, &dir, &made.body) {
        // **出す前の検査は、検出ではなく注記である** ── 見つけるが、直さない。
        // 組めたことを打ち消さないので、終了コードは 0 のままにする
        Ok(bytes) => Outcome::found(
            Vec::new(),
            json!({
                "board": board,
                "path": dir.join("board.html").display().to_string(),
                "bytes": bytes,
                "changed": made.changed,
                "notes": made.notes,
            }),
        ),
        Err(why) => Outcome::misuse(why),
    }
}

fn human_render(out: &Outcome) -> String {
    if out.data.get("checked").and_then(Value::as_bool) == Some(true) {
        let tail = text(out, "message");
        return if out.findings.is_empty() {
            tail
        } else {
            format!("{}\n{tail}", crosses(&out.findings))
        };
    }
    let path = text(out, "path");
    if path.is_empty() {
        return lines_of(out, "lines").join("\n");
    }
    // **出す前の検査は、組めたことを打ち消さない** ── 見つけたことは添えて出す
    let head: Vec<String> = lines_of(out, "notes")
        .iter()
        .map(|x| format!("  △ {x}"))
        .collect();
    if head.is_empty() {
        format!("書き出し: {path}")
    } else {
        format!("{}\n書き出し: {path}", head.join("\n"))
    }
}

fn run_freeze(given: &Given) -> Outcome {
    let board = given.one("board", "");
    if board.is_empty() {
        return Outcome::misuse("ブレストボードのディレクトリを渡していない".to_owned());
    }
    let dir = match render::board_dir(board) {
        Ok(dir) => dir,
        Err(why) => return Outcome::misuse(why),
    };
    match render::freeze(&or_misuse!(references(given)), &dir) {
        Ok(message) => Outcome::found(Vec::new(), json!({ "board": board, "message": message })),
        Err(why) => Outcome::misuse(why),
    }
}

fn human_freeze(out: &Outcome) -> String {
    text(out, "message")
}

fn run_figures(given: &Given) -> Outcome {
    let figures = given.all("figure").to_vec();
    if figures.is_empty() {
        return Outcome::misuse("検査する図を渡していない".to_owned());
    }
    match figcheck::count(&figures) {
        Ok(found) => {
            let total: usize = found.iter().map(figcheck::Found::total).sum();
            let mut lines = Vec::new();
            for one in &found {
                lines.push(one.headline());
                for x in &one.lines {
                    lines.push(format!("   {x}"));
                }
            }
            with_ok(
                total == 0,
                lines,
                json!({ "figures": figures, "count": total }),
            )
        }
        Err(why) => Outcome::misuse(why),
    }
}

fn human_figures(out: &Outcome) -> String {
    let count = out
        .data
        .get("count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let tail = if count == 0 {
        "食い違いは無い".to_owned()
    } else {
        format!("直すところが {count} 件ある")
    };
    let mut lines = out.findings.clone();
    lines.push(tail);
    lines.join("\n")
}

fn run_serve(given: &Given) -> Outcome {
    let port: u16 = given
        .one("port", PORT)
        .parse()
        .unwrap_or(serve::DEFAULT_PORT);
    match serve::run(given.one("root", "."), given.one("answers", ANSWERS), port) {
        Ok(lines) => Outcome::found(Vec::new(), json!({ "lines": lines })),
        Err(why) => Outcome::misuse(why),
    }
}

fn human_serve(out: &Outcome) -> String {
    if out.ok {
        "止めました".to_owned()
    } else {
        "配れなかった".to_owned()
    }
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
    text(out, "css")
}

/// この Skill が持つ道具の一覧。**能力の正本である。**
#[must_use]
pub fn tools() -> Vec<Tool> {
    let root = Arg::opt(
        "skill_root",
        "この Skill の置き場所（既定は、実行ファイルの1つ上）",
        None,
    );
    let board = Arg::need("board", "ブレストボードのディレクトリ");
    vec![
        Tool {
            name: "init",
            summary: "ブレストボードの置き場所と雛形と索引の行を作る",
            args: vec![
                Arg::need("name", "ブレストボードの名前（英小文字とハイフン）"),
                Arg::opt("dir", "置き場所の親", Some(BOARDS)),
                Arg::opt("title", "題。省くと名前をそのまま使う", None),
                root.clone(),
            ],
            run: run_init,
            human: human_init,
        },
        Tool {
            name: "validate",
            summary: "入力（board.json）を検査する",
            args: vec![board.clone(), root.clone()],
            run: run_validate,
            human: human_validate,
        },
        Tool {
            name: "render",
            summary: "JSON からブレストボードを組む",
            args: vec![
                board.clone(),
                Arg::opt("check", "冪等だけを検査する", None),
                root.clone(),
            ],
            run: run_render,
            human: human_render,
        },
        Tool {
            name: "tokens",
            summary: "トークンを検査し、CSS を出す",
            args: vec![Arg::opt("check", "検査だけを実行する", None), root.clone()],
            run: run_tokens,
            human: human_tokens,
        },
        Tool {
            name: "freeze",
            summary: "いまの姿を、前の回の基準として保存する",
            args: vec![board, root.clone()],
            run: run_freeze,
            human: human_freeze,
        },
        Tool {
            name: "figures",
            summary: "図の重なり ・ はみ出し ・ 貫通を検査する",
            args: vec![Arg::many("figure", "検査する SVG（複数可）"), root.clone()],
            run: run_figures,
            human: human_figures,
        },
        Tool {
            name: "serve",
            summary: "ブレストボードを配り、押された回答を蓄積する",
            args: vec![
                Arg::opt("root", "画面を置いてあるディレクトリ", Some(".")),
                Arg::opt("answers", "回答の蓄積先", Some(ANSWERS)),
                Arg::opt("port", "待ち受ける番号", Some(PORT)),
                root,
            ],
            run: run_serve,
            human: human_serve,
        },
    ]
}
