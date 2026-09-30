// SPDX-License-Identifier: MIT
//! 助言型の受け入れの検査（機械の7件）。**見つけるが、直さない**（ACDR 0061）。
//!
//! 学習ノートは `references/archive/notes/*.md` に置く ── 原典の複製を含むので git の管理の外である。
//! 判断基準（`references/criteria.json`）は、語彙を原典の語のまま使い、説明をノートを読んでまとめた言葉で書く
//! （ACDR 0067）。ノートの文を複製せず、ノートの引用（`>` で始まる行）も判断基準へ取り込まない。

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::data_access::files;
use crate::data_access::process;
use crate::profile::Profile;
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

/// 助言型の受け入れの検査を、7件すべて行う。**言語に依存する置き場所と試験は、言語の組の定義が持つ。**
#[must_use]
pub fn accept(root: &Path, others: &[String], lang: &Profile) -> Vec<Check> {
    let references = root.join("references");
    let criteria: Value = files::read_to_string(references.join("criteria.json"))
        .ok()
        .and_then(|b| serde_json::from_str(&b).ok())
        .unwrap_or(Value::Null);
    let items: Vec<Value> = criteria["items"].as_array().cloned().unwrap_or_default();
    let notes = notes(root);
    vec![
        shape(root, &lang.fixtures),
        copied(&items, &notes),
        quotes_brought(&items, &notes),
        concept_units(&items, &notes),
        figures(root, &items),
        other_skills(root, others, lang),
        tests(root, &lang.test),
    ]
}

/// 1 形 ── 判断基準と回答の例が、スキーマの検査に合格する。
fn shape(root: &Path, fixtures: &str) -> Check {
    let references = root.join("references");
    let mut findings = refs::validate(&references).unwrap_or_else(|e| vec![e]);
    let fixtures = root.join(fixtures);
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

/// 複製とみなす最短の長さ（文字）。**これより短い一致は、原典の語彙が続いただけのことが多い。**
const COPY_MIN: usize = 25;

/// 語彙として照らすカタカナ語の最短の長さ（文字）。
const KATAKANA_MIN: usize = 3;

/// 照らすための平文。**空白と Markdown の記号（`*` ・ `` ` `` ・ `>` ・ `#` ・ `|`）を外す** ──
/// ノートは Markdown で書くので、記号や改行が文の間に入る。
fn plain(s: &str) -> Vec<char> {
    squash(s)
        .chars()
        .filter(|c| !c.is_whitespace() && !"*`>#|".contains(*c))
        .collect()
}

/// カタカナ語を取り出す。**長音と連結の `-` は語の内側に数える**（アクター-目的リスト）。
fn katakana(s: &str) -> Vec<String> {
    let kana = |c: char| ('ァ'..='ヶ').contains(&c) || c == 'ー';
    let mut words = Vec::new();
    let mut word = String::new();
    for c in s.chars().chain(std::iter::once(' ')) {
        if kana(c) || (c == '-' && !word.is_empty()) {
            word.push(c);
        } else {
            let w = word.trim_end_matches('-').to_owned();
            if w.chars().count() >= KATAKANA_MIN {
                words.push(w);
            }
            word.clear();
        }
    }
    words
}

/// 出典の節に頁が書かれているか。**頁を書けば、学習ノートとスキャンに戻れる。**
fn pages_missing(item: &Value, id: &str, findings: &mut Vec<String>) {
    let mut check = |at: String, v: &Value| {
        let section = v["source"]["section"].as_str().unwrap_or_default();
        if !section.contains('頁') {
            findings.push(format!(
                "{id}{at}: 出典に頁が無い ──「{section}」── 章 ・ 節と頁を書く"
            ));
        }
    };
    for (i, u) in item["elements"]["units"].as_array().into_iter().flatten().enumerate() {
        check(format!("/elements/units/{i}"), u);
    }
    for (i, a) in item["antipatterns"]["items"].as_array().into_iter().flatten().enumerate() {
        check(format!("/antipatterns/items/{i}"), a);
    }
}

/// 2 複製と語彙と出典 ── 判断基準は学習ノートを読んでまとめた言葉で書く（ACDR 0067）。
///
/// **見るのは3つである。** ノートと長く一致する文字列が無いこと（複製していない）。カタカナ語が
/// ノートに在ること（原典の語彙を言い換えていない）。要素とアンチパターンの出典に頁が在ること。
#[must_use]
pub fn copied(items: &[Value], notes: &[(PathBuf, String)]) -> Check {
    let mut findings = Vec::new();
    if notes.is_empty() {
        findings.push(
            "学習ノートが無い: references/archive/notes/*.md ── 判断基準はノートから作る"
                .to_owned(),
        );
    } else {
        let body: Vec<char> = notes.iter().flat_map(|(_, b)| plain(b)).collect();
        let windows: std::collections::HashSet<String> = body
            .windows(COPY_MIN)
            .map(|w| w.iter().collect())
            .collect();
        let vocabulary: std::collections::HashSet<String> =
            notes.iter().flat_map(|(_, b)| katakana(b)).collect();
        for item in items {
            let id = item["id"].as_str().unwrap_or("?");
            let mut strings = Vec::new();
            prose(item, "", &mut strings);
            for (at, s) in strings {
                let p = plain(&s);
                if let Some(w) = p
                    .windows(COPY_MIN)
                    .map(|w| w.iter().collect::<String>())
                    .find(|w| windows.contains(w))
                {
                    findings.push(format!(
                        "{id}{at}: ノートの文を複製している ──「{w}」── ノートを読んでまとめた言葉で書く"
                    ));
                }
                for w in katakana(&s) {
                    if !vocabulary.contains(&w) {
                        findings.push(format!(
                            "{id}{at}: 原典に無い語 ──「{w}」── 語彙は原典の語のまま使う"
                        ));
                    }
                }
            }
            pages_missing(item, id, &mut findings);
        }
    }
    Check {
        no: 2,
        what: "複製と語彙と出典 ── ノートの文を複製せず、原典に無い語を使わず、要素ごとに頁つきの出典を持つ",
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
///
/// **照らすのは、節の見出しと、図 ・ 表の題である** ── 原典は概念を図や表の題で名付けることもある
/// （図1.1 要求のハブ-スポークモデル）。
#[must_use]
pub fn concept_units(items: &[Value], notes: &[(PathBuf, String)]) -> Check {
    let caption = |l: &str| {
        let l = l.trim_start_matches(['-', '*', ' ']);
        let mut c = l.chars();
        matches!(c.next(), Some('図' | '表')) && c.next().is_some_and(|d| d.is_ascii_digit())
    };
    let headings: Vec<String> = notes
        .iter()
        .flat_map(|(_, b)| b.lines())
        .filter(|l| l.starts_with('#') || caption(l))
        .map(|l| squash(l.trim_start_matches('#')))
        .collect();
    let mut findings = Vec::new();
    for item in items {
        let id = item["id"].as_str().unwrap_or("?");
        let title = squash(item["title"].as_str().unwrap_or_default());
        if !headings.iter().any(|h| h.contains(&title)) {
            findings.push(format!(
                "{id}: 題「{title}」が学習ノートの見出しにも図 ・ 表の題にも無い ── 判断基準は原典が名前を付けた概念を単位にする"
            ));
        }
    }
    Check {
        no: 4,
        what: "概念の単位 ── 判断基準の題が学習ノートの見出しか図 ・ 表の題に在る",
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
fn other_skills(root: &Path, others: &[String], lang: &Profile) -> Check {
    let mut targets = vec![root.join("SKILL.md")];
    for p in files::list(root.join("references")).unwrap_or_default() {
        if p.to_string_lossy().ends_with(".schema.json") {
            targets.push(p);
        }
    }
    // **ソースの拡張子と、入らないフォルダは言語の組が決める**（試験は数える ── 試験も道具の一部である）
    fn sources(dir: &Path, lang: &Profile, out: &mut Vec<PathBuf>) {
        for p in files::list(dir).unwrap_or_default() {
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if files::is_dir(&p) {
                let generated = [
                    "target",
                    "node_modules",
                    ".venv",
                    "bin",
                    "obj",
                    "__pycache__",
                ];
                if !generated.contains(&name.as_str()) {
                    sources(&p, lang, out);
                }
            } else if lang.extensions.iter().any(|e| name.ends_with(e.as_str())) {
                out.push(p);
            }
        }
    }
    sources(&root.join("tool"), lang, &mut targets);
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
