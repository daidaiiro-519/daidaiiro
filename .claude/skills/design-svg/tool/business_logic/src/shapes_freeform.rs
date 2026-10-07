// SPDX-License-Identifier: MIT
//! 自由曲線 ── ベジェも円弧も含む、任意のパスデータをそのまま渡せる部品。
//!
//! 「登録済みの部品の組み合わせでしか表現できない」という制約への回答が、完成された形の部品を
//! 1つ足すことではなく、**任意の形そのものを渡せるこの部品を足すこと**である。
//!
//! **受けるのは SVG のパスの文法そのものである** ── 描く側は座標を書き換えず、外接矩形ぶん
//! だけ平行移動して置く。

use std::sync::OnceLock;

use regex::Regex;

use crate::boolean::boolean_op;
use crate::props;
use crate::py::{self, float as f};
use crate::registry::{Component, Fragment, Props};
use crate::style::Style;

/// 円弧を外接矩形へ含めるときの刻み（度）。**細かくするほど外接矩形は小さくなる。**
const ARC_STEP_DEG: f64 = 5.0;

/// 台帳へ載せる部品。
#[must_use]
pub fn register() -> Vec<(&'static str, Component)> {
    vec![("path", path as Component), ("boolean", boolean)]
}

fn number_pattern() -> &'static Regex {
    static ONCE: OnceLock<Regex> = OnceLock::new();
    ONCE.get_or_init(|| Regex::new(r"[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?").expect("式である"))
}

fn command_pattern() -> &'static Regex {
    static ONCE: OnceLock<Regex> = OnceLock::new();
    ONCE.get_or_init(|| {
        Regex::new(r"([MmLlHhVvCcSsQqTtAaZz])([^MmLlHhVvCcSsQqTtAaZz]*)").expect("式である")
    })
}

fn numbers(chunk: &str) -> Vec<f64> {
    number_pattern()
        .find_iter(chunk)
        .filter_map(|m| py::parse_float(m.as_str()))
        .collect()
}

/// コマンドが1回に取る引数の個数。**SVG のパスの文法が決めている** ── 選んだ値ではない。
const fn arity(up: char) -> usize {
    match up {
        'M' | 'L' | 'T' => 2,
        'H' | 'V' => 1,
        'C' => 6,
        'S' | 'Q' => 4,
        'A' => 7,
        _ => 0,
    }
}

/// 円弧の通る点を刻んで返す。**外接矩形を出すためだけに使う** ── 弧は端点の外側へ膨らむ。
#[allow(clippy::too_many_arguments, clippy::similar_names)]
fn arc_points(
    x0: f64,
    y0: f64,
    rx: f64,
    ry: f64,
    rot: f64,
    large: f64,
    sweep: f64,
    x1: f64,
    y1: f64,
) -> Vec<(f64, f64)> {
    use std::f64::consts::PI;
    if rx == 0.0 || ry == 0.0 || (x0, y0) == (x1, y1) {
        return vec![(x1, y1)];
    }
    let phi = rot.to_radians();
    let (cos_p, sin_p) = (phi.cos(), phi.sin());
    let (dx2, dy2) = ((x0 - x1) / 2.0, (y0 - y1) / 2.0);
    let (x1p, y1p) = (cos_p * dx2 + sin_p * dy2, -sin_p * dx2 + cos_p * dy2);
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    let lam = (x1p / rx).powf(2.0) + (y1p / ry).powf(2.0);
    if lam > 1.0 {
        rx *= lam.sqrt();
        ry *= lam.sqrt();
    }
    let num =
        rx.powf(2.0) * ry.powf(2.0) - rx.powf(2.0) * y1p.powf(2.0) - ry.powf(2.0) * x1p.powf(2.0);
    let den = rx.powf(2.0) * y1p.powf(2.0) + ry.powf(2.0) * x1p.powf(2.0);
    let mut coef = if den != 0.0 {
        (num.max(0.0) / den).sqrt()
    } else {
        0.0
    };
    if large == sweep {
        coef = -coef;
    }
    let (cxp, cyp) = (coef * rx * y1p / ry, -coef * ry * x1p / rx);
    let cx = cos_p * cxp - sin_p * cyp + (x0 + x1) / 2.0;
    let cy = sin_p * cxp + cos_p * cyp + (y0 + y1) / 2.0;
    let start = ((y1p - cyp) / ry).atan2((x1p - cxp) / rx);
    let end = ((-y1p - cyp) / ry).atan2((-x1p - cxp) / rx);
    let mut sweep_angle = end - start;
    if sweep != 0.0 && sweep_angle < 0.0 {
        sweep_angle += 2.0 * PI;
    }
    if sweep == 0.0 && sweep_angle > 0.0 {
        sweep_angle -= 2.0 * PI;
    }
    let steps = ((sweep_angle.abs() / ARC_STEP_DEG.to_radians()) as i64).max(1);
    (0..=steps)
        .map(|i| {
            let t = start + sweep_angle * i as f64 / steps as f64;
            (
                cx + rx * t.cos() * cos_p - ry * t.sin() * sin_p,
                cy + rx * t.cos() * sin_p + ry * t.sin() * cos_p,
            )
        })
        .collect()
}

/// パスが通る点を、絶対座標で返す。
///
/// **曲線は制御点も含める** ── 見積りは実物以上でなければならない。少なく見積もると、申告した
/// 大きさの外へインクが出て、置いた側が重ねてしまう。
///
/// # Errors
///
/// 文法に無いコマンドが在るときと、引数の数が合わないときと、読めない断片が在るときに返す。
pub fn path_points(d: &str) -> Result<Vec<(f64, f64)>, String> {
    let mut unknown: Vec<char> = d
        .chars()
        .filter(|c| {
            c.is_ascii_alphabetic()
                && *c != 'e'
                && *c != 'E'
                && !"MmLlHhVvCcSsQqTtAaZz".contains(*c)
        })
        .collect();
    unknown.sort_unstable();
    unknown.dedup();
    if !unknown.is_empty() {
        let shown: Vec<String> = unknown.iter().map(|c| py::quote(&c.to_string())).collect();
        return Err(format!(
            "パスの文法に無いコマンドがある: [{}]",
            shown.join(", ")
        ));
    }
    let mut out: Vec<(f64, f64)> = Vec::new();
    let (mut x, mut y) = (0.0, 0.0);
    let (mut start_x, mut start_y) = (0.0, 0.0);
    let mut pos = 0;
    for m in command_pattern().captures_iter(d) {
        let whole = m.get(0).map_or(0..0, |g| g.range());
        if whole.start != pos && !d[pos..whole.start].trim().is_empty() {
            return Err(format!(
                "パスとして読めない断片がある: {}",
                py::quote(d[pos..whole.start].trim())
            ));
        }
        pos = whole.end;
        let cmd = m[1].chars().next().unwrap_or('Z');
        let up = cmd.to_ascii_uppercase();
        let rel = cmd.is_ascii_lowercase();
        let nums = numbers(&m[2]);
        let n = arity(up);
        if up == 'Z' {
            x = start_x;
            y = start_y;
            out.push((x, y));
            continue;
        }
        if nums.is_empty() || !nums.len().is_multiple_of(n) {
            let shown: Vec<String> = nums.iter().map(|v| f(*v)).collect();
            return Err(format!(
                "{cmd} の引数の数が {n} の倍数になっていない: [{}]",
                shown.join(", ")
            ));
        }
        for a in nums.chunks(n) {
            match up {
                'H' => x = if rel { x + a[0] } else { a[0] },
                'V' => y = if rel { y + a[0] } else { a[0] },
                'A' => {
                    let (nx, ny) = if rel {
                        (x + a[5], y + a[6])
                    } else {
                        (a[5], a[6])
                    };
                    out.extend(arc_points(x, y, a[0], a[1], a[2], a[3], a[4], nx, ny));
                    x = nx;
                    y = ny;
                    continue;
                }
                _ => {
                    for pair in a.chunks(2) {
                        let (px, py_) = (pair[0], pair[1]);
                        out.push(if rel { (x + px, y + py_) } else { (px, py_) });
                    }
                    if let Some(last) = out.last() {
                        x = last.0;
                        y = last.1;
                    }
                    if up == 'M' {
                        start_x = x;
                        start_y = y;
                    }
                }
            }
            out.push((x, y));
        }
    }
    if pos != d.len() && !d[pos..].trim().is_empty() {
        return Err(format!(
            "パスとして読めない断片がある: {}",
            py::quote(d[pos..].trim())
        ));
    }
    Ok(out)
}

/// パスの外接矩形 ── `(左, 上, 幅, 高さ)`。
///
/// # Errors
///
/// パスとして読めないときに返す。
pub fn path_bounds(d: &str) -> Result<(f64, f64, f64, f64), String> {
    let pts = path_points(d)?;
    if pts.is_empty() {
        return Ok((0.0, 0.0, 0.0, 0.0));
    }
    let x_lo = pts.iter().map(|q| q.0).fold(f64::INFINITY, f64::min);
    let x_hi = pts.iter().map(|q| q.0).fold(f64::NEG_INFINITY, f64::max);
    let y_lo = pts.iter().map(|q| q.1).fold(f64::INFINITY, f64::min);
    let y_hi = pts.iter().map(|q| q.1).fold(f64::NEG_INFINITY, f64::max);
    Ok((x_lo, y_lo, x_hi - x_lo, y_hi - y_lo))
}

/// 任意のパスデータをそのまま描く。**座標を書き換えない** ── 書き換えると、円弧や省略形の
/// 意味まで解釈し直すことになり、そこで形が壊れる。
fn path(p: &Props, style: &Style) -> Result<Fragment, String> {
    let filled = p.get("filled").is_none_or(props::truthy);
    let fill = if filled {
        style.text_or("color.shape-fill", &style.text("color.accent")?)?
    } else {
        "none".to_owned()
    };
    let stroke = style.text_or("color.shape-stroke", &style.text("color.accent")?)?;
    let sw = style.num("size.stroke-width")?;
    let d = props::need_text(p, "d")?;
    let class = if filled { "box focus" } else { "link focus" };
    let (x0, y0, w, h) = path_bounds(&d)?;
    let svg = format!(
        "<g transform=\"translate({:.2},{:.2})\"><path class=\"{class}\" d=\"{d}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{}\"/></g>",
        -x0,
        -y0,
        f(sw)
    );
    Ok(Fragment::own(svg, w, h))
}

/// 多角形どうしの和 ・ 積 ・ 差。
///
/// **結果の輪郭は1つのパスの中の別々の輪郭として置き、偶奇規則で塗る** ── 輪郭ごとにパスを
/// 分けると、内側が穴にならず塗り重なる。
fn boolean(p: &Props, style: &Style) -> Result<Fragment, String> {
    let shapes: Vec<Vec<(f64, f64)>> = props::list(p, "shapes").iter().map(props::points).collect();
    let op = props::need_text(p, "op")?;
    let polygons = boolean_op(&shapes, &op)?;
    let fill = style.text_or("color.shape-fill", &style.text("color.accent")?)?;
    // **結果の外接矩形の左上を (0,0) へ揃え直してから描く** ── 怠ると、描画結果が画布の外へ
    // 出て何も見えなくなる（実測で踏んだ不具合）
    let all: Vec<(f64, f64)> = polygons.iter().flatten().copied().collect();
    let x0 = all.iter().map(|q| q.0).reduce(f64::min).unwrap_or(0.0);
    let y0 = all.iter().map(|q| q.1).reduce(f64::min).unwrap_or(0.0);
    let subpaths: Vec<String> = polygons
        .iter()
        .map(|poly| {
            format!(
                "M{} Z",
                poly.iter()
                    .map(|(x, y)| format!("{:.1},{:.1}", x - x0, y - y0))
                    .collect::<Vec<_>>()
                    .join(" L")
            )
        })
        .collect();
    let parts = if subpaths.is_empty() {
        String::new()
    } else {
        format!(
            "<path class=\"box focus\" d=\"{}\" fill=\"{fill}\" fill-rule=\"evenodd\"/>",
            subpaths.join(" ")
        )
    };
    let w = all.iter().map(|q| q.0 - x0).reduce(f64::max).unwrap_or(0.0);
    let h = all.iter().map(|q| q.1 - y0).reduce(f64::max).unwrap_or(0.0);
    Ok(Fragment::own(format!("<g>{parts}</g>"), w, h))
}
