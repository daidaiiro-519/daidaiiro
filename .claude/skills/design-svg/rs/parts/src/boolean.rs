// SPDX-License-Identifier: MIT
//! 多角形のブーリアン演算 ── 和 ・ 積 ・ 差。**Greiner と Hormann の方式である。**
//!
//! SVG にはブーリアン演算が無いので、「型抜き」に相当する結果を座標そのものを解いて作る。
//! 曲線は扱わず、多角形（点の並び）どうしの演算に限る。
//!
//! 片方がもう片方の内側に完全に収まる差は、外周と内周の2つの輪郭を返す ── 描く側がこの2つを
//! 1つのパスの別々の輪郭として置き、偶奇規則で塗ると、内側が穴になる。
//!
//! **頂点は並びの番号で結ぶ** ── 互いを指し合う環を、番号の対で持つ。

use crate::geometry::Point;

/// 円を多角形で近似するときの既定の分割数。
pub const CIRCLE_FACETS: usize = 48;

#[derive(Debug, Clone)]
struct V {
    x: f64,
    y: f64,
    next: usize,
    prev: usize,
    intersect: bool,
    entry: bool,
    neighbor: Option<usize>,
    alpha: f64,
    visited: bool,
}

impl V {
    fn new(x: f64, y: f64, intersect: bool, alpha: f64, at: usize) -> Self {
        Self {
            x,
            y,
            next: at,
            prev: at,
            intersect,
            entry: true,
            neighbor: None,
            alpha,
            visited: false,
        }
    }
}

/// 点が多角形の内側に在るか（偶奇規則）。
#[must_use]
pub fn point_in_polygon(pt: Point, poly: &[Point]) -> bool {
    let (x, y) = pt;
    let mut inside = false;
    let n = poly.len();
    for i in 0..n {
        let (x1, y1) = poly[i];
        let (x2, y2) = poly[(i + 1) % n];
        if (y1 > y) != (y2 > y) {
            let x_at_y = (x2 - x1) * (y - y1) / (y2 - y1) + x1;
            if x < x_at_y {
                inside = !inside;
            }
        }
    }
    inside
}

/// 2つの線分が、両方とも端点を除く内部で交わるなら `(p に沿った割合, q に沿った割合, x, y)`。
fn seg_intersect(p1: Point, p2: Point, q1: Point, q2: Point) -> Option<(f64, f64, f64, f64)> {
    let ((x1, y1), (x2, y2), (x3, y3), (x4, y4)) = (p1, p2, q1, q2);
    let d = (x2 - x1) * (y4 - y3) - (y2 - y1) * (x4 - x3);
    if d.abs() < 1e-12 {
        return None;
    }
    let a = ((x3 - x1) * (y4 - y3) - (y3 - y1) * (x4 - x3)) / d;
    let b = ((x3 - x1) * (y2 - y1) - (y3 - y1) * (x2 - x1)) / d;
    if 1e-9 < a && a < 1.0 - 1e-9 && 1e-9 < b && b < 1.0 - 1e-9 {
        return Some((a, b, x1 + a * (x2 - x1), y1 + a * (y2 - y1)));
    }
    None
}

/// 環を組み、先頭の番号の並びを返す。
fn build(arena: &mut Vec<V>, points: &[Point]) -> Vec<usize> {
    let base = arena.len();
    let n = points.len();
    for (i, (x, y)) in points.iter().enumerate() {
        arena.push(V::new(*x, *y, false, 0.0, base + i));
    }
    for i in 0..n {
        arena[base + i].next = base + (i + 1) % n;
        arena[base + i].prev = base + (i + n - 1) % n;
    }
    (base..base + n).collect()
}

/// `s1 → s2` の辺の上に、既に挿入済みの交点も含めて割合の順に挿む。
fn insert(arena: &mut [V], vertex: usize, s1: usize, s2: usize) {
    let mut cur = s1;
    while arena[cur].next != s2 && arena[arena[cur].next].alpha < arena[vertex].alpha {
        cur = arena[cur].next;
    }
    let after = arena[cur].next;
    arena[vertex].next = after;
    arena[vertex].prev = cur;
    arena[after].prev = vertex;
    arena[cur].next = vertex;
}

fn mark_entries(arena: &mut [V], head: usize, start_inside: bool) {
    let mut entry = !start_inside;
    let mut v = head;
    loop {
        if arena[v].intersect && !arena[v].visited {
            arena[v].entry = entry;
            entry = !entry;
        }
        v = arena[v].next;
        if v == head {
            break;
        }
    }
}

fn walk(arena: &[V], head: usize) -> Vec<usize> {
    let mut out = Vec::new();
    let mut v = head;
    loop {
        out.push(v);
        v = arena[v].next;
        if v == head {
            break;
        }
    }
    out
}

fn trace(arena: &mut [V], starts: &[usize]) -> Vec<Vec<Point>> {
    let mut polygons = Vec::new();
    for &start in starts {
        if arena[start].visited {
            continue;
        }
        let mut poly = Vec::new();
        let mut current = start;
        loop {
            arena[current].visited = true;
            poly.push((arena[current].x, arena[current].y));
            let forward = arena[current].entry;
            current = if forward {
                arena[current].next
            } else {
                arena[current].prev
            };
            while !arena[current].intersect {
                poly.push((arena[current].x, arena[current].y));
                current = if forward {
                    arena[current].next
                } else {
                    arena[current].prev
                };
            }
            arena[current].visited = true;
            // 交点の頂点は必ず相方を持つ（対で結ぶ）
            let Some(next) = arena[current].neighbor else {
                break;
            };
            current = next;
            if current == start {
                break;
            }
        }
        if poly.len() >= 3 {
            polygons.push(poly);
        }
    }
    polygons
}

fn degenerate(subject: &[Point], clip: &[Point], invert: bool) -> Vec<Vec<Point>> {
    let subj_in_clip = point_in_polygon(subject[0], clip);
    let clip_in_subj = point_in_polygon(clip[0], subject);
    if invert {
        // 和
        if subj_in_clip {
            return vec![clip.to_vec()];
        }
        if clip_in_subj {
            return vec![subject.to_vec()];
        }
        return vec![subject.to_vec(), clip.to_vec()]; // 離れている
    }
    // 積。差は呼ぶ側が clip を反転させて渡す
    if subj_in_clip {
        return vec![subject.to_vec()];
    }
    if clip_in_subj {
        return vec![clip.to_vec()];
    }
    Vec::new()
}

fn do_clip(
    subject: &[Point],
    clip: &[Point],
    invert_subject: bool,
    invert_clip: bool,
) -> Vec<Vec<Point>> {
    let mut arena: Vec<V> = Vec::new();
    let subj = build(&mut arena, subject);
    let clp = build(&mut arena, clip);
    let (ns, nc) = (subj.len(), clp.len());
    let mut found_any = false;
    for i in 0..ns {
        let (s1, s2) = (subj[i], subj[(i + 1) % ns]);
        for j in 0..nc {
            let (c1, c2) = (clp[j], clp[(j + 1) % nc]);
            let hit = seg_intersect(
                (arena[s1].x, arena[s1].y),
                (arena[s2].x, arena[s2].y),
                (arena[c1].x, arena[c1].y),
                (arena[c2].x, arena[c2].y),
            );
            if let Some((a, b, ix, iy)) = hit {
                found_any = true;
                let iv_s = arena.len();
                arena.push(V::new(ix, iy, true, a, iv_s));
                let iv_c = arena.len();
                arena.push(V::new(ix, iy, true, b, iv_c));
                arena[iv_s].neighbor = Some(iv_c);
                arena[iv_c].neighbor = Some(iv_s);
                insert(&mut arena, iv_s, s1, s2);
                insert(&mut arena, iv_c, c1, c2);
            }
        }
    }
    if !found_any {
        return degenerate(subject, clip, invert_subject);
    }
    let mut subj_inside = point_in_polygon((arena[subj[0]].x, arena[subj[0]].y), clip);
    let mut clip_inside = point_in_polygon((arena[clp[0]].x, arena[clp[0]].y), subject);
    if invert_subject {
        subj_inside = !subj_inside;
    }
    if invert_clip {
        clip_inside = !clip_inside;
    }
    mark_entries(&mut arena, subj[0], subj_inside);
    mark_entries(&mut arena, clp[0], clip_inside);
    let starts: Vec<usize> = walk(&arena, subj[0])
        .into_iter()
        .filter(|v| arena[*v].intersect)
        .collect();
    trace(&mut arena, &starts)
}

/// 交点が無いときの差。**包まれる側があるなら穴として返す。**
fn degenerate_subtract(subject: &[Point], clip: &[Point]) -> Vec<Vec<Point>> {
    if point_in_polygon(clip[0], subject) {
        // clip が subject の内側 ── 外周と、向きを逆にした内周
        let mut hole = clip.to_vec();
        hole.reverse();
        return vec![subject.to_vec(), hole];
    }
    if point_in_polygon(subject[0], clip) {
        return Vec::new(); // subject が丸ごと削られる
    }
    vec![subject.to_vec()] // 離れている
}

/// 2つの多角形の辺が交わるか。
fn has_crossing(a: &[Point], b: &[Point]) -> bool {
    for i in 0..a.len() {
        let (p1, p2) = (a[i], a[(i + 1) % a.len()]);
        for j in 0..b.len() {
            let (q1, q2) = (b[j], b[(j + 1) % b.len()]);
            if seg_intersect(p1, p2, q1, q2).is_some() {
                return true;
            }
        }
    }
    false
}

/// 3つ以上の形も、先頭から順に演算を適用して処理する。
///
/// # Errors
///
/// 演算が知らない名前のときと、形が2つ未満のときに返す。
pub fn boolean_op(shapes: &[Vec<Point>], op: &str) -> Result<Vec<Vec<Point>>, String> {
    if shapes.len() < 2 {
        return Err("boolean_opにはshapesを2つ以上渡すこと".to_owned());
    }
    let mut acc = vec![shapes[0].clone()];
    for nxt in &shapes[1..] {
        let mut merged = Vec::new();
        for a in &acc {
            match op {
                "union" => merged.extend(do_clip(a, nxt, true, true)),
                "intersect" => merged.extend(do_clip(a, nxt, false, false)),
                "subtract" => {
                    if has_crossing(a, nxt) {
                        merged.extend(do_clip(a, nxt, false, true));
                    } else {
                        merged.extend(degenerate_subtract(a, nxt));
                    }
                }
                other => return Err(format!("知らない演算です: {other}")),
            }
        }
        acc = merged;
        if acc.is_empty() {
            break;
        }
    }
    Ok(acc)
}

/// 円を多角形にする。
#[must_use]
pub fn circle_polygon(cx: f64, cy: f64, r: f64, n: usize) -> Vec<Point> {
    use std::f64::consts::PI;
    (0..n)
        .map(|i| {
            let a = 2.0 * PI * i as f64 / n as f64;
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

/// 矩形を多角形にする。
#[must_use]
pub fn rect_polygon(x: f64, y: f64, w: f64, h: f64) -> Vec<Point> {
    vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
}
