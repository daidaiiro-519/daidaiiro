// SPDX-License-Identifier: MIT
//! doc-writing-skills の道具の宣言。**能力の正本はここである。**
//!
//! 入口（CLI ・ MCP）はこの宣言から組む ── 能力を2回書くと、片方だけが古くなる。
//! 許可辺は `Cargo.toml` が宣言する ── この crate は部品だけを参照する。

pub mod contract;

use std::path::{Path, PathBuf};

use dws_parts::checks::Words;
use dws_parts::{gate, tails};
use serde_json::json;

pub use contract::{Arg, Given, Outcome, Tool};

/// この Skill の場所。**実行ファイルの位置から辿らない** ── build の置き場所に依存する。
fn skill_root(given: &Given) -> PathBuf {
    PathBuf::from(given.one("skill_root", "."))
}

/// 語の一覧を組む。**この Skill が持つのは和語だけ** ── 同義語と廃語は
/// プロジェクトごとに相違するので、外から受け取る。
fn words(given: &Given, target: &Path) -> Words {
    let predicates = gate::load_predicates(&skill_root(given).join("references/predicates.json"))
        .unwrap_or_default();
    let retired = gate::find_retired(target)
        .and_then(|p| gate::load_retired(&p).ok())
        .unwrap_or_default();
    let synonyms = given
        .all("synonyms")
        .first()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|body| {
            body.lines()
                .filter_map(|line| line.split_once('\t'))
                .map(|(a, b)| (a.trim().to_owned(), b.trim().to_owned()))
                .collect()
        })
        .unwrap_or_default();
    Words::new(predicates, synonyms, retired)
}

fn run_check(given: &Given) -> Outcome {
    let paths = given.all("path");
    if paths.is_empty() {
        return Outcome::misuse("対象のファイルを渡していない".to_owned());
    }
    let first = PathBuf::from(&paths[0]);
    let words = words(given, &first);
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

fn run_tails(given: &Given) -> Outcome {
    let paths: Vec<PathBuf> = given.all("path").iter().map(PathBuf::from).collect();
    if paths.is_empty() {
        return Outcome::misuse("対象のファイルを渡していない".to_owned());
    }
    let refs: Vec<&Path> = paths.iter().map(PathBuf::as_path).collect();
    match tails::lines(&refs) {
        Ok(lines) => Outcome::found(lines, json!({ "looked": paths.len() })),
        Err(e) => Outcome::misuse(format!("読めない ── {e}")),
    }
}

fn human_tails(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    out.findings.join("\n")
}

/// この Skill が持つ道具の一覧。**能力の正本である。**
#[must_use]
pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "check",
            summary: "10の判定を当てる（ゲート1）",
            args: vec![
                Arg::many("path", "対象のファイル（複数可）"),
                Arg::opt("synonyms", "言い換えの一覧", None),
                Arg::opt(
                    "skill_root",
                    "この Skill の場所（語の一覧を読む先）",
                    Some("."),
                ),
            ],
            run: run_check,
            human: human_check,
        },
        Tool {
            name: "tails",
            summary: "述部の末尾を数える",
            args: vec![Arg::many("path", "対象のファイル（複数可）")],
            run: run_tails,
            human: human_tails,
        },
        Tool {
            name: "checks",
            summary: "当てている判定の一覧を出す",
            args: vec![],
            run: run_checks,
            human: human_checks,
        },
    ]
}
