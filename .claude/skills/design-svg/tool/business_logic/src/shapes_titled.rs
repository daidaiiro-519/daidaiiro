// SPDX-License-Identifier: MIT
//! 名前を添えた部品 ── 別の部品を包み、その上に名前を載せる。
//!
//! 節点として使われる部品は、渡された名前を描かなければならない ── 描かないと、図の中でその
//! 節点が何なのかが読めなくなる（実測 ── 階層の中へ円グラフを置いたところ、「内訳」という
//! 名前がどこにも出なかった）。**包む側を1つ足す** ── 中身は既存の部品を台帳越しに呼ぶだけ
//! なので、部品を足しても包む側は変わらない。

use crate::props::{self, esc};
use crate::py::float as f;
use crate::registry::{self, Component, Fragment, Props};
use crate::style::Style;
use crate::text;

/// 台帳へ載せる部品。
#[must_use]
pub fn register() -> Vec<(&'static str, Component)> {
    vec![("titled", titled as Component)]
}

/// 別の部品を、名前の帯つきで描く。**大きさは名前の帯を含む。**
fn titled(p: &Props, style: &Style) -> Result<Fragment, String> {
    let of = props::need_text(p, "of")?;
    let inner = registry::render_node(&of, p, style)?;
    let label = props::text_or(p, "label", "");
    // 名前を描く役目は1か所にしか置けない。**中身が自分で描いたなら、包む側は描かない**
    if label.is_empty() || inner.labels_itself {
        return Ok(inner);
    }
    let size = style.num("font.size-small")?;
    let lead = size * style.num("size.label-line-h")?;
    let pad_x = style.num("size.label-pad-x")?;
    // 帯の幅は、名前が収まる幅と中身の幅の大きいほう
    let w = inner.width.max(text::width(&label, size) + pad_x * 2.0);
    let h = inner.height + lead;
    let dx = (w - inner.width) / 2.0;
    // **帯は字面ぴったりに取る** ── 節点の幅いっぱいへ広げると、名前より遥かに広い面が辺を
    // 丸ごと覆い、矢印が消えたように見える（実測）
    let band_w = text::width(&label, size) + pad_x * 2.0;
    let band_h = size * (style.num("font.cap-ratio")? + style.num("font.descender-ratio")?);
    let band_top = (lead - band_h) / 2.0;
    let base = band_top + size * style.num("font.cap-ratio")?;
    let svg = format!(
        "<g><rect class=\"badge\" x=\"{:.1}\" y=\"{band_top:.1}\" width=\"{band_w:.1}\" height=\"{band_h:.1}\" fill=\"{}\"/><text class=\"label small\" x=\"{:.1}\" y=\"{base:.1}\" text-anchor=\"middle\" font-family=\"{}\" font-size=\"{}\" font-weight=\"{}\" fill=\"{}\">{}</text><g transform=\"translate({dx:.1},{lead:.1})\">{}</g></g>",
        (w - band_w) / 2.0,
        style.text("color.box-fill")?,
        w / 2.0,
        style.text("font.family")?,
        f(size),
        style.text("font.weight-medium")?,
        style.text("color.ink")?,
        esc(&label),
        inner.svg
    );
    Ok(Fragment::own(svg, w, h).labelled())
}
