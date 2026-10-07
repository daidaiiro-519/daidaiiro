// SPDX-License-Identifier: MIT
//! 返す前の道 ── **作成者が記述した SVG も、figure と chart の出力も、同じ解決と検査を通す。**
//!
//! 検査1〜3は解決の前の SVG に、検査4は解決のあとの SVG に適用する。
//!
//! figure と chart は、部品が値と class の両方を出力する。**解決の前に、class が持つ値だけを外す**
//! ── 外したあとの SVG が「作成者が記述した SVG」と同じ形になり、同じ検査1〜3を当てられる。
//! class が持たない値を部品が出力していれば、外されずに残り、検査2が検出する。

use serde_json::{Map, Value};

use crate::checks;
use crate::classes;
use crate::resolve;
use crate::xml::{self, Element};

/// 返すもの。
#[derive(Debug, Clone)]
pub struct Finished {
    /// 解決した SVG。
    pub svg: String,
    /// 検査1〜4の検出。
    pub findings: Vec<String>,
}

/// 作成者が記述した SVG を、検査1〜3 → 解決 → 検査4の順に通す。
///
/// `theme` は既定のテーマに上書きを重ねたもの（`None` なら既定）。`dark` が偽なら、ダークモードの
/// `style` を出さない ── 呼び出し元がトークンを上書きしたときである。
///
/// # Errors
///
/// SVG として読めないときと、class が指すトークンがテーマに無いときに返す。
pub fn authored(
    svg: &str,
    theme: Option<&Map<String, Value>>,
    dark: bool,
) -> Result<Finished, String> {
    let mut findings = checks::check_classes(svg)?;
    findings.extend(checks::check_values(svg)?);
    findings.extend(checks::check_combinations(svg)?);
    let resolved = resolve::resolve(svg, theme, dark)?;
    findings.extend(checks::check_layout(&resolved)?);
    Ok(Finished {
        svg: resolved,
        findings,
    })
}

/// class が持つ値を外す。**class が持たない値は残す**（検査2が検出する）。
fn strip_owned(el: &mut Element) {
    let names: Vec<String> = el.classes().iter().map(|s| (*s).to_owned()).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    for (attr, _) in classes::values_of(&refs) {
        if attr == "rx" && el.tag != "rect" {
            continue;
        }
        el.remove(&attr);
    }
    for ch in el.elements_mut() {
        strip_owned(ch);
    }
}

/// スマホ幅で最小のフォントサイズがしきい値を下回らない、描画の最小幅（px）。**足りていれば `None`。**
///
/// 部品は図の幅を中身から決めるので、横に長い図ほど縮小される。作成者が記述する SVG では
/// 作成者が `min-width` を書くが、figure と chart は design-svg が書く ── 埋め込み先のページが
/// 横スクロールさせる（ブレストボード design-svg-rework の論点5）。
fn needed_min_width(resolved: &str) -> Option<f64> {
    let root = xml::parse(resolved)?;
    let checks = classes::notation().checks;
    let least = checks::min_font_px(resolved)?;
    if least >= checks.min_font_px {
        return None;
    }
    let width = checks::rendered_width(&root);
    Some((width * checks.min_font_px / least).ceil())
}

/// figure と chart が描いた SVG を、class が持つ値を外してから [`authored`] と同じ道に通す。
/// 縮小するとしきい値を下回る図には、`min-width` を足す。
///
/// # Errors
///
/// SVG として読めないときと、class が指すトークンがテーマに無いときに返す。
pub fn generated(
    svg: &str,
    theme: Option<&Map<String, Value>>,
    dark: bool,
) -> Result<Finished, String> {
    let mut root = xml::parse(svg).ok_or("描いた SVG を読めない")?;
    strip_owned(&mut root);
    let draft = resolve::resolve(&xml::write(&root), theme, dark)?;
    if let Some(min) = needed_min_width(&draft) {
        let style = root
            .get("style")
            .map_or_else(String::new, |s| format!("{s};"));
        root.set(
            "style",
            &format!("{style}min-width:{}px", crate::py::fixed(min, 0)),
        );
    }
    authored(&xml::write(&root), theme, dark)
}
