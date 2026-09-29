// SPDX-License-Identifier: MIT
//! 環状配置 ── 節点を輪の上へ等間隔に置く。**層状配置とは別の戦略。**
//!
//! 並びが閉じていることを見せたい図で使う ── 層状配置で縦一列に並べて最後から最初へ長い辺を戻す
//! と、閉じていることが図から読めない。
//!
//! **輪の上の並び順を辺から決める** ── 宣言の順しだいで隣り合うべき節点が輪の反対側へ行き、絵は
//! もつれた星になる（幾何的な破綻は出ないので、検査は通ってしまう）。たどりきれない形は、黙って
//! 歪んだ絵を返さず、戦略の側から申告する。

use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;

use crate::geometry::{segment_hits_rect, shift_to_origin, Point};
use crate::layout_contract::{LayoutResult, Sizes, Unsupported};
use crate::py::sum;

/// 辺をたどって輪の上の並び順を決める。
fn ring_order(ids: &[String], edges: &[(String, String)]) -> Vec<String> {
    let at: HashMap<&str, usize> = ids
        .iter()
        .enumerate()
        .map(|(i, k)| (k.as_str(), i))
        .collect();
    let mut adj: HashMap<&str, HashSet<&str>> =
        ids.iter().map(|k| (k.as_str(), HashSet::new())).collect();
    for (a, b) in edges {
        if at.contains_key(a.as_str()) && at.contains_key(b.as_str()) && a != b {
            adj.get_mut(a.as_str()).expect("在る").insert(b.as_str());
            adj.get_mut(b.as_str()).expect("在る").insert(a.as_str());
        }
    }
    let mut remaining: HashSet<&str> = ids.iter().map(String::as_str).collect();
    // 行き止まり（辺が1本だけ）があればそこから始める
    let start = ids
        .iter()
        .min_by_key(|k| {
            let n = adj[k.as_str()].len();
            (n != 1, n, at[k.as_str()])
        })
        .expect("在る")
        .as_str();
    let mut order = vec![start.to_owned()];
    remaining.remove(start);
    while !remaining.is_empty() {
        let cur = order[order.len() - 1].clone();
        let key = |k: &str| (adj[k].len(), at[k]);
        let nxt: Vec<&str> = adj[cur.as_str()]
            .iter()
            .copied()
            .filter(|k| remaining.contains(k))
            .collect();
        let pool: Vec<&str> = if nxt.is_empty() {
            remaining.iter().copied().collect()
        } else {
            nxt
        };
        let pick = pool.into_iter().min_by_key(|k| key(k)).expect("在る");
        order.push(pick.to_owned());
        remaining.remove(pick);
    }
    order
}

/// 節点を輪の上へ等間隔に置く。**輪の半径は、節点の大きさと個数から決める。**
///
/// # Errors
///
/// 輪の内側を横切る辺が、他の節点の箱を突っ切ってしまうときに返す。
pub fn layout_radial(
    sizes: &Sizes,
    edges: &[(String, String)],
    gap_rank: f64,
    gap_order: f64,
    _direction: &str,
) -> Result<LayoutResult, Unsupported> {
    let ids: Vec<String> = sizes.iter().map(|(k, _)| k.clone()).collect();
    let n = ids.len();
    if n == 0 {
        return Ok(LayoutResult::default());
    }
    let size_of: HashMap<&str, (f64, f64)> = sizes.iter().map(|(k, s)| (k.as_str(), *s)).collect();
    if n == 1 {
        let (w, h) = sizes[0].1;
        return Ok(LayoutResult {
            positions: vec![(ids[0].clone(), (0.0, 0.0))],
            edge_paths: Vec::new(),
            width: w,
            height: h,
        });
    }
    let gap = gap_rank.max(gap_order);
    // 輪の周長は、各節点が占める幅と隙間の合計を下回れない
    let span = sum(sizes.iter().map(|(_, (w, h))| w.max(*h))) + gap * n as f64;
    let mut radius = span / (2.0 * PI);
    // 隣り合う節点が重ならない半径も別に要る
    let biggest = sizes
        .iter()
        .map(|(_, (w, h))| w.max(*h))
        .fold(f64::NEG_INFINITY, f64::max);
    let chord = biggest + gap;
    radius = radius.max(chord / (2.0 * (PI / n as f64).sin()));

    let mut positions: Vec<(String, Point)> = Vec::new();
    let mut centres: Vec<(String, Point)> = Vec::new();
    for (i, nid) in ring_order(&ids, edges).iter().enumerate() {
        let ang = -PI / 2.0 + 2.0 * PI * i as f64 / n as f64; // 真上から時計回り
        let (cx, cy) = (radius * ang.cos(), radius * ang.sin());
        let (w, h) = size_of[nid.as_str()];
        centres.push((nid.clone(), (cx, cy)));
        positions.push((nid.clone(), (cx - w / 2.0, cy - h / 2.0)));
    }
    let (positions, (min_x, min_y)) = shift_to_origin(&positions);
    let centres: HashMap<String, Point> = centres
        .into_iter()
        .map(|(k, (x, y))| (k, (x - min_x, y - min_y)))
        .collect();
    let pos: HashMap<&str, Point> = positions.iter().map(|(k, p)| (k.as_str(), *p)).collect();
    let width = ids
        .iter()
        .map(|k| pos[k.as_str()].0 + size_of[k.as_str()].0)
        .fold(f64::NEG_INFINITY, f64::max);
    let height = ids
        .iter()
        .map(|k| pos[k.as_str()].1 + size_of[k.as_str()].1)
        .fold(f64::NEG_INFINITY, f64::max);
    // 輪の内側を横切る辺が、当事者でない節点の箱を突っ切っていないか ── **閾値は置かない**
    for (a, b) in edges {
        let (Some(ca), Some(cb)) = (centres.get(a), centres.get(b)) else {
            continue;
        };
        if a == b {
            continue;
        }
        for k in &ids {
            if k == a || k == b {
                continue;
            }
            let (x, y) = pos[k.as_str()];
            let (w, h) = size_of[k.as_str()];
            if segment_hits_rect(*ca, *cb, (x, y, x + w, y + h), 0.0) {
                return Err(Unsupported::ByStrategy(format!(
                    "環状配置では描けません: 辺 {a}→{b} が節点 {k} を突っ切ります。輪の上に並べきれない形（1つの節点から3方向以上へ分かれる等）です。layout_tree（放射状の木）か layout_graph（層状）を使ってください。"
                )));
            }
        }
    }
    let edge_paths = edges
        .iter()
        .enumerate()
        .filter_map(|(i, (a, b))| Some((i, vec![*centres.get(a)?, *centres.get(b)?])))
        .collect();
    // 位置は宣言の順で返す
    let positions = ids.iter().map(|k| (k.clone(), pos[k.as_str()])).collect();
    Ok(LayoutResult {
        positions,
        edge_paths,
        width,
        height,
    })
}
