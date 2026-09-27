// SPDX-License-Identifier: MIT
//! 欄の整形。**1つの欄に2つのことが入っているものを、道具が2段へ割る。**
//!
//! 「主張 ── 説明」と書いたものは、1つの欄に2つのことが入っている。区切りで割ると、
//! 読む側が文を解きほぐさずに済む。
//!
//! **書き手に整形させない** ── 整形を散文の規定にすると、ブレストボードごとに違う形が出る。

use crate::template::Parts;

/// 主張と説明の区切り。
pub const SEP: &str = " ── ";

/// 文字列を、HTML の中へそのまま置ける形にする。
#[must_use]
pub fn esc(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// 区切りで割る。**印の内側では割らない。**
///
/// 印を付けたあとの文字列には、属性の中にも区切りが入る ── そこで割ると、属性が本文へ
/// 漏れる（実測 ── 実際に漏れた）。
#[must_use]
pub fn cut(body: &str) -> (String, bool, String) {
    let chars: Vec<char> = body.chars().collect();
    let sep: Vec<char> = SEP.chars().collect();
    let mut depth = 0usize;
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 && chars[i..].starts_with(&sep[..]) => {
                return (
                    chars[..i].iter().collect(),
                    true,
                    chars[i + sep.len()..].iter().collect(),
                );
            }
            _ => {}
        }
        i += 1;
    }
    (body.to_owned(), false, String::new())
}

/// 1つの欄に2つのことが入っているものを、**主張と説明の2段**にする。
///
/// 次の3つは割らない ── 引用（`「` で始まるもの）／ 既に割ってあるもの ／ 区切りを
/// 持たないもの。**引用を割らないのは、原文の形を変えないためである。**
/// **既に割ってあるものを割らないので、何度当てても同じものが出る（冪等）。**
///
/// # Errors
///
/// 型と噛み合わないときに返す。
pub fn cell(parts: &Parts, body: &str) -> Result<String, String> {
    if body.trim_start().starts_with('「') || body.contains("class=\"lead-s\"") {
        return Ok(body.to_owned());
    }
    let (head, split, tail) = cut(body);
    if !split {
        return Ok(body.to_owned());
    }
    let head = head
        .strip_prefix("<b>")
        .and_then(|x| x.strip_suffix("</b>"))
        .map_or(head.clone(), str::to_owned);
    parts.part("cell", &[("head", head), ("tail", tail)])
}

/// 印の抜き書き。**引き出しの一覧に出すので、短くする。**
#[must_use]
pub fn plain(body: &str, count: usize) -> String {
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
    let out = out.trim();
    let taken: String = out.chars().take(count).collect();
    esc(&(taken
        + if out.chars().count() > count {
            "…"
        } else {
            ""
        }))
}

/// 変わった箇所の印。押すと、変更前と理由が開く。
///
/// `cid` を持つ印は、**引き出しから直に跳べる**。
///
/// # Errors
///
/// 型と噛み合わないときに返す。
pub fn mark(
    parts: &Parts,
    text: &str,
    before: &str,
    why: &str,
    deleted: bool,
    cid: Option<&str>,
) -> Result<String, String> {
    parts.part(
        "mark",
        &[
            (
                "del",
                if deleted {
                    " del".to_owned()
                } else {
                    String::new()
                },
            ),
            (
                "id",
                cid.map_or_else(String::new, |x| format!("id=\"{x}\" ")),
            ),
            ("before", esc(before)),
            ("why", esc(why)),
            ("body", text.to_owned()),
        ],
    )
}

/// 表を組む。**見出しが全部空なら、見出しの行を出さない。**
///
/// 空の `<th>` を並べると、中身の無い帯が表の上に1本出る ── 決まりの表のように、行の
/// 名前だけで読める表では見出しが要らない。
///
/// # Errors
///
/// 型と噛み合わないときに返す。
pub fn table(parts: &Parts, head: &[String], rows: &[Vec<String>]) -> Result<String, String> {
    let mut body = String::new();
    for row in rows {
        let mut cells = String::new();
        for value in row {
            cells.push_str(&parts.part("table-td", &[("cell", value.clone())])?);
        }
        body.push_str(&parts.part("table-row", &[("cells", cells)])?);
    }
    if head.iter().all(|c| c.trim().is_empty()) {
        return parts.part("table", &[("head", String::new()), ("rows", body)]);
    }
    let mut cells = String::new();
    for value in head {
        cells.push_str(&parts.part("table-th", &[("cell", value.clone())])?);
    }
    let head = parts.part("table-head", &[("cells", cells)])?;
    parts.part("table", &[("head", head), ("rows", body)])
}

/// 『主張 ── 説明』の並びを、2列の表にする。**区切りが無い行は、右を空にする。**
///
/// # Errors
///
/// 型と噛み合わないときに返す。
pub fn pairs(parts: &Parts, items: &[String], left: &str, right: &str) -> Result<String, String> {
    let mut rows = Vec::new();
    for item in items {
        let (head, split, tail) = cut(item);
        rows.push(vec![
            parts.part("lead", &[("text", head)])?,
            if split { tail } else { String::new() },
        ]);
    }
    table(parts, &[left.to_owned(), right.to_owned()], &rows)
}

/// 案の記号。**空の調子で余分な空白を作らない** ── 部品の側で吸収する。
///
/// # Errors
///
/// 型と噛み合わないときに返す。
pub fn key(parts: &Parts, letter: &str, tone: &str) -> Result<String, String> {
    parts.part(
        "key",
        &[
            (
                "tone",
                if tone.is_empty() {
                    String::new()
                } else {
                    format!(" {tone}")
                },
            ),
            ("letter", letter.to_owned()),
        ],
    )
}
