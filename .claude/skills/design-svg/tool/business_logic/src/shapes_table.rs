// SPDX-License-Identifier: MIT
//! 表の部品 ── **絵の中に表が要るときだけ使う。**
//!
//! 表は絵である必要がない。文字として出せるなら、成果物の書式で書いたほうが読める ── 選択
//! でき、折り返し、幅に追随する。ここが受け持つのは、表が絵の一部でなければならない場合だけ
//! である。**列幅はセルの文字幅から決める** ── 固定の幅にしない。

use crate::props::{self, esc};
use crate::py::{float as f, sum};
use crate::registry::{Component, Fragment, Props};
use crate::style::Style;
use crate::text;

/// 台帳へ載せる部品。
#[must_use]
pub fn register() -> Vec<(&'static str, Component)> {
    vec![("table", table as Component)]
}

/// 見出し行つきの表。**軸の名前を持てる** ── 縦横が何を表すかが絵に出ないと、交点に何が
/// 来るかを読めない。
#[allow(clippy::too_many_lines)]
fn table(p: &Props, style: &Style) -> Result<Fragment, String> {
    let headers: Vec<String> = props::list(p, "headers").iter().map(props::text).collect();
    if p.get("headers").is_none() {
        return Err(crate::py::quote("headers"));
    }
    let rows: Vec<Vec<String>> = props::list(p, "rows")
        .iter()
        .map(|r| {
            r.as_array()
                .map(|c| c.iter().map(props::text).collect())
                .unwrap_or_default()
        })
        .collect();
    if p.get("rows").is_none() {
        return Err(crate::py::quote("rows"));
    }
    let axes: Vec<String> = props::list(p, "axes").iter().map(props::text).collect();
    let fs = style.num("font.size")?;
    let fs_small = style.num("font.size-small")?;
    let pad_x = style.num("chart.table-pad-x")?;
    let row_h = style.num("chart.table-row-h")?;

    let mut col_w = Vec::new();
    for (c, head) in headers.iter().enumerate() {
        let mut widest = text::width(head, fs_small);
        for r in &rows {
            let cell = r
                .get(c)
                .ok_or_else(|| "list index out of range".to_owned())?;
            widest = widest.max(text::width(cell, fs));
        }
        col_w.push(widest + pad_x * 2.0);
    }
    let col_x: Vec<f64> = (0..headers.len())
        .map(|c| sum(col_w[..c].iter().copied()))
        .collect();
    let grid_w = sum(col_w.iter().copied());
    // 軸の名前は表の外側に置く。**帯の厚みは書体から導く**
    let band = if axes.is_empty() {
        0.0
    } else {
        fs_small * style.num("size.label-line-h")?
    };
    let left = if axes.is_empty() { 0.0 } else { band };
    let top = if axes.len() > 1 { band } else { 0.0 };
    let w = grid_w + left;
    let h = row_h * (rows.len() as f64 + 1.0) + top;
    let family = style.text("font.family")?;
    let baseline = style.num("font.baseline-ratio")?;

    let mut body = vec![format!(
        "<g transform=\"translate({left:.1},{top:.1})\"><rect x=\"0\" y=\"0\" width=\"{grid_w:.1}\" height=\"{row_h:.1}\" fill=\"{}\"/>",
        style.text("color.accent-bg")?
    )];
    for (c, head) in headers.iter().enumerate() {
        body.push(format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" font-family=\"{family}\" font-size=\"{}\" font-weight=\"{}\" fill=\"{}\">{}</text>",
            col_x[c] + pad_x,
            row_h / 2.0 + fs_small * baseline,
            f(fs_small),
            style.text("font.weight-medium")?,
            style.text("color.accent")?,
            esc(head)
        ));
    }
    for (r_idx, row) in rows.iter().enumerate() {
        let y = row_h * (r_idx as f64 + 1.0);
        body.push(format!(
            "<line x1=\"0\" y1=\"{y:.1}\" x2=\"{w:.1}\" y2=\"{y:.1}\" stroke=\"{}\"/>",
            style.text("chart.grid")?
        ));
        for (c, cell) in row.iter().enumerate() {
            let x = col_x
                .get(c)
                .ok_or_else(|| "list index out of range".to_owned())?;
            body.push(format!(
                "<text x=\"{:.1}\" y=\"{:.1}\" font-family=\"{family}\" font-size=\"{}\" fill=\"{}\">{}</text>",
                x + pad_x,
                y + row_h / 2.0 + fs * baseline,
                f(fs),
                style.text("color.ink")?,
                esc(cell)
            ));
        }
    }
    let grid_h = row_h * (rows.len() as f64 + 1.0);
    body.push(format!(
        "<rect x=\"0.5\" y=\"0.5\" width=\"{:.1}\" height=\"{:.1}\" fill=\"none\" stroke=\"{}\"/></g>",
        grid_w - 1.0,
        grid_h - 1.0,
        style.text("color.box-stroke")?
    ));
    let base = fs_small * baseline;
    if axes.len() > 1 {
        // 横が何を表すか ── 表の上、いちばん左の列と左端を揃える
        body.push(format!(
            "<text x=\"{left:.1}\" y=\"{:.1}\" font-family=\"{family}\" font-size=\"{}\" fill=\"{}\">{}</text>",
            band / 2.0 + base,
            f(fs_small),
            style.text("color.ink-faint")?,
            esc(&axes[1])
        ));
    }
    if !axes.is_empty() {
        // 縦が何を表すか ── 表の左で、90度回転させて縦書きにする
        let cy = top + grid_h / 2.0;
        let x = band / 2.0 + base - fs_small;
        body.push(format!(
            "<text x=\"{x:.1}\" y=\"{cy:.1}\" text-anchor=\"middle\" transform=\"rotate(-90 {x:.1} {cy:.1})\" font-family=\"{family}\" font-size=\"{}\" fill=\"{}\">{}</text>",
            f(fs_small),
            style.text("color.ink-faint")?,
            esc(&axes[0])
        ));
    }
    Ok(Fragment::own(format!("<g>{}</g>", body.concat()), w, h))
}
