// SPDX-License-Identifier: MIT
//! 描いた結果の機械検査 ── **破綻しているかどうかを、絵を見ずに判定する。**
//!
//! 見るのは3種類:
//!
//! - [`check`] ── 文字どうしの重なりと、画布の外へのはみ出し
//! - [`check_shapes`] ── 箱どうしの重なり ・ 辺が箱を突っ切ること ・ 囲みの中途半端な交差
//! - [`check_attachment`] ── 辺の終端が、相手のインクに着いているか
//!
//! **共有してよいのは「SVG を読むこと」だけ。** 描く側と検査が同じ判断を共有すると、その判断の
//! 誤りを検査が永久に見つけられない。読み取り（path を点列にする ・ 折れ線を刻む）は判断では
//! ないので共有する。
//!
//! **閾値を定数で置かない。** 標本の粗さも内側と見なす余裕も、その図自身の寸法から毎回導く。
//!
//! **この検査が見ないもの**が3つある ── 極端な縦横比、配置戦略の選び違い、詰まり ・ 読み
//! にくさ ・ 配色の良し悪し。目視の代わりにはならない。

use std::collections::HashSet;
use std::f64::consts::PI;
use std::sync::OnceLock;

use regex::Regex;

use crate::geometry::{apply_transform, densify, sample_path, Point, MIN_POLYGON, SAMPLE_DIVISOR};
use crate::py::{parse_float, quote};
use crate::text;
use crate::xml::{self, Element};

/// 曲線を刻まず、命令の区切りの点だけを拾わせるための刻み幅。**大きさそのものに意味は無い。**
const VERTICES_ONLY: f64 = 1e9;

fn theme_num(key: &str) -> f64 {
    crate::theme::num(crate::theme::default_theme(), key).unwrap_or(0.0)
}

/// 読み取りの細かさ。**描く側が輪郭を何向きで表すかと同じ尺度に合わせる。**
fn fineness() -> f64 {
    (theme_num("size.outline-facets") as i64) as f64
}

/// 属性を小数として読む。**読めないときは `None`**（移す前は、そこで読むのをやめた）。
fn attr(el: &Element, name: &str, default: f64) -> Option<f64> {
    match el.get(name) {
        None => Some(default),
        Some(v) => parse_float(v),
    }
}

fn transform(el: &Element, dx: f64, dy: f64, sc: f64) -> (f64, f64, f64) {
    apply_transform(el.get("transform").unwrap_or(""), dx, dy, sc)
}

/// 文字の外接矩形を、祖先の変換を合成した絶対座標で返す。`(左, 上, 右, 下, 中身)`
fn text_boxes(svg: &str) -> Vec<(f64, f64, f64, f64, String)> {
    let Some(root) = xml::parse(svg) else {
        return Vec::new();
    };
    let cap = theme_num("font.cap-ratio");
    let desc = theme_num("font.descender-ratio");
    let default_fs = theme_num("font.size");
    let mut out = Vec::new();
    #[allow(clippy::too_many_arguments)] // 祖先から受け継ぐ状態を、そのまま引数で運ぶ
    fn walk(
        el: &Element,
        dx: f64,
        dy: f64,
        sc: f64,
        cap: f64,
        desc: f64,
        default_fs: f64,
        out: &mut Vec<(f64, f64, f64, f64, String)>,
    ) {
        let t = el.get("transform").unwrap_or("");
        let (dx, dy, sc) = apply_transform(t, dx, dy, sc);
        if el.tag.ends_with("text") {
            let content = el.text().trim().to_owned();
            if !content.is_empty() && !t.contains("rotate") {
                let (Some(x), Some(y), Some(fs)) = (
                    attr(el, "x", 0.0),
                    attr(el, "y", 0.0),
                    attr(el, "font-size", default_fs),
                ) else {
                    return;
                };
                let (x, y, fs) = (x * sc + dx, y * sc + dy, fs * sc);
                let w = text::width(&content, fs);
                let anchor = el.get("text-anchor").unwrap_or("start");
                let x0 = match anchor {
                    "middle" => x - w / 2.0,
                    "end" => x - w,
                    _ => x,
                };
                out.push((x0, y - fs * cap, x0 + w, y + fs * desc, content));
            }
        }
        for ch in el.elements() {
            walk(ch, dx, dy, sc, cap, desc, default_fs, out);
        }
    }
    walk(&root, 0.0, 0.0, 1.0, cap, desc, default_fs, &mut out);
    out
}

type Rect4 = (f64, f64, f64, f64);
/// 囲み ── `(左, 上, 右, 下, 線の太さ)`
type Frame = (f64, f64, f64, f64, f64);
/// 辺 ── `(通る点, 線の太さ)`
type Stroke = (Vec<Point>, f64);

/// 箱 ・ 囲み ・ 辺を、祖先の変換を合成した絶対座標で収集する。
fn shapes(svg: &str) -> (Vec<Rect4>, Vec<Frame>, Vec<Stroke>) {
    let Some(root) = xml::parse(svg) else {
        return (Vec::new(), Vec::new(), Vec::new());
    };
    let mut boxes = Vec::new();
    let mut frames = Vec::new();
    let mut paths = Vec::new();
    #[allow(clippy::too_many_arguments)] // 祖先から受け継ぐ状態を、そのまま引数で運ぶ
    fn walk(
        el: &Element,
        dx: f64,
        dy: f64,
        sc: f64,
        in_box: bool,
        boxes: &mut Vec<Rect4>,
        frames: &mut Vec<(f64, f64, f64, f64, f64)>,
        paths: &mut Vec<(Vec<Point>, f64)>,
    ) {
        let (dx, dy, sc) = transform(el, dx, dy, sc);
        let cls = el.get("class").unwrap_or("");
        let here_box = in_box || cls.contains("svg-box");
        if el.tag == "rect" {
            let read = (|| {
                Some((
                    attr(el, "x", 0.0)? * sc + dx,
                    attr(el, "y", 0.0)? * sc + dy,
                    attr(el, "width", 0.0)? * sc,
                    attr(el, "height", 0.0)? * sc,
                ))
            })();
            if let Some((x, y, w, h)) = read {
                if w != 0.0 {
                    if here_box {
                        boxes.push((x, y, x + w, y + h));
                    } else if el.classes().contains(&"area")
                        || (el.get("stroke-dasharray").is_some_and(|v| !v.is_empty())
                            && el.get("fill") == Some("none"))
                    {
                        let fsw = attr(el, "stroke-width", 1.0).map_or(1.0, |v| v * sc);
                        frames.push((x, y, x + w, y + h, fsw));
                    }
                }
            }
        }
        if el.tag == "path" && el.get("fill") == Some("none") {
            if let Some(d) = el.get("d").filter(|d| !d.is_empty()) {
                let pts = sample(d);
                let sw = attr(el, "stroke-width", 1.0).map_or(1.0, |v| v * sc);
                paths.push((
                    pts.iter()
                        .map(|(px, py)| (px * sc + dx, py * sc + dy))
                        .collect(),
                    sw,
                ));
            }
        }
        for ch in el.elements() {
            walk(ch, dx, dy, sc, here_box, boxes, frames, paths);
        }
    }
    walk(
        &root,
        0.0,
        0.0,
        1.0,
        false,
        &mut boxes,
        &mut frames,
        &mut paths,
    );
    (boxes, frames, paths)
}

/// その図形自身の広がりから、刻み幅を決める。
fn step_for(points: &[Point]) -> f64 {
    if points.is_empty() {
        return 1.0;
    }
    let (mut x0, mut x1, mut y0, mut y1) = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for (x, y) in points {
        x0 = x0.min(*x);
        x1 = x1.max(*x);
        y0 = y0.min(*y);
        y1 = y1.max(*y);
    }
    let diag = ((x1 - x0).powf(2.0) + (y1 - y0).powf(2.0)).powf(0.5);
    diag.max(1.0) / fineness()
}

/// path を点列にする。**刻み幅はその path 自身の広がりから決める。**
fn sample(d: &str) -> Vec<Point> {
    let rough = sample_path(d, VERTICES_ONLY);
    sample_path(d, step_for(&rough))
}

fn inside(pt: Point, b: Rect4, margin: f64) -> bool {
    b.0 + margin < pt.0 && pt.0 < b.2 - margin && b.1 + margin < pt.1 && pt.1 < b.3 - margin
}

/// 線と箱の関係を検査する。**閾値は置かず、すべて図から決める。**
#[must_use]
pub fn check_shapes(svg: &str) -> Vec<String> {
    let mut faults = Vec::new();
    let (boxes, frames, paths) = shapes(svg);
    for i in 0..boxes.len() {
        for j in i + 1..boxes.len() {
            let (a, b) = (boxes[i], boxes[j]);
            if a.0 < b.2 && a.2 > b.0 && a.1 < b.3 && a.3 > b.1 {
                faults.push("箱どうしが重なる".to_owned());
            }
        }
    }
    if !boxes.is_empty() {
        let step = (boxes
            .iter()
            .map(|b| (b.2 - b.0).min(b.3 - b.1))
            .fold(f64::INFINITY, f64::min)
            / SAMPLE_DIVISOR)
            .max(0.5);
        for (pts, sw) in &paths {
            if pts.len() < 2 {
                continue;
            }
            let dense = densify(pts, step);
            // 始点 ・ 終点が触れている箱は、その辺の相手なので除く（**位置で切らない**）
            let ends = [pts[0], pts[pts.len() - 1]];
            for bx in &boxes {
                let touching = ends.iter().any(|(x, y)| {
                    bx.0 - sw <= *x && *x <= bx.2 + sw && bx.1 - sw <= *y && *y <= bx.3 + sw
                });
                if touching {
                    continue;
                }
                if dense.iter().any(|p| inside(*p, *bx, *sw)) {
                    faults.push("辺が箱を突っ切る".to_owned());
                    break;
                }
            }
        }
    }
    for i in 0..frames.len() {
        for j in i + 1..frames.len() {
            let (a, b) = (frames[i], frames[j]);
            let hit = a.0 < b.2 && a.2 > b.0 && a.1 < b.3 && a.3 > b.1;
            let nest = (a.0 <= b.0 && a.1 <= b.1 && a.2 >= b.2 && a.3 >= b.3)
                || (b.0 <= a.0 && b.1 <= a.1 && b.2 >= a.2 && b.3 >= a.3);
            if hit && !nest {
                faults.push("囲みが中途半端に交差する".to_owned());
            }
        }
    }
    // 囲みの線が中身へ食い込んでいないか ── **空けるべき量はその囲み自身の線幅から決める**
    for (fx0, fy0, fx1, fy1, fsw) in &frames {
        for (bx0, by0, bx1, by1) in &boxes {
            let overlaps = fx0 < bx1 && fx1 > bx0 && fy0 < by1 && fy1 > by0;
            if !overlaps {
                continue;
            }
            let clear = fsw / 2.0;
            if *bx0 < fx0 + clear || *by0 < fy0 + clear || *bx1 > fx1 - clear || *by1 > fy1 - clear
            {
                faults.push("囲みの線が中身に重なる".to_owned());
                break;
            }
        }
    }
    faults
}

fn viewbox() -> &'static Regex {
    static ONCE: OnceLock<Regex> = OnceLock::new();
    ONCE.get_or_init(|| Regex::new(r#"viewBox="0 0 ([\d.]+) ([\d.]+)""#).expect("式である"))
}

/// 文字の重なりと、画布の外へのはみ出しを列挙する。**空なら破綻無し。**
#[must_use]
pub fn check(svg: &str) -> Vec<String> {
    let Some(m) = viewbox().captures(svg) else {
        return vec!["viewBoxが無い".to_owned()];
    };
    let (vw, vh) = (
        m[1].parse::<f64>().unwrap_or(0.0),
        m[2].parse::<f64>().unwrap_or(0.0),
    );
    let boxes = text_boxes(svg);
    let mut faults = Vec::new();
    for i in 0..boxes.len() {
        let bx = &boxes[i];
        if bx.0 < -1.0 || bx.1 < -1.0 || bx.2 > vw + 1.0 || bx.3 > vh + 1.0 {
            faults.push(format!("画布の外: {}", quote(&bx.4)));
        }
        for other in &boxes[i + 1..] {
            if bx.0 < other.2 && bx.2 > other.0 && bx.1 < other.3 && bx.3 > other.1 {
                faults.push(format!(
                    "文字が重なる: {} × {}",
                    quote(&bx.4),
                    quote(&other.4)
                ));
            }
        }
    }
    faults
}

/// 点から線分までの距離。**線分の外側なら端点までの距離になる。**
fn point_to_segment(p: Point, a: Point, b: Point) -> f64 {
    let (ex, ey) = (b.0 - a.0, b.1 - a.1);
    let length2 = ex * ex + ey * ey;
    if length2 <= 0.0 {
        return ((p.0 - a.0).powf(2.0) + (p.1 - a.1).powf(2.0)).powf(0.5);
    }
    let t = (((p.0 - a.0) * ex + (p.1 - a.1) * ey) / length2).clamp(0.0, 1.0);
    let (qx, qy) = (a.0 + ex * t, a.1 + ey * t);
    ((p.0 - qx).powf(2.0) + (p.1 - qy).powf(2.0)).powf(0.5)
}

type Segment = (Point, Point, f64);

fn polygon_numbers(s: &str) -> Vec<f64> {
    static ONCE: OnceLock<Regex> = OnceLock::new();
    let re = ONCE.get_or_init(|| Regex::new(r"-?[\d.]+").expect("式である"));
    re.find_iter(s)
        .filter_map(|m| parse_float(m.as_str()))
        .collect()
}

/// 節点ごとに、その節点が実際に置いたインクを**線分の並び**として集める。
///
/// 点までの距離で測ると、標本と標本のあいだに入った終端が、標本の粗さのぶんだけ「離れている」
/// と出る（実測 ── 着いているのに 1.5〜1.7 の隔たりと報告した）。
fn node_ink(root: &Element, fine: f64) -> Vec<Vec<Segment>> {
    fn drawable(el: &Element, dx: f64, dy: f64, sc: f64, fine: f64, into: &mut Vec<Segment>) {
        let (dx, dy, sc) = transform(el, dx, dy, sc);
        let tag = el.tag.as_str();
        let read = || -> Option<Vec<Point>> {
            Some(match tag {
                "rect" => {
                    let (x, y) = (attr(el, "x", 0.0)?, attr(el, "y", 0.0)?);
                    let (w, h) = (attr(el, "width", 0.0)?, attr(el, "height", 0.0)?);
                    if w != 0.0 && h != 0.0 {
                        vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h), (x, y)]
                    } else {
                        Vec::new()
                    }
                }
                "polygon" | "polyline" => {
                    let v = polygon_numbers(el.get("points").unwrap_or(""));
                    v.as_chunks::<2>().0.iter().map(|c| (c[0], c[1])).collect()
                }
                "path" => el
                    .get("d")
                    .filter(|d| !d.is_empty())
                    .map(sample)
                    .unwrap_or_default(),
                "circle" => {
                    let (cx, cy, r) = (
                        attr(el, "cx", 0.0)?,
                        attr(el, "cy", 0.0)?,
                        attr(el, "r", 0.0)?,
                    );
                    let n = ((2.0 * PI * fine) as usize).max(MIN_POLYGON);
                    (0..n)
                        .map(|k| {
                            let a = 2.0 * PI * k as f64 / n as f64;
                            (cx + r * a.cos(), cy + r * a.sin())
                        })
                        .collect()
                }
                t if t.ends_with("text") => {
                    let content = el.text().trim().to_owned();
                    if content.is_empty() {
                        Vec::new()
                    } else {
                        let (x, y) = (attr(el, "x", 0.0)?, attr(el, "y", 0.0)?);
                        let fs = attr(el, "font-size", theme_num("font.size"))?;
                        let w = text::width(&content, fs);
                        let x0 = match el.get("text-anchor").unwrap_or("start") {
                            "middle" => x - w / 2.0,
                            "end" => x - w,
                            _ => x,
                        };
                        let (top, bot) = (
                            y - fs * theme_num("font.cap-ratio"),
                            y + fs * theme_num("font.descender-ratio"),
                        );
                        vec![
                            (x0, top),
                            (x0 + w, top),
                            (x0 + w, bot),
                            (x0, bot),
                            (x0, top),
                        ]
                    }
                }
                _ => Vec::new(),
            })
        };
        let local = read().unwrap_or_default();
        if !local.is_empty() {
            let halo = match el.get("stroke") {
                Some(s) if !s.is_empty() && s != "none" => {
                    attr(el, "stroke-width", 1.0).map_or(0.0, |v| v / 2.0 * sc)
                }
                _ => 0.0,
            };
            let moved: Vec<Point> = local
                .iter()
                .map(|(px, py)| (px * sc + dx, py * sc + dy))
                .collect();
            if moved.len() == 1 {
                into.push((moved[0], moved[0], halo));
            }
            for w in moved.windows(2) {
                into.push((w[0], w[1], halo));
            }
        }
        for ch in el.elements() {
            drawable(ch, dx, dy, sc, fine, into);
        }
    }
    fn find(el: &Element, dx: f64, dy: f64, sc: f64, fine: f64, groups: &mut Vec<Vec<Segment>>) {
        let (dx, dy, sc) = transform(el, dx, dy, sc);
        if el.get("class").unwrap_or("").contains("wf-node") {
            let mut acc = Vec::new();
            for ch in el.elements() {
                drawable(ch, dx, dy, sc, fine, &mut acc);
            }
            groups.push(acc);
            return;
        }
        for ch in el.elements() {
            find(ch, dx, dy, sc, fine, groups);
        }
    }
    let mut groups = Vec::new();
    find(root, 0.0, 0.0, 1.0, fine, &mut groups);
    groups
}

/// 辺の終端が、どこかの節点のインクに着いているかを検査する。
///
/// **離れてよい量は、その辺自身の線幅から決める** ── 定数を置くと、倍率を変えた瞬間に効かなく
/// なる。
#[must_use]
pub fn check_attachment(svg: &str) -> Vec<String> {
    let Some(root) = xml::parse(svg) else {
        return vec!["SVGとして読めない".to_owned()];
    };
    let inks = node_ink(&root, fineness());
    if inks.is_empty() {
        return Vec::new();
    }
    let all_ink: Vec<Segment> = inks.into_iter().flatten().collect();
    let mut faults = Vec::new();
    let mut seen: HashSet<(String, String)> = HashSet::new();
    #[allow(clippy::too_many_arguments)] // 祖先から受け継ぐ状態を、そのまま引数で運ぶ
    fn walk(
        el: &Element,
        dx: f64,
        dy: f64,
        sc: f64,
        inside_node: bool,
        in_head: bool,
        all_ink: &[Segment],
        seen: &mut HashSet<(String, String)>,
        faults: &mut Vec<String>,
    ) {
        let (dx, dy, sc) = transform(el, dx, dy, sc);
        let inside_node = inside_node || el.get("class").unwrap_or("").contains("wf-node");
        // 矢じり（wf-head）は辺の端の飾りであって辺ではない ── かぎ ・ 開いた矢じりの先は
        // 節点に着かなくてよい
        let head = in_head || el.get("class").unwrap_or("").contains("wf-head");
        if !inside_node && !head && el.tag == "path" && el.get("fill") == Some("none") {
            if let Some(d) = el.get("d").filter(|d| !d.is_empty()) {
                let pts: Vec<Point> = sample(d)
                    .iter()
                    .map(|(x, y)| (x * sc + dx, y * sc + dy))
                    .collect();
                if pts.len() >= 2 {
                    let tol = attr(el, "stroke-width", 1.0).map_or(1.0, |v| v * sc);
                    for end in [pts[0], pts[pts.len() - 1]] {
                        let gap = all_ink
                            .iter()
                            .map(|(a, b, halo)| point_to_segment(end, *a, *b) - halo)
                            .fold(f64::INFINITY, f64::min);
                        if gap > tol {
                            let key = (format!("{:.1}", end.0), format!("{:.1}", end.1));
                            if seen.insert(key) {
                                faults.push(format!(
                                    "辺の終端が何にも着いていない: ({:.1},{:.1}) から最も近いインクまで {gap:.1}（許される隔たり {tol:.1}）",
                                    end.0, end.1
                                ));
                            }
                        }
                    }
                }
            }
        }
        for ch in el.elements() {
            walk(ch, dx, dy, sc, inside_node, head, all_ink, seen, faults);
        }
    }
    walk(
        &root,
        0.0,
        0.0,
        1.0,
        false,
        false,
        &all_ink,
        &mut seen,
        &mut faults,
    );
    faults
}

/// 3種の検査を当てる。**通ったことを返り値で示す。**
#[must_use]
pub fn all(svg: &str) -> Vec<String> {
    let mut out = check(svg);
    out.extend(check_shapes(svg));
    out.extend(check_attachment(svg));
    out
}
