// SPDX-License-Identifier: MIT
//! 量を描く部品 ── **値から座標が一意に決まる図。**
//!
//! 内訳 ・ 大小 ・ 並び順 ・ 区間 ・ 分布 ・ 基準からのずれ ・ 2軸上の点 ・ 流れる量 ・ 位置を、
//! それぞれ1つの形で描く。どれも値さえ決まれば座標が決まるので、配置の解決は要らない。
//!
//! どの図がどんな言い分を運ぶかは、ここでは決めない ── それは呼ぶ側が決めることである。

use std::collections::BTreeSet;

use serde_json::Value;

use crate::labels::place_avoiding;
use crate::props::{self, esc};
use crate::py::{float as f, sum, sum_values};
use crate::registry::{self, Component, Fragment, Props};
use crate::style::Style;
use crate::text;

/// 台帳へ載せる部品。
#[must_use]
pub fn register() -> Vec<(&'static str, Component)> {
    vec![
        ("donut", donut as Component),
        ("pie", pie),
        ("bars", bars),
        ("ranking", ranking),
        ("lanes", lanes),
        ("scatter", scatter),
        ("flow", flow),
        ("spatial", spatial),
    ]
}

fn tone(style: &Style, i: usize) -> Result<String, String> {
    let tones = style.tones("chart.tones")?;
    Ok(tones[i % tones.len()].clone())
}

/// 文字を1つ置く。
#[allow(clippy::too_many_arguments)]
fn t(
    x: f64,
    y: f64,
    s: &str,
    style: &Style,
    color: &str,
    anchor: &str,
    size: Option<f64>,
    weight: &str,
) -> Result<String, String> {
    let small = style.num("font.size-small")?;
    let fs = match size {
        Some(v) if v != 0.0 => v,
        _ => small,
    };
    let class = crate::classes::text_class(color, (fs - small).abs() < f64::EPSILON);
    Ok(format!(
        "<text class=\"{class}\" x=\"{x:.1}\" y=\"{y:.1}\" text-anchor=\"{anchor}\" font-family=\"{}\" font-size=\"{}\" font-weight=\"{weight}\" fill=\"{}\">{}</text>",
        style.text("font.family")?,
        f(fs),
        style.text(color)?,
        esc(s)
    ))
}

fn items<'a>(p: &'a Props, key: &str) -> Result<&'a [Value], String> {
    p.get(key)
        .and_then(|x| x.as_array())
        .map(Vec::as_slice)
        .ok_or_else(|| crate::py::quote(key))
}

fn field<'a>(v: &'a Value, key: &str) -> Result<&'a Value, String> {
    v.get(key).ok_or_else(|| crate::py::quote(key))
}

fn fnum(v: &Value, key: &str) -> Result<f64, String> {
    field(v, key)?
        .as_f64()
        .ok_or_else(|| format!("'{key}' は数でなければならない"))
}

fn ftext(v: &Value, key: &str) -> Result<String, String> {
    Ok(props::text(field(v, key)?))
}

/// 割合の輪だけを描く部品。**凡例も余白も持たない** ── 部品であって図ではない。
fn donut(p: &Props, style: &Style) -> Result<Fragment, String> {
    let slices = items(p, "slices")?;
    let values: Vec<Value> = slices
        .iter()
        .map(|s| field(s, "value").cloned())
        .collect::<Result<_, _>>()?;
    let total = sum_values(&values);
    let total = if total == 0.0 { 1.0 } else { total };
    let r = style.num("chart.pie-radius")?;
    let thickness = style.num("chart.pie-donut-thickness")?;
    if thickness >= r {
        return Err(format!(
            "chart.pie-donut-thickness は chart.pie-radius より小さくすること(thickness={}, radius={})",
            f(thickness),
            f(r)
        ));
    }
    let size = (r + thickness / 2.0) * 2.0;
    let (cx, cy) = (size / 2.0, size / 2.0);
    // **輪は、太い線ではなく塗りつぶした円弧で描く** ── 線の太さはストローク幅の段階に収まらない
    // （ブレストボード design-svg-rework の論点3の追加1-1）
    let (outer, inner) = (r + thickness / 2.0, r - thickness / 2.0);
    let sector = |a0: f64, a1: f64| -> String {
        let pt = |a: f64, rad: f64| {
            let t = (a - 90.0).to_radians();
            (cx + rad * t.cos(), cy + rad * t.sin())
        };
        let (ox0, oy0) = pt(a0, outer);
        let (ox1, oy1) = pt(a1, outer);
        let (ix1, iy1) = pt(a1, inner);
        let (ix0, iy0) = pt(a0, inner);
        let large = i32::from(a1 - a0 > 180.0);
        format!(
            "M{ox0:.1},{oy0:.1} A{outer:.1},{outer:.1} 0 {large},1 {ox1:.1},{oy1:.1} L{ix1:.1},{iy1:.1} A{inner:.1},{inner:.1} 0 {large},0 {ix0:.1},{iy0:.1} Z"
        )
    };
    let mut body = Vec::new();
    let mut a = 0.0;
    for (i, s) in slices.iter().enumerate() {
        let a1 = a + fnum(s, "value")? / total * 360.0;
        // 1周ぶんの円弧は始点と終点が重なって描けないので、半分ずつに分ける
        let halves = if a1 - a >= 360.0 {
            vec![(a, a + 180.0), (a + 180.0, a1)]
        } else {
            vec![(a, a1)]
        };
        for (h0, h1) in halves {
            body.push(format!(
                "<path class=\"{}\" d=\"{}\" fill=\"{}\"/>",
                crate::classes::series(i),
                sector(h0, h1),
                tone(style, i)?,
            ));
        }
        a = a1;
    }
    if props::flag(p, "centre") {
        let fs = style.num("font.size")?;
        body.push(t(
            cx,
            cy + fs * style.num("font.baseline-ratio")?,
            &props::text_or(p, "centre", ""),
            style,
            "color.ink",
            "middle",
            Some(fs),
            "600",
        )?);
    }
    Ok(Fragment::own(
        format!("<g>{}</g>", body.concat()),
        size,
        size,
    ))
}

/// 輪と凡例を並べた図。**輪そのものは donut が描く。**
fn pie(p: &Props, style: &Style) -> Result<Fragment, String> {
    let ring = registry::render_node("donut", p, style)?;
    if !p.get("legend").is_none_or(props::truthy) {
        return Ok(ring);
    }
    let slices = items(p, "slices")?;
    let pad = style.num("chart.pad")?;
    let gap = style.num("chart.gap")?;
    let fs_small = style.num("font.size-small")?;
    let cap = style.num("font.cap-ratio")?;
    let row_h = style.num("chart.legend-row-h")?;
    // **凡例の幅は中身から決める** ── 決め打ちにすると、値だけが遠くへ取り残される（実測）
    let swatch = fs_small;
    let names: Vec<String> = slices
        .iter()
        .map(|s| ftext(s, "name"))
        .collect::<Result<_, _>>()?;
    let vals: Vec<String> = slices
        .iter()
        .map(|s| ftext(s, "value"))
        .collect::<Result<_, _>>()?;
    let name_w = text::column(&names, fs_small, gap);
    let value_w = text::column(&vals, fs_small, 0.0);
    let legend_w = swatch + gap / 2.0 + name_w + value_w;
    let w = pad * 2.0 + ring.width + gap + legend_w;
    let h = pad * 2.0 + ring.height.max(row_h * slices.len() as f64);
    let mut body = vec![format!(
        "<g transform=\"translate({},{:.1})\">{}</g>",
        f(pad),
        (h - ring.height) / 2.0,
        ring.svg
    )];
    let lx = pad + ring.width + gap;
    for (i, _) in slices.iter().enumerate() {
        let y = pad + row_h * i as f64 + (row_h + fs_small * cap) / 2.0;
        let sw = fs_small;
        body.push(format!(
            "<rect class=\"{}\" x=\"{}\" y=\"{:.1}\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
            crate::classes::series(i),
            f(lx),
            y - sw * cap - (sw - sw * cap) / 2.0,
            f(sw),
            f(sw),
            tone(style, i)?
        ));
        body.push(t(
            lx + sw + gap / 2.0,
            y,
            &names[i],
            style,
            "color.ink-soft",
            "start",
            None,
            "400",
        )?);
        body.push(t(
            w - pad,
            y,
            &vals[i],
            style,
            "color.ink",
            "end",
            None,
            "400",
        )?);
    }
    Ok(Fragment::own(format!("<g>{}</g>", body.concat()), w, h))
}

/// 縦棒。**基準を与えると、そこからの正負の差として伸びる（偏差）。**
///
/// 軸を2本持てるのは、「量の大小」が読む枠に軸を2本要求するため ── 仕様が必須と定めた欄を
/// 欠落させない。
#[allow(clippy::too_many_lines)]
fn bars(p: &Props, style: &Style) -> Result<Fragment, String> {
    let list = items(p, "bars")?;
    let baseline = props::num(p, "baseline").unwrap_or(0.0);
    let values: Vec<f64> = list
        .iter()
        .map(|it| Ok(fnum(it, "value")? - baseline))
        .collect::<Result<_, String>>()?;
    let pad = style.num("chart.pad")?;
    let bw = style.num("chart.bar-width")?;
    let bar_gap = style.num("chart.bar-gap")?;
    let names: Vec<String> = list
        .iter()
        .map(|it| ftext(it, "name"))
        .collect::<Result<_, _>>()?;
    // **1本ぶんの持ち場は、棒の幅と、その下に付く項目名の広いほう**（実測 ── 名前が棒より
    // 長いときに隣どうしで重なった）
    let slot = (bw + bar_gap).max(text::column(&names, style.num("font.size-small")?, bar_gap));
    let ph = style.num("chart.bar-plot-h")?;
    let left_margin = style.num("chart.bar-left-margin")?;
    let bottom_margin = style.num("chart.bar-bottom-margin")?;
    let fs_small = style.num("font.size-small")?;
    let cap_ratio = style.num("font.cap-ratio")?;
    // 軸ラベルは、棒の上に出る値と同じ高さの帯を奪い合う ── ラベル専用の帯を確保する
    let plot_top = if props::flag(p, "axis_label") {
        let label_baseline = pad + fs_small;
        let gap = style.num("chart.gap")?;
        let value_offset = gap / 2.0;
        let cap = fs_small * cap_ratio;
        let breathing = gap / 2.0;
        label_baseline + breathing + value_offset + cap
    } else {
        pad
    };
    let gap = style.num("chart.gap")?;
    let w = pad * 2.0 + left_margin + list.len() as f64 * slot;
    let lowest = values.iter().copied().fold(f64::INFINITY, f64::min);
    // **負に伸びる棒は、値のラベルを作図領域の下へ出す** ── 帯を先に確保してから名前を置く
    let value_band = if lowest < 0.0 {
        gap / 2.0 + fs_small * (cap_ratio + style.num("font.descender-ratio")?)
    } else {
        0.0
    };
    let item_axis_band = if props::flag(p, "item_axis_label") {
        fs_small * style.num("size.label-line-h")?
    } else {
        0.0
    };
    let h = plot_top + ph + value_band + item_axis_band + bottom_margin + pad;
    let x0 = pad + left_margin;
    let zero_y = if lowest < 0.0 {
        plot_top + ph / 2.0
    } else {
        plot_top + ph
    };
    let top = values
        .iter()
        .map(|v| v.abs())
        .fold(f64::NEG_INFINITY, f64::max);
    let top = if top == 0.0 { 1.0 } else { top };
    let scale = (if lowest < 0.0 { ph / 2.0 } else { ph }) / top;
    let axis = style.text("chart.axis")?;
    let mut body = vec![
        format!(
            "<line class=\"grid\" x1=\"{}\" y1=\"{plot_top:.1}\" x2=\"{}\" y2=\"{:.1}\" stroke=\"{axis}\"/>",
            f(x0),
            f(x0),
            plot_top + ph
        ),
        format!(
            "<line class=\"grid\" x1=\"{}\" y1=\"{zero_y:.1}\" x2=\"{}\" y2=\"{zero_y:.1}\" stroke=\"{axis}\"/>",
            f(x0),
            f(w - pad)
        ),
    ];
    if props::flag(p, "axis_label") {
        body.push(t(
            x0,
            pad + fs_small,
            &props::text_or(p, "axis_label", ""),
            style,
            "color.ink-faint",
            "start",
            None,
            "400",
        )?);
    }
    for (i, it) in list.iter().enumerate() {
        let v = values[i];
        let bx = x0 + i as f64 * slot + (slot - bw) / 2.0;
        let bh = v.abs() * scale;
        let by = if v >= 0.0 { zero_y - bh } else { zero_y };
        body.push(format!(
            "<rect class=\"{}\" x=\"{bx:.1}\" y=\"{by:.1}\" width=\"{}\" height=\"{bh:.1}\" fill=\"{}\"/>",
            crate::classes::series(i),
            f(bw),
            tone(style, i)?
        ));
        let num_y = if v >= 0.0 {
            by - gap / 2.0
        } else {
            by + bh + gap / 2.0 + fs_small * cap_ratio
        };
        body.push(t(
            bx + bw / 2.0,
            num_y,
            &ftext(it, "value")?,
            style,
            "color.ink",
            "middle",
            Some(fs_small),
            "400",
        )?);
        body.push(t(
            bx + bw / 2.0,
            plot_top + ph + value_band + gap + fs_small * cap_ratio,
            &names[i],
            style,
            "color.ink-faint",
            "middle",
            None,
            "400",
        )?);
    }
    if props::flag(p, "item_axis_label") {
        // 項目の軸の名前は、項目名の行のさらに下 ── **行の高さぶん下げる**
        let name_row = plot_top + ph + value_band + gap + fs_small * cap_ratio;
        body.push(t(
            x0 + (w - pad - x0) / 2.0,
            name_row + fs_small * style.num("size.label-line-h")?,
            &props::text_or(p, "item_axis_label", ""),
            style,
            "color.ink-faint",
            "middle",
            None,
            "400",
        )?);
    }
    Ok(Fragment::own(format!("<g>{}</g>", body.concat()), w, h))
}

/// 並びの順をそのまま順位とする、横棒の一覧。**読ませたいのは順番で、値は添え物。**
fn ranking(p: &Props, style: &Style) -> Result<Fragment, String> {
    let list = items(p, "items")?;
    let values: Vec<f64> = list
        .iter()
        .map(|it| fnum(it, "value"))
        .collect::<Result<_, _>>()?;
    let top = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let top = if top == 0.0 { 1.0 } else { top };
    let pad = style.num("chart.pad")?;
    let row_h = style.num("chart.rank-row-h")?;
    let bar_w = style.num("chart.rank-bar-w")?;
    let gap0 = style.num("chart.gap")?;
    let fs0 = style.num("font.size-small")?;
    // 名前 ・ 順位 ・ 値の欄は、それぞれ実際に入る文字から決める
    let names: Vec<String> = list
        .iter()
        .map(|it| ftext(it, "name"))
        .collect::<Result<_, _>>()?;
    let vals: Vec<String> = list
        .iter()
        .map(|it| ftext(it, "value"))
        .collect::<Result<_, _>>()?;
    let ranks: Vec<String> = (1..=list.len()).map(|i| i.to_string()).collect();
    let name_w = text::column(&names, fs0, gap0);
    let left_margin = text::column(&ranks, fs0, gap0 * 2.0);
    let value_w = text::column(&vals, fs0, gap0 * 2.0);
    let w = pad * 2.0 + left_margin + name_w + bar_w + value_w;
    let h = pad * 2.0 + row_h * list.len() as f64;
    let gap = style.num("chart.gap")?;
    let base = style.num("font.baseline-ratio")?;
    let fs_small = style.num("font.size-small")?;
    let track_h = style.num("chart.rank-track-h")?;
    let radius = f(style.num("size.radius-small")?);
    let mut body = Vec::new();
    for (i, v) in values.iter().enumerate() {
        let y = pad + row_h * i as f64;
        body.push(t(
            pad + left_margin - gap,
            y + row_h / 2.0 + fs_small * base,
            &ranks[i],
            style,
            "color.ink-faint",
            "end",
            None,
            "400",
        )?);
        body.push(t(
            pad + left_margin + name_w,
            y + row_h / 2.0 + fs_small * base,
            &names[i],
            style,
            "color.ink-soft",
            "end",
            None,
            "400",
        )?);
        let track_x = pad + left_margin + name_w + gap;
        let track_y = y + (row_h - track_h) / 2.0;
        body.push(format!(
            "<rect class=\"box\" x=\"{}\" y=\"{track_y:.1}\" width=\"{}\" height=\"{track_h:.1}\" rx=\"{radius}\" fill=\"{}\"/>",
            f(track_x),
            f(bar_w),
            style.text("chart.grid")?
        ));
        let fill_w = bar_w * v / top;
        body.push(format!(
            "<rect class=\"{}\" x=\"{}\" y=\"{track_y:.1}\" width=\"{fill_w:.1}\" height=\"{track_h:.1}\" fill=\"{}\"/>",
            crate::classes::series(0),
            f(track_x),
            tone(style, 0)?
        ));
        body.push(t(
            track_x + bar_w + value_w - gap,
            y + row_h / 2.0 + fs_small * base,
            &vals[i],
            style,
            "color.ink",
            "end",
            None,
            "400",
        )?);
    }
    Ok(Fragment::own(format!("<g>{}</g>", body.concat()), w, h))
}

/// 行ごとの区間を、帯として並べる。**列（時間の軸）は共有する。**
fn lanes(p: &Props, style: &Style) -> Result<Fragment, String> {
    let rows = items(p, "rows")?;
    // 区間の上限は宣言が持つ。**無ければ実際の最大値から決める**（既定値を置かない）
    let span = match p
        .get("span")
        .filter(|v| props::truthy(v))
        .and_then(Value::as_f64)
    {
        Some(s) => s,
        None => {
            let mut hi: Option<f64> = None;
            for r in rows {
                for b in field(r, "bars")?
                    .as_array()
                    .map_or(&[][..], |x| x.as_slice())
                {
                    let to = fnum(b, "to")?;
                    hi = Some(hi.map_or(to, |h| h.max(to)));
                }
            }
            hi.unwrap_or(1.0)
        }
    };
    let pad = style.num("chart.pad")?;
    let tw = style.num("chart.lane-track-w")?;
    let rh = style.num("chart.lane-row-h")?;
    let fs_small = style.num("font.size-small")?;
    let base = style.num("font.baseline-ratio")?;
    let gap = style.num("chart.gap")?;
    let inset = style.num("chart.lane-bar-inset")?;
    let axis_h = style.num("chart.lane-axis-h")?;
    let names: Vec<String> = rows
        .iter()
        .map(|r| ftext(r, "name"))
        .collect::<Result<_, _>>()?;
    let lw = text::column(&names, fs_small, gap * 2.0);
    let w = pad * 2.0 + lw + tw;
    let h = pad * 2.0
        + rh * rows.len() as f64
        + if props::flag(p, "axis_label") {
            axis_h
        } else {
            0.0
        };
    let radius = f(style.num("size.radius-small")?);
    let mut body = Vec::new();
    for (i, r) in rows.iter().enumerate() {
        let y = pad + rh * i as f64;
        body.push(t(
            pad + lw - gap,
            y + rh / 2.0 + fs_small * base,
            &names[i],
            style,
            "color.ink-soft",
            "end",
            None,
            "400",
        )?);
        body.push(format!(
            "<rect class=\"box\" x=\"{}\" y=\"{:.1}\" width=\"{}\" height=\"{:.1}\" rx=\"{radius}\" fill=\"{}\"/>",
            f(pad + lw),
            y + inset,
            f(tw),
            rh - inset * 2.0,
            style.text("chart.grid")?
        ));
        for (j, bar) in field(r, "bars")?
            .as_array()
            .map_or(&[][..], |x| x.as_slice())
            .iter()
            .enumerate()
        {
            let from = fnum(bar, "from")?;
            let to = fnum(bar, "to")?;
            let bx = pad + lw + tw * from / span;
            let bwid = tw * (to - from) / span;
            body.push(format!(
                "<rect class=\"{}\" x=\"{bx:.1}\" y=\"{:.1}\" width=\"{bwid:.1}\" height=\"{:.1}\" fill=\"{}\"/>",
                crate::classes::series(j),
                y + inset,
                rh - inset * 2.0,
                tone(style, j)?
            ));
            if bar.get("label").is_some_and(props::truthy) {
                body.push(t(
                    bx + bwid / 2.0,
                    y + rh / 2.0 + fs_small * base,
                    &ftext(bar, "label")?,
                    style,
                    "color.ink",
                    "middle",
                    Some(fs_small),
                    "400",
                )?);
            }
        }
    }
    if props::flag(p, "axis_label") {
        let ay = pad + rh * rows.len() as f64 + gap / 2.0;
        body.push(format!(
            "<line class=\"grid\" x1=\"{}\" y1=\"{ay:.1}\" x2=\"{}\" y2=\"{ay:.1}\" stroke=\"{}\"/>",
            f(pad + lw),
            f(pad + lw + tw),
            style.text("chart.axis")?
        ));
        body.push(t(
            pad + lw,
            ay + gap / 2.0 + fs_small,
            &props::text_or(p, "axis_label", ""),
            style,
            "color.ink-faint",
            "start",
            None,
            "400",
        )?);
    }
    Ok(Fragment::own(format!("<g>{}</g>", body.concat()), w, h))
}

/// 2つの軸上の座標として点を置く。**点が端に寄っても、ラベルが枠の外へはみ出さないよう、点の
/// 位置に応じて寄せる向きを変える。**
#[allow(clippy::too_many_lines)]
fn scatter(p: &Props, style: &Style) -> Result<Fragment, String> {
    let pts = items(p, "points")?;
    let w = style.num("chart.scatter-w")?;
    let h = style.num("chart.scatter-h")?;
    let x0 = style.num("chart.scatter-margin-left")?;
    let y0 = style.num("chart.scatter-margin-top")?;
    let x1 = w - style.num("chart.scatter-margin-left")?;
    let y1 = h - style.num("chart.scatter-margin-bottom")?;
    let r = style.num("chart.scatter-point-r")?;
    let fs_small = style.num("font.size-small")?;
    let cap = fs_small * style.num("font.cap-ratio")?;
    let gap = style.num("chart.gap")?;
    let xs: Vec<f64> = pts.iter().map(|q| fnum(q, "x")).collect::<Result<_, _>>()?;
    let ys: Vec<f64> = pts.iter().map(|q| fnum(q, "y")).collect::<Result<_, _>>()?;
    let mx = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let my = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mx = if xs.is_empty() || mx == 0.0 { 1.0 } else { mx };
    let my = if ys.is_empty() || my == 0.0 { 1.0 } else { my };
    let axis = style.text("chart.axis")?;
    let mut body = vec![
        format!(
            "<line class=\"grid\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{axis}\"/>",
            f(x0),
            f(y0),
            f(x0),
            f(y1)
        ),
        format!(
            "<line class=\"grid\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{axis}\"/>",
            f(x0),
            f(y1),
            f(x1),
            f(y1)
        ),
    ];
    // **点の位置を先に全部決めてから、ラベルをまとめて置く**（実測 ── 1点ずつ置くと12点で15件重なった）
    let mut placed = Vec::new();
    for i in 0..pts.len() {
        let px = x0 + (x1 - x0) * (xs[i] / mx);
        let py = y1 - (y1 - y0) * (ys[i] / my);
        placed.push((px, py));
        body.push(format!(
            "<circle class=\"{}\" cx=\"{px:.1}\" cy=\"{py:.1}\" r=\"{}\" fill=\"{}\"/>",
            crate::classes::series(0),
            f(r),
            tone(style, 0)?
        ));
    }
    let off = r + gap / 2.0; // 点から離す量
    let mut to_place = Vec::new();
    let mut names = Vec::new();
    for (i, (px, py)) in placed.iter().enumerate() {
        let name = ftext(&pts[i], "name")?;
        let lw = text::width(&name, fs_small) + gap;
        let lh = cap + gap / 2.0;
        // 候補は点の周り8方向。**上を最優先にし、外側から順に試す**
        let cands = vec![
            (*px, py - off - cap / 2.0),
            (*px, py + off + cap / 2.0),
            (px - off - lw / 2.0, *py),
            (px + off + lw / 2.0, *py),
            (px - off - lw / 2.0, py - off),
            (px + off + lw / 2.0, py - off),
            (px - off - lw / 2.0, py + off),
            (px + off + lw / 2.0, py + off),
        ];
        // 画布の外へ出る候補は使わない
        let inside: Vec<(f64, f64)> = cands
            .iter()
            .copied()
            .filter(|(cx, cy)| {
                cx - lw / 2.0 >= 0.0 && cx + lw / 2.0 <= w && cy - cap >= 0.0 && *cy <= h
            })
            .collect();
        to_place.push(((lw, lh), if inside.is_empty() { cands } else { inside }));
        names.push(name);
    }
    // 点そのものも避ける相手に入れる ── **ラベルが点に乗ると読めない**
    let occupied: Vec<(f64, f64, f64, f64)> = placed
        .iter()
        .map(|(px, py)| (px - r, py - r, px + r, py + r))
        .collect();
    for ((cx, cy), name) in place_avoiding(&to_place, &occupied)
        .into_iter()
        .zip(names.iter())
    {
        body.push(t(
            cx,
            cy,
            name,
            style,
            "color.ink-soft",
            "middle",
            None,
            "400",
        )?);
    }
    if props::flag(p, "x_label") {
        body.push(t(
            (x0 + x1) / 2.0,
            h - gap,
            &props::text_or(p, "x_label", ""),
            style,
            "color.ink-faint",
            "middle",
            None,
            "400",
        )?);
    }
    if props::flag(p, "y_label") {
        let fs = style.num("font.size-small")?;
        let label_x = f(style.num("chart.pad")?);
        let cy = (y0 + y1) / 2.0;
        body.push(format!(
            "<text class=\"note small\" x=\"{label_x}\" y=\"{cy:.1}\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\" text-anchor=\"middle\" transform=\"rotate(-90 {label_x} {cy:.1})\">{}</text>",
            style.text("font.family")?,
            f(fs),
            style.text("color.ink-faint")?,
            esc(&props::text_or(p, "y_label", ""))
        ));
    }
    Ok(Fragment::own(format!("<g>{}</g>", body.concat()), w, h))
}

/// 左右2列の間を、**太さが値を表すリボン**で結ぶ。
#[allow(clippy::too_many_lines)]
fn flow(p: &Props, style: &Style) -> Result<Fragment, String> {
    let links = items(p, "links")?;
    let values: Vec<Value> = links
        .iter()
        .map(|l| field(l, "value").cloned())
        .collect::<Result<_, _>>()?;
    let total = sum_values(&values);
    let total = if total == 0.0 { 1.0 } else { total };
    let mut w = style.num("chart.flow-w")?;
    let h = style.num("chart.flow-h")?;
    let pad = style.num("chart.pad")?;
    let margin = style.num("chart.flow-margin")?;
    let bw = style.num("chart.flow-bar-w")?;
    let node_gap = style.num("chart.flow-node-gap")?;
    let gap = style.num("chart.gap")?;
    let base = style.num("font.baseline-ratio")?;
    let fs_small = style.num("font.size-small")?;
    // 左右の節点を、現れた順に集める
    let mut lefts: Vec<(String, f64)> = Vec::new();
    let mut rights: Vec<(String, f64)> = Vec::new();
    for l in links {
        let (from, to, v) = (ftext(l, "from")?, ftext(l, "to")?, fnum(l, "value")?);
        match lefts.iter_mut().find(|(k, _)| *k == from) {
            Some(slot) => slot.1 += v,
            None => lefts.push((from, v)),
        }
        match rights.iter_mut().find(|(k, _)| *k == to) {
            Some(slot) => slot.1 += v,
            None => rights.push((to, v)),
        }
    }
    // **左右の余白は、実際に入る名前から決める** ── 固定の余白のままだと、長い名前が画布の外
    // へ出る（実測）
    let span = w - (pad + margin) * 2.0;
    let widest = |side: &[(String, f64)]| {
        side.iter()
            .map(|(k, _)| text::width(k, fs_small))
            .fold(0.0_f64, f64::max)
    };
    let margin_l = margin.max(widest(&lefts) + gap);
    let margin_r = margin.max(widest(&rights) + gap);
    w = pad + margin_l + span + margin_r + pad;
    let (xl, xr) = (pad + margin_l, pad + margin_l + span);
    // 節点どうしの隙間は、値に比例する高さとは別に要る ── **節点の多いほうに合わせて1つの
    // 配分にする**
    let most = (lefts.len() as i64 - 1).max(rights.len() as i64 - 1).max(0);
    let span_h = h - pad * 2.0 - node_gap * most as f64;
    let place = |side: &[(String, f64)]| -> Vec<(String, [f64; 2])> {
        let mut cy = pad;
        side.iter()
            .map(|(k, v)| {
                let pair = [cy, cy + span_h * v / total];
                cy = pair[1] + node_gap;
                (k.clone(), pair)
            })
            .collect()
    };
    let mut pos_l = place(&lefts);
    let mut pos_r = place(&rights);
    let opacity = f(style.num("chart.ribbon-opacity")?);
    let mut body = Vec::new();
    for (i, l) in links.iter().enumerate() {
        let hgt = span_h * fnum(l, "value")? / total;
        let from = ftext(l, "from")?;
        let to = ftext(l, "to")?;
        let li = pos_l
            .iter()
            .position(|(k, _)| *k == from)
            .ok_or("左の節点が無い")?;
        let ri = pos_r
            .iter()
            .position(|(k, _)| *k == to)
            .ok_or("右の節点が無い")?;
        let (a, b) = (pos_l[li].1[0], pos_r[ri].1[0]);
        pos_l[li].1[0] += hgt;
        pos_r[ri].1[0] += hgt;
        let m = (xl + xr) / 2.0;
        let (sx, sm, sr) = (f(xl + bw), f(m), f(xr));
        body.push(format!(
            "<path class=\"{}\" d=\"M{sx},{a:.1} C{sm},{a:.1} {sm},{b:.1} {sr},{b:.1} L{sr},{:.1} C{sm},{:.1} {sm},{:.1} {sx},{:.1} Z\" fill=\"{}\" opacity=\"{opacity}\"/>",
            crate::classes::series(i),
            b + hgt,
            b + hgt,
            a + hgt,
            a + hgt,
            tone(style, i)?
        ));
    }
    let faint = style.text("color.ink-faint")?;
    for (k, [y0, y1]) in &pos_l {
        body.push(format!(
            "<rect class=\"box\" x=\"{}\" y=\"{y0:.1}\" width=\"{}\" height=\"{:.1}\" fill=\"{faint}\"/>",
            f(xl),
            f(bw),
            y1 - y0
        ));
        body.push(t(
            xl - gap,
            (y0 + y1) / 2.0 + fs_small * base,
            k,
            style,
            "color.ink-soft",
            "end",
            None,
            "400",
        )?);
    }
    for (k, [y0, y1]) in &pos_r {
        body.push(format!(
            "<rect class=\"box\" x=\"{}\" y=\"{y0:.1}\" width=\"{}\" height=\"{:.1}\" fill=\"{faint}\"/>",
            f(xr - bw),
            f(bw),
            y1 - y0
        ));
        body.push(t(
            xr + gap,
            (y0 + y1) / 2.0 + fs_small * base,
            k,
            style,
            "color.ink-soft",
            "start",
            None,
            "400",
        )?);
    }
    Ok(Fragment::own(format!("<g>{}</g>", body.concat()), w, h))
}

/// 並べた区画を置く。**読ませたいのは位置そのもの。**
///
/// **座標が在れば座標で置く。** 無ければ宣言の並び順のまま並べる。箱の幅は、列ごとに最長の
/// ラベルへ合わせる（実測 ── 固定の幅だと長いラベルがはみ出した）。
#[allow(clippy::too_many_lines)]
fn spatial(p: &Props, style: &Style) -> Result<Fragment, String> {
    let list = items(p, "items")?;
    let at = |it: &Value| -> Option<(f64, f64)> {
        it.get("at").filter(|v| !v.is_null()).and_then(props::point)
    };
    // 座標の値を、重ねずに並べる
    let axis_values = |pick: fn((f64, f64)) -> f64| -> Vec<f64> {
        let set: BTreeSet<u64> = list
            .iter()
            .filter_map(at)
            .map(|q| pick(q).to_bits())
            .collect();
        let mut out: Vec<f64> = set.into_iter().map(f64::from_bits).collect();
        out.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        out.dedup();
        out
    };
    let xs = axis_values(|q| q.0);
    let ys = axis_values(|q| q.1);
    let by_at = !xs.is_empty();
    let cols = if by_at {
        xs.len()
    } else {
        props::num(p, "cols").map_or(1, |c| c as usize)
    };
    let ch = style.num("chart.spatial-row-h")?;
    let gap = style.num("chart.spatial-gap")?;
    let pad_x = style.num("chart.spatial-pad-x")?;
    let fs = style.num("font.size")?;
    let fs_small = style.num("font.size-small")?;
    let rows = if by_at {
        ys.len()
    } else {
        list.len().div_ceil(cols)
    };
    // 名前の右には、深さの数字が右寄せで入る ── **その欄も中身から決める**
    let depths: Vec<String> = list
        .iter()
        .filter_map(|it| it.get("depth").filter(|v| !v.is_null()).map(props::text))
        .collect();
    let depth_w = text::column(&depths, fs_small, gap * 2.0);
    let cell = |i: usize, it: &Value| -> (usize, usize) {
        match at(it) {
            Some((x, y)) if by_at => (
                xs.iter().position(|v| *v == x).unwrap_or(0),
                ys.iter().position(|v| *v == y).unwrap_or(0),
            ),
            _ => (i % cols, i / cols),
        }
    };
    let mut col_w = vec![0.0_f64; cols];
    for (i, it) in list.iter().enumerate() {
        let (c, _) = cell(i, it);
        col_w[c] = col_w[c].max(text::width(&ftext(it, "name")?, fs) + pad_x * 2.0 + depth_w);
    }
    let col_x: Vec<f64> = (0..cols)
        .map(|c| sum(col_w[..c].iter().copied()) + gap * c as f64)
        .collect();
    let grid_w = sum(col_w.iter().copied()) + gap * (cols as f64 - 1.0);
    let grid_h = rows as f64 * ch + (rows as f64 - 1.0) * gap;
    // 地と軸の名前は、置いたものの外側に帯を取る。**厚みは書体から導く**
    let band = fs_small * style.num("size.label-line-h")?;
    let left = if props::flag(p, "axis_label") {
        band
    } else {
        0.0
    };
    let topb = if props::flag(p, "ground") { band } else { 0.0 };
    let w = grid_w + left;
    let h = grid_h + topb;
    let baseline = style.num("font.baseline-ratio")?;
    let mut body = Vec::new();
    if props::flag(p, "ground") {
        // 地 ── 置いたものが何の上にあるか。**背に敷き、名前を左上へ置く**
        body.push(format!(
            "<rect class=\"area\" x=\"{left:.1}\" y=\"{topb:.1}\" width=\"{grid_w:.1}\" height=\"{grid_h:.1}\" rx=\"{}\" fill=\"{}\" opacity=\"{}\"/>",
            f(style.num("size.radius-small")?),
            style.text("chart.grid")?,
            f(style.num("opacity.faint")?)
        ));
        body.push(t(
            left,
            band / 2.0 + fs_small * baseline,
            &props::text_or(p, "ground", ""),
            style,
            "color.ink-faint",
            "start",
            Some(fs_small),
            "400",
        )?);
    }
    if props::flag(p, "axis_label") {
        let cy = topb + grid_h / 2.0;
        let ax = band / 2.0 + fs_small * baseline - fs_small;
        body.push(format!(
            "<text class=\"note small\" x=\"{ax:.1}\" y=\"{cy:.1}\" text-anchor=\"middle\" transform=\"rotate(-90 {ax:.1} {cy:.1})\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\">{}</text>",
            style.text("font.family")?,
            f(fs_small),
            style.text("color.ink-faint")?,
            esc(&props::text_or(p, "axis_label", ""))
        ));
    }
    for (i, it) in list.iter().enumerate() {
        let (c, r) = cell(i, it);
        let x = left + col_x[c];
        let y = topb + r as f64 * (ch + gap);
        let cw = col_w[c];
        let focus = it.get("role").and_then(Value::as_str) == Some("focus");
        let fill = style.text(if focus {
            "color.accent-bg"
        } else {
            "color.box-fill"
        })?;
        let stroke = style.text(if focus {
            "color.accent"
        } else {
            "color.box-stroke"
        })?;
        body.push(format!(
            "<rect class=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{fill}\" stroke=\"{stroke}\"/>",
            if focus { "box focus" } else { "box" },
            f(x),
            f(y),
            f(cw),
            f(ch),
            f(style.num("size.radius")?)
        ));
        body.push(t(
            x + pad_x,
            y + ch / 2.0 + fs * baseline,
            &ftext(it, "name")?,
            style,
            if focus { "color.accent" } else { "color.ink" },
            "start",
            Some(fs),
            "400",
        )?);
        if let Some(depth) = it.get("depth").filter(|v| !v.is_null()) {
            body.push(t(
                x + cw - gap,
                y + ch / 2.0 + fs_small * baseline,
                &props::text(depth),
                style,
                "color.ink-faint",
                "end",
                Some(fs_small),
                "400",
            )?);
        }
    }
    Ok(Fragment::own(format!("<g>{}</g>", body.concat()), w, h))
}
