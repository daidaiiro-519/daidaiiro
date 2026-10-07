// SPDX-License-Identifier: MIT
//! 試しに足す部品 ── 六角形の節点。
//!
//! **核を触らずに足せるかを測るために作った。** 台帳へ登録し、テーマの `parts.node` をこの
//! 名前へ向ければ、節点が六角形になる。

use crate::props::{self, esc};
use crate::py::float as f;
use crate::registry::{Component, Fragment, Props};
use crate::style::Style;
use crate::text;

/// 台帳へ載せる部品。
#[must_use]
pub fn register() -> Vec<(&'static str, Component)> {
    vec![("hex", hexagon as Component)]
}

/// 名前を1つ持つ六角形。**箱と同じ契約**（構造だけ受け取り、見た目は見た目から）。
fn hexagon(p: &Props, style: &Style) -> Result<Fragment, String> {
    let label = props::text_or(p, "label", "");
    let fs = style.num("font.size")?;
    let pad_x = style.num("size.box-pad-x")?;
    let h = style.num("size.box-h")?;
    let w = style
        .num("size.box-min-w")?
        .max(text::width_with(&label, fs, style.num("font.latin-width-ratio")?) + pad_x * 2.0 + h);
    let cut = h / 2.0; // 斜めに切り取る幅。左右で h/2 ずつ要るので幅にも足す
    let pts = format!(
        "{},0 {},0 {},{} {},{} {},{} 0,{}",
        f(cut),
        f(w - cut),
        f(w),
        f(h / 2.0),
        f(w - cut),
        f(h),
        f(cut),
        f(h),
        f(h / 2.0)
    );
    let svg = format!(
        "<g class=\"svg-box\"><polygon class=\"box\" points=\"{pts}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/><text class=\"label\" x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\">{}</text></g>",
        style.text("color.box-fill")?,
        style.text("color.box-stroke")?,
        f(style.num("size.stroke-width")?),
        w / 2.0,
        h / 2.0 + fs * style.num("font.baseline-ratio")?,
        style.text("font.family")?,
        f(fs),
        style.text_or("color.text", &style.text("color.ink")?)?,
        esc(&label)
    );
    Ok(Fragment::own(svg, w, h).labelled())
}
