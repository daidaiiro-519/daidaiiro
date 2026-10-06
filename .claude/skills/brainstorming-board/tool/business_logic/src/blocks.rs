// SPDX-License-Identifier: MIT
//! 宣言の並びを HTML へ組み、入力から論点を起こす。
//!
//! **種類ごとに1つの形だけを持つ** ── 同じものを2つの形で書けると、ブレストボードごとに違う形が出る。
//!
//! **正規化した SVG を JSON へ格納しない** ── 固定幅を外す処理は自身の出力へ再適用すると
//! `max-width` が消失し、冪等でなくなる。

use crate::data_access::files;

use std::path::Path;

use serde_json::Value;

use crate::cell::{cell, esc};
use crate::template::Parts;
use crate::topic::{Option_, Table, Topic};

/// 出来事の種別。**種別の色は、前提の表と同じ意味で割り当てる。**
const EVENTS: [(&str, &str, &str); 4] = [
    ("returned", "k-given", "差し戻し"),
    ("obsolete", "k-open", "失効した点"),
    ("finding", "k-fact", "判明事項"),
    ("content", "k-rule", "内容"),
];

/// 扱いの3値。出どころの5値とは別の系列だが、種別の形は共有する。
const TREATMENT: [(&str, &str, &str); 3] = [
    ("out", "k-rule", "対象外"),
    ("later", "k-given", "後続で決定"),
    ("resolved", "k-fact", "解消済"),
];

/// 確かさの順。**外の根拠 → こちらの決まり → 実測 → 前提 → 未確認。**
const CERTAINTY: [&str; 5] = ["primary", "rule", "measured", "assumption", "unverified"];

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

fn event_of(tag: &str) -> Option<(&'static str, &'static str)> {
    EVENTS
        .iter()
        .find(|(k, _, _)| *k == tag)
        .map(|(_, cls, label)| (*cls, *label))
}

/// 出来事を、種別ごとのまとまりにする。
///
/// **同じ種別が複数あるなら、1つの見出しの下へ束ねて番号を付与する。**
fn events(parts: &Parts, rows: &[(String, String)]) -> Result<String, String> {
    if rows.is_empty() {
        return Ok(String::new());
    }
    let mut body = String::new();
    let mut i = 0;
    while i < rows.len() {
        let tag = rows[i].0.clone();
        let mut j = i;
        while j < rows.len() && rows[j].0 == tag {
            j += 1;
        }
        let n = j - i;
        let count = if n > 1 {
            parts.part("event-count", &[("count", n.to_string())])?
        } else {
            String::new()
        };
        let Some((cls, label)) = event_of(&tag) else {
            return Err(format!("知らない出来事の種別: {tag}"));
        };
        for m in 0..n {
            let head = if m == 0 {
                parts.part(
                    "event-head",
                    &[
                        ("span", n.to_string()),
                        ("kind", cls.to_owned()),
                        ("label", label.to_owned()),
                        ("count", count.clone()),
                    ],
                )?
            } else {
                String::new()
            };
            let num = if n > 1 {
                parts.part("event-num", &[("n", (m + 1).to_string())])?
            } else {
                String::new()
            };
            let end = if m == n - 1 && j < rows.len() {
                " g-end"
            } else {
                ""
            };
            body.push_str(&parts.part(
                "event-row",
                &[
                    ("tag", tag.clone()),
                    ("end", end.to_owned()),
                    ("head", head),
                    ("num", num),
                    ("body", rows[i + m].1.clone()),
                ],
            )?);
        }
        i = j;
    }
    parts.part("events", &[("rows", body)])
}

/// 固定幅を外し、2つの上限を同時に当てる。
///
/// **元の幅（引き伸ばさない）と、入れ物の幅（はみ出さない）である** ── 片方だけを行内の
/// style へ書くと、それがブレストボードの CSS に勝ち、狭い入れ物から図がはみ出す（実測 ── はみ出した）。
#[must_use]
pub fn fit(svg: &str) -> String {
    let Some(at) = svg.find(" width=\"") else {
        return svg.to_owned();
    };
    let head = at + " width=\"".len();
    let Some(end) = svg[head..].find('"').map(|x| head + x) else {
        return svg.to_owned();
    };
    let value = &svg[head..end];
    if value.is_empty() || !value.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return svg.to_owned();
    }
    let taken = format!(" width=\"{value}\"");
    let without = svg.replacen(&taken, "", 1);
    without.replacen(
        "<svg ",
        &format!("<svg style=\"max-width:min(100%,{value}px)\" "),
        1,
    )
}

/// 図は `figures/<名前>.svg` を毎回読む。
fn figure(parts: &Parts, b: &Value, figures: &Path) -> Result<String, String> {
    let name = text_of(b, "name");
    let path = figures.join(format!("{name}.svg"));
    let svg =
        files::read_to_string(&path).map_err(|e| format!("{}: 読めない ── {e}", path.display()))?;
    let caption = text_of(b, "caption");
    let note = if caption.is_empty() {
        String::new()
    } else {
        parts.part("figcaption", &[("text", caption)])?
    };
    parts.part("figure-top", &[("svg", fit(&svg)), ("caption", note)])
}

/// 宣言の並びを HTML へ組む。
///
/// # Errors
///
/// 知らない種類のときと、図を読めないときと、型と噛み合わないときに返す。
#[allow(clippy::too_many_lines)]
pub fn build(parts: &Parts, blocks: &[Value], figures: &Path) -> Result<String, String> {
    let mut out = String::new();
    for b in blocks {
        let kind = text_of(b, "kind");
        let nested = |key: &str| -> &[Value] { array_of(b, key) };
        match kind.as_str() {
            "para" => {
                let body = text_of(b, "text") + &build(parts, nested("nested"), figures)?;
                out.push_str(&if body.starts_with("<b>") {
                    body
                } else {
                    parts.part("para", &[("body", body)])?
                });
            }
            "note" => out.push_str(&parts.part("note-s", &[("body", text_of(b, "text"))])?),
            "heading" => out.push_str(&parts.part(
                "heading",
                &[("level", text_of(b, "level")), ("text", text_of(b, "text"))],
            )?),
            "html" => out.push_str(&parts.part("code", &[("text", text_of(b, "text"))])?),
            "list" => {
                let tag = if b.get("ordered").and_then(Value::as_bool) == Some(true) {
                    "ol"
                } else {
                    "ul"
                };
                let mut items = String::new();
                for x in array_of(b, "items") {
                    let body = cell(parts, &text_of(x, "text"))?
                        + &build(parts, array_of(x, "nested"), figures)?;
                    items.push_str(&parts.part("list-item", &[("body", body)])?);
                }
                out.push_str(&parts.part("list", &[("tag", tag.to_owned()), ("items", items)])?);
            }
            "fold" => out.push_str(&parts.part(
                "fold-why",
                &[
                    ("summary", text_of(b, "heading")),
                    ("body", build(parts, array_of(b, "body"), figures)?),
                ],
            )?),
            "card" => {
                let letter = text_of(b, "letter");
                let mark = if letter.is_empty() {
                    String::new()
                } else {
                    parts.part("card-mark", &[("letter", letter)])?
                };
                let mut rows = Vec::new();
                for e in array_of(b, "events") {
                    let tag = text_of(e, "tag");
                    let text = text_of(e, "text");
                    let body = match tag.as_str() {
                        "returned" => parts.part("quote", &[("text", text)])?,
                        "finding" => cell(parts, &text)?,
                        _ => text,
                    };
                    rows.push((tag, body));
                }
                out.push_str(&parts.part(
                    "card",
                    &[
                        ("mark", mark),
                        ("heading", text_of(b, "heading")),
                        ("events", events(parts, &rows)?),
                    ],
                )?);
            }
            "table" => {
                let mut rows = String::new();
                for row in array_of(b, "rows") {
                    let pair = row.as_array().map_or(&[][..], |x| x.as_slice());
                    let head = pair.first().map_or_else(String::new, value_text);
                    let mut cells = String::new();
                    for v in pair
                        .get(1)
                        .and_then(|x| x.as_array())
                        .map_or(&[][..], |x| x.as_slice())
                    {
                        cells.push_str(
                            &parts.part("table-td", &[("cell", cell(parts, &value_text(v))?)])?,
                        );
                    }
                    rows.push_str(
                        &parts.part("table-row-head", &[("head", head), ("cells", cells)])?,
                    );
                }
                let mut head = parts.part("table-th", &[("cell", String::new())])?;
                for c in array_of(b, "cols") {
                    head.push_str(&parts.part("table-th", &[("cell", value_text(c))])?);
                }
                let thead = parts.part(
                    "table-thead",
                    &[("row", parts.part("table-head", &[("cells", head)])?)],
                )?;
                out.push_str(&parts.part(
                    "table-cls",
                    &[
                        ("cls", "fact".to_owned()),
                        ("head", thead),
                        ("rows", parts.part("table-tbody", &[("rows", rows)])?),
                    ],
                )?);
            }
            "grid" => {
                let mut head = String::new();
                for c in array_of(b, "cols") {
                    head.push_str(&parts.part("table-th", &[("cell", value_text(c))])?);
                }
                let mut body = String::new();
                for row in array_of(b, "rows") {
                    let mut cells = String::new();
                    for v in row.as_array().map_or(&[][..], |x| x.as_slice()) {
                        cells.push_str(&parts.part("table-td", &[("cell", value_text(v))])?);
                    }
                    body.push_str(&parts.part("table-row", &[("cells", cells)])?);
                }
                let thead = parts.part(
                    "table-thead",
                    &[("row", parts.part("table-head", &[("cells", head)])?)],
                )?;
                out.push_str(&parts.part(
                    "table",
                    &[
                        ("head", thead),
                        ("rows", parts.part("table-tbody", &[("rows", body)])?),
                    ],
                )?);
            }
            "previous" => {
                let mut rows = vec![
                    (
                        "前の答え".to_owned(),
                        parts.part("lead", &[("text", text_of(b, "letter"))])?
                            + "　"
                            + &text_of(b, "body"),
                    ),
                    (
                        "なぜ作り直したか".to_owned(),
                        cell(parts, &text_of(b, "why"))?,
                    ),
                ];
                let words = text_of(b, "user_words");
                if !words.is_empty() {
                    rows.push(("利用者の言葉（そのまま）".to_owned(), words));
                }
                let mut body = String::new();
                for (head, value) in rows {
                    body.push_str(&parts.part(
                        "table-row-head",
                        &[
                            ("head", head),
                            ("cells", parts.part("table-td", &[("cell", value)])?),
                        ],
                    )?);
                }
                out.push_str(&parts.part("table", &[("head", String::new()), ("rows", body)])?);
            }
            "figure" => out.push_str(&figure(parts, b, figures)?),
            other => return Err(format!("知らない宣言の種類: {other}")),
        }
    }
    Ok(out)
}

fn value_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// 扱いを札にする ── 太字だけでは、同じ意味の印が2種になる。
///
/// **札は入力が3値で持ち、画面へ出す語はこの対応表が保持する** ── 入力へ印を書くと、
/// 散文の検査がそこへ当たらなくなる。
fn tagged(parts: &Parts, w: &Value) -> Result<String, String> {
    let tag = text_of(w, "treatment");
    let Some((cls, label)) = TREATMENT
        .iter()
        .find(|(k, _, _)| *k == tag)
        .map(|(_, cls, label)| (*cls, *label))
    else {
        return Err(format!("知らない扱い: {tag}"));
    };
    let note = text_of(w, "note");
    Ok(parts.part(
        "kind",
        &[("cls", cls.to_owned()), ("label", label.to_owned())],
    )? + &if note.is_empty() {
        String::new()
    } else {
        format!(" ── {note}")
    })
}

/// 図を、面へ置ける形へ剥く。**囲みは面の側が持つ。**
fn figure_svg(parts: &Parts, f: &Value, figures: &Path) -> Result<String, String> {
    let built = figure(parts, f, figures)?;
    let caption = text_of(f, "caption");
    let stripped = built
        .replace("<figure class=\"fig-top\">", "")
        .replace("</figure>", "");
    Ok(if caption.is_empty() {
        stripped
    } else {
        stripped.replace(&format!("<figcaption>{caption}</figcaption>"), "")
    })
}

/// 入力1件から論点を起こす。
///
/// # Errors
///
/// 知らない種類のときと、図を読めないときと、型と噛み合わないときに返す。
pub fn to_topic(parts: &Parts, d: &Value, figures: &Path) -> Result<Topic, String> {
    let pick = match d.get("decision") {
        None | Some(Value::Null) => None,
        Some(decision) => Some((
            text_of(decision, "letter"),
            build(parts, array_of(decision, "text"), figures)?,
        )),
    };
    let mut kept = Vec::new();
    for o in array_of(d, "passed") {
        kept.push(Option_::new(
            text_of(o, "name"),
            text_of(o, "body"),
            text_of(o, "cost"),
        ));
    }
    let mut figures_out = Vec::new();
    for f in array_of(d, "figures") {
        figures_out.push((figure_svg(parts, f, figures)?, text_of(f, "caption")));
    }
    let mut path = Vec::new();
    for b in array_of(d, "path") {
        path.push(build(parts, std::slice::from_ref(b), figures)?);
    }
    let mut found = Vec::new();
    for x in array_of(d, "findings") {
        found.push(cell(parts, &value_text(x))?);
    }
    let mut grounds = Vec::new();
    for g in array_of(d, "grounds") {
        grounds.push((
            text_of(g, "supports"),
            cell(parts, &text_of(g, "basis"))?,
            text_of(g, "tag"),
            text_of(g, "source"),
        ));
    }
    // **確かさの順に並べ替える。** 組み立てと基準の保存が、同じものを見るようにする ──
    // 片方だけが並べ替えていたので、根拠の欄が毎回「変わった」と出ていた（実測 140 か所）
    grounds.sort_by_key(|(_, _, tag, _)| CERTAINTY.iter().position(|k| k == tag).unwrap_or(9));
    let mut costs = Vec::new();
    for x in array_of(d, "requirements") {
        costs.push(cell(parts, &value_text(x))?);
    }
    let mut weaknesses = Vec::new();
    for w in array_of(d, "out_of_scope") {
        let how = if w.get("treatment").is_some() {
            tagged(parts, w)?
        } else {
            String::new()
        };
        weaknesses.push((text_of(w, "item"), how));
    }
    let mut extras = Vec::new();
    for e in array_of(d, "panels") {
        extras.push((
            text_of(e, "heading"),
            build(parts, array_of(e, "body"), figures)?,
        ));
    }
    let mut tables = Vec::new();
    for tb in array_of(d, "tables") {
        let mut rows = Vec::new();
        for row in array_of(tb, "rows") {
            let pair = row.as_array().map_or(&[][..], |x| x.as_slice());
            rows.push((
                pair.first().map_or_else(String::new, value_text),
                pair.get(1)
                    .and_then(|x| x.as_array())
                    .map(|xs| xs.iter().map(value_text).collect())
                    .unwrap_or_default(),
            ));
        }
        tables.push(Table {
            caption: text_of(tb, "caption"),
            columns: array_of(tb, "cols").iter().map(value_text).collect(),
            rows,
            lead: text_of(tb, "lead"),
            plain: tb.get("plain").and_then(Value::as_bool) == Some(true),
        });
    }
    let mut dropped = Vec::new();
    for x in array_of(d, "dropped") {
        dropped.push((text_of(x, "body"), text_of(x, "reason")));
    }
    Ok(Topic {
        no: d.get("no").and_then(Value::as_u64).unwrap_or(0) as usize,
        label: text_of(d, "name"),
        question: text_of(d, "question"),
        status: text_of(d, "status"),
        answer: cell(parts, &text_of(d, "answer"))?,
        note: text_of(d, "intro"),
        figures: figures_out,
        kept,
        tables,
        dropped,
        found,
        pick,
        path,
        grounds,
        costs,
        example: build(parts, array_of(d, "example"), figures)?,
        weaknesses,
        defects: Vec::new(),
        decision: Vec::new(),
        extras,
    })
}

/// 入力から論点を起こし、**並べ替えまで済ませる。**
///
/// # Errors
///
/// 知らない種類のときと、図を読めないときと、型と噛み合わないときに返す。
pub fn prepare(parts: &Parts, d: &Value, figures: &Path) -> Result<Vec<Topic>, String> {
    let mut out = Vec::new();
    for t in array_of(d, "topics") {
        out.push(to_topic(parts, t, figures)?);
    }
    Ok(out)
}

/// 題を、そのまま置ける形にする。
#[must_use]
pub fn title_of(d: &Value) -> String {
    esc(&text_of(d, "title"))
}
