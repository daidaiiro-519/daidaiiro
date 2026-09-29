// SPDX-License-Identifier: MIT
//! narration の道具の宣言。**能力の正本はここである。**
//!
//! 入口（CLI ・ MCP）はこの宣言から組む ── 能力を2回書くと、片方だけが古くなる。
//! 許可辺は `Cargo.toml` が宣言する ── この crate は部品だけを参照する。

pub mod contract;

use std::path::PathBuf;

use nar_parts::{mp3, voice};
use serde_json::json;

pub use contract::{catalog, Arg, Given, Outcome, Tool};

fn script_of(given: &Given) -> Result<(PathBuf, voice::Script, String), String> {
    let dir = PathBuf::from(given.one("directory", "."));
    let script = voice::load(&dir).map_err(|e| format!("原稿を読めない ── {e}"))?;
    let asked = given.one("voice", "");
    let chosen = if asked.is_empty() {
        script.voice.clone()
    } else {
        asked.to_owned()
    };
    Ok((dir, script, chosen))
}

fn run_plan(given: &Given) -> Outcome {
    let (dir, script, chosen) = match script_of(given) {
        Ok(got) => got,
        Err(why) => return Outcome::misuse(why),
    };
    let rows = voice::plan(&dir, &script, &chosen);
    let listed: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            json!({ "id": r.id, "key": r.key, "text": r.text,
                    "audio": r.audio, "marks": r.marks, "cached": r.cached })
        })
        .collect();
    Outcome::found(
        Vec::new(),
        json!({
            "voice": chosen, "engine": script.engine,
            "plan": listed,
            "make": rows.iter().filter(|r| !r.cached).count(),
            "take": rows.iter().filter(|r| r.cached).count(),
        }),
    )
}

fn human_plan(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    let n = |k: &str| {
        out.data
            .get(k)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
    };
    let (make, take) = (n("make"), n("take"));
    format!(
        "{} 枚　／　合成する {make} 枚　／　取り出す {take} 枚",
        make + take
    )
}

fn run_synth(given: &Given) -> Outcome {
    let (dir, script, chosen) = match script_of(given) {
        Ok(got) => got,
        Err(why) => return Outcome::misuse(why),
    };
    // **外部の道具は tool.json から読んで渡す** ── 部品は名前を直書きしない
    let aws = match given.external("aws") {
        Ok(aws) => aws,
        Err(why) => return Outcome::misuse(why),
    };
    match voice::run(&aws, &dir, &script, &chosen) {
        Ok(made) => Outcome::found(
            Vec::new(),
            json!({
                "made": made.made, "taken": made.taken,
                "durations": made.durations.iter()
                    .map(|(id, ms)| json!({ "id": id, "durationMs": ms }))
                    .collect::<Vec<_>>(),
            }),
        ),
        Err(e) => Outcome::misuse(e.to_string()),
    }
}

fn human_synth(out: &Outcome) -> String {
    if !out.ok {
        return "合成できなかった".to_owned();
    }
    let count = |k: &str| {
        out.data
            .get(k)
            .and_then(|x| x.as_array())
            .map_or(0, Vec::len)
    };
    let empty = vec![];
    let rows = out
        .data
        .get("durations")
        .and_then(|x| x.as_array())
        .unwrap_or(&empty);
    let total: u64 = rows
        .iter()
        .filter_map(|r| r.get("durationMs").and_then(serde_json::Value::as_u64))
        .sum();
    #[allow(clippy::cast_precision_loss)]
    let seconds = total as f64 / 1000.0;
    format!(
        "{} 枚　／　合成 {} 枚　／　取り出し {} 枚　／　合計 {seconds:.1} 秒",
        rows.len(),
        count("made"),
        count("taken")
    )
}

fn run_lexicon(given: &Given) -> Outcome {
    let path = PathBuf::from(given.one("path", ""));
    let name = given.one("name", "");
    if path.as_os_str().is_empty() || name.is_empty() {
        return Outcome::misuse("辞書の場所と名前の両方を渡す".to_owned());
    }
    let aws = match given.external("aws") {
        Ok(aws) => aws,
        Err(why) => return Outcome::misuse(why),
    };
    match nar_parts::voice::put_lexicon(&aws, &path, name) {
        Ok(()) => Outcome::found(
            Vec::new(),
            json!({ "name": name, "path": path.display().to_string() }),
        ),
        Err(e) => Outcome::misuse(e.to_string()),
    }
}

fn human_lexicon(out: &Outcome) -> String {
    if !out.ok {
        return "登録できなかった".to_owned();
    }
    format!(
        "登録した：{}",
        out.data
            .get("name")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
    )
}

fn run_measure(given: &Given) -> Outcome {
    let paths = given.all("path");
    if paths.is_empty() {
        return Outcome::misuse("音声を渡していない".to_owned());
    }
    let mut rows = Vec::new();
    for raw in paths {
        let Ok(data) = std::fs::read(raw) else {
            return Outcome::misuse(format!("読めない ── {raw}"));
        };
        rows.push(json!({ "path": raw, "durationMs": mp3::duration_ms(&data) }));
    }
    Outcome::found(Vec::new(), json!({ "measured": rows }))
}

fn human_measure(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    let empty = vec![];
    let rows = out
        .data
        .get("measured")
        .and_then(|x| x.as_array())
        .unwrap_or(&empty);
    let ms = |r: &serde_json::Value| {
        r.get("durationMs")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
    };
    // **1件なら、その尺だけを出す** ── 経路を添えると、呼ぶ側が数を取り出せない
    if let [only] = rows.as_slice() {
        return format!("{} ms", ms(only));
    }
    rows.iter()
        .map(|r| {
            format!(
                "{:>8}  {}",
                ms(r),
                r.get("path").and_then(|v| v.as_str()).unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// この Skill が持つ道具の一覧。**能力の正本である。**
#[must_use]
pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "plan",
            summary: "合成せずに、合成と取り出しの内訳を出す",
            args: vec![
                Arg::need("directory", "原稿の場所"),
                Arg::opt("voice", "読み上げる声", None),
            ],
            run: run_plan,
            human: human_plan,
        },
        Tool {
            name: "synth",
            summary: "原稿から、枚ごとの音声と尺を作る",
            args: vec![
                Arg::need("directory", "原稿の場所"),
                Arg::opt("voice", "読み上げる声", None),
            ],
            run: run_synth,
            human: human_synth,
        },
        Tool {
            name: "lexicon",
            summary: "読みの辞書を登録する",
            args: vec![
                Arg::need("path", "辞書のファイル"),
                Arg::need("name", "登録する名前"),
            ],
            run: run_lexicon,
            human: human_lexicon,
        },
        Tool {
            name: "measure",
            summary: "音声の長さを測る",
            args: vec![Arg::many("path", "音声（複数可）")],
            run: run_measure,
            human: human_measure,
        },
    ]
}
