// SPDX-License-Identifier: MIT
//! 装飾系の部品 ── 実務図には要らないが、1枚絵の完成度を上げるためだけの部品。
//!
//! グラデーション ・ 見出しの大きな活字 ・ 区切り ・ 簡単な絵記号は、どれも意味を持つ構造では
//! なく飾りなので、既存の部品とは別の層として独立させる。

use serde_json::Value;

use crate::props::{self, esc};
use crate::py::{self, float as f};
use crate::registry::{Component, Fragment, Props};
use crate::shapes::tone_color;
use crate::style::Style;

/// 波の1周期を4つに割る ── 上り ・ 頂点 ・ 下り ・ 谷という波の形そのもの。
const WAVE_QUARTER: f64 = 4.0;

/// 台帳へ載せる部品。
#[must_use]
pub fn register() -> Vec<(&'static str, Component)> {
    vec![
        ("gradient_rect", gradient_rect as Component),
        ("title", title),
        ("divider", divider),
        ("icon", icon),
    ]
}

/// グラデーションで塗った矩形。**背景や強調帯に使う。**
fn gradient_rect(p: &Props, style: &Style) -> Result<Fragment, String> {
    let wv = p.get("width").ok_or_else(|| py::quote("width"))?;
    let hv = p.get("height").ok_or_else(|| py::quote("height"))?;
    let (w, h) = (wv.as_f64().unwrap_or(0.0), hv.as_f64().unwrap_or(0.0));
    let given = p.get("stops").filter(|v| props::truthy(v));
    // 既定の刻みは、移す前は組で持っていた ── 識別子の材料として、その書き方で書く
    let (stops, stops_repr): (Vec<(f64, String)>, String) = match given {
        Some(v) => {
            let list: Vec<(f64, String)> = v
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|s| {
                            let pair = s.as_array()?;
                            Some((pair.first()?.as_f64()?, props::text(pair.get(1)?)))
                        })
                        .collect()
                })
                .unwrap_or_default();
            (list, py::repr(v))
        }
        None => {
            let a = style.text("color.accent")?;
            let b = style.text("color.accent-bg")?;
            let repr = format!("[(0.0, {}), (1.0, {})]", py::quote(&a), py::quote(&b));
            (vec![(0.0, a), (1.0, b)], repr)
        }
    };
    let direction = props::text_or(p, "direction", "v");
    let radius = p.get("radius").cloned().unwrap_or(Value::from(0));
    let gid = crate::ids::stable(
        "grad",
        &[
            stops_repr,
            py::quote(&direction),
            py::repr(&radius),
            py::repr(wv),
            py::repr(hv),
        ],
    );
    let stop_svg: String = stops
        .iter()
        .map(|(o, c)| format!("<stop offset=\"{:.0}%\" stop-color=\"{c}\"/>", o * 100.0))
        .collect();
    let defs = if direction == "radial" {
        format!("<radialGradient id=\"{gid}\" cx=\"50%\" cy=\"50%\" r=\"75%\">{stop_svg}</radialGradient>")
    } else {
        let (x2, y2) = if direction == "h" {
            ("100%", "0%")
        } else {
            ("0%", "100%")
        };
        format!("<linearGradient id=\"{gid}\" x1=\"0%\" y1=\"0%\" x2=\"{x2}\" y2=\"{y2}\">{stop_svg}</linearGradient>")
    };
    let svg = format!(
        "<defs>{defs}</defs><rect x=\"0\" y=\"0\" width=\"{w:.1}\" height=\"{h:.1}\" rx=\"{}\" fill=\"url(#{gid})\"/>",
        py::num(&radius)
    );
    Ok(Fragment::own(svg, w, h).ints(wv.is_i64() || wv.is_u64(), hv.is_i64() || hv.is_u64()))
}

/// 大きな見出しの活字。**本文の書体とは別の、表題用の書体を使う。**
fn title(p: &Props, style: &Style) -> Result<Fragment, String> {
    let text = props::need_text(p, "text")?;
    let size = style.num("font.size-display")?;
    let family = style.text_or("font.family-display", &style.text("font.family")?)?;
    let color = style.text_or("color.title", &style.text("color.ink")?)?;
    let align = props::text_or(p, "align", "start");
    let w = match props::num(p, "width") {
        Some(w) => w,
        None => style.num("size.decor-title-w")?,
    };
    let anchor_x = if align == "middle" {
        f(w / 2.0)
    } else {
        "0".to_owned()
    };
    let mut body = vec![format!(
        "<text x=\"{anchor_x}\" y=\"{size:.0}\" text-anchor=\"{align}\" font-family=\"{family}\" font-size=\"{size:.0}\" font-weight=\"{}\" letter-spacing=\"0.01em\" fill=\"{color}\">{}</text>",
        style.text("font.weight-bold")?,
        esc(&text)
    )];
    let mut h = size + size * style.num("font.baseline-ratio")?;
    if props::flag(p, "subtitle") {
        let sub_size = style.num("font.size")?;
        let lead = style.num("chart.gap")?;
        body.push(format!(
            "<text x=\"{anchor_x}\" y=\"{:.0}\" text-anchor=\"{align}\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\">{}</text>",
            size + sub_size + lead,
            style.text("font.family")?,
            f(sub_size),
            style.text("color.ink-soft")?,
            esc(&props::text_or(p, "subtitle", ""))
        ));
        h += sub_size + lead + sub_size * style.num("font.baseline-ratio")?;
    }
    Ok(Fragment::own(format!("<g>{}</g>", body.concat()), w, h))
}

/// 区切り。**飾りの波線 ／ 既定は直線。**
fn divider(p: &Props, style: &Style) -> Result<Fragment, String> {
    let w = props::need_num(p, "width")?;
    let color = style.text_or("color.title", &style.text("color.accent")?)?;
    let rule = f(style.num("size.rule-width")?);
    if props::text_or(p, "kind", "") == "wave" {
        let amp = style.num("size.divider-amp")?;
        let period = style.num("size.divider-period")?;
        let mut pts = Vec::new();
        let mut x = 0.0;
        while x <= w {
            pts.push((x, amp * (x / period * std::f64::consts::PI).sin()));
            x += period / WAVE_QUARTER;
        }
        let d = format!(
            "M{}",
            pts.iter()
                .map(|(x, y)| format!("{x:.1},{:.1}", y + amp))
                .collect::<Vec<_>>()
                .join(" L")
        );
        let svg =
            format!("<path d=\"{d}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{rule}\"/>");
        return Ok(Fragment::own(svg, w, amp * 2.0 + 2.0).ints(props::is_int(p, "width"), false));
    }
    let svg = format!("<line x1=\"0\" y1=\"1\" x2=\"{w:.1}\" y2=\"1\" stroke=\"{color}\" stroke-width=\"{rule}\"/>");
    // 直線の高さは、移す前も整数で書いてある
    Ok(Fragment::own(svg, w, 2.0).ints(props::is_int(p, "width"), true))
}

/// 絵記号の定義。**どれも 0..24 の同じ枠で描く** ── 大きさの意味が絵ごとに違うと、並べたとき
/// に揃わない。`(名前, 面, 円, 線)` の並びであり、**対応表であって、式の中の数ではない。**
type Circle = (f64, f64, f64, Option<&'static str>);
type Pictogram = (
    &'static str,
    Option<&'static str>,
    &'static [Circle],
    &'static [&'static str],
);
const PICTOGRAMS: [Pictogram; 13] = [
    ("arrow-up", None, &[], &["M12,20 L12,4 M5,11 L12,4 L19,11"]),
    ("book", None, &[], &["M2,4 C7,2 10,2 12,4 C14,2 17,2 22,4 L22,20 C17,18 14,18 12,20 C10,18 7,18 2,20 Z M12,4 L12,20"]),
    ("branch", None, &[], &["M2,6 L10,6 L10,18 L22,18 M10,6 L22,6 M10,12 L22,12"]),
    ("calendar", None, &[], &["M3,5 L21,5 L21,21 L3,21 Z M3,10 L21,10 M8,2 L8,7 M16,2 L16,7 M7,14 L11,14 M14,14 L18,14 M7,18 L11,18"]),
    ("chat", None, &[], &["M2,3 L22,3 L22,15 L10,15 L5,20 L5,15 L2,15 Z M6,7 L18,7 M6,11 L14,11"]),
    ("check", None, &[], &["M4,13 L10,19 L20,6"]),
    ("doc", None, &[], &["M5,2 L15,2 L19,6 L19,22 L5,22 Z M15,2 L15,6 L19,6 M8,11 L16,11 M8,15 L16,15 M8,19 L13,19"]),
    ("gear", Some("M12,3 L14,3 L14.6,6 L17,7.4 L19.6,6.4 L20.6,8.2 L18.6,10.2 L18.6,13 L20.6,15 L19.6,16.8 L17,15.8 L14.6,17.2 L14,20 L12,20 L10,20 L9.4,17.2 L7,15.8 L4.4,16.8 L3.4,15 L5.4,13 L5.4,10.2 L3.4,8.2 L4.4,6.4 L7,7.4 L9.4,6 L10,3 Z"), &[], &[]),
    // 人 ── 頭は面を持つ絵記号である。**別の部品にしない** ── 同じ枠 ・ 同じ線の決まりで
    // 描くものを2つの仕組みに分けると、片方だけが直る
    ("person", None, &[(10.67, 5.33, 5.33, Some("fill"))], &["M0,24 L0,21.33 C0,9.33 21.33,9.33 21.33,21.33 L21.33,24"]),
    ("ring", None, &[(12.0, 12.0, 9.0, None)], &[]),
    ("screen", None, &[], &["M2,4 L22,4 L22,17 L2,17 Z M12,17 L12,21 M7,21 L17,21"]),
    ("search", None, &[(10.0, 10.0, 7.0, None)], &["M15,15 L21,21"]),
    ("spark", Some("M12,1 L14.6,9.4 L23,12 L14.6,14.6 L12,23 L9.4,14.6 L1,12 L9.4,9.4 Z"), &[], &[]),
];

/// 描ける絵の名前。**目録がここから引く** ── 一覧を2か所に持たない。
#[must_use]
pub fn icon_names() -> Vec<&'static str> {
    let mut names: Vec<&str> = PICTOGRAMS.iter().map(|(n, ..)| *n).collect();
    names.sort_unstable();
    names
}

/// 絵記号。**線1本ぶんの意匠で、写実ではない。拡大しても線は太らない。**
fn icon(p: &Props, style: &Style) -> Result<Fragment, String> {
    let name = props::text_or(p, "name", "spark");
    let Some((_, fill, circles, paths)) = PICTOGRAMS.iter().find(|(n, ..)| *n == name) else {
        let known: Vec<String> = icon_names().iter().map(|k| py::quote(k)).collect();
        return Err(format!(
            "知らない絵の名前: {name}。使えるのは [{}] である",
            known.join(", ")
        ));
    };
    let size = match props::num(p, "size") {
        Some(s) => s,
        None => style.num("size.decor-icon")?,
    };
    let color = match p.get("tone").filter(|v| props::truthy(v)) {
        Some(t) => tone_color(style, &props::text(t))?,
        None => style.text_or("color.title", &style.text("color.accent")?)?,
    };
    let scale = size / style.num("size.decor-icon")?;
    let width = style.num("size.stroke-width-icon")? / scale;
    let line = format!("stroke=\"{color}\" stroke-width=\"{width:.3}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"");
    let mut body = Vec::new();
    if let Some(d) = fill {
        body.push(format!("<path d=\"{d}\" fill=\"{color}\"/>"));
    }
    for (cx, cy, r, fill_tone) in *circles {
        let fill = match fill_tone {
            Some(t) => tone_color(style, t)?,
            None => "none".to_owned(),
        };
        body.push(format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{fill}\" {line}/>",
            f(*cx),
            f(*cy),
            f(*r)
        ));
    }
    for d in *paths {
        body.push(format!("<path d=\"{d}\" fill=\"none\" {line}/>"));
    }
    let svg = format!("<g transform=\"scale({scale:.3})\">{}</g>", body.concat());
    Ok(Fragment::own(svg, size, size))
}
