// SPDX-License-Identifier: MIT
//! デッキの入力（JSON）から、1枚の HTML を組む。
//!
//! **3つで組む。** 入力の形は `references/slide-deck.schema.json` が、出来上がりの形は
//! `references/slide-deck.template.html` が、配色は `references/themes/<名前>.css` が
//! 持つ。ここが持つのは、**どの値をどの部品へ差し込むか**だけである ── HTML の形を
//! ここへ書かない。
//!
//! **同じ入力からは、同じ1枚が出る。** 日付も乱数も読まない。

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::data_access::files;
use crate::template::Parts;
use crate::{theme, validate};

/// 型の置き場所。
#[must_use]
pub fn template_path(references: &Path) -> PathBuf {
    references.join("slide-deck.template.html")
}

/// 入力の契約の置き場所。
#[must_use]
pub fn schema_path(references: &Path) -> PathBuf {
    references.join("slide-deck.schema.json")
}

/// 枚の中で主従を付けるために通す印。
const KEPT: [&str; 6] = ["b", "i", "em", "strong", "mark", "small"];

/// 文字列を、HTML の中へそのまま置ける形にする。
///
/// **太字と強調だけは通す** ── `<b>` ・ `<i>` ・ `<em>` ・ `<mark>` ・ `<br>` は、
/// 枚の中で主従を付けるために要る。それ以外の印は文字として出る。
#[must_use]
pub fn esc(value: &str) -> String {
    let mut out = value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    for tag in KEPT {
        out = out
            .replace(&format!("&lt;{tag}&gt;"), &format!("<{tag}>"))
            .replace(&format!("&lt;/{tag}&gt;"), &format!("</{tag}>"));
    }
    out.replace("&lt;br&gt;", "<br>")
}

fn text_of(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_owned()
}

fn array_of<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(|x| x.as_array())
        .map_or(&[], |x| x.as_slice())
}

/// 値が在るときだけ、その部品で包む。**無い欄は、空の文字列になる。**
fn wrap(parts: &Parts, part: &str, slot: &str, value: &str) -> Result<String, String> {
    if value.is_empty() {
        return Ok(String::new());
    }
    parts.part(part, &[(slot, esc(value))])
}

/// 要素1つを組む。
///
/// # Errors
///
/// 知らない種類のときと、型と噛み合わないときに返す。
pub fn block(parts: &Parts, b: &Value) -> Result<String, String> {
    let kind = text_of(b, "kind");
    match kind.as_str() {
        "text" => parts.part("text", &[("body", esc(&text_of(b, "body")))]),
        "lede" => parts.part("lede", &[("body", esc(&text_of(b, "body")))]),
        "caveat" => parts.part("caveat", &[("body", esc(&text_of(b, "body")))]),
        "card" => {
            let mut rows = String::new();
            for r in array_of(b, "rows") {
                let note = wrap(parts, "card-row-note", "text", &text_of(r, "note"))?;
                rows.push_str(&parts.part(
                    "card-row",
                    &[("text", esc(&text_of(r, "text"))), ("note", note)],
                )?);
            }
            parts.part(
                "card",
                &[
                    ("label", esc(&text_of(b, "label"))),
                    ("claim", esc(&text_of(b, "claim"))),
                    ("rows", rows),
                ],
            )
        }
        "flow" => {
            let mut rows = String::new();
            for r in array_of(b, "rows") {
                let marked = r.get("mark").and_then(Value::as_bool) == Some(true);
                rows.push_str(&parts.part(
                    if marked { "flow-row-mark" } else { "flow-row" },
                    &[
                        ("title", esc(&text_of(r, "title"))),
                        ("note", esc(&text_of(r, "note"))),
                    ],
                )?);
            }
            parts.part("flow", &[("rows", rows)])
        }
        "stat" => parts.part(
            "stat",
            &[
                ("value", esc(&text_of(b, "value"))),
                (
                    "unit",
                    wrap(parts, "stat-unit", "text", &text_of(b, "unit"))?,
                ),
                ("caption", esc(&text_of(b, "caption"))),
                (
                    "source",
                    wrap(parts, "stat-source", "text", &text_of(b, "source"))?,
                ),
            ],
        ),
        "boxes" => {
            let mut items = String::new();
            for i in array_of(b, "items") {
                items.push_str(&parts.part(
                    "boxes-item",
                    &[
                        ("title", esc(&text_of(i, "title"))),
                        ("note", esc(&text_of(i, "note"))),
                    ],
                )?);
            }
            parts.part("boxes", &[("items", items)])
        }
        "pair" => {
            let side = |text: &str, note: &str| -> Result<String, String> {
                Ok(esc(text) + &wrap(parts, "pair-note", "text", note)?)
            };
            parts.part(
                "pair",
                &[
                    ("left", side(&text_of(b, "left"), &text_of(b, "left_note"))?),
                    (
                        "right",
                        side(&text_of(b, "right"), &text_of(b, "right_note"))?,
                    ),
                    ("link", esc(&text_of(b, "link"))),
                ],
            )
        }
        "recap" => {
            let mut items = String::new();
            for i in array_of(b, "items") {
                items.push_str(&parts.part(
                    "recap-item",
                    &[
                        ("no", esc(&text_of(i, "no"))),
                        ("text", esc(&text_of(i, "text"))),
                    ],
                )?);
            }
            parts.part("recap", &[("items", items)])
        }
        "punch" => parts.part(
            "punch",
            &[
                ("body", esc(&text_of(b, "body"))),
                ("sub", wrap(parts, "punch-sub", "text", &text_of(b, "sub"))?),
            ],
        ),
        // 図は SVG の文字列のまま置く。**この Skill は図を描かない**
        "figure" => parts.part(
            "figure",
            &[
                ("svg", text_of(b, "svg")),
                (
                    "caption",
                    wrap(parts, "figure-caption", "text", &text_of(b, "caption"))?,
                ),
            ],
        ),
        "table" => {
            let head = if array_of(b, "head").is_empty() {
                String::new()
            } else {
                let mut cells = String::new();
                for c in array_of(b, "head") {
                    cells.push_str(&parts.part("table-th", &[("text", esc(&as_text(c)))])?);
                }
                parts.part("table-head", &[("cells", cells)])?
            };
            let mut rows = String::new();
            for r in array_of(b, "rows") {
                let mut cells = String::new();
                for c in r.as_array().map_or(&[][..], |x| x.as_slice()) {
                    cells.push_str(&parts.part("table-td", &[("text", esc(&as_text(c)))])?);
                }
                rows.push_str(&parts.part("table-row", &[("cells", cells)])?);
            }
            parts.part("table", &[("head", head), ("rows", rows)])
        }
        "list" => {
            let mut items = String::new();
            for i in array_of(b, "items") {
                items.push_str(&parts.part("list-item", &[("text", esc(&as_text(i)))])?);
            }
            parts.part("list", &[("items", items)])
        }
        "who" => {
            let mut items = String::new();
            for i in array_of(b, "items") {
                items.push_str(&parts.part("who-item", &[("text", esc(&as_text(i)))])?);
            }
            parts.part("who", &[("items", items)])
        }
        other => Err(format!(
            "知らない要素: {other} ── 使えるのは {:?} である",
            KINDS
        )),
    }
}

/// 扱える要素の種類。**種類を足すときは、型にも部品を足す** ── 片方だけに足すと、
/// 入力が通って出力が空になる。
pub const KINDS: [&str; 14] = [
    "boxes", "card", "caveat", "figure", "flow", "lede", "list", "pair", "punch", "recap", "stat",
    "table", "text", "who",
];

/// 文字でない値も、そのまま文字として出す。
fn as_text(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| value.to_string(), std::borrow::ToOwned::to_owned)
}

fn column(parts: &Parts, c: &Value) -> Result<String, String> {
    let mut body = String::new();
    for b in array_of(c, "blocks") {
        body.push_str(&block(parts, b)?);
    }
    parts.part(
        "column",
        &[
            (
                "role",
                wrap(parts, "column-role", "text", &text_of(c, "role"))?,
            ),
            ("blocks", body),
            (
                "close",
                wrap(parts, "column-close", "text", &text_of(c, "close"))?,
            ),
        ],
    )
}

fn kicker(parts: &Parts, s: &Value, center: bool) -> Result<String, String> {
    let Some(k) = s.get("kicker") else {
        return Ok(String::new());
    };
    let text = text_of(k, "text");
    if text.is_empty() {
        return Ok(String::new());
    }
    let tone = if text_of(k, "tone") == "warm" {
        " warm"
    } else {
        ""
    };
    parts.part(
        if center { "kicker-center" } else { "kicker" },
        &[("text", esc(&text)), ("tone", tone.to_owned())],
    )
}

/// 枚を1つ組む。**並べ方は layout が決める** ── 枚ごとに器を選ばせない。
///
/// # Errors
///
/// 型と噛み合わないときに返す。
pub fn slide(parts: &Parts, s: &Value) -> Result<String, String> {
    let layout = text_of(s, "layout");
    if layout == "cover" {
        return parts.part(
            "slide-cover",
            &[
                ("kicker", kicker(parts, s, true)?),
                ("heading", esc(&text_of(s, "heading"))),
                (
                    "lede",
                    wrap(parts, "cover-lede", "text", &text_of(s, "lede"))?,
                ),
                (
                    "byline",
                    wrap(parts, "byline", "text", &text_of(s, "byline"))?,
                ),
                (
                    "submitted",
                    wrap(parts, "submitted", "text", &text_of(s, "submitted"))?,
                ),
            ],
        );
    }
    let body = if matches!(layout.as_str(), "cols" | "cols-3") {
        let mut columns = String::new();
        for c in array_of(s, "columns") {
            columns.push_str(&column(parts, c)?);
        }
        parts.part(&layout, &[("columns", columns)])?
    } else {
        let mut all = String::new();
        for b in array_of(s, "blocks") {
            all.push_str(&block(parts, b)?);
        }
        parts.part("stack", &[("blocks", all)])?
    };
    let center = layout == "center";
    parts.part(
        if center { "slide-center" } else { "slide" },
        &[
            ("kicker", kicker(parts, s, center)?),
            (
                "heading",
                wrap(parts, "heading", "text", &text_of(s, "heading"))?,
            ),
            ("body", body),
            ("note", wrap(parts, "note", "text", &text_of(s, "note"))?),
        ],
    )
}

/// デッキ1本を組む。**入力が通っていなければ、1バイトも出さない。**
///
/// # Errors
///
/// 入力の検査が通らないときと、型と噛み合わないときに返す。
pub fn build(references: &Path, deck: &Value) -> Result<String, String> {
    let bad = validate::check(references, &schema_path(references), deck);
    if !bad.is_empty() {
        return Err(format!(
            "入力の検査が通っていない ── HTML は書き出さない:\n  {}",
            bad.iter()
                .map(|e| format!("× {e}"))
                .collect::<Vec<_>>()
                .join("\n  ")
        ));
    }
    let parts = Parts::load(&template_path(references))?;
    let colors = theme::read(references, &text_of(deck, "theme"))?;
    let slides = array_of(deck, "slides");
    // **ラベルは0から数える** ── めくる仕掛けが見るのは `LABELS[i]` で、i は0から
    // 始まる。先頭に空を足すと、全部の枚が1つ前のラベルを出す
    let labels = format!(
        "[{}]",
        slides
            .iter()
            .map(|s| {
                let label = s.get("label").and_then(|x| x.as_str()).unwrap_or_default();
                serde_json::to_string(label).unwrap_or_else(|_| "\"\"".to_owned())
            })
            .collect::<Vec<_>>()
            .join(", ")
    );
    let mut body = String::new();
    for (i, s) in slides.iter().enumerate() {
        if i > 0 {
            body.push('\n');
        }
        body.push_str(&slide(&parts, s)?);
    }
    parts.part(
        "page",
        &[
            ("title", esc(&text_of(deck, "title"))),
            ("theme_name", esc(&text_of(deck, "theme"))),
            ("theme", colors.trim_end().to_owned()),
            ("total", slides.len().to_string()),
            ("labels", labels),
            ("slides", body),
        ],
    )
}

/// 入力を読む。
///
/// # Errors
///
/// 読めないときと、JSON として読めないときに返す。
pub fn load(path: &Path) -> Result<Value, String> {
    let body =
        files::read_to_string(path).map_err(|e| format!("{}: 読めない ── {e}", path.display()))?;
    serde_json::from_str(&body)
        .map_err(|e| format!("{}: JSON として読めない ── {e}", path.display()))
}

/// 書き出し先に、既に何かが在るか。**作り直さないための判定に使う。**
#[must_use]
pub fn exists(path: &Path) -> bool {
    files::exists(path)
}

/// 入力を書き出す。**書き出し先のフォルダが無ければ作る。**
///
/// # Errors
///
/// フォルダを作れないときと、書けないときに返す。
pub fn save(path: &Path, body: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            files::create_dir_all(parent)
                .map_err(|e| format!("{}: 作れない ── {e}", parent.display()))?;
        }
    }
    files::write(path, body).map_err(|e| format!("{}: 書けない ── {e}", path.display()))
}

/// 組んだ結果。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Built {
    /// 検査だけのとき、既に在るものと同一か。
    pub same: bool,
    /// 人へ見せる一言。
    pub note: String,
}

/// 入力から1枚を書き出す。`check_only` なら、差が無いかだけを検査する。
///
/// # Errors
///
/// 入力の検査が通らないときと、書けないときに返す。
pub fn build_deck(
    references: &Path,
    source: &Path,
    out: &Path,
    check_only: bool,
) -> Result<Built, String> {
    let body = build(references, &load(source)?)?;
    if check_only {
        let same = files::read_to_string(out).is_ok_and(|now| now == body);
        return Ok(Built {
            same,
            note: if same { "同一" } else { "差が在る" }.to_owned(),
        });
    }
    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            files::create_dir_all(parent)
                .map_err(|e| format!("{}: 作れない ── {e}", parent.display()))?;
        }
    }
    files::write(out, &body).map_err(|e| format!("{}: 書けない ── {e}", out.display()))?;
    Ok(Built {
        same: true,
        note: format!("{} 字", body.chars().count()),
    })
}
