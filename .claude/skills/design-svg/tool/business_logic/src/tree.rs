// SPDX-License-Identifier: MIT
//! 放射の木 ── 根を中心に、深さごとの輪へ節点を置く。
//!
//! 輪の半径も扇の広さも、その深さに実際に居る節点の大きさと数から決める ── **決め打ちの半径 ・
//! 決め打ちの角度を持たない。**

use std::collections::{HashMap, VecDeque};
use std::f64::consts::PI;

use crate::geometry::{shift_to_origin, Point};
use crate::layout_contract::{LayoutResult, Sizes, Unsupported};
use crate::py::sum;

/// 最大のもの。**同じ大きさなら、先に在るものを選ぶ** ── 移す前と同じ選び方である。
fn first_max<T, K: PartialOrd>(items: &[T], key: impl Fn(&T) -> K) -> Option<&T> {
    let mut best: Option<(&T, K)> = None;
    for it in items {
        let k = key(it);
        if best.as_ref().is_none_or(|(_, b)| k > *b) {
            best = Some((it, k));
        }
    }
    best.map(|(it, _)| it)
}

/// 入ってくる辺が無い節点を根とする。**無ければ出る辺が最も多いものを選ぶ。**
fn pick_root(ids: &[String], edges: &[(String, String)]) -> String {
    let mut incoming: HashMap<&str, i64> = ids.iter().map(|i| (i.as_str(), 0)).collect();
    let mut outgoing = incoming.clone();
    for (a, b) in edges {
        if let Some(v) = incoming.get_mut(b.as_str()) {
            *v += 1;
        }
        if let Some(v) = outgoing.get_mut(a.as_str()) {
            *v += 1;
        }
    }
    let roots: Vec<String> = ids
        .iter()
        .filter(|i| incoming[i.as_str()] == 0)
        .cloned()
        .collect();
    let pool = if roots.is_empty() {
        ids.to_vec()
    } else {
        roots
    };
    first_max(&pool, |i| outgoing[i.as_str()])
        .expect("在る")
        .clone()
}

/// 幅優先で木を張る。**輪になっていても、後から届いた辺は木に加えない。**
fn spanning_tree(
    root: &str,
    ids: &[String],
    edges: &[(String, String)],
) -> (HashMap<String, Vec<String>>, HashMap<String, i64>) {
    let mut adj: HashMap<&str, Vec<&str>> = ids.iter().map(|i| (i.as_str(), Vec::new())).collect();
    for (a, b) in edges {
        if adj.contains_key(a.as_str()) && adj.contains_key(b.as_str()) {
            adj.get_mut(a.as_str()).expect("在る").push(b.as_str());
        }
    }
    let mut children: HashMap<String, Vec<String>> =
        ids.iter().map(|i| (i.clone(), Vec::new())).collect();
    let mut depth: HashMap<String, i64> = HashMap::from([(root.to_owned(), 0)]);
    let mut queue = VecDeque::from([root.to_owned()]);
    while let Some(cur) = queue.pop_front() {
        for nxt in &adj[cur.as_str()] {
            if !depth.contains_key(*nxt) {
                depth.insert((*nxt).to_owned(), depth[&cur] + 1);
                children
                    .get_mut(&cur)
                    .expect("在る")
                    .push((*nxt).to_owned());
                queue.push_back((*nxt).to_owned());
            }
        }
    }
    // 根から届かないものは、いちばん外の輪へまとめて置く
    let stray: Vec<String> = ids
        .iter()
        .filter(|i| !depth.contains_key(*i))
        .cloned()
        .collect();
    if !stray.is_empty() {
        let outer = depth.values().copied().max().unwrap_or(0) + 1;
        for i in stray {
            depth.insert(i.clone(), outer);
            children.get_mut(root).expect("在る").push(i);
        }
    }
    (children, depth)
}

/// その節点がぶら下げる葉の数。**扇の広さを分けるのに使う。**
fn leaves(node: &str, children: &HashMap<String, Vec<String>>) -> i64 {
    let kids = &children[node];
    if kids.is_empty() {
        return 1;
    }
    kids.iter().map(|c| leaves(c, children)).sum()
}

/// 根を中心に、深さごとの輪へ節点を置く。
///
/// # Errors
///
/// 返さない（契約を他の戦略と揃えるために、形だけ持つ）。
pub fn layout_tree(
    sizes: &Sizes,
    edges: &[(String, String)],
    gap_rank: f64,
    gap_order: f64,
    _direction: &str,
) -> Result<LayoutResult, Unsupported> {
    let ids: Vec<String> = sizes.iter().map(|(k, _)| k.clone()).collect();
    if ids.is_empty() {
        return Ok(LayoutResult::default());
    }
    let size_of: HashMap<&str, (f64, f64)> = sizes.iter().map(|(k, s)| (k.as_str(), *s)).collect();
    if ids.len() == 1 {
        let (w, h) = sizes[0].1;
        return Ok(LayoutResult {
            positions: vec![(ids[0].clone(), (0.0, 0.0))],
            edge_paths: Vec::new(),
            width: w,
            height: h,
        });
    }
    let root = pick_root(&ids, edges);
    let (children, depth) = spanning_tree(&root, &ids, edges);
    let max_depth = depth.values().copied().max().unwrap_or(0);
    let big = |k: &str| {
        let (w, h) = size_of[k];
        w.max(h)
    };
    // 深さごとの輪の半径。**1つ前の輪から、両側の節点の張り出しと隙間だけ離す**
    let mut radius: HashMap<i64, f64> = HashMap::from([(0, 0.0)]);
    for d in 1..=max_depth {
        let here: Vec<&String> = ids.iter().filter(|k| depth[*k] == d).collect();
        let prev: Vec<&String> = ids.iter().filter(|k| depth[*k] == d - 1).collect();
        let out_prev = prev.iter().map(|k| big(k)).reduce(f64::max).unwrap_or(0.0) / 2.0;
        let out_here = here.iter().map(|k| big(k)).reduce(f64::max).unwrap_or(0.0) / 2.0;
        let step = out_prev + out_here + gap_rank;
        // その輪に並ぶ節点が触れ合わないだけの周長も要る
        let need = sum(here.iter().map(|k| big(k) + gap_order));
        radius.insert(d, (radius[&(d - 1)] + step).max(need / (2.0 * PI)));
    }
    let mut centres: Vec<(String, Point)> = vec![(root.clone(), (0.0, 0.0))];
    fn place(
        node: &str,
        a0: f64,
        a1: f64,
        children: &HashMap<String, Vec<String>>,
        depth: &HashMap<String, i64>,
        radius: &HashMap<i64, f64>,
        centres: &mut Vec<(String, Point)>,
    ) {
        let kids = &children[node];
        if kids.is_empty() {
            return;
        }
        let total: i64 = kids.iter().map(|c| leaves(c, children)).sum();
        let mut cur = a0;
        for c in kids {
            let share = (a1 - a0) * leaves(c, children) as f64 / total as f64;
            let mid = cur + share / 2.0;
            let r = radius[&depth[c]];
            centres.push((c.clone(), (r * mid.cos(), r * mid.sin())));
            place(c, cur, cur + share, children, depth, radius, centres);
            cur += share;
        }
    }
    place(
        &root,
        -PI / 2.0,
        -PI / 2.0 + 2.0 * PI,
        &children,
        &depth,
        &radius,
        &mut centres,
    );
    let positions: Vec<(String, Point)> = centres
        .iter()
        .map(|(k, (cx, cy))| {
            let (w, h) = size_of[k.as_str()];
            (k.clone(), (cx - w / 2.0, cy - h / 2.0))
        })
        .collect();
    let (positions, (min_x, min_y)) = shift_to_origin(&positions);
    let centres: HashMap<String, Point> = centres
        .into_iter()
        .map(|(k, (x, y))| (k, (x - min_x, y - min_y)))
        .collect();
    let width = positions
        .iter()
        .map(|(k, p)| p.0 + size_of[k.as_str()].0)
        .fold(f64::NEG_INFINITY, f64::max);
    let height = positions
        .iter()
        .map(|(k, p)| p.1 + size_of[k.as_str()].1)
        .fold(f64::NEG_INFINITY, f64::max);
    let edge_paths = edges
        .iter()
        .enumerate()
        .filter_map(|(i, (a, b))| Some((i, vec![*centres.get(a)?, *centres.get(b)?])))
        .collect();
    Ok(LayoutResult {
        positions,
        edge_paths,
        width,
        height,
    })
}
