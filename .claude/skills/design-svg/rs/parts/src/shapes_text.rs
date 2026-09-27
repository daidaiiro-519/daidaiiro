// SPDX-License-Identifier: MIT
//! 素の文字 ── 箱にもラベルにも属さない、任意の位置へ置く1つ。
//!
//! **この部品が無いと、注記も、見出しの添えも、図の中の一言も置けない。** 入力は中身と構造
//! だけを持ち、色も寸法もトークンから引く。

use serde_json::Value;

use crate::props::{self, esc};
use crate::registry::{Component, Fragment, Props};
use crate::style::Style;
use crate::text;

/// 太さ ── 役割の名前からトークンへ。
const WEIGHT: [(&str, &str); 3] = [
    ("normal", "font.weight-normal"),
    ("medium", "font.weight-medium"),
    ("bold", "font.weight-bold"),
];

/// 台帳へ載せる部品。
#[must_use]
pub fn register() -> Vec<(&'static str, Component)> {
    vec![("text", text_ as Component)]
}

/// 1行でも、行の並びでもよい。**呼ぶ側に形を揃えさせない。**
fn lines(v: &Value) -> Vec<String> {
    match v {
        Value::Array(items) => items.iter().map(props::text).collect(),
        other => vec![props::text(other)],
    }
}

/// 任意の位置へ置く文字。
///
/// **インクは申告した大きさの中に収める。** 揃え方を変えても外接矩形は動かない ── 動かすと、
/// 置いた側が知らないところで重なりが起きる。
fn text_(p: &Props, style: &Style) -> Result<Fragment, String> {
    let rows = lines(p.get("text").ok_or_else(|| crate::py::quote("text"))?);
    let size = match props::num(p, "size") {
        Some(s) => s,
        None => style.num("font.size")?,
    };
    let tone = props::text_or(p, "tone", "ink");
    let tone_key = crate::theme::tone(&tone).ok_or_else(|| crate::py::quote(&tone))?;
    let color = style.text_or(tone_key, &style.text("color.ink")?)?;
    let wname = props::text_or(p, "weight", "normal");
    let weight_key = WEIGHT
        .iter()
        .find(|(k, _)| *k == wname)
        .map(|(_, v)| *v)
        .ok_or_else(|| crate::py::quote(&wname))?;
    let weight = style.text(weight_key)?;
    let align = props::text_or(p, "align", "start");
    let family = style.text("font.family")?;
    let line_h = size * style.num("size.label-line-h")?;
    let w = rows
        .iter()
        .map(|s| text::width(s, size))
        .fold(f64::NEG_INFINITY, f64::max);
    let anchor_x = match align.as_str() {
        "start" => 0.0,
        "middle" => w / 2.0,
        "end" => w,
        other => return Err(crate::py::quote(other)),
    };
    let cap = size * style.num("font.cap-ratio")?;
    let body: String = rows
        .iter()
        .enumerate()
        .map(|(i, s)| {
            format!(
                "<text x=\"{anchor_x:.1}\" y=\"{:.1}\" text-anchor=\"{align}\" font-family=\"{family}\" font-size=\"{size:.1}\" font-weight=\"{weight}\" fill=\"{color}\">{}</text>",
                cap + i as f64 * line_h,
                esc(s)
            )
        })
        .collect();
    let h = cap + (rows.len() as f64 - 1.0) * line_h + size * style.num("font.descender-ratio")?;
    Ok(Fragment::own(format!("<g>{body}</g>"), w, h))
}
