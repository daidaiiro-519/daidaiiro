// SPDX-License-Identifier: MIT
//! 層状グラフ描画 ── Graphviz の `dot` と同じ系統（Sugiyama 法）。
//!
//! 1. **サイクルの分断** ── 深さ優先で逆流する辺を見つけ、層の計算時だけ向きを仮に反転する
//! 2. **層の配分** ── 網状単体法で、辺の長さの総和を最小にする
//! 3. **複数層をまたぐ辺の経路** ── またぐ層の分だけ仮の節点を挟み、隣接層だけを結ぶ鎖にする
//! 4. **交差の最小化** ── 中央値法と転置法を上り下り交互に反復し、交差が最少の並びを採る
//! 5. **座標の整列** ── 上下の中央値へ寄せる反復平均。並びを保ったまま望んだ位置との差の総和を
//!    最小にする配置を、等調回帰で厳密に解く。最後に、繋がっていない塊どうしを詰める
//!
//! **辿る順も、移す前と同じにする** ── 同じ近さの候補が複数あると、辿る順で採るものが決まる。
//! 整数の集合は [`crate::intset::IntSet`] が、移す前の集合と同じ順で辿る。

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::geometry::{shift_to_origin, Point};
use crate::intset::IntSet;
use crate::layout_contract::{LayoutResult, Sizes, Unsupported};
use crate::py::{sum, sum_nums, Num};

/// 入れ替えの空回りへの保険 ── 浮動小数の丸めで振動した場合に備えて上限を置く。
const SWAP_LIMIT_PER_EDGE: usize = 4;
const SWAP_LIMIT_BASE: usize = 16;

/// 辺。`(from, to)`
type Edge = (String, String);

// ── 1. サイクルの分断 ──────────────────────────────────────

/// 深さ優先で逆流する辺を見つけ、`(from, to, 逆向きか)` の並びを返す。
fn break_cycles(nodes: &[String], edges: &[Edge]) -> Vec<(String, String, bool)> {
    let known: HashSet<&str> = nodes.iter().map(String::as_str).collect();
    let mut adj: HashMap<&str, Vec<&str>> =
        nodes.iter().map(|n| (n.as_str(), Vec::new())).collect();
    for (a, b) in edges {
        if known.contains(a.as_str()) && known.contains(b.as_str()) {
            adj.get_mut(a.as_str()).expect("在る").push(b.as_str());
        }
    }
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum State {
        White,
        Gray,
        Black,
    }
    let mut state: HashMap<&str, State> =
        nodes.iter().map(|n| (n.as_str(), State::White)).collect();
    let mut out: Vec<(String, String, bool)> = Vec::new();
    for start in nodes {
        if state[start.as_str()] != State::White {
            continue;
        }
        let mut stack: Vec<(&str, usize)> = vec![(start.as_str(), 0)];
        state.insert(start.as_str(), State::Gray);
        while let Some(&(u, at)) = stack.last() {
            let list = &adj[u];
            let mut advanced = false;
            let mut i = at;
            while i < list.len() {
                let v = list[i];
                i += 1;
                match state[v] {
                    State::White => {
                        state.insert(v, State::Gray);
                        let top = stack.len() - 1;
                        stack[top].1 = i;
                        stack.push((v, 0));
                        advanced = true;
                        break;
                    }
                    State::Gray => out.push((v.to_owned(), u.to_owned(), true)), // 後退辺
                    State::Black => out.push((u.to_owned(), v.to_owned(), false)),
                }
            }
            if !advanced {
                state.insert(u, State::Black);
                stack.pop();
            }
        }
    }
    // 同じ (u, v) を二重に数えないよう、辺の並びから作り直す
    let mut rank_edges: HashMap<(String, String), bool> = HashMap::new();
    for (a, b, rev) in out {
        let key = if rev { (b, a) } else { (a, b) };
        rank_edges.insert(key, rev);
    }
    edges
        .iter()
        .filter(|(a, b)| known.contains(a.as_str()) && known.contains(b.as_str()))
        .map(|(a, b)| {
            let rev = rank_edges
                .get(&(a.clone(), b.clone()))
                .copied()
                .unwrap_or(false);
            (a.clone(), b.clone(), rev)
        })
        .collect()
}

// ── 2. 層の配分 ──────────────────────────────────────────

/// どの辺も1層以上またぐ、いちばん素朴な配分。**単体法の出発点にする。**
fn longest_path_ranks(nodes: &[String], edges: &[Edge]) -> HashMap<String, i64> {
    let mut rank: HashMap<String, i64> = nodes.iter().map(|n| (n.clone(), 0)).collect();
    for _ in 0..=nodes.len() {
        let mut changed = false;
        for (a, b) in edges {
            let ra = rank[a];
            if rank[b] < ra + 1 {
                rank.insert(b.clone(), ra + 1);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    rank
}

/// ちょうど1層だけまたぐ辺（たるみ0）だけを辿って、広がれるだけ広げる。
fn tight_tree(
    edges: &[Edge],
    rank: &HashMap<String, i64>,
    root: &str,
) -> (HashSet<String>, IntSet) {
    let mut reached: HashSet<String> = HashSet::from([root.to_owned()]);
    let mut used = IntSet::new();
    let mut grew = true;
    while grew {
        grew = false;
        for (i, (a, b)) in edges.iter().enumerate() {
            if used.contains(i) || rank[b] - rank[a] != 1 {
                continue;
            }
            if reached.contains(a) != reached.contains(b) {
                reached.insert(a.clone());
                reached.insert(b.clone());
                used.add(i);
                grew = true;
            }
        }
    }
    (reached, used)
}

/// 木の辺がすべてちょうど1層になるように、層を振り直す。
fn ranks_from_tree(
    nodes: &[String],
    edges: &[Edge],
    tree: &IntSet,
    root: &str,
) -> HashMap<String, i64> {
    let mut adj: HashMap<&str, Vec<(&str, i64)>> =
        nodes.iter().map(|n| (n.as_str(), Vec::new())).collect();
    for i in tree.iter() {
        let (a, b) = &edges[i];
        adj.get_mut(a.as_str()).expect("在る").push((b.as_str(), 1));
        adj.get_mut(b.as_str())
            .expect("在る")
            .push((a.as_str(), -1));
    }
    let mut rank: HashMap<String, i64> = HashMap::from([(root.to_owned(), 0)]);
    let mut stack = vec![root.to_owned()];
    while let Some(n) = stack.pop() {
        for (m, step) in &adj[n.as_str()] {
            if !rank.contains_key(*m) {
                rank.insert((*m).to_owned(), rank[&n] + step);
                stack.push((*m).to_owned());
            }
        }
    }
    rank
}

/// 木から1本抜いたとき、その辺の根元側に残る節点の集合。
fn tail_side(nodes: &[String], edges: &[Edge], tree: &IntSet, leaving: usize) -> HashSet<String> {
    let mut adj: HashMap<&str, Vec<&str>> =
        nodes.iter().map(|n| (n.as_str(), Vec::new())).collect();
    for i in tree.iter() {
        if i == leaving {
            continue;
        }
        let (a, b) = &edges[i];
        adj.get_mut(a.as_str()).expect("在る").push(b.as_str());
        adj.get_mut(b.as_str()).expect("在る").push(a.as_str());
    }
    let start = edges[leaving].0.clone();
    let mut side = HashSet::from([start.clone()]);
    let mut stack = vec![start];
    while let Some(n) = stack.pop() {
        for m in &adj[n.as_str()] {
            if !side.contains(*m) {
                side.insert((*m).to_owned());
                stack.push((*m).to_owned());
            }
        }
    }
    side
}

/// 辺の長さの総和が最小になる層の配分を解く（網状単体法）。
///
/// **層数（縦の長さ）は増えない** ── 制約は最長経路法と同じで、その中で最短を選ぶだけ。
fn network_simplex(nodes: &[String], edges: &[Edge]) -> HashMap<String, i64> {
    if nodes.is_empty() {
        return HashMap::new();
    }
    let mut rank = longest_path_ranks(nodes, edges);
    let root = nodes[0].clone();
    // たるみ0の辺だけでは全体に届かないうちは、いちばんたるみの小さい辺の分だけ木ごと動かす
    let mut tree;
    loop {
        let (reached, used) = tight_tree(edges, &rank, &root);
        tree = used;
        if reached.len() == nodes.len() {
            break;
        }
        let mut best: Option<(usize, i64)> = None;
        for (i, (a, b)) in edges.iter().enumerate() {
            if reached.contains(a) == reached.contains(b) {
                continue;
            }
            let slack = rank[b] - rank[a] - 1;
            if best.is_none_or(|(_, s)| slack < s) {
                best = Some((i, slack));
            }
        }
        let Some((best_edge, best_slack)) = best else {
            break; // 繋がっていない ── 呼ぶ側が塊ごとに分けている前提
        };
        let shift = if reached.contains(&edges[best_edge].0) {
            best_slack
        } else {
            -best_slack
        };
        for n in &reached {
            *rank.get_mut(n).expect("在る") += shift;
        }
    }
    let limit = SWAP_LIMIT_PER_EDGE * edges.len() + SWAP_LIMIT_BASE;
    for _ in 0..limit {
        let mut leaving: Option<(usize, HashSet<String>)> = None;
        for i in tree.iter() {
            let side = tail_side(nodes, edges, &tree, i);
            let cut: i64 = edges
                .iter()
                .map(|(a, b)| {
                    let (ia, ib) = (side.contains(a), side.contains(b));
                    if ia != ib && ia {
                        1
                    } else if ia != ib {
                        -1
                    } else {
                        0
                    }
                })
                .sum();
            if cut < 0 {
                leaving = Some((i, side));
                break;
            }
        }
        let Some((i, side)) = leaving else { break };
        // 切り口を逆向きに跨ぐ辺のうち、いちばんたるみの小さいものを入れる
        let mut entering: Option<(usize, i64)> = None;
        for (j, (a, b)) in edges.iter().enumerate() {
            if tree.contains(j) || side.contains(a) || !side.contains(b) {
                continue;
            }
            let slack = rank[b] - rank[a] - 1;
            if entering.is_none_or(|(_, s)| slack < s) {
                entering = Some((j, slack));
            }
        }
        let Some((j, _)) = entering else { break };
        tree = tree.without(i).with(j);
        rank = ranks_from_tree(nodes, edges, &tree, &root);
    }
    let low = rank.values().copied().min().unwrap_or(0);
    nodes.iter().map(|n| (n.clone(), rank[n] - low)).collect()
}

/// 層を配分する。**繋がっていない塊は、それぞれ独立に解く。**
fn assign_ranks(nodes: &[String], dag_edges: &[Edge]) -> HashMap<String, i64> {
    let mut adj: HashMap<&str, Vec<&str>> =
        nodes.iter().map(|n| (n.as_str(), Vec::new())).collect();
    for (a, b) in dag_edges {
        adj.get_mut(a.as_str()).expect("在る").push(b.as_str());
        adj.get_mut(b.as_str()).expect("在る").push(a.as_str());
    }
    let mut seen: HashSet<&str> = HashSet::new();
    let mut rank = HashMap::new();
    for start in nodes {
        if seen.contains(start.as_str()) {
            continue;
        }
        let mut group: Vec<&str> = Vec::new();
        let mut stack = vec![start.as_str()];
        seen.insert(start.as_str());
        while let Some(n) = stack.pop() {
            group.push(n);
            for m in &adj[n] {
                if !seen.contains(m) {
                    seen.insert(m);
                    stack.push(m);
                }
            }
        }
        let members: HashSet<&str> = group.into_iter().collect();
        let comp_nodes: Vec<String> = nodes
            .iter()
            .filter(|n| members.contains(n.as_str()))
            .cloned()
            .collect();
        let comp_edges: Vec<Edge> = dag_edges
            .iter()
            .filter(|(a, _)| members.contains(a.as_str()))
            .cloned()
            .collect();
        rank.extend(network_simplex(&comp_nodes, &comp_edges));
    }
    rank
}

// ── 3. 複数層をまたぐ辺へ仮の節点を挟む ────────────────────────

/// 仮の節点を挟んだグラフ。
#[derive(Debug, Clone)]
struct Expanded {
    all_nodes: Vec<String>,
    real: HashSet<String>,
    rank: HashMap<String, i64>,
    unit_edges: Vec<Edge>,
    /// 元の辺ごとの、通る節点の並び（実節点と仮節点、層の昇順）
    chains: Vec<Vec<String>>,
}

fn expand(
    nodes: &[String],
    edges_with_rev: &[(String, String, bool)],
    mut rank: HashMap<String, i64>,
) -> Expanded {
    let mut all_nodes = nodes.to_vec();
    let mut unit_edges = Vec::new();
    let mut chains = Vec::new();
    let mut vcount = 0;
    for (a, b, _) in edges_with_rev {
        let (ra, rb) = (rank[a], rank[b]);
        let (lo, hi) = if ra <= rb { (a, b) } else { (b, a) };
        let mut chain = vec![lo.clone()];
        let mut cur = lo.clone();
        for r in (rank[lo] + 1)..rank[hi] {
            let vid = format!("__v{vcount}");
            vcount += 1;
            rank.insert(vid.clone(), r);
            all_nodes.push(vid.clone());
            unit_edges.push((cur.clone(), vid.clone()));
            chain.push(vid.clone());
            cur = vid;
        }
        unit_edges.push((cur, hi.clone()));
        chain.push(hi.clone());
        if lo != a {
            chain.reverse();
        }
        chains.push(chain);
    }
    Expanded {
        all_nodes,
        real: nodes.iter().cloned().collect(),
        rank,
        unit_edges,
        chains,
    }
}

// ── 4. 層内の並び順 ──────────────────────────────────────────

/// その節点が属する群の番号。**入れ子は、より内側（後に宣言された群）を優先する。**
fn block_of(node: &str, groups: &[Vec<String>]) -> i64 {
    let mut found = -1;
    for (i, members) in groups.iter().enumerate() {
        if members.iter().any(|m| m == node) {
            found = i as i64;
        }
    }
    found
}

/// n1 が n2 の左に居るとして、隣接層との間で何本交差するか。
fn local_crossings(
    n1: &str,
    n2: &str,
    order: &HashMap<String, i64>,
    incoming: &HashMap<String, Vec<String>>,
    outgoing: &HashMap<String, Vec<String>>,
) -> i64 {
    let mut c = 0;
    for neigh in [incoming, outgoing] {
        let empty = Vec::new();
        for p1 in neigh.get(n1).unwrap_or(&empty) {
            for p2 in neigh.get(n2).unwrap_or(&empty) {
                if order.get(p1).copied().unwrap_or(0) > order.get(p2).copied().unwrap_or(0) {
                    c += 1;
                }
            }
        }
    }
    c
}

fn crossing_total(
    expanded: &Expanded,
    by_rank: &BTreeMap<i64, Vec<String>>,
    order: &HashMap<String, i64>,
) -> i64 {
    let mut total = 0;
    let ranks: Vec<i64> = by_rank.keys().copied().collect();
    let mut edges_by_top: BTreeMap<i64, Vec<(&str, &str)>> =
        ranks.iter().map(|r| (*r, Vec::new())).collect();
    for (a, b) in &expanded.unit_edges {
        edges_by_top
            .entry(expanded.rank[a])
            .or_default()
            .push((a, b));
    }
    for r in ranks.iter().take(ranks.len().saturating_sub(1)) {
        let pairs: Vec<(i64, i64)> = edges_by_top.get(r).map_or_else(Vec::new, |es| {
            es.iter().map(|(a, b)| (order[*a], order[*b])).collect()
        });
        for i in 0..pairs.len() {
            for j in i + 1..pairs.len() {
                let ((a1, b1), (a2, b2)) = (pairs[i], pairs[j]);
                if (a1 - a2) * (b1 - b2) < 0 {
                    total += 1;
                }
            }
        }
    }
    total
}

/// 中央値。**偶数個なら中2つの平均** ── どちらへも同じだけ動けるように。
fn median(vals: &[f64]) -> f64 {
    let mut s = vals.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let m = s.len() / 2;
    if s.len() % 2 == 1 {
        s[m]
    } else {
        (s[m - 1] + s[m]) / 2.0
    }
}

fn cmp_f(a: f64, b: f64) -> std::cmp::Ordering {
    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
}

/// 層内の並びを解く。
///
/// **群があるときは階層的に解く** ── まず同じ群の要素をひとつの塊として並べ、次に塊の中を並べる。
/// 隣り合うことは優先度ではなく要件である ── 離れて置かれた要素を箱で囲むと、間に居る非メンバー
/// まで囲んでしまう。
#[allow(clippy::too_many_lines)]
fn order_within_ranks(expanded: &Expanded, groups: &[Vec<String>]) -> HashMap<String, i64> {
    let mut by_rank: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for n in &expanded.all_nodes {
        by_rank.entry(expanded.rank[n]).or_default().push(n.clone());
    }
    let mut incoming: HashMap<String, Vec<String>> = expanded
        .all_nodes
        .iter()
        .map(|n| (n.clone(), Vec::new()))
        .collect();
    let mut outgoing = incoming.clone();
    for (a, b) in &expanded.unit_edges {
        outgoing.get_mut(a).expect("在る").push(b.clone());
        incoming.get_mut(b).expect("在る").push(a.clone());
    }
    // 初期の並びは、**宣言順ではなく辺をたどった順**にする（実測 ── 宣言順から始めると、2つの
    // 塊に分かれる図で28交差。塊を分けて並べれば10交差）
    let mut seen: HashSet<String> = HashSet::new();
    let mut walked: Vec<String> = Vec::new();
    for start in &expanded.all_nodes {
        if seen.contains(start) {
            continue;
        }
        seen.insert(start.clone());
        let mut queue = std::collections::VecDeque::from([start.clone()]);
        while let Some(n) = queue.pop_front() {
            walked.push(n.clone());
            for m in outgoing[&n].iter().chain(incoming[&n].iter()) {
                if !seen.contains(m) {
                    seen.insert(m.clone());
                    queue.push_back(m.clone());
                }
            }
        }
    }
    let walk_at: HashMap<&str, usize> = walked
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();
    let mut order: HashMap<String, i64> = HashMap::new();
    for row in by_rank.values_mut() {
        row.sort_by_key(|n| walk_at[n.as_str()]);
        for (i, n) in row.iter().enumerate() {
            order.insert(n.clone(), i as i64);
        }
    }

    let med =
        |n: &str, neighbours: &HashMap<String, Vec<String>>, order: &HashMap<String, i64>| -> Num {
            let mut ords: Vec<i64> = neighbours[n]
                .iter()
                .filter_map(|p| order.get(p).copied())
                .collect();
            ords.sort_unstable();
            if ords.is_empty() {
                return Num::I(order[n]);
            }
            let m = ords.len() / 2;
            if ords.len() % 2 == 1 {
                return Num::F(ords[m] as f64);
            }
            if ords.len() == 2 {
                return Num::F((ords[0] + ords[1]) as f64 / 2.0);
            }
            let left = ords[m - 1] - ords[0];
            let right = ords[ords.len() - 1] - ords[m];
            if left + right == 0 {
                return Num::F((ords[m - 1] + ords[m]) as f64 / 2.0);
            }
            Num::F((ords[m - 1] * right + ords[m] * left) as f64 / (left + right) as f64)
        };

    let median_pass = |neighbours: &HashMap<String, Vec<String>>,
                       seq: &[i64],
                       by_rank: &mut BTreeMap<i64, Vec<String>>,
                       order: &mut HashMap<String, i64>| {
        for r in seq {
            let row = by_rank[r].clone();
            let keyed: Vec<String> = if groups.is_empty() {
                let mut k: Vec<(f64, String)> = row
                    .iter()
                    .map(|n| (med(n, neighbours, order).f(), n.clone()))
                    .collect();
                k.sort_by(|a, b| cmp_f(a.0, b.0));
                k.into_iter().map(|(_, n)| n).collect()
            } else {
                // 階層的に解く。**塊は分断されないので隣接が保たれる**
                let mut blocks: Vec<(i64, Vec<String>)> = Vec::new();
                for n in &row {
                    let bi = block_of(n, groups);
                    match blocks.iter_mut().find(|(k, _)| *k == bi) {
                        Some(slot) => slot.1.push(n.clone()),
                        None => blocks.push((bi, vec![n.clone()])),
                    }
                }
                let free: Vec<(i64, Vec<String>)> = blocks
                    .iter()
                    .find(|(k, _)| *k == -1)
                    .map(|(_, ms)| ms.iter().map(|n| (-1, vec![n.clone()])).collect())
                    .unwrap_or_default();
                let mut units: Vec<(i64, Vec<String>)> = free;
                units.extend(blocks.into_iter().filter(|(k, _)| *k != -1));
                let mut keyed_units: Vec<(f64, Vec<String>)> = units
                    .into_iter()
                    .map(|(_, members)| {
                        let meds: Vec<Num> =
                            members.iter().map(|m| med(m, neighbours, order)).collect();
                        (sum_nums(&meds).f() / members.len() as f64, members)
                    })
                    .collect();
                keyed_units.sort_by(|a, b| cmp_f(a.0, b.0));
                let mut out = Vec::new();
                for (_, members) in keyed_units {
                    if members.len() > 1 {
                        let mut k: Vec<(f64, String)> = members
                            .iter()
                            .map(|n| (med(n, neighbours, order).f(), n.clone()))
                            .collect();
                        k.sort_by(|a, b| cmp_f(a.0, b.0));
                        out.extend(k.into_iter().map(|(_, n)| n));
                    } else {
                        out.extend(members);
                    }
                }
                out
            };
            for (i, n) in keyed.iter().enumerate() {
                order.insert(n.clone(), i as i64);
            }
        }
    };

    let transpose = |by_rank: &mut BTreeMap<i64, Vec<String>>, order: &mut HashMap<String, i64>| {
        let mut improved = true;
        while improved {
            improved = false;
            for row in by_rank.values_mut() {
                for i in 0..row.len().saturating_sub(1) {
                    let (n1, n2) = (row[i].clone(), row[i + 1].clone());
                    if !groups.is_empty() && block_of(&n1, groups) != block_of(&n2, groups) {
                        continue;
                    }
                    let before = local_crossings(&n1, &n2, order, &incoming, &outgoing);
                    let after = local_crossings(&n2, &n1, order, &incoming, &outgoing);
                    if after < before {
                        row.swap(i, i + 1);
                        order.insert(n1, i as i64 + 1);
                        order.insert(n2, i as i64);
                        improved = true;
                    }
                }
            }
        }
    };

    let ranks_sorted: Vec<i64> = by_rank.keys().copied().collect();
    let mut best_order = order.clone();
    let mut best_score = crossing_total(expanded, &by_rank, &order);
    // **回数は決め打ちにしない** ── 交差が0になるか、1往復しても並びが変わらなくなったら終わり
    let limit = expanded.rank.len().max(1);
    let mut it = 0;
    while it < limit * 2 && best_score > 0 {
        let before_cycle = order.clone();
        for pass in 0..2 {
            let seq: Vec<i64> = if pass == 0 {
                ranks_sorted.clone()
            } else {
                ranks_sorted.iter().rev().copied().collect()
            };
            // **中央値法は順位だけを書き、行の並びは動かさない** ── 移す前もそうである。転置は、
            // 動かなかった行の並びの上で隣どうしを入れ替える
            median_pass(
                if pass == 0 { &incoming } else { &outgoing },
                &seq,
                &mut by_rank,
                &mut order,
            );
            transpose(&mut by_rank, &mut order);
            let score = crossing_total(expanded, &by_rank, &order);
            if score < best_score {
                best_score = score;
                best_order = order.clone();
            }
        }
        if order == before_cycle {
            break; // 一往復して何も動かなかった ── これ以上良くならない
        }
        it += 1;
    }
    best_order
}

// ── 5. 座標の整列 ──────────────────────────────────────────

/// 層ごとの並び。**層が最初に現れた順に持つ** ── 塊を詰める判定が、この順で層を見る。
type Rows = Vec<(i64, Vec<String>)>;

fn row_of(rows: &Rows, r: i64) -> &Vec<String> {
    &rows.iter().find(|(k, _)| *k == r).expect("在る").1
}

fn row_of_mut(rows: &mut Rows, r: i64) -> &mut Vec<String> {
    &mut rows.iter_mut().find(|(k, _)| *k == r).expect("在る").1
}

/// 繋がっていない塊どうしを、隣り合うまで詰める。
///
/// **決まらないなら詰める、と決める** ── 塊の相対位置は目的から決まらず、反復に委ねると寄せる
/// 力が無いまま離れていく（実測 ── 節点を1つ足すと幅が570から1290へ広がった）。
fn pack_components(
    expanded: &Expanded,
    by_rank: &Rows,
    cross: &mut HashMap<String, f64>,
    sizes: &HashMap<String, (f64, f64)>,
    gap_order: f64,
    direction: &str,
) {
    let mut comp: Vec<(String, usize)> = Vec::new();
    let mut comp_of: HashMap<String, usize> = HashMap::new();
    let mut adj: HashMap<&str, Vec<&str>> = expanded
        .all_nodes
        .iter()
        .map(|n| (n.as_str(), Vec::new()))
        .collect();
    for (a, b) in &expanded.unit_edges {
        adj.get_mut(a.as_str()).expect("在る").push(b.as_str());
        adj.get_mut(b.as_str()).expect("在る").push(a.as_str());
    }
    let mut seen: HashSet<&str> = HashSet::new();
    let mut count = 0;
    for start in &expanded.all_nodes {
        if seen.contains(start.as_str()) {
            continue;
        }
        let cid = count;
        count += 1;
        seen.insert(start.as_str());
        let mut queue = std::collections::VecDeque::from([start.as_str()]);
        while let Some(n) = queue.pop_front() {
            comp.push((n.to_owned(), cid));
            comp_of.insert(n.to_owned(), cid);
            for m in &adj[n] {
                if !seen.contains(m) {
                    seen.insert(m);
                    queue.push_back(m);
                }
            }
        }
    }
    if count < 2 {
        return;
    }
    let mut sequences: Vec<Vec<usize>> = Vec::new();
    for (_, row) in by_rank {
        let mut row = row.clone();
        row.sort_by(|a, b| cmp_f(cross[a], cross[b]));
        let mut seq = vec![comp_of[&row[0]]];
        for n in &row[1..] {
            let c = comp_of[n];
            if c != seq[seq.len() - 1] {
                if seq.contains(&c) {
                    return; // 塊が層の中で途切れている
                }
                seq.push(c);
            }
        }
        sequences.push(seq);
    }
    let mut order_seen: HashMap<usize, usize> = HashMap::new();
    for seq in &sequences {
        for (i, c) in seq.iter().enumerate() {
            if order_seen.get(c).is_some_and(|o| *o != i) && seq.len() == sequences[0].len() {
                return; // 塊どうしの前後関係が層によって違う
            }
        }
        for (i, c) in seq.iter().enumerate() {
            order_seen.entry(*c).or_insert(i);
        }
    }
    let span = |n: &str| {
        let s = sizes.get(n).copied().unwrap_or((2.0, 2.0));
        if direction == "TB" {
            s.0
        } else {
            s.1
        }
    };
    let mut extents: Vec<(usize, (f64, f64))> = Vec::new();
    for (n, c) in &comp {
        let (lo, hi) = (cross[n] - span(n) / 2.0, cross[n] + span(n) / 2.0);
        match extents.iter_mut().find(|(k, _)| k == c) {
            Some(slot) => slot.1 = (lo.min(slot.1 .0), hi.max(slot.1 .1)),
            None => extents.push((*c, (lo, hi))),
        }
    }
    let mut ordered = extents.clone();
    ordered.sort_by(|a, b| cmp_f(a.1 .0, b.1 .0));
    let mut cursor: Option<f64> = None;
    for (c, (lo, hi)) in ordered {
        let Some(at) = cursor else {
            cursor = Some(hi + gap_order);
            continue;
        };
        let shift = at - lo;
        let (mut lo2, mut hi2) = (lo, hi);
        if shift < 0.0 {
            for (n, cc) in &comp {
                if *cc == c {
                    *cross.get_mut(n).expect("在る") += shift;
                }
            }
            lo2 += shift;
            hi2 += shift;
        }
        let _ = lo2;
        cursor = Some(hi2 + gap_order);
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn assign_coordinates(
    expanded: &Expanded,
    order: &HashMap<String, i64>,
    sizes: &HashMap<String, (f64, f64)>,
    gap_rank: f64,
    gap_order: f64,
    direction: &str,
    tolerance: f64,
    groups: &[Vec<String>],
    group_margin: f64,
) -> HashMap<String, Point> {
    let mut by_rank: Rows = Vec::new();
    for n in &expanded.all_nodes {
        let r = expanded.rank[n];
        match by_rank.iter_mut().find(|(k, _)| *k == r) {
            Some(slot) => slot.1.push(n.clone()),
            None => by_rank.push((r, vec![n.clone()])),
        }
    }
    for (_, row) in &mut by_rank {
        row.sort_by_key(|n| order[n]);
    }
    let size_of = |n: &str| sizes.get(n).copied().unwrap_or((2.0, 2.0));
    let along = |n: &str| {
        if direction == "TB" {
            size_of(n).0
        } else {
            size_of(n).1
        }
    };
    // 群の境目だけ、枠が入るぶんを余分に空ける ── **全体の間隔を一律に広げると、群の内側まで
    // 間延びする**
    let gap_between = |prev: &str, cur: &str| -> f64 {
        if !groups.is_empty() && block_of(prev, groups) != block_of(cur, groups) {
            gap_order + group_margin
        } else {
            gap_order
        }
    };
    // 初期位置 ── 層内をそのまま均等に置く
    let mut cross: HashMap<String, f64> = HashMap::new();
    let mut ranks_sorted: Vec<i64> = by_rank.iter().map(|(k, _)| *k).collect();
    ranks_sorted.sort_unstable();
    for r in &ranks_sorted {
        let row = row_of(&by_rank, *r);
        let mut cursor = 0.0;
        for (i, n) in row.iter().enumerate() {
            let w = along(n);
            if i > 0 {
                cursor += gap_between(&row[i - 1], n) - gap_order;
            }
            cross.insert(n.clone(), cursor + w / 2.0);
            cursor += w + gap_order;
        }
    }
    let mut incoming: HashMap<String, Vec<String>> = expanded
        .all_nodes
        .iter()
        .map(|n| (n.clone(), Vec::new()))
        .collect();
    let mut outgoing = incoming.clone();
    for (a, b) in &expanded.unit_edges {
        outgoing.get_mut(a).expect("在る").push(b.clone());
        incoming.get_mut(b).expect("在る").push(a.clone());
    }

    // 層の並びを保ったまま、望んだ位置に最も近い座標へ詰める（等調回帰で厳密に解く）。
    // **開くのと閉じるのを、1つの規則で同時に扱う**（実測 ── 押す力しか無いと、一度開いた隙間が
    // 閉じず、幅が570から1974へ膨らんだ）
    let resolve_overlaps = |row: &[String], cross: &mut HashMap<String, f64>| {
        if row.is_empty() {
            return;
        }
        let span: Vec<f64> = row.iter().map(|n| along(n)).collect();
        let mut base = vec![0.0];
        for i in 1..row.len() {
            base.push(
                base[i - 1] + span[i - 1] / 2.0 + span[i] / 2.0 + gap_between(&row[i - 1], &row[i]),
            );
        }
        let mut stack: Vec<(f64, Vec<f64>)> = Vec::new();
        for (i, n) in row.iter().enumerate() {
            let mut block = vec![cross[n] - base[i]];
            let mut centre = block[0];
            while stack.last().is_some_and(|(c, _)| *c >= centre) {
                let (_, prev_block) = stack.pop().expect("在る");
                let mut merged = prev_block;
                merged.extend(block);
                block = merged;
                centre = median(&block);
            }
            stack.push((centre, block));
        }
        let mut i = 0;
        for (centre, block) in &stack {
            for _ in block {
                cross.insert(row[i].clone(), centre + base[i]);
                i += 1;
            }
        }
    };

    // **回数は決め打ちにしない** ── 1往復で動いた最大の量が、描いても見えない大きさ未満になったら止める
    let limit = order.len().max(1);
    let mut it: i64 = -1;
    let mut moved = tolerance + 1.0;
    while ((it + 1) as usize) < limit * 2 && moved > tolerance {
        it += 1;
        let snapshot = cross.clone();
        let neigh = if it % 2 == 0 { &incoming } else { &outgoing };
        let seq: Vec<i64> = if it % 2 == 0 {
            ranks_sorted.clone()
        } else {
            ranks_sorted.iter().rev().copied().collect()
        };
        for r in &seq {
            let row = row_of(&by_rank, *r).clone();
            for n in &row {
                let ns = neigh.get(n).map_or(&[][..], |x| x.as_slice());
                if !ns.is_empty() {
                    let v = sum(ns.iter().map(|p| cross[p])) / ns.len() as f64;
                    cross.insert(n.clone(), v);
                }
            }
            // 位置で並べ替え直すと、順序の層で解いた群の隣接が壊れる ── **群があるときは塊ごと
            // 動かし、塊の中だけを並べ替える**
            let new_row: Vec<String> = if groups.is_empty() {
                let mut r2 = row.clone();
                r2.sort_by(|a, b| cmp_f(cross[a], cross[b]));
                r2
            } else {
                let mut units: Vec<(i64, Vec<String>)> = Vec::new();
                for n in &row {
                    let gi = block_of(n, groups);
                    match units.last_mut() {
                        Some(last) if last.0 == gi && gi != -1 => last.1.push(n.clone()),
                        _ => units.push((gi, vec![n.clone()])),
                    }
                }
                let key =
                    |u: &(i64, Vec<String>)| sum(u.1.iter().map(|m| cross[m])) / u.1.len() as f64;
                let mut keyed: Vec<(f64, (i64, Vec<String>))> =
                    units.into_iter().map(|u| (key(&u), u)).collect();
                keyed.sort_by(|a, b| cmp_f(a.0, b.0));
                keyed
                    .into_iter()
                    .flat_map(|(_, (_, mut members))| {
                        members.sort_by(|a, b| cmp_f(cross[a], cross[b]));
                        members
                    })
                    .collect()
            };
            *row_of_mut(&mut by_rank, *r) = new_row.clone();
            resolve_overlaps(&new_row, &mut cross);
        }
        moved = cross
            .iter()
            .map(|(k, v)| (v - snapshot[k]).abs())
            .fold(0.0_f64, f64::max);
    }

    pack_components(expanded, &by_rank, &mut cross, sizes, gap_order, direction);

    let mut main: HashMap<String, f64> = HashMap::new();
    let mut cursor = 0.0;
    for r in &ranks_sorted {
        let row = row_of(&by_rank, *r);
        let extent = row
            .iter()
            .map(|n| {
                if direction == "TB" {
                    size_of(n).1
                } else {
                    size_of(n).0
                }
            })
            .fold(f64::NEG_INFINITY, f64::max);
        for n in row {
            main.insert(n.clone(), cursor);
        }
        cursor += extent + gap_rank;
    }
    let mut positions = HashMap::new();
    for n in &expanded.all_nodes {
        let (w, h) = size_of(n);
        let p = if direction == "TB" {
            (cross[n] - w / 2.0, main[n])
        } else {
            (main[n], cross[n] - h / 2.0)
        };
        positions.insert(n.clone(), p);
    }
    positions
}

// ── 公開 ────────────────────────────────────────────

/// 点と辺から、実座標つきの完全な配置を解く。**呼ばれ方は他の戦略と同じ5つの引数である。**
///
/// # Errors
///
/// 返さない（契約を他の戦略と揃えるために、形だけ持つ）。
pub fn layout_graph(
    sizes: &Sizes,
    edges: &[(String, String)],
    gap_rank: f64,
    gap_order: f64,
    direction: &str,
) -> Result<LayoutResult, Unsupported> {
    Ok(layout_graph_with(
        sizes,
        edges,
        gap_rank,
        gap_order,
        direction,
        1.0,
        &[],
        0.0,
    ))
}

/// 群と止め方まで指定して解く。
///
/// **サイクル ・ 複数層をまたぐ辺 ・ 交差する辺のいずれにも耐える**（耐える、とは ── 落ちない、
/// かつ辺が節点を突っ切らないことを指す）。
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
#[must_use]
pub fn layout_graph_with(
    sizes: &Sizes,
    edges: &[(String, String)],
    gap_rank: f64,
    gap_order: f64,
    direction: &str,
    tolerance: f64,
    groups: &[Vec<String>],
    group_margin: f64,
) -> LayoutResult {
    let nodes: Vec<String> = sizes.iter().map(|(k, _)| k.clone()).collect();
    let edges_with_rev = break_cycles(&nodes, edges);
    let dag_edges: Vec<Edge> = edges_with_rev
        .iter()
        .map(|(a, b, rev)| {
            if *rev {
                (b.clone(), a.clone())
            } else {
                (a.clone(), b.clone())
            }
        })
        .collect();
    let rank = assign_ranks(&nodes, &dag_edges);
    let expanded = expand(&nodes, &edges_with_rev, rank);
    let order = order_within_ranks(&expanded, groups);
    let mut all_sizes: HashMap<String, (f64, f64)> = sizes.iter().cloned().collect();
    for n in &expanded.all_nodes {
        all_sizes.entry(n.clone()).or_insert((1.0, 1.0)); // 仮節点は点として扱う
    }
    let positions = assign_coordinates(
        &expanded,
        &order,
        &all_sizes,
        gap_rank,
        gap_order,
        direction,
        tolerance,
        groups,
        group_margin,
    );
    // 層ごとに重心へ戻すと、左（または上）へはみ出すことがある ── **原点を左上へ取り直す**
    let ordered: Vec<(String, Point)> = expanded
        .all_nodes
        .iter()
        .map(|n| (n.clone(), positions[n]))
        .collect();
    let (shifted, _) = shift_to_origin(&ordered);
    let pos: HashMap<String, Point> = shifted.into_iter().collect();
    let centre = |n: &str| {
        let (x, y) = pos[n];
        let (w, h) = all_sizes[n];
        (x + w / 2.0, y + h / 2.0)
    };
    // 仮節点では層の入口と出口の2点を出し、**辺がその層の高さぶん並走するようにする**
    let axis_tb = direction == "TB";
    let mut rank_lo: HashMap<i64, f64> = HashMap::new();
    let mut rank_hi: HashMap<i64, f64> = HashMap::new();
    for n in &nodes {
        let r = expanded.rank[n];
        let lo = if axis_tb { pos[n].1 } else { pos[n].0 };
        let hi = lo
            + if axis_tb {
                all_sizes[n].1
            } else {
                all_sizes[n].0
            };
        rank_lo.insert(r, rank_lo.get(&r).copied().unwrap_or(lo).min(lo));
        rank_hi.insert(r, rank_hi.get(&r).copied().unwrap_or(hi).max(hi));
    }
    let mut edge_paths = Vec::new();
    for (idx, chain) in expanded.chains.iter().enumerate() {
        let mut pts: Vec<Point> = Vec::new();
        for n in chain {
            if expanded.real.contains(n) {
                pts.push(centre(n));
                continue;
            }
            let r = expanded.rank[n];
            let (cx, cy) = centre(n);
            let fallback = if axis_tb { cy } else { cx };
            let lo = rank_lo.get(&r).copied().unwrap_or(fallback);
            let hi = rank_hi.get(&r).copied().unwrap_or(fallback);
            if axis_tb {
                pts.push((cx, lo));
                pts.push((cx, hi));
            } else {
                pts.push((lo, cy));
                pts.push((hi, cy));
            }
        }
        edge_paths.push((idx, pts));
    }
    let max_x = expanded
        .all_nodes
        .iter()
        .map(|n| pos[n].0 + all_sizes[n].0)
        .fold(0.0_f64, f64::max);
    let max_y = expanded
        .all_nodes
        .iter()
        .map(|n| pos[n].1 + all_sizes[n].1)
        .fold(0.0_f64, f64::max);
    LayoutResult {
        positions: nodes.iter().map(|n| (n.clone(), pos[n])).collect(),
        edge_paths,
        width: max_x,
        height: max_y,
    }
}
