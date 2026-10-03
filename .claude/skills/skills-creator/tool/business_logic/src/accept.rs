// SPDX-License-Identifier: MIT
//! 助言型の受け入れの検査（機械の7件）。**見つけるが、直さない**（ACDR 0061）。
//!
//! 学習ノートは `references/archive/notes/*.md` に置く ── 原典の複製を含むので git の管理の外である。
//! 判断基準（`references/criteria.json`）は、語彙を原典の語のまま使い、説明をノートを読んでまとめた言葉で書く
//! （ACDR 0067）。ノートの文を複製せず、ノートの引用（`>` で始まる行）も判断基準へ取り込まない。

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::behavior;
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

/// 組み立ての出力と、処理系が作るフォルダ。**道具のソースではない** ── 中を探さない。
const GENERATED: [&str; 6] = [
    "target",
    "node_modules",
    ".venv",
    "bin",
    "obj",
    "__pycache__",
];

/// `tool/` の下のファイルを集める。**組み立ての出力は見ない。** どの言語で書いても同じ規則で集める
/// （ACDR 0097）── 拡張子を言語ごとに持つと、言語を足すたびに定義が要る。
fn tool_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for p in files::list(dir).unwrap_or_default() {
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if files::is_dir(&p) {
            if !GENERATED.contains(&name.as_str()) {
                tool_files(&p, out);
            }
        } else {
            out.push(p);
        }
    }
}

/// 助言型の受け入れの検査を、7件すべて行う。**言語に依存しない** ── 回答の例は `tool/` の下の
/// `answer*.json` を探し、試験のコマンドは `tool.json` の `test` から読む（ACDR 0097）。
#[must_use]
pub fn accept(root: &Path, others: &[String]) -> Vec<Check> {
    let references = root.join("references");
    let criteria: Value = files::read_to_string(references.join("criteria.json"))
        .ok()
        .and_then(|b| serde_json::from_str(&b).ok())
        .unwrap_or(Value::Null);
    let items: Vec<Value> = criteria["items"].as_array().cloned().unwrap_or_default();
    let notes = notes(root);
    vec![
        shape(root),
        copied(&items, &notes),
        quotes_brought(&items, &notes),
        concept_units(&items, &notes),
        figures(root, &items),
        other_skills(root, others),
        tests(root),
    ]
}

/// 1 形 ── 判断基準と回答の例が、スキーマの検査に合格する。
fn shape(root: &Path) -> Check {
    let references = root.join("references");
    let mut findings = refs::validate(&references).unwrap_or_else(|e| vec![e]);
    let mut found = Vec::new();
    tool_files(&root.join("tool"), &mut found);
    found.sort();
    for f in found {
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

/// 写しとみなす並びに入る、日本語の文字の最少の数。**これより少ない並びは、英語の名前かコードである。**
const PROSE_MIN: usize = 10;

/// 原典の名前を書く欄。**ノートと一致するのが正しい状態である** ── 題 ・ 関連する基準の題 ・ 定義の語。
const NAME_KEYS: [&str; 2] = ["title", "term"];

/// 「」で囲んだ部分を、照らさない区切りに置き換える。**引用は複製ではない** ── 原典の名前を示すときに
/// 使う（入れ子の「「欠陥ゼロ」の落とし穴」も1つの引用として外す）。区切りを残すので、引用の前後の
/// 地の文が1続きの文字列にならない。
fn unquoted(p: &[char]) -> Vec<char> {
    let mut depth = 0usize;
    let mut out = Vec::with_capacity(p.len());
    for &c in p {
        match c {
            '「' => depth += 1,
            '」' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    out.push('\u{0}');
                }
            }
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

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

/// 原典の中を指す番号（表 ・ 図 ・ 指針 ・ 練習 ・ メモ ・ 付録）と「本書」を取り出す。
///
/// **判断基準の頁に原典は無い** ── 読み手は「表20.1」も「指針14」も辿れない。どこから取ったかは
/// 出典の欄が持つので、本文には書かない。
fn numbers_of_original(s: &str) -> Vec<String> {
    const MARKS: [&str; 6] = ["表", "図", "指針", "練習", "メモ", "付録"];
    let mut found = Vec::new();
    for m in MARKS {
        for (i, _) in s.match_indices(m) {
            let rest = &s[i + m.len()..];
            let next = rest.chars().next();
            if next.is_some_and(|c| c.is_ascii_digit() || (m == "付録" && c.is_ascii_uppercase()))
            {
                let n: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '.')
                    .collect();
                found.push(format!("{m}{n}"));
            }
        }
    }
    // 「1本書き」「数本書く」の「本書」は原典を指さない ── 後ろが動詞の活用なら数えない
    for (i, _) in s.match_indices("本書") {
        let next = s[i + "本書".len()..].chars().next();
        if !next.is_some_and(|c| "かきくけこいっ".contains(c)) {
            found.push("本書".to_owned());
            break;
        }
    }
    found
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
    for (i, u) in item["elements"]["units"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        check(format!("/elements/units/{i}"), u);
    }
    for (i, a) in item["antipatterns"]["items"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
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
        let windows: std::collections::HashSet<String> =
            body.windows(COPY_MIN).map(|w| w.iter().collect()).collect();
        let vocabulary: std::collections::HashSet<String> =
            notes.iter().flat_map(|(_, b)| katakana(b)).collect();
        for item in items {
            let id = item["id"].as_str().unwrap_or("?");
            let mut strings = Vec::new();
            prose(item, "", &mut strings);
            for (at, s) in strings {
                let p = unquoted(&plain(&s));
                let named = NAME_KEYS.iter().any(|k| at.ends_with(&format!("/{k}")));
                if let Some(w) = p
                    .windows(COPY_MIN)
                    .filter(|_| !named)
                    // **日本語の文字が少ない並びは数えない** ── 原典の英語の名前とコードは、そのまま書くのが正しい。
                    // 空白を外すと25字を超える名前（Database Partitioning Scheme）が在り、助詞や括弧が付いても
                    // 名前のままである。日本語の文を写したものは、25字の中に日本語が十分に入るので検出できる
                    .filter(|w| w.iter().filter(|c| !c.is_ascii()).count() >= PROSE_MIN)
                    .map(|w| w.iter().collect::<String>())
                    .find(|w| windows.contains(w))
                {
                    findings.push(format!(
                        "{id}{at}: ノートの文を複製している ──「{w}」── ノートを読んでまとめた言葉で書く"
                    ));
                }
                for n in numbers_of_original(&s) {
                    findings.push(format!(
                        "{id}{at}: 原典の番号 ──「{n}」── 判断基準の頁に原典は無い。番号は出典の欄に書く"
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
        what: "複製と語彙と出典 ── ノートの文を複製せず、原典に無い語と原典の番号を本文に書かず、要素ごとに頁つきの出典を持つ",
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
fn other_skills(root: &Path, others: &[String]) -> Check {
    let mut targets = vec![root.join("SKILL.md")];
    for p in files::list(root.join("references")).unwrap_or_default() {
        if p.to_string_lossy().ends_with(".schema.json") {
            targets.push(p);
        }
    }
    // **道具のファイルはすべて見る**（試験も道具の一部である）。文字として読めないものは飛ばす
    tool_files(&root.join("tool"), &mut targets);
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

/// 7 試験 ── 道具の試験が通る。**試験のコマンドは `tool.json` の `test` が持つ**（契約の版3）。
fn tests(root: &Path) -> Check {
    let findings = match behavior::commands(root, "test") {
        Err(why) => vec![why],
        Ok(list) if list.is_empty() => {
            vec!["試験のコマンドが無い ── tool.json の test に書く".to_owned()]
        }
        Ok(list) => list
            .iter()
            .filter_map(|command| {
                let (cmd, args) = command.split_first()?;
                match process::run(cmd, args, root, std::time::Duration::from_secs(600)) {
                    Ok(ran) if ran.code == 0 => None,
                    Ok(ran) => {
                        let tail: Vec<&str> = ran.stdout.lines().rev().take(5).collect();
                        Some(format!(
                            "試験が通らない（終了コード {}）── {}",
                            ran.code,
                            tail.into_iter().rev().collect::<Vec<_>>().join(" ／ ")
                        ))
                    }
                    Err(e) => Some(format!("試験を起動できない: {cmd} ── {e:?}")),
                }
            })
            .collect(),
    };
    Check {
        no: 7,
        what: "試験 ── 道具の試験が通る",
        findings,
    }
}
