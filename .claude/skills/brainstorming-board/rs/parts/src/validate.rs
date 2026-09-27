// SPDX-License-Identifier: MIT
//! 入力（board.json）を検査する。**生成物ではなく、入力を検査する。**
//!
//! 生成物を検査する形では、記録が正しく生成されたことしか判明しない。しかも不合格の時点で
//! HTML は既に出ている ── それはゲートではなく注記である。
//! **不合格なら HTML を1バイトも出さない。**
//!
//! 検査は4系統である ── 形（スキーマ）・ 散文 ・ 見出しや表の混入 ・ 参照の解決。

use std::path::{Path, PathBuf};

use serde_json::Value;

/// 主張の区切り。
const SEP: &str = "──";
/// 1文の字数の上限。
const LONGEST: usize = 120;
/// 1つの行に置ける主張の数。
const COHABIT: usize = 1;

/// 形の契約の場所。
#[must_use]
pub fn schema_path(references: &Path) -> PathBuf {
    references.join("board.schema.json")
}

fn text_of(value: &Value, key: &str) -> String {
    match value.get(key) {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

fn array_of<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(|x| x.as_array())
        .map_or(&[], |x| x.as_slice())
}

fn value_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// 宣言の並びの中の欄を、場所の名前つきで列挙する。
fn declare(blocks: &[Value], place: &str, out: &mut Vec<(String, String)>) {
    for (i, b) in blocks.iter().enumerate() {
        let kind = text_of(b, "kind");
        let p = format!("{place}[{i}]{kind}");
        if matches!(kind.as_str(), "para" | "note" | "heading" | "card") {
            let body = if text_of(b, "text").is_empty() {
                text_of(b, "heading")
            } else {
                text_of(b, "text")
            };
            out.push((p.clone(), body));
        }
        match kind.as_str() {
            "card" => {
                for (j, e) in array_of(b, "events").iter().enumerate() {
                    out.push((
                        format!("{p}/events[{j}]{}", text_of(e, "tag")),
                        text_of(e, "text"),
                    ));
                }
            }
            "list" => {
                for (j, x) in array_of(b, "items").iter().enumerate() {
                    out.push((format!("{p}/items[{j}]"), text_of(x, "text")));
                    declare(array_of(x, "nested"), &format!("{p}/items[{j}]"), out);
                }
            }
            "table" => {
                for row in array_of(b, "rows") {
                    let pair = row.as_array().map_or(&[][..], |x| x.as_slice());
                    let head = pair.first().map_or_else(String::new, value_text);
                    for v in pair
                        .get(1)
                        .and_then(|x| x.as_array())
                        .map_or(&[][..], |x| x.as_slice())
                    {
                        out.push((format!("{p}/{head}"), value_text(v)));
                    }
                }
            }
            "grid" => {
                for row in array_of(b, "rows") {
                    for v in row.as_array().map_or(&[][..], |x| x.as_slice()) {
                        out.push((p.clone(), value_text(v)));
                    }
                }
            }
            "previous" => {
                out.push((format!("{p}/body"), text_of(b, "body")));
                out.push((format!("{p}/why"), text_of(b, "why")));
            }
            "fold" => declare(
                array_of(b, "body"),
                &format!("{p}/{}", text_of(b, "heading")),
                out,
            ),
            _ => {}
        }
        if matches!(kind.as_str(), "para" | "note") {
            declare(array_of(b, "nested"), &p, out);
        }
    }
}

/// 検査する欄を、場所の名前つきで全部列挙する。**取りこぼしを作らない。**
#[must_use]
pub fn cells(d: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    declare(array_of(d, "intro"), "intro", &mut out);
    for e in array_of(d, "panels") {
        declare(
            array_of(e, "body"),
            &format!("panels/{}", text_of(e, "heading")),
            &mut out,
        );
    }
    for q in array_of(d, "queue") {
        out.push((format!("queue/{}", text_of(q, "no")), text_of(q, "why")));
    }
    for t in array_of(d, "topics") {
        let n = format!("topics[{}]", text_of(t, "no"));
        out.push((format!("{n}/answer"), text_of(t, "answer")));
        if !text_of(t, "intro").is_empty() {
            out.push((format!("{n}/intro"), text_of(t, "intro")));
        }
        if let Some(decision) = t.get("decision").filter(|x| !x.is_null()) {
            declare(
                array_of(decision, "text"),
                &format!("{n}/decision"),
                &mut out,
            );
        }
        declare(array_of(t, "example"), &format!("{n}/example"), &mut out);
        declare(array_of(t, "path"), &format!("{n}/path"), &mut out);
        for (i, o) in array_of(t, "passed").iter().enumerate() {
            out.push((format!("{n}/passed[{i}]/name"), text_of(o, "name")));
            out.push((format!("{n}/passed[{i}]/body"), text_of(o, "body")));
            out.push((format!("{n}/passed[{i}]/cost"), text_of(o, "cost")));
        }
        for (i, o) in array_of(t, "dropped").iter().enumerate() {
            out.push((format!("{n}/dropped[{i}]"), text_of(o, "body")));
            out.push((format!("{n}/dropped[{i}]/reason"), text_of(o, "reason")));
        }
        for (i, g) in array_of(t, "grounds").iter().enumerate() {
            out.push((format!("{n}/grounds[{i}]"), text_of(g, "basis")));
        }
        for (i, x) in array_of(t, "requirements").iter().enumerate() {
            out.push((format!("{n}/requirements[{i}]"), value_text(x)));
        }
        for (i, x) in array_of(t, "findings").iter().enumerate() {
            out.push((format!("{n}/findings[{i}]"), value_text(x)));
        }
        for (i, w) in array_of(t, "out_of_scope").iter().enumerate() {
            out.push((format!("{n}/out_of_scope[{i}]"), text_of(w, "item")));
            if !text_of(w, "note").is_empty() {
                out.push((format!("{n}/out_of_scope[{i}]/note"), text_of(w, "note")));
            }
        }
        for (i, tb) in array_of(t, "tables").iter().enumerate() {
            if !text_of(tb, "lead").is_empty() {
                out.push((format!("{n}/tables[{i}]/lead"), text_of(tb, "lead")));
            }
            for row in array_of(tb, "rows") {
                let pair = row.as_array().map_or(&[][..], |x| x.as_slice());
                let head = pair.first().map_or_else(String::new, value_text);
                for v in pair
                    .get(1)
                    .and_then(|x| x.as_array())
                    .map_or(&[][..], |x| x.as_slice())
                {
                    out.push((format!("{n}/tables[{i}]/{head}"), value_text(v)));
                }
            }
        }
        for e in array_of(t, "panels") {
            declare(
                array_of(e, "body"),
                &format!("{n}/panels/{}", text_of(e, "heading")),
                &mut out,
            );
        }
    }
    out
}

/// 印と引用を落とした、素の文。
fn plain(body: &str) -> String {
    // 印を落とす
    let mut out = String::new();
    let mut inside = false;
    for c in body.chars() {
        match c {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(c),
            _ => {}
        }
    }
    // **引用は除外する** ── 原文の形を変えないと決めているためである
    let mut kept = String::new();
    let mut quoted = false;
    for c in out.chars() {
        match c {
            '「' => quoted = true,
            '」' => quoted = false,
            _ if !quoted => kept.push(c),
            _ => {}
        }
    }
    kept
}

/// 欄の中の行へ割る。**改行は欄の中の行を分ける。**
fn lines(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut now = String::new();
    let mut i = 0;
    let bytes = body.as_bytes();
    while i < body.len() {
        if bytes[i] == b'<' {
            let lower: String = body[i..].chars().take(6).collect::<String>().to_lowercase();
            if lower.starts_with("<br>")
                || lower.starts_with("<br/>")
                || lower.starts_with("<br />")
            {
                let end = body[i..].find('>').map_or(body.len(), |x| i + x + 1);
                out.push(std::mem::take(&mut now));
                i = end;
                continue;
            }
        }
        let c = body[i..].chars().next().unwrap_or('\0');
        now.push(c);
        i += c.len_utf8();
    }
    out.push(now);
    out
}

/// 「。」のあとで文を割る。**区切りは前の文に残す。**
fn sentences(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut now = String::new();
    for c in body.chars() {
        now.push(c);
        if c == '。' {
            out.push(std::mem::take(&mut now));
        }
    }
    if !now.is_empty() {
        out.push(now);
    }
    out
}

/// 1つの行に2つのことが入っていないか、1文が長すぎないかを見る。
///
/// **箇条書きごと除外してはならない** ── 以前の検査は箇条書きを含む欄を丸ごと素通しに
/// していた。列挙はこの道具が欄へ割ってから渡す。
#[must_use]
pub fn prose(place: &str, body: &str) -> Vec<String> {
    let mut bad = Vec::new();
    for line in lines(body) {
        let raw = plain(&line);
        let count = raw.matches(SEP).count();
        if count > COHABIT {
            bad.push(format!(
                "{place}: 1つの行に区切り「{SEP}」が {count} 個ある"
            ));
        }
    }
    let whole = plain(body);
    for sentence in sentences(&whole) {
        let sentence = sentence.trim();
        let length = sentence.chars().count();
        if length > LONGEST {
            bad.push(format!(
                "{place}: 1文が {length} 字ある（上限 {LONGEST}）── {}…",
                sentence.chars().take(34).collect::<String>()
            ));
        }
    }
    bad
}

/// 欄の中に在ってはならない印。
const LEVEL: [&str; 16] = [
    "p", "h1", "h2", "h3", "h4", "h5", "h6", "ul", "ol", "li", "pre", "div", "table", "tr", "td",
    "th",
];
/// 同じ理由で在ってはならない、畳みと図の印。
const LEVEL_MORE: [&str; 4] = ["details", "summary", "figure", "blockquote"];

/// **入力は宣言だけを保持する。** 見出しや表が入っていれば、宣言の外に構造がある。
#[must_use]
pub fn level_mix(place: &str, body: &str) -> Vec<String> {
    let lower = body.to_lowercase();
    let mut from = 0;
    while let Some(at) = lower[from..].find('<').map(|x| x + from) {
        let head = at + 1;
        let head = if lower[head..].starts_with('/') {
            head + 1
        } else {
            head
        };
        let name: String = lower[head..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();
        let after = lower[head + name.len()..].chars().next();
        let bounded = after.is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '-'));
        if bounded && (LEVEL.contains(&name.as_str()) || LEVEL_MORE.contains(&name.as_str())) {
            let shown = &body[at..head + name.len()];
            return vec![format!(
                "{place}: 見出しや表「{shown}」が欄の中に在る ── 宣言へ割る"
            )];
        }
        from = at + 1;
    }
    Vec::new()
}

/// 節の見出しを手で書いた畳みを探す。**見出しは道具が作る。**
#[must_use]
pub fn authored_sections(d: &Value) -> Vec<String> {
    fn walk(value: &Value, where_: &str, bad: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                if map.get("kind").and_then(|x| x.as_str()) == Some("fold") {
                    let heading = map
                        .get("heading")
                        .and_then(|x| x.as_str())
                        .unwrap_or_default();
                    if heading.contains("この答えの") || heading.contains("この答えが") {
                        bad.push(format!(
                            "{where_}: 節の見出しを手で書いている「{}」 ── 見出しは道具が作る。中身だけを置く",
                            heading.chars().take(28).collect::<String>()
                        ));
                    }
                }
                for v in map.values() {
                    walk(v, where_, bad);
                }
            }
            Value::Array(items) => {
                for v in items {
                    walk(v, where_, bad);
                }
            }
            _ => {}
        }
    }
    let mut bad = Vec::new();
    for t in array_of(d, "topics") {
        let no = text_of(t, "no");
        if let Some(x) = t.get("example") {
            walk(x, &format!("topics[{no}]/example"), &mut bad);
        }
        if let Some(x) = t.get("panels") {
            walk(x, &format!("topics[{no}]/panels"), &mut bad);
        }
    }
    if let Some(x) = d.get("panels") {
        walk(x, "panels", &mut bad);
    }
    bad
}

/// 図の参照先が実在するかを見る。
#[must_use]
pub fn refs(dir: &Path, d: &Value) -> Vec<String> {
    fn walk(value: &Value, dir: &Path, seen: &mut Vec<String>, bad: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                if map.get("kind").and_then(|x| x.as_str()) == Some("figure") {
                    if let Some(name) = map.get("name").and_then(|x| x.as_str()) {
                        if !seen.contains(&name.to_owned()) {
                            seen.push(name.to_owned());
                            if !dir.join("figures").join(format!("{name}.svg")).exists() {
                                bad.push(format!("図 figures/{name}.svg が無い"));
                            }
                        }
                    }
                }
                for v in map.values() {
                    walk(v, dir, seen, bad);
                }
            }
            Value::Array(items) => {
                for v in items {
                    walk(v, dir, seen, bad);
                }
            }
            _ => {}
        }
    }
    let mut seen = Vec::new();
    let mut bad = Vec::new();
    walk(d, dir, &mut seen, &mut bad);
    bad
}

/// 入力1つを、4系統すべてで検査する。**呼ぶ側は、0件のときだけ組む。**
#[must_use]
pub fn inspect(references: &Path, dir: &Path, d: &Value) -> Vec<String> {
    let mut bad = crate::shape::against(&schema_path(references), d, "形");
    bad.extend(refs(dir, d));
    bad.extend(authored_sections(d));
    for (place, body) in cells(d) {
        bad.extend(prose(&place, &body));
        bad.extend(level_mix(&place, &body));
    }
    bad
}

/// ブレストボードのフォルダ1つを検査する。
///
/// # Errors
///
/// 入力を読めないときに返す。
pub fn check(references: &Path, dir: &Path) -> Result<Vec<String>, String> {
    let path = dir.join("board.json");
    let body = std::fs::read_to_string(&path)
        .map_err(|e| format!("{}: 読めない ── {e}", path.display()))?;
    let d: Value = serde_json::from_str(&body)
        .map_err(|e| format!("{}: JSON として読めない ── {e}", path.display()))?;
    Ok(inspect(references, dir, &d))
}
