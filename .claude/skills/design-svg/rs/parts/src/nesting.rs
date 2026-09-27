// SPDX-License-Identifier: MIT
//! 群を含む配置 ── **群を再帰的に解き、親は群を1個の節点として扱う。**
//!
//! 層ごとに要素を隣り合わせるだけでは足りない ── 群が層をまたぐと、斜めに離れた要素の外接矩形
//! が間の非メンバーまで飲み込む（実測）。「群の要素だけが、群の矩形の内側に居る」ことを構造と
//! して保証するには、群を先に配置して1つの大きさへ集約し、親はそれを1個として置くしかない。
//!
//! **辺は群の箱ではなく、実際の節点の位置へ描く。**

use std::collections::{HashMap, HashSet};

use crate::geometry::Point;
use crate::layout_contract::{Sizes, Strategy, Unsupported};

/// 配置の結果。**節点1つの箱か、群の箱。**
#[derive(Debug, Clone, Copy, Default)]
pub struct Box_ {
    /// 左。
    pub x: f64,
    /// 上。
    pub y: f64,
    /// 幅。
    pub width: f64,
    /// 高さ。
    pub height: f64,
}

/// 群の宣言。`(名前, 要素)`
pub type Group = (Option<String>, Vec<String>);

#[derive(Debug, Clone)]
struct Container {
    key: String,
    label: Option<String>,
    node_ids: Vec<String>,
    children: Vec<usize>,
}

/// 群の包含関係から木を組む。**要素の集合が包まれる側を子とする。**
fn build_tree(
    node_ids: &[String],
    groups: &[Group],
) -> Result<(Vec<Container>, usize), Unsupported> {
    let keys: Vec<String> = (0..groups.len()).map(|i| format!("__g{i}")).collect();
    let sets: Vec<HashSet<&str>> = groups
        .iter()
        .map(|(_, m)| m.iter().map(String::as_str).collect())
        .collect();
    for i in 0..groups.len() {
        for j in i + 1..groups.len() {
            let (sa, sb) = (&sets[i], &sets[j]);
            let common: Vec<&str> = {
                let mut c: Vec<&str> = sa.intersection(sb).copied().collect();
                c.sort_unstable();
                c
            };
            if !common.is_empty() && !(sa.is_subset(sb) || sb.is_subset(sa)) {
                let name = |k: usize| {
                    groups[k]
                        .0
                        .clone()
                        .filter(|l| !l.is_empty())
                        .unwrap_or_else(|| keys[k].clone())
                };
                let shown: Vec<String> = common.iter().map(|c| crate::py::quote(c)).collect();
                return Err(Unsupported::ByStrategy(format!(
                    "群 {} と {} が入れ子でなく重なっている（共通の要素: [{}]）。この配置は群を入れ子の箱として集約するので、この重なりは描けない。",
                    crate::py::quote(&name(i)),
                    crate::py::quote(&name(j)),
                    shown.join(", ")
                )));
            }
        }
    }
    let proper = |a: usize, b: usize| sets[a].is_subset(&sets[b]) && sets[a].len() < sets[b].len();
    let mut parent: Vec<Option<usize>> = Vec::new();
    for k in 0..groups.len() {
        let mut best: Option<usize> = None;
        for other in 0..groups.len() {
            if other == k || !proper(k, other) {
                continue;
            }
            if best.is_none_or(|b| proper(other, b)) {
                best = Some(other);
            }
        }
        parent.push(best);
    }
    let mut arena: Vec<Container> = groups
        .iter()
        .enumerate()
        .map(|(i, (label, _))| Container {
            key: keys[i].clone(),
            label: label.clone(),
            node_ids: Vec::new(),
            children: Vec::new(),
        })
        .collect();
    let root = arena.len();
    arena.push(Container {
        key: "__root".to_owned(),
        label: None,
        node_ids: Vec::new(),
        children: Vec::new(),
    });
    for (k, up) in parent.iter().enumerate() {
        let at = up.unwrap_or(root);
        arena[at].children.push(k);
    }
    // 節点は、それを含む最も内側の群へ配る。**どの群にも属さないものは最上位へ**
    for nid in node_ids {
        let mut owner: Option<usize> = None;
        for (k, set) in sets.iter().enumerate().take(groups.len()) {
            if set.contains(nid.as_str()) && owner.is_none_or(|o| proper(k, o)) {
                owner = Some(k);
            }
        }
        arena[owner.unwrap_or(root)].node_ids.push(nid.clone());
    }
    Ok((arena, root))
}

fn descendant_nodes(arena: &[Container], c: usize, out: &mut HashSet<String>) {
    out.extend(arena[c].node_ids.iter().cloned());
    for ch in &arena[c].children {
        descendant_nodes(arena, *ch, out);
    }
}

/// 解いた結果。
#[derive(Debug, Clone, Default)]
pub struct Nested {
    /// 節点 → 箱。**置いた順に持つ。**
    pub node_boxes: Vec<(String, Box_)>,
    /// 群のキー → 箱。
    pub group_boxes: HashMap<String, Box_>,
    /// 辺の番号 → 経路。
    pub edge_paths: HashMap<usize, Vec<Point>>,
    /// 全体の幅。
    pub width: f64,
    /// 全体の高さ。
    pub height: f64,
}

struct Solved {
    positions: HashMap<String, Point>,
    local_of: Vec<(usize, usize)>,
    paths: HashMap<usize, Vec<Point>>,
}

/// 群を再帰的に解き、節点と群それぞれの絶対座標を返す。
///
/// **各層の戦略を受け取って使う** ── 受け取らないと、呼ぶ側が戦略を選んだのに群があるという
/// だけで黙って層状に描かれる（以前はそうなっていた）。
///
/// # Errors
///
/// 入れ子でない群の重なりがあるときと、戦略が描けないときに返す。
#[allow(clippy::too_many_arguments)]
pub fn layout_nested(
    sizes: &Sizes,
    edges: &[(String, String)],
    groups: &[Group],
    gap_rank: f64,
    gap_order: f64,
    direction: &str,
    frame_pad: f64,
    label_h: f64,
    layout: Strategy,
) -> Result<Nested, Unsupported> {
    let node_ids: Vec<String> = sizes.iter().map(|(k, _)| k.clone()).collect();
    let size_of: HashMap<&str, (f64, f64)> = sizes.iter().map(|(k, s)| (k.as_str(), *s)).collect();
    let (arena, root) = build_tree(&node_ids, groups)?;
    let mut solved: HashMap<usize, Solved> = HashMap::new();
    let mut child_size: HashMap<usize, (f64, f64)> = HashMap::new();

    #[allow(clippy::too_many_arguments)]
    fn measure(
        c: usize,
        arena: &[Container],
        size_of: &HashMap<&str, (f64, f64)>,
        edges: &[(String, String)],
        gap_rank: f64,
        gap_order: f64,
        direction: &str,
        frame_pad: f64,
        label_h: f64,
        layout: Strategy,
        solved: &mut HashMap<usize, Solved>,
        child_size: &mut HashMap<usize, (f64, f64)>,
    ) -> Result<(f64, f64), Unsupported> {
        let mut sizes: Vec<(String, (f64, f64))> = Vec::new();
        for nid in &arena[c].node_ids {
            sizes.push((nid.clone(), size_of[nid.as_str()]));
        }
        for ch in &arena[c].children {
            let (cw, chh) = measure(
                *ch, arena, size_of, edges, gap_rank, gap_order, direction, frame_pad, label_h,
                layout, solved, child_size,
            )?;
            let pad_w = frame_pad * 2.0;
            let pad_h = frame_pad * 2.0
                + if arena[*ch].label.is_some() {
                    label_h
                } else {
                    0.0
                };
            child_size.insert(*ch, (cw + pad_w, chh + pad_h));
            sizes.push((arena[*ch].key.clone(), (cw + pad_w, chh + pad_h)));
        }
        let mut owner: HashMap<String, String> = arena[c]
            .node_ids
            .iter()
            .map(|n| (n.clone(), n.clone()))
            .collect();
        for ch in &arena[c].children {
            let mut under = HashSet::new();
            descendant_nodes(arena, *ch, &mut under);
            for nid in under {
                owner.insert(nid, arena[*ch].key.clone());
            }
        }
        let mut pairs: Vec<(String, String)> = Vec::new();
        let mut local_of: Vec<(usize, usize)> = Vec::new(); // 元の辺の番号 → この層での辺の番号
        for (idx, (a, b)) in edges.iter().enumerate() {
            let (Some(ra), Some(rb)) = (owner.get(a), owner.get(b)) else {
                continue;
            };
            if ra == rb {
                continue;
            }
            local_of.push((idx, pairs.len()));
            pairs.push((ra.clone(), rb.clone()));
        }
        if sizes.is_empty() {
            solved.insert(
                c,
                Solved {
                    positions: HashMap::new(),
                    local_of: Vec::new(),
                    paths: HashMap::new(),
                },
            );
            return Ok((0.0, 0.0));
        }
        let res = layout(&sizes, &pairs, gap_rank, gap_order, direction)?;
        solved.insert(
            c,
            Solved {
                positions: res.positions.iter().cloned().collect(),
                local_of,
                paths: res.edge_paths.iter().cloned().collect(),
            },
        );
        Ok((res.width, res.height))
    }

    let (total_w, total_h) = measure(
        root,
        &arena,
        &size_of,
        edges,
        gap_rank,
        gap_order,
        direction,
        frame_pad,
        label_h,
        layout,
        &mut solved,
        &mut child_size,
    )?;

    let mut out = Nested {
        width: total_w,
        height: total_h,
        ..Nested::default()
    };
    #[allow(clippy::too_many_arguments)]
    fn emplace(
        c: usize,
        ox: f64,
        oy: f64,
        arena: &[Container],
        solved: &HashMap<usize, Solved>,
        child_size: &HashMap<usize, (f64, f64)>,
        size_of: &HashMap<&str, (f64, f64)>,
        frame_pad: f64,
        label_h: f64,
        out: &mut Nested,
    ) {
        let s = &solved[&c];
        for (gidx, lidx) in &s.local_of {
            if let Some(pts) = s.paths.get(lidx).filter(|p| !p.is_empty()) {
                out.edge_paths
                    .insert(*gidx, pts.iter().map(|(x, y)| (ox + x, oy + y)).collect());
            }
        }
        for nid in &arena[c].node_ids {
            let (x, y) = s.positions[nid];
            let (w, h) = size_of[nid.as_str()];
            out.node_boxes.push((
                nid.clone(),
                Box_ {
                    x: ox + x,
                    y: oy + y,
                    width: w,
                    height: h,
                },
            ));
        }
        for ch in &arena[c].children {
            let (x, y) = s.positions[&arena[*ch].key];
            let (cw, chh) = child_size[ch];
            out.group_boxes.insert(
                arena[*ch].key.clone(),
                Box_ {
                    x: ox + x,
                    y: oy + y,
                    width: cw,
                    height: chh,
                },
            );
            // 子の中身は、枠の余白とラベルのぶん内側へ入る
            let extra = if arena[*ch].label.is_some() {
                label_h
            } else {
                0.0
            };
            emplace(
                *ch,
                ox + x + frame_pad,
                oy + y + frame_pad + extra,
                arena,
                solved,
                child_size,
                size_of,
                frame_pad,
                label_h,
                out,
            );
        }
    }
    emplace(
        root,
        0.0,
        0.0,
        &arena,
        &solved,
        &child_size,
        &size_of,
        frame_pad,
        label_h,
        &mut out,
    );
    Ok(out)
}
