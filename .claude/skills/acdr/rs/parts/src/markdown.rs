// SPDX-License-Identifier: MIT
//! マークダウンを描画し、変更箇所に印を付ける。
//!
//! 変更1件の形は `{"find": …, "before": 変更前の原文, "why": なぜ変えたか}` である。
//! `find` は**描画後の HTML に現れる文字列**である。
//!
//! 一致しなかった `find` は報告へ出す。**黙って除外しない。**
//!
//! **HTML の形は、ここが持たない** ── `references/acdr.template.html` が持つ。

use regex::Regex;
use serde_json::Value;

use crate::template::Parts;

/// 文字列を、HTML の中へそのまま置ける形にする。
///
/// **引用符も逃がす** ── 本文と属性で逃がし方を分けると、同じ値が2つの姿を持つ。
#[must_use]
pub fn esc(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn text_of(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_owned()
}

/// 行の中の印。**`<br>` だけは通す** ── 欄の中の行を分けるために要る。
fn inline(parts: &Parts, body: &str) -> String {
    let escaped = esc(body);
    let code = Regex::new(r"`([^`]+)`").expect("式である");
    let strong = Regex::new(r"\*\*(.+?)\*\*").expect("式である");
    let wrap = |pat: &Regex, part: &str, into: &str| -> String {
        pat.replace_all(into, |c: &regex::Captures| {
            parts
                .part(part, &[("body", c[1].to_owned())])
                .unwrap_or_default()
        })
        .into_owned()
    };
    let got = wrap(&strong, "md-strong", &wrap(&code, "md-code", &escaped));
    got.replace("&lt;br&gt;", "<br>")
        .replace("&lt;br/&gt;", "<br>")
}

/// 区切りの行か。
fn is_separator(cells: &[String]) -> bool {
    let rule = Regex::new(r"^:?-{2,}:?$").expect("式である");
    cells.iter().any(|c| !c.trim().is_empty())
        && cells
            .iter()
            .filter(|c| !c.trim().is_empty())
            .all(|c| rule.is_match(c.trim()))
}

/// マークダウンを描画する。
///
/// # Errors
///
/// 型と噛み合わないときに返す。
pub fn render(parts: &Parts, md: &str) -> Result<String, String> {
    // HTML のコメントは、描画すると文字として出る。除去する
    let comment = Regex::new(r"(?s)<!--.*?-->").expect("式である");
    let md = comment.replace_all(md, "");
    let heading = Regex::new(r"^(#{1,4})\s+(.*)").expect("式である");
    let lines: Vec<&str> = md.split('\n').collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some(rest) = line.strip_prefix("```") {
            let lang = rest.trim();
            let mut j = i + 1;
            let mut body = Vec::new();
            while j < lines.len() && !lines[j].starts_with("```") {
                body.push(lines[j]);
                j += 1;
            }
            let cls = if lang == "mermaid" { "mermaid" } else { "code" };
            out.push(parts.part(
                "md-pre",
                &[("cls", cls.to_owned()), ("body", esc(&body.join("\n")))],
            )?);
            i = j + 1;
            continue;
        }
        if let Some(m) = heading.captures(line) {
            out.push(parts.part(
                "md-heading",
                &[
                    ("level", m[1].len().to_string()),
                    ("body", inline(parts, &m[2])),
                ],
            )?);
            i += 1;
            continue;
        }
        if line.starts_with('|') {
            let mut cells: Vec<Vec<String>> = Vec::new();
            while i < lines.len() && lines[i].starts_with('|') {
                cells.push(
                    lines[i]
                        .trim()
                        .trim_matches('|')
                        .split('|')
                        .map(|c| c.trim().to_owned())
                        .collect(),
                );
                i += 1;
            }
            let mut body: Vec<Vec<String>> =
                cells.into_iter().filter(|c| !is_separator(c)).collect();
            let mut rows = String::new();
            // 見出しが全部空なら、見出しの行を出さない ── 空の `<th>` を並べると、
            // 中身の無い帯が表の上に1本出る
            if body
                .first()
                .is_some_and(|r| r.iter().all(|c| c.trim().is_empty()))
            {
                body.remove(0);
            } else if body.len() > 1 {
                let head = body.remove(0);
                let mut inner = String::new();
                for cell in &head {
                    inner.push_str(&parts.part("md-th", &[("cell", inline(parts, cell))])?);
                }
                rows.push_str(&parts.part("md-tr", &[("cells", inner)])?);
            }
            for row in &body {
                let mut inner = String::new();
                for cell in row {
                    inner.push_str(&parts.part("md-td", &[("cell", inline(parts, cell))])?);
                }
                rows.push_str(&parts.part("md-tr", &[("cells", inner)])?);
            }
            out.push(parts.part("md-table", &[("rows", rows)])?);
            continue;
        }
        if line.starts_with('>') {
            let mut body = Vec::new();
            while i < lines.len() && lines[i].starts_with('>') {
                body.push(lines[i][1..].trim().to_owned());
                i += 1;
            }
            out.push(parts.part("md-quote", &[("body", inline(parts, &body.join("<br>")))])?);
            continue;
        }
        if line.trim() == "---" {
            out.push(parts.part("md-hr", &[])?);
            i += 1;
            continue;
        }
        if line.trim().is_empty() {
            i += 1;
            continue;
        }
        let mut body = Vec::new();
        while i < lines.len()
            && !lines[i].trim().is_empty()
            && !lines[i].starts_with(['|', '>', '#'])
            && !lines[i].starts_with("```")
            && lines[i].trim() != "---"
        {
            body.push(lines[i]);
            i += 1;
        }
        out.push(parts.part("para", &[("body", inline(parts, &body.join("<br>")))])?);
    }
    Ok(out.join("\n"))
}

/// `<pre>` の外だけを、印を付けてよい範囲として返す。
///
/// コードと図の中に印を差し込むと、その中身が壊れる ── mermaid は差し込んだ時点で
/// 描画されなくなる。
#[must_use]
pub fn outside_pre(body: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut i = 0;
    loop {
        match body[i..].find("<pre").map(|x| i + x) {
            None => {
                spans.push((i, body.len()));
                break;
            }
            Some(at) => {
                spans.push((i, at));
                i = body[at..]
                    .find("</pre>")
                    .map_or(body.len(), |x| at + x + "</pre>".len());
            }
        }
    }
    spans
}

/// 印を付けた結果。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Marked {
    /// 印を差し込んだ本文。
    pub body: String,
    /// 印を付けた数。
    pub kept: usize,
    /// 渡された数。
    pub asked: usize,
    /// 除外したものと、その理由。**黙って除外しない。**
    pub dropped: Vec<String>,
}

/// 変更後の HTML に、印を差し込む。
///
/// **位置は、差し込む前の HTML に対して先に全部決める。** 差し込んだ `data-b` ・
/// `data-w` は HTML の一部になるので、差し込みながら探すと、**次の印が前の印の理由文の
/// 中へ入る** ── 実際にそれで属性の中へ `<mark>` が入り、面が壊れた。
///
/// # Errors
///
/// 型と噛み合わないときに返す。
pub fn mark(parts: &Parts, body: &str, marks: &[Value]) -> Result<Marked, String> {
    let spans = outside_pre(body);
    let mut plan: Vec<(usize, usize, &Value)> = Vec::new();
    let mut dropped = Vec::new();
    for change in marks {
        let find = text_of(change, "find");
        if find.is_empty() || !body.contains(&find) {
            dropped.push(format!("一致せず: {}", head(&find, 60)));
            continue;
        }
        let at = spans.iter().find_map(|(a, b)| {
            body.get(*a..*b)
                .and_then(|slice| slice.find(&find))
                .map(|x| a + x)
        });
        let Some(at) = at else {
            dropped.push(format!("コードか図の中にしかない: {}", head(&find, 60)));
            continue;
        };
        if text_of(change, "why").is_empty() {
            dropped.push(format!("なぜが無い: {}", head(&find, 60)));
        }
        plan.push((at, find.len(), change));
    }
    // 重なりを除外する。**同じ場所へ2つ差し込むと、片方が他方の中へ入る**
    plan.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
    let mut kept: Vec<(usize, usize, &Value)> = Vec::new();
    let mut end = 0;
    for (at, len, change) in plan {
        if !kept.is_empty() && at < end {
            dropped.push(format!(
                "位置が重なる: {}",
                head(&text_of(change, "find"), 60)
            ));
            continue;
        }
        kept.push((at, len, change));
        end = at + len;
    }
    // 後ろから差し込む。**前の位置がずれない**
    let mut out = body.to_owned();
    for (at, len, change) in kept.iter().rev() {
        let piece = parts.part(
            "mark",
            &[
                ("before", esc(&text_of(change, "before"))),
                ("why", esc(&text_of(change, "why"))),
                ("body", out[*at..at + len].to_owned()),
            ],
        )?;
        out = format!("{}{piece}{}", &out[..*at], &out[at + len..]);
    }
    Ok(Marked {
        body: out,
        kept: kept.len(),
        asked: marks.len(),
        dropped,
    })
}

/// 先頭の何文字かを取る。**文字で数える** ── バイトで切ると日本語が割れる。
fn head(body: &str, count: usize) -> String {
    body.chars().take(count).collect()
}
