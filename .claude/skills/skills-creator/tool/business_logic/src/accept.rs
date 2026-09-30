// SPDX-License-Identifier: MIT
//! 助言型の受け入れの検査（機械の7件）。**見つけるが、直さない**（ACDR 0061）。
//!
//! 学習ノートは `references/archive/notes/*.md` に置く ── 原典の複製を含むので git の管理の外である。
//! 判断基準（`references/criteria.json`）は、ノートの言葉だけで書く。ノートの引用（`>` で始まる行）は
//! 原文のままの文なので、判断基準へ取り込まない。

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::data_access::files;
use crate::data_access::process;
use crate::refs;

/// 検査1件の結果。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Check {
    /// 検査の番号（1〜7）。
    pub no: u8,
    /// 何を見るか。
    pub what: &'static str,
    /// 検出。**空なら合格である。**
    pub findings: Vec<String>,
}

/// 判断基準の文字列のうち、ノートと照らさない欄。**識別子 ・ 種類 ・ 図の経路 ・ コード ・ 出典である。**
const NOT_PROSE: [&str; 5] = ["id", "kind", "svg", "code", "source"];

/// 取り込んだ引用とみなす最短の長さ（文字）。**短い語句は、ノートの言葉と重なって当然である。**
const QUOTE_MIN: usize = 12;

/// 空白を1つにそろえ、Markdown の記法（太字の `**` ・ コードの `` ` `` ・ リンクの記法）を外す。
/// **ノートは Markdown で書くので、記法が文字列の間に入る** ── 外さないと、同じ言葉が照合から外れる。
fn squash(s: &str) -> String {
    let plain = s.replace("**", "").replace('`', "");
    let mut out = String::new();
    let mut rest = plain.as_str();
    // [語](経路) は、語だけを残す
    while let Some(open) = rest.find('[') {
        let Some(mid) = rest[open..].find("](") else {
            break;
        };
        let Some(close) = rest[open + mid..].find(')') else {
            break;
        };
        out.push_str(&rest[..open]);
        out.push_str(&rest[open + 1..open + mid]);
        rest = &rest[open + mid + close + 1..];
    }
    out.push_str(rest);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 学習ノートを読む。**ファイルの名前の順に並べる。**
fn notes(root: &Path) -> Vec<(PathBuf, String)> {
    let dir = root.join("references/archive/notes");
    files::list(&dir)
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .filter_map(|p| files::read_to_string(&p).ok().map(|b| (p, b)))
        .collect()
}

/// 判断基準の中の文字列を、欄の経路と一緒に集める。**照らさない欄の下は入らない。**
fn prose(v: &Value, at: &str, out: &mut Vec<(String, String)>) {
    match v {
        Value::String(s) => out.push((at.to_owned(), s.clone())),
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                prose(x, &format!("{at}/{i}"), out);
            }
        }
        Value::Object(m) => {
            for (k, x) in m {
                if !NOT_PROSE.contains(&k.as_str()) && k != "$schema" {
                    prose(x, &format!("{at}/{k}"), out);
                }
            }
        }
        _ => {}
    }
}

/// 助言型の受け入れの検査を、7件すべて行う。
#[must_use]
pub fn accept(root: &Path, others: &[String], test: &[String]) -> Vec<Check> {
    let references = root.join("references");
    let criteria: Value = files::read_to_string(references.join("criteria.json"))
        .ok()
        .and_then(|b| serde_json::from_str(&b).ok())
        .unwrap_or(Value::Null);
    let items: Vec<Value> = criteria["items"].as_array().cloned().unwrap_or_default();
    let notes = notes(root);
    let body = squash(
        &notes
            .iter()
            .map(|(_, b)| b.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    );
    vec![
        shape(root),
        not_in_notes(&items, &notes, &body),
        quotes_brought(&items, &notes),
        concept_units(&items, &notes),
        figures(root, &items),
        other_skills(root, others),
        tests(root, test),
    ]
}

/// 1 形 ── 判断基準と回答の例が、スキーマの検査に合格する。
fn shape(root: &Path) -> Check {
    let references = root.join("references");
    let mut findings = refs::validate(&references).unwrap_or_else(|e| vec![e]);
    let fixtures = root.join("tool/business_logic/tests/fixtures");
    for f in files::list(&fixtures).unwrap_or_default() {
        let name = f
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name.starts_with("answer") && name.ends_with(".json") {
            findings
                .extend(refs::validate_file(&references, "answer", &f).unwrap_or_else(|e| vec![e]));
        }
    }
    Check {
        no: 1,
        what: "形 ── 判断基準と回答の例がスキーマに合う",
        findings,
    }
}

/// 2 ノートに無い文字列 ── 判断基準のすべての文字列が、学習ノートの本文の一部である。
fn not_in_notes(items: &[Value], notes: &[(PathBuf, String)], body: &str) -> Check {
    let mut findings = Vec::new();
    if notes.is_empty() {
        findings.push(
            "学習ノートが無い: references/archive/notes/*.md ── 判断基準はノートから作る"
                .to_owned(),
        );
    } else {
        for item in items {
            let id = item["id"].as_str().unwrap_or("?");
            let mut strings = Vec::new();
            prose(item, "", &mut strings);
            for (at, s) in strings {
                if !body.contains(&squash(&s)) {
                    let head: String = s.chars().take(30).collect();
                    findings.push(format!("{id}{at}: ノートに無い ──「{head}…」"));
                }
            }
        }
    }
    Check {
        no: 2,
        what: "ノートに無い文字列 ── 判断基準の文字列がすべて学習ノートに在る",
        findings,
    }
}

/// 3 取り込んだ引用 ── 学習ノートの引用（原文のままの文）が、判断基準に無い。
fn quotes_brought(items: &[Value], notes: &[(PathBuf, String)]) -> Check {
    let quotes: Vec<String> = notes
        .iter()
        .flat_map(|(_, b)| b.lines())
        .filter_map(|l| l.trim_start().strip_prefix('>'))
        .map(squash)
        .filter(|q| q.chars().count() >= QUOTE_MIN)
        .collect();
    let mut findings = Vec::new();
    for item in items {
        let id = item["id"].as_str().unwrap_or("?");
        let mut strings = Vec::new();
        prose(item, "", &mut strings);
        for (at, s) in strings {
            let s = squash(&s);
            if let Some(q) = quotes.iter().find(|q| s.contains(q.as_str())) {
                let head: String = q.chars().take(30).collect();
                findings.push(format!(
                    "{id}{at}: ノートの引用を取り込んでいる ──「{head}…」"
                ));
            }
        }
    }
    Check {
        no: 3,
        what: "取り込んだ引用 ── ノートの引用（原文のままの文）が判断基準に無い",
        findings,
    }
}

/// 4 概念の単位 ── 各判断基準の題が、学習ノートの見出しに原典の概念の名前として在る。
fn concept_units(items: &[Value], notes: &[(PathBuf, String)]) -> Check {
    let headings: Vec<String> = notes
        .iter()
        .flat_map(|(_, b)| b.lines())
        .filter(|l| l.starts_with('#'))
        .map(|l| squash(l.trim_start_matches('#')))
        .collect();
    let mut findings = Vec::new();
    for item in items {
        let id = item["id"].as_str().unwrap_or("?");
        let title = squash(item["title"].as_str().unwrap_or_default());
        if !headings.iter().any(|h| h.contains(&title)) {
            findings.push(format!(
                "{id}: 題「{title}」が学習ノートの見出しに無い ── 判断基準は原典が名前を付けた概念を単位にする"
            ));
        }
    }
    Check {
        no: 4,
        what: "概念の単位 ── 判断基準の題が学習ノートの見出しに在る",
        findings,
    }
}

/// 5 図の同梱 ── 判断基準が指す SVG が Skill の中に在り、SVG として読める。
fn figures(root: &Path, items: &[Value]) -> Check {
    fn walk(v: &Value, out: &mut Vec<String>) {
        match v {
            Value::Object(m) => {
                if let Some(Value::String(p)) = m.get("svg") {
                    out.push(p.clone());
                }
                m.values().for_each(|x| walk(x, out));
            }
            Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    let mut findings = Vec::new();
    for item in items {
        let id = item["id"].as_str().unwrap_or("?");
        let mut paths = Vec::new();
        walk(item, &mut paths);
        for p in paths {
            let at = root.join("references").join(&p);
            match files::read_to_string(&at) {
                Ok(b) if b.contains("<svg") => {}
                Ok(_) => findings.push(format!("{id}: {p} が SVG として読めない")),
                Err(_) => findings.push(format!("{id}: {p} が Skill の中に無い")),
            }
        }
    }
    Check {
        no: 5,
        what: "図の同梱 ── 判断基準が指す SVG が Skill の中に在る",
        findings,
    }
}

/// 6 他の Skill の名前 ── SKILL.md ・ スキーマ ・ 道具に、同じ置き場所の他の Skill の名前が無い。
fn other_skills(root: &Path, others: &[String]) -> Check {
    let mut targets = vec![root.join("SKILL.md")];
    for p in files::list(root.join("references")).unwrap_or_default() {
        if p.to_string_lossy().ends_with(".schema.json") {
            targets.push(p);
        }
    }
    fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
        for p in files::list(dir).unwrap_or_default() {
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if files::is_dir(&p) {
                if name != "target" {
                    sources(&p, out);
                }
            } else if name.ends_with(".rs") {
                out.push(p);
            }
        }
    }
    sources(&root.join("tool"), &mut targets);
    let mut findings = Vec::new();
    for t in targets {
        let Ok(body) = files::read_to_string(&t) else {
            continue;
        };
        let at = t.strip_prefix(root).unwrap_or(&t).display().to_string();
        for name in others {
            if body.contains(name.as_str()) {
                findings.push(format!(
                    "{at}: 他の Skill の名前「{name}」が在る ── 助言型は単体で動く"
                ));
            }
        }
    }
    Check {
        no: 6,
        what: "他の Skill の名前 ── SKILL.md ・ スキーマ ・ 道具に他の Skill の名前が無い",
        findings,
    }
}

/// 7 試験 ── 雛形の試験が通る。**試験のコマンドは言語の組の定義が持つ。**
fn tests(root: &Path, test: &[String]) -> Check {
    let findings = match test.split_first() {
        None => vec!["試験のコマンドが無い ── 言語の組の定義に test を書く".to_owned()],
        Some((cmd, args)) => {
            match process::run(cmd, args, root, std::time::Duration::from_secs(600)) {
                Ok(ran) if ran.code == 0 => Vec::new(),
                Ok(ran) => {
                    let tail: Vec<&str> = ran.stdout.lines().rev().take(5).collect();
                    vec![format!(
                        "試験が通らない（終了コード {}）── {}",
                        ran.code,
                        tail.into_iter().rev().collect::<Vec<_>>().join(" ／ ")
                    )]
                }
                Err(e) => vec![format!("試験を起動できない: {cmd} ── {e:?}")],
            }
        }
    };
    Check {
        no: 7,
        what: "試験 ── 雛形の試験が通る",
        findings,
    }
}
