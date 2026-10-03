// SPDX-License-Identifier: MIT
//! doc-writing-skills のサービス層。道具の一覧を持ち、**能力の正本はここである。**
//!
//! プレゼンテーション層（CLI ・ MCP）はこの一覧から組む ── 能力を2回書くと、片方だけが古くなる。
//! 依存の向きは `Cargo.toml` が宣言する ── この crate は業務ロジック層だけを参照する。

pub mod contract;
mod refs;

use std::path::{Path, PathBuf};

use dws_business_logic::checks::Words;
use dws_business_logic::{gate, input, instruction};
use serde_json::json;

pub use contract::{catalog, Arg, Given, Outcome, Tool};

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

fn skill_root(given: &Given) -> Result<PathBuf, String> {
    given.skill_root()
}

/// 語の一覧を組む。同義語と廃語はプロジェクトごとに違うので、外から受け取る。
fn words(given: &Given, target: &Path) -> Result<Words, String> {
    // 一覧が無いことは失敗にしない。**在るのに読めないことは、失敗にする** ── 空の一覧として続行すると、
    // 廃語の検査を実施しないまま0件と報告する
    let retired = match gate::find_retired(target) {
        Some(p) => gate::load_retired(&p)
            .map_err(|e| format!("廃語の一覧を読めない ── {} ── {e}", p.display()))?,
        None => Vec::new(),
    };
    let synonyms = given
        .all("synonyms")
        .first()
        .map(|p| gate::load_synonyms(Path::new(p)))
        .unwrap_or_default();
    Ok(Words::new(synonyms, retired))
}

fn run_check(given: &Given) -> Outcome {
    let paths = given.all("path");
    if paths.is_empty() {
        return Outcome::misuse("対象のファイルを渡していない".to_owned());
    }
    let first = PathBuf::from(&paths[0]);
    let words = match words(given, &first) {
        Ok(w) => w,
        Err(e) => return Outcome::misuse(e),
    };
    let mut findings = Vec::new();
    let mut looked = 0_usize;
    for raw in paths {
        let path = PathBuf::from(raw);
        match gate::inspect(&path, &words) {
            Ok(found) => {
                looked += 1;
                let name = path
                    .file_name()
                    .map_or_else(String::new, |x| x.to_string_lossy().into_owned());
                findings.extend(found.into_iter().map(|f| {
                    format!(
                        "× {name}:{} [{}] {}：{}",
                        f.line, f.basis, f.check, f.excerpt
                    )
                }));
            }
            Err(e) => return Outcome::misuse(format!("読めない ── {} ── {e}", path.display())),
        }
    }
    Outcome::found(findings, json!({ "looked": looked }))
}

fn human_check(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    let looked = out
        .data
        .get("looked")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let mut lines: Vec<String> = out.findings.clone();
    if out.findings.is_empty() {
        lines.push(format!("\nゲート1　通った　／　対象 {looked} ファイル"));
    } else {
        lines.push(format!(
            "\nゲート1　通っていない（{} 件）　／　対象 {looked} ファイル",
            out.findings.len()
        ));
        lines.push("**0件にできるものだけを検査している。直してから出す。**".to_owned());
    }
    lines.join("\n")
}

fn run_checks(_given: &Given) -> Outcome {
    let listed: Vec<serde_json::Value> = gate::all()
        .iter()
        .map(|c| json!({ "name": c.name, "basis": c.basis, "scope": c.scope.label(), "note": c.note }))
        .collect();
    Outcome::found(Vec::new(), json!({ "checks": listed }))
}

fn human_checks(out: &Outcome) -> String {
    let empty = vec![];
    let listed = out
        .data
        .get("checks")
        .and_then(|x| x.as_array())
        .unwrap_or(&empty);
    let mut lines = vec![format!(
        "{:30}{:16}{}",
        "検査", "拠って立つもの", "適用する単位"
    )];
    for c in listed {
        let get = |k: &str| {
            c.get(k)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_owned()
        };
        lines.push(format!(
            "{:30}{:16}{}",
            get("name"),
            get("basis"),
            get("scope")
        ));
        lines.push(format!("{:46}{}", "", get("note")));
    }
    lines.push(String::new());
    lines.join("\n")
}

/// 利用者への応答に当てる判定。**語彙表で決まる1つだけである** ── 見出しや表の判定は、
/// 会話の応答の形には当てない。
const REPLY_CHECKS: [&str; 1] = ["廃語を使用している"];

/// 応答の本文と、廃語の一覧を探し始める場所を取る。
///
/// `hook` を渡すと、Claude Code の Stop フックの入力（JSON）を読む ── 本文は
/// `last_assistant_message`、探し始める場所は `cwd` である。`-` なら標準入力から読む。
/// 項目名は <https://code.claude.com/docs/en/hooks> の「Stop input」の原文で照合した。
fn reply_source(given: &Given) -> Result<(String, PathBuf), String> {
    let here = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if given.has("message") {
        return Ok((given.one("message", "").to_owned(), here));
    }
    let from = given.one("hook", "-");
    let body = input::hook_body(from)?;
    let input: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("フックの入力が JSON ではない ── {e}"))?;
    let message = input
        .get("last_assistant_message")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let base = input
        .get("cwd")
        .and_then(serde_json::Value::as_str)
        .map_or(here, PathBuf::from);
    Ok((message, base))
}

fn run_reply(given: &Given) -> Outcome {
    let (message, base) = match reply_source(given) {
        Ok(x) => x,
        Err(e) => return Outcome::misuse(e),
    };
    let words = match words(given, &base) {
        Ok(w) => w,
        Err(e) => return Outcome::misuse(e),
    };
    let findings: Vec<String> = gate::inspect_text(&message, &words)
        .into_iter()
        .filter(|f| REPLY_CHECKS.contains(&f.check.as_str()))
        .map(|f| format!("{}行 {}：{}", f.line, f.check, f.excerpt))
        .collect();
    if findings.is_empty() {
        return Outcome::found(Vec::new(), json!({}));
    }
    let reason = format!(
        "直前の応答に、語彙表が検出した語が在る。表示済みの応答の該当箇所を言い換え先へ修正し、\
         修正した全文を出し直す。謝罪と経緯の説明は書かない。\n{}",
        findings.join("\n")
    );
    // **`decision: "block"` を使わない** ── 原典は、それを記録上 hook error として表示する。
    // `additionalContext` なら同じく会話を続けさせ、表示は `Stop hook feedback` になる
    // （code.claude.com/docs/en/hooks :2620・2631、2026-09-27 取得）
    Outcome::found(
        findings,
        json!({ "hookSpecificOutput": { "hookEventName": "Stop", "additionalContext": reason } }),
    )
}

/// **Stop フックが読む形で出す。** 検出があれば `hookSpecificOutput.additionalContext` の JSON、
/// 無ければ何も出さない ── Claude Code は、終了コードに関係なく JSON の中身で判定する。
fn human_reply(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    if out.findings.is_empty() {
        return String::new();
    }
    serde_json::to_string(&out.data).unwrap_or_default()
}

/// 日本語の審査の依頼文を組む。**判定はしない** ── 判定するのはモデルであり、この道具は
/// 判定基準・手順・事例を1つの依頼文へ組むだけである。
///
/// 正本は3つである ── 手順（`references/document.json` の `review-instruction`）と判定基準
/// （`references/review-criteria.json`）はこの Skill が持ち、事例
/// （`.doc-writing/review-examples.json`）はプロジェクトが持つ。
/// 審査の手順を持つ document の id。
const REVIEW_INSTRUCTION: &str = "review-instruction";

fn run_review(given: &Given) -> Outcome {
    let (message, base) = match reply_source(given) {
        Ok(x) => x,
        Err(e) => return Outcome::misuse(e),
    };
    let root = or_misuse!(skill_root(given));
    let refs_dir = root.join("references");
    let instruction =
        match dws_business_logic::refs::get(&refs_dir, "document", Some(REVIEW_INSTRUCTION)) {
            Ok(doc) => instruction::text(&doc),
            Err(e) => return Outcome::misuse(format!("審査の手順を読めない ── {e}")),
        };
    let read = |p: PathBuf| input::read_text(&p);
    let criteria: serde_json::Value =
        match read(refs_dir.join("review-criteria.json")).and_then(|b| {
            serde_json::from_str(&b).map_err(|e| format!("判定基準が JSON ではない ── {e}"))
        }) {
            Ok(x) => x,
            Err(e) => return Outcome::misuse(e),
        };
    let whole = given.one("scope", "document") == "document";
    let applied: Vec<&serde_json::Value> = criteria["items"]
        .as_array()
        .map(|xs| {
            xs.iter()
                .filter(|c| whole || !c["needs_context"].as_bool().unwrap_or(false))
                .collect()
        })
        .unwrap_or_default();
    // 事例はプロジェクトが持つ ── 廃語の一覧と同じく、上へたどって探す。無ければ無しで組む
    let examples = input::find_examples(&base);
    let prompt = format!(
        "{instruction}\n## 判定基準（適用するもの {} 件）\n\n{}\n\n## 事例\n\n{}\n\n## 本文\n\n{message}\n",
        applied.len(),
        serde_json::to_string_pretty(&applied).unwrap_or_default(),
        if examples.is_empty() { "（無い）" } else { examples.trim() },
    );
    Outcome::found(
        Vec::new(),
        json!({ "prompt": prompt, "criteria": applied.len() }),
    )
}

fn human_review(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    out.data["prompt"].as_str().unwrap_or_default().to_owned()
}

/// この Skill が持つ道具の一覧。**能力の正本である。**
#[must_use]
pub fn tools() -> Vec<Tool> {
    let mut all = vec![
        Tool {
            name: "check",
            summary: "10の判定を当てる（ゲート1）",
            args: vec![
                Arg::many("path", "対象のファイル（複数可）"),
                Arg::opt("synonyms", "言い換えの一覧", None),
                Arg::opt("skill_root", "この Skill の場所（語の一覧を読む先）", None),
            ],
            run: run_check,
            human: human_check,
        },
        Tool {
            name: "reply",
            summary: "利用者への応答に語彙表の照合を当てる（Stop フックの入出力）",
            args: vec![
                Arg::opt("message", "応答の本文。渡さなければ hook を読む", None),
                Arg::opt(
                    "hook",
                    "Stop フックの入力（JSON）のファイル。- なら標準入力",
                    Some("-"),
                ),
                Arg::opt("skill_root", "この Skill の場所（語の一覧を読む先）", None),
            ],
            run: run_reply,
            human: human_reply,
        },
        Tool {
            name: "review",
            summary: "日本語の審査の依頼文を組む（判定はモデルが実施する）",
            args: vec![
                Arg::opt("message", "審査する本文。渡さなければ hook を読む", None),
                Arg::opt(
                    "hook",
                    "Stop フックの入力（JSON）のファイル。- なら標準入力",
                    Some("-"),
                ),
                Arg::opt(
                    "scope",
                    "document（文書。全部の基準を適用する）か reply（文脈を要する基準を外す）",
                    Some("document"),
                ),
                Arg::opt("skill_root", "この Skill の場所（判定基準を読む先）", None),
            ],
            run: run_review,
            human: human_review,
        },
        Tool {
            name: "checks",
            summary: "当てている判定の一覧を出す",
            args: vec![],
            run: run_checks,
            human: human_checks,
        },
    ];
    // references の4つの道具（get ・ validate ・ view ・ import）。実体は雛形の複製 `refs` が持つ
    all.extend(refs::tools());
    all
}
