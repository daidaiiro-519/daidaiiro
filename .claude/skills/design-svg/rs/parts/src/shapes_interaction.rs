// SPDX-License-Identifier: MIT
//! やり取り専用の部品 ── 参加者ごとの縦のライフラインと、層ごとに行き来する横向きの
//! メッセージ。
//!
//! 「順序」や「つながり」の汎用グラフでは、『誰と誰の間か』という軸を表現できない ── やり
//! 取りだけがこの軸を持つので、専用の部品として起こす。
//!
//! **分かれ（cases）を持つ囲みは、ケースの見出しぶんの高さを行として確保してから層を並べる**
//! ── 見出しをメッセージの行へ後から重ね書きすると、文字と線が衝突した（実測）。

use std::collections::BTreeMap;

use serde_json::Value;

use crate::props::{self, esc};
use crate::py::float as f;
use crate::registry::{Component, Fragment, Props};
use crate::shapes::arrow_head;
use crate::style::Style;
use crate::text;

/// 台帳へ載せる部品。
#[must_use]
pub fn register() -> Vec<(&'static str, Component)> {
    vec![("exchange", exchange as Component)]
}

fn span_of(v: &Value) -> Option<(i64, i64)> {
    let a = v.get("span")?.as_array()?;
    Some((a.first()?.as_i64()?, a.get(1)?.as_i64()?))
}

/// 参加者の縦のライフラインと、層ごとのメッセージ。
#[allow(clippy::too_many_lines)]
fn exchange(p: &Props, style: &Style) -> Result<Fragment, String> {
    let who: Vec<String> = props::list(p, "participants")
        .iter()
        .map(props::text)
        .collect();
    let steps = props::list(p, "steps");
    let groups = props::list(p, "groups");

    let pad = style.num("chart.pad")?;
    let colw = style.num("chart.exchange-col-w")?;
    let head_h = style.num("chart.exchange-head-h")?;
    let row_h = style.num("chart.exchange-row-h")?;
    let case_head_h = style.num("chart.exchange-case-head-h")?;
    // 参加者の箱の幅は、入る名前から決める（決め打ちだと長い名前がはみ出す）
    let box_h = style.num("chart.exchange-box-h")?;
    let fs = style.num("font.size")?;
    let fs_small = style.num("font.size-small")?;
    let box_w = text::column(&who, fs, style.num("chart.gap")? * 2.0);
    let w = pad * 2.0 + colw * who.len() as f64;

    // 層ごとの縦位置を、分かれの見出し行ぶんも織り込んで先に確定させる
    let mut case_header_at: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for g in groups {
        for c in g
            .get("cases")
            .and_then(|x| x.as_array())
            .map_or(&[][..], |x| x.as_slice())
        {
            let (s, _) = span_of(c).ok_or("span が無い")?;
            case_header_at
                .entry(s)
                .or_default()
                .push(c.get("name").map(props::text).unwrap_or_default());
        }
    }
    let mut row_top: BTreeMap<i64, f64> = BTreeMap::new();
    let mut header_top: BTreeMap<i64, Vec<(f64, String)>> = BTreeMap::new();
    let mut cursor = pad + head_h;
    for idx in 0..steps.len() as i64 {
        if let Some(names) = case_header_at.get(&idx) {
            let mut slots = Vec::new();
            for name in names {
                slots.push((cursor, name.clone()));
                cursor += case_head_h;
            }
            header_top.insert(idx, slots);
        }
        row_top.insert(idx, cursor);
        cursor += row_h;
    }
    let h = cursor + pad / 2.0;
    let x_of = |i: usize| pad + colw * i as f64 + colw / 2.0;

    let mut body = Vec::new();
    let family = style.text("font.family")?;
    let tab_h = style.num("chart.exchange-tab-h")?;
    let base = style.num("font.baseline-ratio")?;
    let gap = style.num("chart.gap")?;

    // 囲み ── ライフラインより手前、メッセージより奥に描く
    for g in groups {
        let cases = g
            .get("cases")
            .and_then(|x| x.as_array())
            .filter(|x| !x.is_empty());
        let spans: Vec<(i64, i64)> = match cases {
            Some(cs) => cs.iter().filter_map(span_of).collect(),
            None => vec![span_of(g).ok_or("span が無い")?],
        };
        let s = spans.iter().map(|x| x.0).min().unwrap_or(0);
        let e = spans.iter().map(|x| x.1).max().unwrap_or(0);
        let top = match header_top.get(&s) {
            Some(slots) => slots[0].0 - tab_h / 2.0,
            None => row_top.get(&s).copied().ok_or("層が無い")? - tab_h,
        };
        let bottom = row_top.get(&e).copied().ok_or("層が無い")? + row_h - gap;
        body.push(format!(
            "<rect x=\"{}\" y=\"{top:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"{}\" fill=\"none\" stroke=\"{}\"/>",
            f(pad),
            w - pad * 2.0,
            bottom - top,
            f(style.num("size.radius-small")?),
            style.text("color.accent")?
        ));
        let label = g.get("label").map(props::text).unwrap_or_default();
        let tab_w = (text::width(&label, fs_small - 1.0) + fs_small).min(w - pad * 2.0);
        body.push(format!(
            "<rect x=\"{}\" y=\"{top:.1}\" width=\"{tab_w:.1}\" height=\"{tab_h:.1}\" fill=\"{}\"/>",
            f(pad),
            style.text("color.accent-bg")?
        ));
        body.push(format!(
            "<text x=\"{}\" y=\"{:.1}\" font-family=\"{family}\" font-size=\"{}\" fill=\"{}\">{}</text>",
            f(pad + gap / 2.0),
            top + tab_h / 2.0 + fs_small * base,
            f(fs_small - 1.0),
            style.text("color.accent")?,
            esc(&label)
        ));
        if let Some(cs) = cases {
            for (i, c) in cs.iter().enumerate() {
                let name = c.get("name").map(props::text).unwrap_or_default();
                let (s0, _) = span_of(c).ok_or("span が無い")?;
                let head_y = header_top
                    .get(&s0)
                    .and_then(|slots| slots.iter().find(|(_, n)| *n == name))
                    .map(|(y, _)| *y)
                    .ok_or("見出しが無い")?;
                if i > 0 {
                    body.push(format!(
                        "<line x1=\"{}\" y1=\"{head_y:.1}\" x2=\"{}\" y2=\"{head_y:.1}\" stroke=\"{}\" stroke-dasharray=\"5 4\"/>",
                        f(pad),
                        f(w - pad),
                        style.text("color.accent")?
                    ));
                }
                body.push(format!(
                    "<text x=\"{}\" y=\"{:.1}\" font-weight=\"{}\" font-family=\"{family}\" font-size=\"{}\" fill=\"{}\">{}</text>",
                    f(pad + fs_small),
                    head_y + case_head_h / 2.0 + fs_small * base,
                    style.text("font.weight-medium")?,
                    f(fs),
                    style.text("color.ink")?,
                    esc(&name)
                ));
            }
        }
    }

    // 参加者の箱とライフライン ── メッセージの下敷きになるので、後に描く
    for i in 0..who.len() {
        let cx = x_of(i);
        body.push(format!(
            "<line x1=\"{cx:.1}\" y1=\"{:.1}\" x2=\"{cx:.1}\" y2=\"{:.1}\" stroke=\"{}\" stroke-dasharray=\"3 4\"/>",
            pad + box_h,
            h - pad / 2.0,
            style.text("color.box-stroke")?
        ));
    }
    for (i, name) in who.iter().enumerate() {
        let cx = x_of(i);
        body.push(format!(
            "<rect x=\"{:.1}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{}\" stroke=\"{}\"/>",
            cx - box_w / 2.0,
            f(pad),
            f(box_w),
            f(box_h),
            f(style.num("size.radius")?),
            style.text("color.box-fill")?,
            style.text("color.box-stroke")?
        ));
        body.push(format!(
            "<text x=\"{cx:.1}\" y=\"{:.1}\" text-anchor=\"middle\" font-family=\"{family}\" font-size=\"{}\" fill=\"{}\">{}</text>",
            pad + box_h / 2.0 + fs_small * base,
            f(fs),
            style.text("color.ink")?,
            esc(name)
        ));
    }

    // メッセージ ── 一番手前
    for (r, s) in steps.iter().enumerate() {
        let y = row_top.get(&(r as i64)).copied().ok_or("層が無い")? + row_h / 2.0;
        let from = s.get("from").map(props::text).unwrap_or_default();
        let to = s.get("to").map(props::text).unwrap_or_default();
        let a = who
            .iter()
            .position(|x| *x == from)
            .ok_or_else(|| format!("{} is not in list", crate::py::quote(&from)))?;
        let b = who
            .iter()
            .position(|x| *x == to)
            .ok_or_else(|| format!("{} is not in list", crate::py::quote(&to)))?;
        let (xa, xb) = (x_of(a), x_of(b));
        let dashed = s.get("kind").and_then(|x| x.as_str()) == Some("return");
        let dash_attr = if dashed {
            " stroke-dasharray=\"4 3\""
        } else {
            ""
        };
        body.push(format!(
            "<line x1=\"{xa:.1}\" y1=\"{y:.1}\" x2=\"{xb:.1}\" y2=\"{y:.1}\" stroke=\"{}\" stroke-width=\"{}\"{dash_attr}/>",
            style.text("color.ink-faint")?,
            f(style.num("size.stroke-width")?)
        ));
        // 矢じりの底辺は、先端より「来た方」へ置く（実測 ── 進む方へ置くと三角が逆を向いた）
        let back = if xb > xa { -1 } else { 1 };
        let ink = style.text("color.ink-faint")?;
        body.push(arrow_head(
            (xb, y),
            if back > 0 { std::f64::consts::PI } else { 0.0 },
            style,
            Some(&ink),
            "solid",
        )?);
        if s.get("label").is_some_and(props::truthy) {
            let mx = (xa + xb) / 2.0;
            body.push(format!(
                "<text x=\"{mx:.1}\" y=\"{:.1}\" text-anchor=\"middle\" font-family=\"{family}\" font-size=\"{}\" fill=\"{}\">{}</text>",
                y - gap / 2.0,
                f(fs_small),
                style.text("color.ink-soft")?,
                esc(&s.get("label").map(props::text).unwrap_or_default())
            ));
        }
    }
    Ok(Fragment::own(format!("<g>{}</g>", body.concat()), w, h))
}
