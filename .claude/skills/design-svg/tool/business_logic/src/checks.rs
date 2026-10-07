// SPDX-License-Identifier: MIT
//! 図のデザインシステムを守っているかの検査 ── **検査1〜4。**
//!
//! - [`check_classes`]（検査1）── 図形とテキストが、一覧に在る class を持つ
//! - [`check_values`]（検査2）── 作成者が色 ・ ストローク幅 ・ 角丸の半径 ・ 破線のパターン ・
//!   フォントサイズの値を記述していない
//! - [`check_combinations`]（検査3）── 禁止した組み合わせが無い
//! - [`check_layout`]（検査4）── テキストの重なり ・ キャンバスやボックスからのはみ出し ・
//!   スマホ幅で描画したときの最小のフォントサイズ
//!
//! **検査1〜3は解決の前の SVG（作成者が記述したもの）に、検査4は解決のあとの SVG に適用する** ──
//! 解決のあとは値が属性に在るので、検査2を適用すると全部が検出になる。
//!
//! class の選択の誤り（イベントに warn を指定するなど）は機械では検査できない。だから
//! [`labels`] が class ごとのラベルの一覧と、複数行のラベルの一覧を返し、描画して目視で確かめる。

use std::collections::BTreeMap;

use crate::classes::{self, Kind};
use crate::geometry::apply_transform;
use crate::py::{parse_float, quote};
use crate::resolve::GENERATED;
use crate::text;
use crate::theme;
use crate::verify;
use crate::xml::{self, Element};

/// class を持たなければならない要素。**tspan は親の text の class に従う。**
const TARGETS: [&str; 9] = [
    "rect", "circle", "ellipse", "line", "polyline", "polygon", "path", "text", "image",
];
/// 中身を描かない入れ物。**この中の要素は検査1の対象外である**（marker は別に検出する）。
const CONTAINERS: [&str; 6] = [
    "defs",
    "clipPath",
    "mask",
    "pattern",
    "symbol",
    "linearGradient",
];
/// 作成者が記述してはならない値の属性。
const VALUE_ATTRS: [&str; 5] = [
    "fill",
    "stroke",
    "stroke-width",
    "stroke-dasharray",
    "font-size",
];
/// 角丸の半径。**矩形のときだけ値である**（楕円の rx は形そのもの）。
const RADIUS_ATTRS: [&str; 2] = ["rx", "ry"];
/// swatch の要素が記述してよい色。
const SWATCH_ATTRS: [&str; 3] = ["fill", "stroke", "stop-color"];
/// 色の値の属性（swatch でない要素では値として検出する）。
const STOP_COLOR: &str = "stop-color";
/// 色ではない値 ── どの要素に記述してもよい。
const NOT_A_VALUE: &str = "none";
/// 複数行のラベルを連結する記号。
pub const LINE_JOIN: &str = "｜";

fn is_generated(el: &Element) -> bool {
    el.get(GENERATED).is_some()
}

/// 要素を、読み手が探せる短い名前で言う。**テキストは中身、図形は位置。**
fn name_of(el: &Element) -> String {
    if el.tag == "text" {
        return format!("text {}", quote(el.text().trim()));
    }
    let at: Vec<String> = ["x", "y", "cx", "cy", "x1", "y1", "d", "points"]
        .iter()
        .filter_map(|k| el.get(k).map(|v| format!("{k}={}", short(v))))
        .take(2)
        .collect();
    if at.is_empty() {
        el.tag.clone()
    } else {
        format!("{}({})", el.tag, at.join(","))
    }
}

/// 長い値（path の d など）を、最初の語までに縮める。
fn short(v: &str) -> String {
    v.split_whitespace().next().unwrap_or(v).to_owned()
}

/// 解決の前の SVG を、生成した要素を除いて歩く。`f(要素, 入れ物の中か)`
fn walk<'a>(el: &'a Element, inside: bool, f: &mut dyn FnMut(&'a Element, bool)) {
    if is_generated(el) {
        return;
    }
    f(el, inside);
    let inside = inside || CONTAINERS.contains(&el.tag.as_str());
    for ch in el.elements() {
        walk(ch, inside, f);
    }
}

fn parse(svg: &str) -> Result<Element, String> {
    xml::parse(svg).ok_or_else(|| "SVG として読めない".to_owned())
}

/// 検査1 ── 図形とテキストが、一覧に在る class を持つか。作成者が記述した marker も検出する。
///
/// # Errors
///
/// SVG として読めないときに返す。
pub fn check_classes(svg: &str) -> Result<Vec<String>, String> {
    let root = parse(svg)?;
    let mut out = Vec::new();
    walk(&root, false, &mut |el, inside| {
        if el.tag == "marker" {
            out.push(format!(
                "検査1 marker {}: 矢じりは design-svg が flow の class から生成する。marker を記述しない",
                el.get("id").map_or_else(String::new, quote)
            ));
            return;
        }
        if inside {
            return;
        }
        let names = el.classes();
        let unknown: Vec<&str> = names
            .iter()
            .copied()
            .filter(|n| !classes::known(n))
            .collect();
        if TARGETS.contains(&el.tag.as_str()) && names.is_empty() {
            out.push(format!("検査1 {}: class が無い", name_of(el)));
        } else if (TARGETS.contains(&el.tag.as_str()) || el.tag == "tspan") && !unknown.is_empty() {
            out.push(format!(
                "検査1 {}: 一覧に無い class {}",
                name_of(el),
                unknown.join(" ・ ")
            ));
        }
    });
    Ok(out)
}

/// style 属性の宣言を `(名前, 値)` に分ける。
fn declarations(style: &str) -> Vec<(String, String)> {
    style
        .split(';')
        .filter_map(|d| {
            let (k, v) = d.split_once(':')?;
            Some((k.trim().to_owned(), v.trim().to_owned()))
        })
        .collect()
}

/// その属性が、その要素では値として検出されるか。
fn is_value(el: &Element, attr: &str, value: &str) -> bool {
    let swatch = el.classes().contains(&"swatch");
    if swatch && SWATCH_ATTRS.contains(&attr) {
        return false;
    }
    if (attr == "fill" || attr == "stroke") && value.trim().eq_ignore_ascii_case(NOT_A_VALUE) {
        return false;
    }
    VALUE_ATTRS.contains(&attr)
        || attr == STOP_COLOR
        || (RADIUS_ATTRS.contains(&attr) && el.tag == "rect")
}

/// 検査2 ── 作成者が値を記述していないか。**swatch の要素自身の色だけは記述してよい。**
///
/// # Errors
///
/// SVG として読めないときに返す。
pub fn check_values(svg: &str) -> Result<Vec<String>, String> {
    let root = parse(svg)?;
    let mut out = Vec::new();
    walk(&root, false, &mut |el, _| {
        if el.tag == "style" {
            out.push(
                "検査2 style: 作成者が style 要素を記述した。スタイルは class で指定する"
                    .to_owned(),
            );
            return;
        }
        let mut hits: Vec<String> = el
            .attrs
            .iter()
            .filter(|(k, v)| is_value(el, k, v))
            .map(|(k, v)| format!("{k}={}", quote(v)))
            .collect();
        if let Some(style) = el.get("style") {
            hits.extend(
                declarations(style)
                    .into_iter()
                    .filter(|(k, v)| is_value(el, k, v))
                    .map(|(k, v)| format!("style の {k}:{v}")),
            );
        }
        if !hits.is_empty() {
            out.push(format!(
                "検査2 {}: 値を記述した {}",
                name_of(el),
                hits.join(" ・ ")
            ));
        }
    });
    Ok(out)
}

/// 要素の種類が、その class の種類と合うか。
fn tag_fits(tag: &str, kind: Kind) -> bool {
    match tag {
        "text" => kind == Kind::Text,
        "line" => kind == Kind::Line,
        "path" | "polyline" => kind == Kind::Shape || kind == Kind::Line,
        _ => kind == Kind::Shape,
    }
}

/// 検査3 ── 禁止した組み合わせが無いか。**組ごとに1件だけ報告する。**
///
/// # Errors
///
/// SVG として読めないときに返す。
pub fn check_combinations(svg: &str) -> Result<Vec<String>, String> {
    let root = parse(svg)?;
    let mut out = Vec::new();
    walk(&root, false, &mut |el, inside| {
        if inside || !TARGETS.contains(&el.tag.as_str()) {
            return;
        }
        let names = el.classes();
        let list = classes::ordered(&names);
        if list.is_empty() {
            return;
        }
        for (i, a) in list.iter().enumerate() {
            for b in &list[i + 1..] {
                if let Some(why) = classes::conflict(&a.name, &b.name) {
                    out.push(format!(
                        "検査3 {}: {} と {} ── {why}",
                        name_of(el),
                        a.name,
                        b.name
                    ));
                }
            }
        }
        match list.iter().find(|c| c.kind != Kind::Modifier) {
            None => out.push(format!(
                "検査3 {}: 修飾の class だけで、図形 ・ 線 ・ テキストの class が無い",
                name_of(el)
            )),
            Some(base) if !tag_fits(&el.tag, base.kind) => out.push(format!(
                "検査3 {}: {} は {} の class で、{} 要素には使わない",
                name_of(el),
                base.name,
                base.kind.slot(),
                el.tag
            )),
            Some(_) => {}
        }
    });
    Ok(out)
}

/// 外接矩形。`(左, 上, 右, 下)`
type Rect = (f64, f64, f64, f64);

/// テキスト1つ ── 外接矩形 ・ 中身 ・ class
struct Label {
    rect: Rect,
    text: String,
    classes: Vec<String>,
}

/// 図形1つ ── 外接矩形 ・ class
struct Shape {
    rect: Rect,
    classes: Vec<String>,
}

fn theme_num(key: &str) -> f64 {
    theme::num(theme::default_theme(), key).unwrap_or_default()
}

fn num(el: &Element, k: &str) -> f64 {
    el.get(k).and_then(parse_float).unwrap_or_default()
}

/// 解決したあとの SVG から、テキストと図形を絶対座標で集める。**回転したテキストは除く。**
fn gather(root: &Element) -> (Vec<Label>, Vec<Shape>) {
    let cap = theme_num("font.cap-ratio");
    let desc = theme_num("font.descender-ratio");
    let default_fs = theme_num("font.size");
    let mut labels = Vec::new();
    let mut shapes = Vec::new();
    fn go(
        el: &Element,
        (dx, dy, sc): (f64, f64, f64),
        ratios: (f64, f64, f64),
        labels: &mut Vec<Label>,
        shapes: &mut Vec<Shape>,
    ) {
        if is_generated(el) || CONTAINERS.contains(&el.tag.as_str()) {
            return;
        }
        let t = el.get("transform").unwrap_or("");
        if t.contains("rotate") {
            return;
        }
        let (dx, dy, sc) = apply_transform(t, dx, dy, sc);
        let names: Vec<String> = el.classes().iter().map(|s| (*s).to_owned()).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let base = classes::ordered(&refs)
            .into_iter()
            .find(|c| c.kind != Kind::Modifier);
        let (cap, desc, default_fs) = ratios;
        let at = |x: f64, y: f64| (x * sc + dx, y * sc + dy);
        match el.tag.as_str() {
            "text" => {
                let content = el.text().trim().to_owned();
                if !content.is_empty() {
                    let fs = el
                        .get("font-size")
                        .and_then(parse_float)
                        .unwrap_or(default_fs)
                        * sc;
                    let (x, y) = at(num(el, "x"), num(el, "y"));
                    let w = text::width(&content, fs);
                    let x0 = match el.get("text-anchor").unwrap_or("start") {
                        "middle" => x - w / 2.0,
                        "end" => x - w,
                        _ => x,
                    };
                    labels.push(Label {
                        rect: (x0, y - fs * cap, x0 + w, y + fs * desc),
                        text: content,
                        classes: names.clone(),
                    });
                }
            }
            "rect" | "circle" | "ellipse" if base.is_some_and(|b| b.kind == Kind::Shape) => {
                let rect = match el.tag.as_str() {
                    "rect" => {
                        let (x, y) = at(num(el, "x"), num(el, "y"));
                        (x, y, x + num(el, "width") * sc, y + num(el, "height") * sc)
                    }
                    "circle" => {
                        let (cx, cy) = at(num(el, "cx"), num(el, "cy"));
                        let r = num(el, "r") * sc;
                        (cx - r, cy - r, cx + r, cy + r)
                    }
                    _ => {
                        let (cx, cy) = at(num(el, "cx"), num(el, "cy"));
                        let (rx, ry) = (num(el, "rx") * sc, num(el, "ry") * sc);
                        (cx - rx, cy - ry, cx + rx, cy + ry)
                    }
                };
                shapes.push(Shape {
                    rect,
                    classes: names.clone(),
                });
            }
            _ => {}
        }
        for ch in el.elements() {
            go(ch, (dx, dy, sc), ratios, labels, shapes);
        }
    }
    go(
        root,
        (0.0, 0.0, 1.0),
        (cap, desc, default_fs),
        &mut labels,
        &mut shapes,
    );
    (labels, shapes)
}

fn area(r: Rect) -> f64 {
    (r.2 - r.0) * (r.3 - r.1)
}

/// テキストを含む図形のうち、最も小さいもの。**テキストの中心が図形の内側に在るものから選ぶ。**
fn container<'a>(label: &Label, shapes: &'a [Shape]) -> Option<&'a Shape> {
    let (cx, cy) = (
        (label.rect.0 + label.rect.2) / 2.0,
        (label.rect.1 + label.rect.3) / 2.0,
    );
    shapes
        .iter()
        .filter(|s| s.rect.0 < cx && cx < s.rect.2 && s.rect.1 < cy && cy < s.rect.3)
        .min_by(|a, b| area(a.rect).total_cmp(&area(b.rect)))
}

/// 値を精度の刻みへ丸める。**境目の比較が、浮動小数の誤差で揺れないようにする。**
fn round_to(v: f64, precision: f64) -> f64 {
    if precision <= 0.0 {
        return v;
    }
    (v / precision).round() * precision
}

/// style 属性から、長さ（px）を読む。
fn style_px(root: &Element, key: &str) -> Option<f64> {
    let style = root.get("style")?;
    let v = declarations(style).into_iter().find(|(k, _)| k == key)?.1;
    parse_float(v.trim_end_matches("px"))
}

/// スマホ幅で描画したときの幅（px）── スマホ幅か、図が指定した min-width の大きいほう。
/// max-width と、数で書いた width がそれより狭ければ、そちらで描画される。
#[must_use]
pub fn rendered_width(root: &Element) -> f64 {
    let phone = classes::notation().checks.phone_width;
    let mut w = phone.max(style_px(root, "min-width").unwrap_or_default());
    if let Some(max) = style_px(root, "max-width") {
        w = w.min(max);
    }
    if let Some(attr) = root.get("width").filter(|v| !v.ends_with('%')) {
        if let Some(fixed) = parse_float(attr.trim_end_matches("px")) {
            w = w.min(fixed);
        }
    }
    w
}

/// viewBox の幅。
fn viewbox_width(root: &Element) -> Option<f64> {
    let parts: Vec<f64> = root
        .get("viewBox")?
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter_map(parse_float)
        .collect();
    let [_, _, w, _] = parts[..] else { return None };
    Some(w)
}

/// スマホ幅で描画したときの最小のフォントサイズ（px）。**テキストが無ければ `None`。**
#[must_use]
pub fn min_font_px(svg: &str) -> Option<f64> {
    let root = xml::parse(svg)?;
    let vw = viewbox_width(&root)?;
    let scale = rendered_width(&root) / vw;
    let mut least: Option<f64> = None;
    fn go(el: &Element, sc: f64, default_fs: f64, least: &mut Option<f64>) {
        if is_generated(el) || CONTAINERS.contains(&el.tag.as_str()) {
            return;
        }
        let (_, _, sc) = apply_transform(el.get("transform").unwrap_or(""), 0.0, 0.0, sc);
        if el.tag == "text" && !el.text().trim().is_empty() {
            let fs = el
                .get("font-size")
                .and_then(parse_float)
                .unwrap_or(default_fs)
                * sc;
            *least = Some(least.map_or(fs, |l: f64| l.min(fs)));
        }
        for ch in el.elements() {
            go(ch, sc, default_fs, least);
        }
    }
    go(&root, scale, theme_num("font.size"), &mut least);
    let precision = classes::notation().checks.precision;
    least.map(|v| round_to(v, precision))
}

/// 検査4 ── テキストの重なり ・ キャンバスやボックスからのはみ出し ・ スマホ幅の最小のフォントサイズ。
/// 解決のあとの SVG に適用する。
///
/// # Errors
///
/// SVG として読めないときに返す。
pub fn check_layout(svg: &str) -> Result<Vec<String>, String> {
    let root = parse(svg)?;
    let mut out: Vec<String> = verify::all(svg)
        .into_iter()
        .map(|f| format!("検査4 {f}"))
        .collect();
    let precision = classes::notation().checks.precision;
    let (labels, shapes) = gather(&root);
    for l in &labels {
        let Some(c) = container(l, &shapes) else {
            continue;
        };
        let (a, b) = (l.rect, c.rect);
        let out_by = (b.0 - a.0).max(a.2 - b.2).max(b.1 - a.1).max(a.3 - b.3);
        if round_to(out_by, precision) > 0.0 {
            out.push(format!(
                "検査4 文字がボックスからはみ出す: {}（{} はみ出す）",
                quote(&l.text),
                crate::py::fixed(out_by, 1)
            ));
        }
    }
    let threshold = classes::notation().checks.min_font_px;
    if let Some(least) = min_font_px(svg) {
        if least < threshold {
            out.push(format!(
                "検査4 スマホ幅（{}px）で描画すると、最小のフォントサイズが {}px になる（しきい値 {}px）",
                crate::py::fixed(rendered_width(&root), 0),
                crate::py::fixed(least, 2),
                crate::py::fixed(threshold, 2)
            ));
        }
    }
    Ok(out)
}

/// 目視で確かめる一覧。
#[derive(Debug, Default, Clone)]
pub struct Labels {
    /// class から、その class を持つテキストと、その class を持つ図形の中のテキストへ。
    pub by_class: BTreeMap<String, Vec<String>>,
    /// 同じボックスの中の複数の行を、上から連結したもの。
    pub lines: Vec<String>,
}

/// class ごとのラベルの一覧と、複数行のラベルの一覧を返す。**合否ではなく、目視のための出力である。**
///
/// # Errors
///
/// SVG として読めないときに返す。
pub fn labels(svg: &str) -> Result<Labels, String> {
    let root = parse(svg)?;
    let (labels, shapes) = gather(&root);
    let mut out = Labels::default();
    let mut boxes: Vec<(usize, Vec<&Label>)> = Vec::new();
    for l in &labels {
        let mut names: Vec<&str> = l.classes.iter().map(String::as_str).collect();
        let holder = shapes
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                let (cx, cy) = ((l.rect.0 + l.rect.2) / 2.0, (l.rect.1 + l.rect.3) / 2.0);
                s.rect.0 < cx && cx < s.rect.2 && s.rect.1 < cy && cy < s.rect.3
            })
            .min_by(|a, b| area(a.1.rect).total_cmp(&area(b.1.rect)));
        if let Some((i, s)) = holder {
            names.extend(s.classes.iter().map(String::as_str));
            match boxes.iter_mut().find(|(k, _)| *k == i) {
                Some((_, v)) => v.push(l),
                None => boxes.push((i, vec![l])),
            }
        }
        for c in classes::ordered(&names) {
            let list = out.by_class.entry(c.name.clone()).or_default();
            if !list.contains(&l.text) {
                list.push(l.text.clone());
            }
        }
    }
    for (_, mut ls) in boxes {
        if ls.len() < 2 {
            continue;
        }
        ls.sort_by(|a, b| a.rect.1.total_cmp(&b.rect.1));
        out.lines.push(
            ls.iter()
                .map(|l| l.text.as_str())
                .collect::<Vec<_>>()
                .join(LINE_JOIN),
        );
    }
    Ok(out)
}
