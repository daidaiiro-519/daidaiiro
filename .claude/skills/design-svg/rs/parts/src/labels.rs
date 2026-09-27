// SPDX-License-Identifier: MIT
//! ラベルの置き場所 ── **互いに重ならないよう、まとめて調整する。**
//!
//! 1つずつ「決まった場所」に置くだけだと、近くに集まったときに重なる。まとめて受け取り、候補を
//! いくつか試して、既に置いたものと重ならない位置を選ぶ。
//!
//! **辺のラベルも点のラベルも、この1つの仕組みを使う** ── 別々に書くと、片方だけが重なりを
//! 見なくなる（実測 ── 散布図のラベルは他のラベルを一切見ておらず、点を12個近づけると15件
//! 重なった）。

use crate::geometry::{rects_overlap, Point, Rect};
use crate::style::Style;
use crate::text;

/// 置き場所を試す刻みの上限。**極端に短い経路では、刻みが細かくなりすぎて回り続ける。**
const MAX_STEPS: i64 = 20;

/// 置き場所の候補を、経路の長さとラベルの幅から決める。**真ん中から外へ交互に出す。**
#[must_use]
pub fn candidates(path_len: f64, label_w: f64) -> Vec<f64> {
    if path_len <= 0.0 {
        return vec![0.5];
    }
    let step = (label_w / path_len).min(0.5);
    if step <= 0.0 {
        return vec![0.5];
    }
    // 端の余白は、ラベルの半分が経路からはみ出さない位置
    let edge = (label_w / 2.0 / path_len).min(0.5);
    let (lo, hi) = (edge, 1.0 - edge);
    let mut out = vec![0.5];
    let mut k: i64 = 1;
    loop {
        let kf = k as f64;
        if !(0.5 - step * kf > lo || 0.5 + step * kf < hi) {
            break;
        }
        for f in [0.5 - step * kf, 0.5 + step * kf] {
            if lo <= f && f <= hi {
                out.push(f);
            }
        }
        k += 1;
        if k > MAX_STEPS {
            break;
        }
    }
    out
}

/// 折れ線の弧長に沿って、割合 `t` の位置の点を返す。
#[must_use]
pub fn point_at_fraction(points: &[Point], t: f64) -> Point {
    let seg: Vec<f64> = points
        .windows(2)
        .map(|w| ((w[1].0 - w[0].0).powf(2.0) + (w[1].1 - w[0].1).powf(2.0)).powf(0.5))
        .collect();
    let total = crate::py::sum(seg.iter().copied());
    let total = if total == 0.0 { 1.0 } else { total };
    let target = total * t;
    let mut cursor = 0.0;
    for (i, s) in seg.iter().enumerate() {
        if cursor + s >= target || i == seg.len() - 1 {
            let local = if *s == 0.0 {
                0.0
            } else {
                (target - cursor) / s
            };
            let (x1, y1) = points[i];
            let (x2, y2) = points[i + 1];
            return (x1 + (x2 - x1) * local, y1 + (y2 - y1) * local);
        }
        cursor += s;
    }
    points[points.len() - 1]
}

/// 置く1つ。`(大きさ, 良い順の候補)`
pub type Item = ((f64, f64), Vec<Point>);

/// 候補の中から、既に置いたものと重ならない場所を1つずつ選ぶ。
///
/// **どの候補も重なるときは最後の候補を採る**（最善努力）。
#[must_use]
pub fn place_avoiding(items: &[Item], occupied: &[Rect]) -> Vec<Point> {
    let mut placed: Vec<Rect> = occupied.to_vec();
    let mut out = Vec::new();
    for ((w, h), cands) in items {
        let cands: Vec<Point> = if cands.is_empty() {
            vec![(0.0, 0.0)]
        } else {
            cands.clone()
        };
        let mut chosen = None;
        for (cx, cy) in &cands {
            let bx = (cx - w / 2.0, cy - h / 2.0, cx + w / 2.0, cy + h / 2.0);
            if !placed.iter().any(|p| rects_overlap(bx, *p)) {
                chosen = Some((*cx, *cy));
                placed.push(bx);
                break;
            }
        }
        let chosen = chosen.unwrap_or_else(|| {
            let (cx, cy) = cands[cands.len() - 1];
            placed.push((cx - w / 2.0, cy - h / 2.0, cx + w / 2.0, cy + h / 2.0));
            (cx, cy)
        });
        out.push(chosen);
    }
    out
}

/// ラベルを持つ辺それぞれについて、重ならない置き場所を1つ返す。`(辺の番号, 中心)`
///
/// # Errors
///
/// 見た目に要るトークンが無いときに返す。
pub fn place_edge_labels(
    edges: &[(usize, Vec<Point>, String)],
    style: &Style,
    occupied: &[Rect],
) -> Result<Vec<(usize, Point)>, String> {
    let fs = style.num("font.size-small")?;
    let mut items = Vec::new();
    for (_, pts, label) in edges {
        let w = text::width(label, fs) + style.num("size.label-pad-x")?;
        let h = fs * style.num("size.label-line-h")?;
        let seg = crate::py::sum(
            pts.windows(2)
                .map(|p| ((p[1].0 - p[0].0).powf(2.0) + (p[1].1 - p[0].1).powf(2.0)).powf(0.5)),
        );
        // 辺のラベルの候補は「経路上の点」。**真ん中から外へ交互に**
        items.push((
            (w, h),
            candidates(seg, w)
                .iter()
                .map(|f| point_at_fraction(pts, *f))
                .collect(),
        ));
    }
    let chosen = place_avoiding(&items, occupied);
    Ok(edges
        .iter()
        .zip(chosen)
        .map(|((i, _, _), c)| (*i, c))
        .collect())
}
