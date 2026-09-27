// SPDX-License-Identifier: MIT
//! 座標まわりの共通の道具 ── **同じ計算を複数の場所で書かないための置き場。**
//!
//! 同じ概念が別々に実装されていると、片方だけ直したときに気づけない ── 実測 ── 「折れ線を
//! 細かく刻む」が2か所、「線分が箱を通るか」が2か所、「原点を左上へ寄せる」が5か所に在り、
//! 刻みの細かさや余裕の取り方が場所ごとに違っていた。
//!
//! **検査（verify）はここを使わない。** 検査が計算と同じコードを使うと、その計算の誤りを
//! 検査が永久に見つけられなくなる。
//!
//! **冪は冪のまま計算する** ── 2乗を掛け算へ、平方根を専用の手順へ置き換えると、最後の桁が
//! 移す前と変わり、同じ宣言から別の座標が出る。

use std::sync::OnceLock;

use regex::Regex;

use crate::xml::{self, Element};

/// 点。
pub type Point = (f64, f64);
/// 矩形。`(x0, y0, x1, y1)`
pub type Rect = (f64, f64, f64, f64);

/// 標本の粗さ ── 最も小さい辺を何等分するか。**検査側も同じ粗さを使う** ── 描く側と検査側で
/// 粗さが違うと、片方だけが見つける破綻ができる。
pub const SAMPLE_DIVISOR: f64 = 4.0;

/// 多角形として閉じるのに要る最小の頂点数。
pub const MIN_POLYGON: usize = 3;

/// 0で割らないための下限。
const TINY: f64 = 1e-9;

/// 2点の隔たり。
#[must_use]
pub fn dist(a: Point, b: Point) -> f64 {
    ((b.0 - a.0).powf(2.0) + (b.1 - a.1).powf(2.0)).powf(0.5)
}

/// 折れ線の途中も含めて点を収集する。**刻みは呼ぶ側が渡す** ── 既定値を置かない。
#[must_use]
pub fn densify(pts: &[Point], step: f64) -> Vec<Point> {
    if pts.len() < 2 {
        return pts.to_vec();
    }
    let mut out = vec![pts[0]];
    for w in pts.windows(2) {
        let ((x1, y1), (x2, y2)) = (w[0], w[1]);
        let d = ((x2 - x1).powf(2.0) + (y2 - y1).powf(2.0)).powf(0.5);
        let n = ((d / step.max(TINY)) as i64).max(1);
        for i in 1..=n {
            let t = i as f64 / n as f64;
            out.push((x1 + (x2 - x1) * t, y1 + (y2 - y1) * t));
        }
    }
    out
}

/// 線分が矩形の内側を通るか。**縁に触れるのは通ったとみなさない。**
#[must_use]
pub fn segment_hits_rect(p0: Point, p1: Point, rect: Rect, margin: f64) -> bool {
    let (x0, y0, x1, y1) = (
        rect.0 + margin,
        rect.1 + margin,
        rect.2 - margin,
        rect.3 - margin,
    );
    if x1 <= x0 || y1 <= y0 {
        return false;
    }
    let step = (x1 - x0).min(y1 - y0) / SAMPLE_DIVISOR;
    densify(&[p0, p1], step.max(TINY))
        .iter()
        .any(|(x, y)| x0 < *x && *x < x1 && y0 < *y && *y < y1)
}

/// 2つの矩形が重なるか。**縁で接するだけは重なりとみなさない。**
#[must_use]
pub fn rects_overlap(a: Rect, b: Rect) -> bool {
    a.0 < b.2 && a.2 > b.0 && a.1 < b.3 && a.3 > b.1
}

/// 左上が原点に来るよう、全体を平行移動する。**描く側は座標が0から始まる前提で画布を決める。**
#[must_use]
pub fn shift_to_origin(points: &[(String, Point)]) -> (Vec<(String, Point)>, Point) {
    if points.is_empty() {
        return (Vec::new(), (0.0, 0.0));
    }
    let off_x = points
        .iter()
        .map(|(_, p)| p.0)
        .fold(f64::INFINITY, f64::min);
    let off_y = points
        .iter()
        .map(|(_, p)| p.1)
        .fold(f64::INFINITY, f64::min);
    if off_x == 0.0 && off_y == 0.0 {
        return (points.to_vec(), (0.0, 0.0));
    }
    (
        points
            .iter()
            .map(|(k, (x, y))| (k.clone(), (x - off_x, y - off_y)))
            .collect(),
        (off_x, off_y),
    )
}

fn number_pattern() -> &'static Regex {
    static ONCE: OnceLock<Regex> = OnceLock::new();
    ONCE.get_or_init(|| Regex::new(r"-?\d*\.?\d+(?:[eE][-+]?\d+)?").expect("式である"))
}

/// 文字から数を全部拾う。
#[must_use]
pub fn numbers(text: &str) -> Vec<f64> {
    number_pattern()
        .find_iter(text)
        .filter_map(|m| m.as_str().parse().ok())
        .collect()
}

/// 円弧を点列にする。**SVG の終点指定を中心指定へ直してから刻む。**
///
/// 刻む数は弧の長さと刻み幅から決める ── 決め打ちにすると、大きな弧では点と点の間が開き、
/// その間の向きが輪郭から欠落する（実測 ── 半径52の輪の輪郭が、向きによって60〜64.5と
/// ばらついた）。
#[allow(clippy::too_many_arguments, clippy::many_single_char_names)]
#[must_use]
pub fn arc_points(
    p0: Point,
    rx: f64,
    ry: f64,
    rot: f64,
    large: i64,
    sweep: i64,
    p1: Point,
    step: f64,
) -> Vec<Point> {
    use std::f64::consts::PI;
    if rx == 0.0 || ry == 0.0 {
        return vec![p1];
    }
    let phi = rot.to_radians();
    let (cos_p, sin_p) = (phi.cos(), phi.sin());
    let (dx2, dy2) = ((p0.0 - p1.0) / 2.0, (p0.1 - p1.1) / 2.0);
    let x1 = cos_p * dx2 + sin_p * dy2;
    let y1 = -sin_p * dx2 + cos_p * dy2;
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    let lam = x1 * x1 / (rx * rx) + y1 * y1 / (ry * ry);
    if lam > 1.0 {
        rx *= lam.powf(0.5);
        ry *= lam.powf(0.5);
    }
    let num = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1;
    let den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let mut coef = if den != 0.0 {
        (num.max(0.0) / den).powf(0.5)
    } else {
        0.0
    };
    if large == sweep {
        coef = -coef;
    }
    let cx1 = coef * rx * y1 / ry;
    let cy1 = -coef * ry * x1 / rx;
    let cx = cos_p * cx1 - sin_p * cy1 + (p0.0 + p1.0) / 2.0;
    let cy = sin_p * cx1 + cos_p * cy1 + (p0.1 + p1.1) / 2.0;
    let a0 = ((y1 - cy1) / ry).atan2((x1 - cx1) / rx);
    let a1 = ((-y1 - cy1) / ry).atan2((-x1 - cx1) / rx);
    let mut sweep_angle = a1 - a0;
    if sweep != 0 && sweep_angle < 0.0 {
        sweep_angle += 2.0 * PI;
    } else if sweep == 0 && sweep_angle > 0.0 {
        sweep_angle -= 2.0 * PI;
    }
    let steps = ((sweep_angle.abs() * rx.max(ry) / step.max(TINY)) as i64 + 1).max(2);
    (1..=steps)
        .map(|i| {
            let a = a0 + sweep_angle * i as f64 / steps as f64;
            (
                cos_p * rx * a.cos() - sin_p * ry * a.sin() + cx,
                sin_p * rx * a.cos() + cos_p * ry * a.sin() + cy,
            )
        })
        .collect()
}

/// 制御点をたどった長さから、その曲線を何分割するかを決める。
fn curve_steps(control: &[Point], step: f64) -> i64 {
    let length = crate::py::sum(
        control
            .windows(2)
            .map(|w| ((w[1].0 - w[0].0).powf(2.0) + (w[1].1 - w[0].1).powf(2.0)).powf(0.5)),
    );
    ((length / step.max(TINY)) as i64 + 1).max(2)
}

fn command_pattern() -> &'static Regex {
    static ONCE: OnceLock<Regex> = OnceLock::new();
    ONCE.get_or_init(|| {
        Regex::new(r"([MmLlHhVvCcSsQqTtAaZz])([^MmLlHhVvCcSsQqTtAaZz]*)").expect("式である")
    })
}

/// path の d を点列にする。**曲線は制御点ではなく曲線上を刻む** ── 制御点を輪郭に混ぜると、
/// 実際にはインクが無いところまで輪郭が張り出す。
#[allow(clippy::too_many_lines, clippy::many_single_char_names)]
#[must_use]
pub fn sample_path(d: &str, step: f64) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::new();
    let mut cur = (0.0, 0.0);
    let mut start = (0.0, 0.0);
    let mut prev_c: Option<Point> = None;
    let mut prev_q: Option<Point> = None;
    for m in command_pattern().captures_iter(d) {
        let cmd = m[1].chars().next().unwrap_or('Z');
        let nums = numbers(&m[2]);
        let rel = cmd.is_ascii_lowercase();
        let mut up = cmd.to_ascii_uppercase();
        let mut i = 0;
        let mut first = true;
        loop {
            let need = match up {
                'M' | 'L' | 'T' => 2,
                'H' | 'V' => 1,
                'C' => 6,
                'S' | 'Q' => 4,
                'A' => 7,
                _ => 0,
            };
            if up == 'Z' {
                if !out.is_empty() {
                    out.push(start);
                }
                cur = start;
                break;
            }
            if i + need > nums.len() {
                break;
            }
            let v = &nums[i..i + need];
            i += need;
            let (ox, oy) = if rel { cur } else { (0.0, 0.0) };
            match up {
                'M' => {
                    cur = (v[0] + ox, v[1] + oy);
                    if first {
                        start = cur;
                    }
                    out.push(cur);
                    up = 'L'; // 2つ目以降の座標対は暗黙の L
                }
                'L' => {
                    cur = (v[0] + ox, v[1] + oy);
                    out.push(cur);
                }
                'H' => {
                    cur = (v[0] + ox, cur.1);
                    out.push(cur);
                }
                'V' => {
                    cur = (cur.0, v[0] + oy);
                    out.push(cur);
                }
                'C' | 'S' => {
                    let (c1, c2, end) = if up == 'C' {
                        (
                            (v[0] + ox, v[1] + oy),
                            (v[2] + ox, v[3] + oy),
                            (v[4] + ox, v[5] + oy),
                        )
                    } else {
                        let c1 = prev_c.map_or(cur, |p| (2.0 * cur.0 - p.0, 2.0 * cur.1 - p.1));
                        (c1, (v[0] + ox, v[1] + oy), (v[2] + ox, v[3] + oy))
                    };
                    let n = curve_steps(&[cur, c1, c2, end], step);
                    for k in 1..=n {
                        let t = k as f64 / n as f64;
                        let u = 1.0 - t;
                        out.push((
                            u.powf(3.0) * cur.0
                                + 3.0 * u * u * t * c1.0
                                + 3.0 * u * t * t * c2.0
                                + t.powf(3.0) * end.0,
                            u.powf(3.0) * cur.1
                                + 3.0 * u * u * t * c1.1
                                + 3.0 * u * t * t * c2.1
                                + t.powf(3.0) * end.1,
                        ));
                    }
                    prev_c = Some(c2);
                    prev_q = None;
                    cur = end;
                }
                'Q' | 'T' => {
                    let (c1, end) = if up == 'Q' {
                        ((v[0] + ox, v[1] + oy), (v[2] + ox, v[3] + oy))
                    } else {
                        let c1 = prev_q.map_or(cur, |p| (2.0 * cur.0 - p.0, 2.0 * cur.1 - p.1));
                        (c1, (v[0] + ox, v[1] + oy))
                    };
                    let n = curve_steps(&[cur, c1, end], step);
                    for k in 1..=n {
                        let t = k as f64 / n as f64;
                        let u = 1.0 - t;
                        out.push((
                            u * u * cur.0 + 2.0 * u * t * c1.0 + t * t * end.0,
                            u * u * cur.1 + 2.0 * u * t * c1.1 + t * t * end.1,
                        ));
                    }
                    prev_q = Some(c1);
                    prev_c = None;
                    cur = end;
                }
                'A' => {
                    let end = (v[5] + ox, v[6] + oy);
                    out.extend(arc_points(
                        cur,
                        v[0],
                        v[1],
                        v[2],
                        v[3] as i64,
                        v[4] as i64,
                        end,
                        step,
                    ));
                    prev_c = None;
                    prev_q = None;
                    cur = end;
                }
                _ => {}
            }
            if up != 'C' && up != 'S' {
                prev_c = None;
            }
            if up != 'Q' && up != 'T' {
                prev_q = None;
            }
            first = false;
        }
    }
    out
}

fn translate_pattern() -> &'static Regex {
    static ONCE: OnceLock<Regex> = OnceLock::new();
    ONCE.get_or_init(|| {
        Regex::new(r"translate\(\s*(-?[\d.]+)\s*[, ]\s*(-?[\d.]+)\s*\)").expect("式である")
    })
}

fn scale_pattern() -> &'static Regex {
    static ONCE: OnceLock<Regex> = OnceLock::new();
    ONCE.get_or_init(|| Regex::new(r"scale\(\s*(-?[\d.]+)\s*\)").expect("式である"))
}

/// 変換を読み、平行移動と拡大を重ねる。**検査も同じ読み方をする。**
#[must_use]
pub fn apply_transform(t: &str, dx: f64, dy: f64, s: f64) -> (f64, f64, f64) {
    let (mut dx, mut dy, mut s) = (dx, dy, s);
    if let Some(m) = translate_pattern().captures(t) {
        dx += m[1].parse::<f64>().unwrap_or(0.0) * s;
        dy += m[2].parse::<f64>().unwrap_or(0.0) * s;
    }
    if let Some(m) = scale_pattern().captures(t) {
        s *= m[1].parse::<f64>().unwrap_or(1.0);
    }
    (dx, dy, s)
}

/// 属性を数として読む。**読めなければ既定値。**
#[must_use]
pub fn attr_num(el: &Element, name: &str, default: f64) -> f64 {
    el.get(name)
        .and_then(crate::py::parse_float)
        .unwrap_or(default)
}

/// 部品が描いた SVG 断片から、インクの通る点を収集する。
///
/// **文字は含めない** ── 名前は形の内側に置かれるもので、輪郭を広げる役目を持たない。
/// 返すのは `(点, その点でインクが中心線からどれだけ外へ及ぶか)` の並びである ── 線は幅を
/// 持つので、中心線だけを輪郭にすると線の太さの半分だけ内側になる。
#[must_use]
pub fn sample_ink(fragment: &str, step: f64) -> Vec<(Point, f64)> {
    let Some(root) = xml::parse(&format!("<g>{fragment}</g>")) else {
        return Vec::new();
    };
    let mut pts = Vec::new();
    walk_ink(&root, 0.0, 0.0, 1.0, step, &mut pts);
    pts
}

fn walk_ink(el: &Element, dx: f64, dy: f64, s: f64, step: f64, pts: &mut Vec<(Point, f64)>) {
    use std::f64::consts::PI;
    let (dx, dy, s) = apply_transform(el.get("transform").unwrap_or(""), dx, dy, s);
    let mut local: Vec<Point> = Vec::new();
    match el.tag.as_str() {
        "rect" => {
            let (x, y) = (attr_num(el, "x", 0.0), attr_num(el, "y", 0.0));
            let (w, h) = (attr_num(el, "width", 0.0), attr_num(el, "height", 0.0));
            if w != 0.0 && h != 0.0 {
                local = vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h), (x, y)];
            }
        }
        "polygon" | "polyline" => {
            let v = numbers(el.get("points").unwrap_or(""));
            local = v.as_chunks::<2>().0.iter().map(|c| (c[0], c[1])).collect();
            if el.tag == "polygon" && !local.is_empty() {
                local.push(local[0]);
            }
        }
        "path" => local = sample_path(el.get("d").unwrap_or(""), step),
        "line" => {
            local = vec![
                (attr_num(el, "x1", 0.0), attr_num(el, "y1", 0.0)),
                (attr_num(el, "x2", 0.0), attr_num(el, "y2", 0.0)),
            ];
        }
        "circle" | "ellipse" => {
            let (cx, cy) = (attr_num(el, "cx", 0.0), attr_num(el, "cy", 0.0));
            let r = attr_num(el, "r", 0.0);
            let rx = if r != 0.0 { r } else { attr_num(el, "rx", 0.0) };
            let ry = if r != 0.0 { r } else { attr_num(el, "ry", 0.0) };
            // 刻み幅に見合う数へ分ける。**最低でも三角形にはする**
            let n = ((2.0 * PI * rx.max(ry) / step.max(TINY)) as usize + 1).max(MIN_POLYGON);
            local = (0..=n)
                .map(|k| {
                    let a = 2.0 * PI * k as f64 / n as f64;
                    (cx + rx * a.cos(), cy + ry * a.sin())
                })
                .collect();
        }
        _ => {}
    }
    let halo = match el.get("stroke") {
        Some(s_) if !s_.is_empty() && s_ != "none" => attr_num(el, "stroke-width", 1.0) / 2.0 * s,
        _ => 0.0,
    };
    let pts_local = if local.len() > 1 {
        densify(&local, step)
    } else {
        local
    };
    for (px, py) in pts_local {
        pts.push(((px * s + dx, py * s + dy), halo));
    }
    for ch in el.elements() {
        walk_ink(ch, dx, dy, s, step, pts);
    }
}

/// 部品が描いたインクの「表面」の点を返す。**線は、中心から見て外へ線幅の半分だけ押し出す。**
///
/// 輪郭（閉じた多角形）を作らないのは、作れない形があるため ── 波やチェック記号のように
/// インクが角度方向に途切れる形では、輪郭の頂点を結んだ弦がインクの無いところをまたぐ。
#[must_use]
pub fn ink_surface(fragment: &str, width: f64, height: f64, fineness: i64) -> Vec<Point> {
    let step = width.min(height).max(1.0) / fineness.max(1) as f64;
    let (cx, cy) = (width / 2.0, height / 2.0);
    let mut out = Vec::new();
    for ((px, py), halo) in sample_ink(fragment, step) {
        let (mut px, mut py) = (px, py);
        if halo != 0.0 {
            let (dx, dy) = (px - cx, py - cy);
            let r = (dx * dx + dy * dy).powf(0.5);
            if r > 0.0 {
                px = cx + dx / r * (r + halo);
                py = cy + dy / r * (r + halo);
            }
        }
        out.push((px, py));
    }
    if !out.is_empty() {
        return out;
    }
    let bx = [
        (0.0, 0.0),
        (width, 0.0),
        (width, height),
        (0.0, height),
        (0.0, 0.0),
    ];
    densify(&bx, step)
}

/// 狙った場所にいちばん近い点を返す。**同じ近さなら、先に在るものを選ぶ。**
#[must_use]
pub fn nearest(points: &[Point], aim: Point) -> Point {
    let mut best: Option<(f64, Point)> = None;
    for p in points {
        let d = (p.0 - aim.0).powf(2.0) + (p.1 - aim.1).powf(2.0);
        if best.is_none_or(|(b, _)| d < b) {
            best = Some((d, *p));
        }
    }
    best.map_or(aim, |(_, p)| p)
}
