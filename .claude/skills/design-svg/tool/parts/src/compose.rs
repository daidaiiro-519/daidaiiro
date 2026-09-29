// SPDX-License-Identifier: MIT
//! 合成 ── **宣言（節点 ・ 辺 ・ 囲み）から、1枚の SVG を組み立てる核。**
//!
//! この核が保証する性質:
//!
//! 1. **辺は必ず、両端のインクに着く。** 着き先は部品が申告した形ではなく、実際に描いたインクから
//!    選ぶ。迂回で入り方が変わったら決め直す
//! 2. **同じ節点へ集まる辺は、同じ点へ収束する。** どの辺から出すかだけを相手の位置で決め、その辺の
//!    中央を狙う
//! 3. **線は迂回させない。迂回させるのは節点を避けるときだけ。** ラベルは動かせるので障害物に数えない
//! 4. **迂回用の車線は、節点どうしと同じ間隔だけ離れる**
//! 5. **動かせるものが譲り、動かせないものは動かない。** 譲れないときは不透明な帯で線を断って上に載る
//! 6. **ラベルには逃げ場がある。** 層の間隔は、その間を通る辺のラベルが収まるだけ空ける
//! 7. **画布は、描いたものを全部含む。** 余白は四辺へ均等に
//! 8. **群の要素だけが、群の矩形の内側に居る**（群の入れ子の配置が保証する）
//!
//! **描く順序は 囲み → 辺 → ラベル → 節点。**

use std::collections::HashMap;

use serde_json::{json, Map, Value};

use crate::geometry::{densify, ink_surface, nearest, segment_hits_rect, Point, Rect};
use crate::labels::place_edge_labels;
use crate::layout_contract::{Strategy, Unsupported};
use crate::nesting::{layout_nested, Group};
use crate::props;
use crate::py::sum;
use crate::registry::{self, Fragment};
use crate::style::{self, Style};
use crate::sugiyama::layout_graph;
use crate::text;
use crate::theme;

type Theme = Map<String, Value>;

fn centre(pos: Point, size: (f64, f64)) -> Point {
    (pos.0 + size.0 / 2.0, pos.1 + size.1 / 2.0)
}

/// 迂回の位置が囲みの内側なら、近いほうの縁の外へ出す。
fn push_out(v: f64, s0: f64, s1: f64, keep_out: &[Rect], margin: f64, axis: usize) -> f64 {
    let (lo_s, hi_s) = if s0 <= s1 { (s0, s1) } else { (s1, s0) };
    let get = |r: &Rect, i: usize| [r.0, r.1, r.2, r.3][i];
    let other = 1 - axis;
    let mut v = v;
    for r in keep_out {
        // 迂回が伸びる向きに、この囲みと重なりがあるか
        if hi_s < get(r, other) || lo_s > get(r, other + 2) {
            continue;
        }
        if get(r, axis) < v && v < get(r, axis + 2) {
            let near_lo = v - get(r, axis);
            let near_hi = get(r, axis + 2) - v;
            v = if near_lo <= near_hi {
                get(r, axis) - margin * 2.0
            } else {
                get(r, axis + 2) + margin * 2.0
            };
        }
    }
    v
}

/// 経路が箱を突っ切るなら、ぶつかった箱の脇を回る点を挟む。
fn avoid(
    pts: &[Point],
    obstacles: &[Rect],
    direction: &str,
    margin: f64,
    keep_out: &[Rect],
) -> Vec<Point> {
    let mut out = vec![pts[0]];
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let hit: Vec<&Rect> = obstacles
            .iter()
            .filter(|o| segment_hits_rect(a, b, **o, margin))
            .collect();
        if !hit.is_empty() {
            let lo = (
                hit.iter().map(|o| o.0).fold(f64::INFINITY, f64::min),
                hit.iter().map(|o| o.1).fold(f64::INFINITY, f64::min),
            );
            let hi = (
                hit.iter().map(|o| o.2).fold(f64::NEG_INFINITY, f64::max),
                hit.iter().map(|o| o.3).fold(f64::NEG_INFINITY, f64::max),
            );
            if direction == "TB" {
                // 近い側の外へ寄せて、縦に迂回させる
                let left = (a.0 - lo.0).abs() + (b.0 - lo.0).abs();
                let right = (a.0 - hi.0).abs() + (b.0 - hi.0).abs();
                let x = if left <= right {
                    lo.0 - margin * 2.0
                } else {
                    hi.0 + margin * 2.0
                };
                let x = if keep_out.is_empty() {
                    x
                } else {
                    push_out(x, a.1, b.1, keep_out, margin, 0)
                };
                out.push((x, a.1));
                out.push((x, b.1));
            } else {
                let top = (a.1 - lo.1).abs() + (b.1 - lo.1).abs();
                let bottom = (a.1 - hi.1).abs() + (b.1 - hi.1).abs();
                let y = if top <= bottom {
                    lo.1 - margin * 2.0
                } else {
                    hi.1 + margin * 2.0
                };
                let y = if keep_out.is_empty() {
                    y
                } else {
                    push_out(y, a.0, b.0, keep_out, margin, 1)
                };
                out.push((a.0, y));
                out.push((b.0, y));
            }
        }
        out.push(b);
    }
    out
}

/// 自分から自分へ戻る辺の経路。**節点の脇へ小さな輪を作る。**
fn self_loop(pos: Point, size: (f64, f64), gap: f64, style: &Style) -> Result<Vec<Point>, String> {
    let (x, y) = pos;
    let (w, h) = size;
    let attach = style.num("size.self-loop-attach")?;
    let bulge = style.num("size.self-loop-bulge")?;
    let lift = style.num("size.self-loop-lift")?;
    let r = w.min(h) / 2.0 + gap;
    let (cx, cy) = (x + w, y + h / 2.0);
    let (top, bottom) = (y + h * attach, y + h * (1.0 - attach));
    Ok(vec![
        (x + w, top),
        (cx + r, top - r * lift),
        (cx + r * bulge, cy),
        (cx + r, bottom + r * lift),
        (x + w, bottom),
    ])
}

/// 相手が居る側を上下左右のどれかに決め、**その側のインクへ着ける。**
fn cardinal(
    pos: Point,
    size: (f64, f64),
    ink: &[Point],
    toward: Point,
    flow: Option<usize>,
) -> Point {
    let (cx, cy) = (pos.0 + size.0 / 2.0, pos.1 + size.1 / 2.0);
    let (dx, dy) = (toward.0 - cx, toward.1 - cy);
    if dx == 0.0 && dy == 0.0 {
        return (cx, cy);
    }
    // **層が進む向きで離れているなら、その向きの辺から出す**（実測 ── 箱の形だけで決めると、
    // 枝10本の木で根から両端の枝への辺が列の上を横切った）
    if let Some(flow) = flow {
        let d_flow = if flow == 1 { dy } else { dx };
        let half = if flow == 1 { size.1 } else { size.0 } / 2.0;
        if d_flow.abs() > half {
            let aim = if flow == 1 {
                (cx, pos.1 + if dy >= 0.0 { size.1 } else { 0.0 })
            } else {
                (pos.0 + if dx >= 0.0 { size.0 } else { 0.0 }, cy)
            };
            let here = nearest(ink, (aim.0 - pos.0, aim.1 - pos.1));
            return (pos.0 + here.0, pos.1 + here.1);
        }
    }
    let aim = if dx.abs() * size.1 >= dy.abs() * size.0 {
        (pos.0 + if dx >= 0.0 { size.0 } else { 0.0 }, cy)
    } else {
        (cx, pos.1 + if dy >= 0.0 { size.1 } else { 0.0 })
    };
    let here = nearest(ink, (aim.0 - pos.0, aim.1 - pos.1));
    (pos.0 + here.0, pos.1 + here.1)
}

/// 迂回経路の辺が「どちら側を回るか」を返す。**まっすぐなら `None`。**
fn detour_aim(pos: Point, size: (f64, f64), routed: &[Point], direction: &str) -> Option<Point> {
    if routed.len() <= 2 {
        return None;
    }
    let axis = usize::from(direction != "TB");
    let get = |p: Point, i: usize| if i == 0 { p.0 } else { p.1 };
    let c = get(pos, axis) + get(size, axis) / 2.0;
    // 最も離れたもの ── **同じ離れ方なら、先に在るものを選ぶ**
    let mut far: Option<(f64, Point)> = None;
    for p in &routed[1..routed.len() - 1] {
        let d = (get(*p, axis) - c).abs();
        if far.is_none_or(|(b, _)| d > b) {
            far = Some((d, *p));
        }
    }
    let (_, far) = far?;
    if (get(far, axis) - c).abs() <= get(size, axis) / 2.0 {
        return None;
    }
    let other = 1 - axis;
    let mut aim = [0.0, 0.0];
    aim[axis] = get(far, axis);
    aim[other] = get(pos, other) + get(size, other) / 2.0;
    Some((aim[0], aim[1]))
}

fn text_of(v: &Value, key: &str) -> String {
    v.get(key).map_or_else(String::new, props::text)
}

fn truthy(v: &Value, key: &str) -> bool {
    v.get(key).is_some_and(props::truthy)
}

fn array_of<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

/// 節点の中身として置く子図を組み立てる。**深さに上限を置く** ── 無いと、自分を指す宣言で
/// 終わらなくなる。
fn nested(decl: &Value, theme_: &Theme, depth: i64, label: &str) -> Result<Fragment, String> {
    let limit = theme::num(theme_, "size.figure-depth-limit")? as i64;
    if depth >= limit {
        return Err(format!("図の入れ子が深すぎる（上限 {limit}）"));
    }
    let direction = decl
        .get("direction")
        .map_or_else(|| "TB".to_owned(), props::text);
    let inner = figure_fragment(
        array_of(decl, "nodes"),
        array_of(decl, "edges"),
        array_of(decl, "groups"),
        &direction,
        theme_,
        None,
        depth + 1,
    )?;
    if label.is_empty() {
        return Ok(inner);
    }
    // **名前を持つ入れ子は、囲んで名札を付ける**（実測 ── 付けないと、親から子図への辺が子の先頭の
    // 節点を指しているようにしか見えなかった）
    let st = style::resolve("plain", None, Some(theme_))?;
    let pad = st.num("font.size-small")? * st.num("size.frame-pad-ratio")?;
    let label_h = st.num("font.size-small")? * st.num("size.label-line-h")?;
    let (w, h) = (inner.width + pad * 2.0, inner.height + pad * 2.0);
    let frame = registry::render(
        &st.text("parts.group")?,
        &as_props(&json!({"x": 0, "y": label_h, "width": w, "height": h, "label": null})),
        &st,
    )?;
    let tag = registry::render(
        "frame_label",
        &as_props(&json!({"x": st.num("size.label-pad-x")?, "y": label_h, "label": label})),
        &st,
    )?;
    let svg = format!(
        "{}<g transform=\"translate({pad:.1},{:.1})\">{}</g>{}",
        frame.svg,
        label_h + pad,
        inner.svg,
        tag.svg
    );
    Ok(Fragment::own(svg, w, h + label_h).labelled())
}

/// 部品へ渡す入力を組む。
fn as_props(v: &Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

/// 節点 ・ 辺 ・ 囲みの宣言から、**部品として置ける断片**を組み立てる。
///
/// ルートタグを被せない ── 返すのは中身と、それを囲む大きさ、つまり部品と同じ契約である。
/// だから図を他の図の中へ置ける。
///
/// # Errors
///
/// 宣言が存在しない節点を指したときと、戦略が描けないときと、部品が描けないときに返す。
#[allow(clippy::too_many_lines)]
pub fn figure_fragment(
    nodes: &[Value],
    edges: &[Value],
    groups: &[Value],
    direction: &str,
    theme_: &Theme,
    layout: Option<Strategy>,
    depth: i64,
) -> Result<Fragment, String> {
    // 層が進む向き。**層状のときだけ辺の出入りに効かせる**
    let flow = if layout.is_none() {
        Some(usize::from(direction == "TB"))
    } else {
        None
    };
    let strategy: Strategy = layout.unwrap_or(layout_graph);

    let mut rendered: Vec<(String, Fragment)> = Vec::new();
    for n in nodes {
        let role = n
            .get("role")
            .map_or_else(|| "plain".to_owned(), props::text);
        let st = style::resolve(
            &role,
            n.get("style").and_then(Value::as_object),
            Some(theme_),
        )?;
        let id = n
            .get("id")
            .map(props::text)
            .ok_or_else(|| crate::py::quote("id"))?;
        let frag = if truthy(n, "figure") {
            // 節点の中身が図 ── **子図を先に組み立てて、大きさの分かった1つにする**
            nested(&n["figure"], theme_, depth, &text_of(n, "label"))?
        } else {
            let node_props = n.as_object().cloned().unwrap_or_default();
            registry::render_node(&st.text("parts.node")?, &node_props, &st)?
        };
        match rendered.iter_mut().find(|(k, _)| *k == id) {
            Some(slot) => slot.1 = frag,
            None => rendered.push((id, frag)),
        }
    }
    let sizes: Vec<(String, (f64, f64))> = rendered
        .iter()
        .map(|(k, r)| (k.clone(), (r.width, r.height)))
        .collect();
    let size_of: HashMap<String, (f64, f64)> = sizes.iter().cloned().collect();
    // 辺の着き先は、**部品が描いたインクそのものから選ぶ**
    let fineness = theme::num(theme_, "size.outline-facets")? as i64;
    let inks: HashMap<String, Vec<Point>> = rendered
        .iter()
        .map(|(k, r)| (k.clone(), ink_surface(&r.svg, r.width, r.height, fineness)))
        .collect();
    let edge_pairs: Vec<(String, String)> = edges
        .iter()
        .map(|e| (text_of(e, "from"), text_of(e, "to")))
        .collect();

    // 囲みの余白とラベルの高さは、書体のトークンから導く
    let frame_style = style::resolve("plain", None, Some(theme_))?;
    let frame_pad =
        frame_style.num("font.size-small")? * frame_style.num("size.frame-pad-ratio")?;
    let label_h = frame_style.num("font.size-small")? * frame_style.num("size.label-line-h")?;

    // **層の間隔は、その間を通る辺のラベルが収まるだけ空ける**（実測 ── 3節点の鎖で4件重なった）
    let mut gap_rank = theme::num(theme_, "size.gap-rank")?;
    let gap_order = theme::num(theme_, "size.gap-order")?;
    if !edges.is_empty() {
        let fs = frame_style.num("font.size-small")?;
        let pad = theme::num(theme_, "size.label-pad-x")?;
        let need = edges
            .iter()
            .filter(|e| truthy(e, "label"))
            .map(|e| text::width(&text_of(e, "label"), fs) + pad)
            .fold(0.0_f64, f64::max);
        if direction == "LR" {
            gap_rank = gap_rank.max(need + gap_order);
        } else {
            // 縦に進む辺では、ラベルは帯の高さぶんしか層を占めない
            gap_rank = gap_rank.max(theme::num(theme_, "size.label-band-h")? + gap_order);
        }
    }

    let group_list: Vec<Group> = groups
        .iter()
        .map(|g| {
            let label = g.get("label").filter(|v| props::truthy(v)).map(props::text);
            let members = array_of(g, "members").iter().map(props::text).collect();
            (label, members)
        })
        .collect();
    let (coords, group_boxes, edge_paths, total_w, total_h) = if groups.is_empty() {
        let result = strategy(&sizes, &edge_pairs, gap_rank, gap_order, direction)
            .map_err(|e| e.to_string())?;
        let coords: HashMap<String, Point> = result.positions.iter().cloned().collect();
        let paths: HashMap<usize, Vec<Point>> = result.edge_paths.iter().cloned().collect();
        (coords, HashMap::new(), paths, result.width, result.height)
    } else {
        // **群を先に解いて1つの大きさへ集約し、親はそれを1個として置く**
        let res = layout_nested(
            &sizes,
            &edge_pairs,
            &group_list,
            gap_rank,
            gap_order,
            direction,
            frame_pad,
            label_h,
            strategy,
        )
        .map_err(|e: Unsupported| e.to_string())?;
        let coords: HashMap<String, Point> = res
            .node_boxes
            .iter()
            .map(|(k, b)| (k.clone(), (b.x, b.y)))
            .collect();
        // 経路が解けなかった辺（群の内側で完結するなど）だけ、両端を直結する
        let mut paths = HashMap::new();
        for (i, (a, b)) in edge_pairs.iter().enumerate() {
            let p = match res.edge_paths.get(&i) {
                Some(p) => p.clone(),
                None => {
                    let ca = coords.get(a).ok_or_else(|| crate::py::quote(a))?;
                    let cb = coords.get(b).ok_or_else(|| crate::py::quote(b))?;
                    vec![centre(*ca, size_of[a]), centre(*cb, size_of[b])]
                }
            };
            paths.insert(i, p);
        }
        (coords, res.group_boxes, paths, res.width, res.height)
    };
    let coord = |k: &str| coords.get(k).copied().ok_or_else(|| crate::py::quote(k));
    let size = |k: &str| size_of.get(k).copied().ok_or_else(|| crate::py::quote(k));

    // 囲みは節点の外側へはみ出す ── **画布の大きさを節点だけから決めると、この分が切れる**
    let frame_bounds: Vec<Rect> = (0..groups.len())
        .map(|i| {
            let b = group_boxes[&format!("__g{i}")];
            (b.x, b.y, b.x + b.width, b.y + b.height)
        })
        .collect();
    let edge_style = style::resolve("plain", None, Some(theme_))?;

    let mut edge_points: Vec<Vec<Point>> = Vec::new();
    for (idx, (a, b)) in edge_pairs.iter().enumerate() {
        if a == b {
            // 自分へ戻る辺は、配置の解いた経路（同じ点が2つ）では表せない
            edge_points.push(self_loop(
                coord(a)?,
                size(a)?,
                edge_style.num("size.gap-order")? / 2.0,
                &edge_style,
            )?);
            continue;
        }
        let mut pts = edge_paths
            .get(&idx)
            .cloned()
            .ok_or_else(|| idx.to_string())?;
        let nxt = if pts.len() > 1 {
            pts[1]
        } else {
            pts[pts.len() - 1]
        };
        let prv = if pts.len() > 1 {
            pts[pts.len() - 2]
        } else {
            pts[0]
        };
        let (ia, ib) = (&inks[a], &inks[b]);
        pts[0] = cardinal(coord(a)?, size(a)?, ia, nxt, flow);
        let last = pts.len() - 1;
        pts[last] = cardinal(coord(b)?, size(b)?, ib, prv, flow);
        // 端の2つ以外が占めている領域のうち、**動かせないもの（節点）だけが障害物**
        let obstacles: Vec<Rect> = coords
            .iter()
            .filter(|(n, _)| *n != a && *n != b)
            .map(|(n, p)| {
                let s = size_of[n];
                (p.0, p.1, p.0 + s.0, p.1 + s.1)
            })
            .collect();
        let mut routed = avoid(
            &pts,
            &obstacles,
            direction,
            edge_style.num("size.stroke-width")? * 2.0,
            &frame_bounds,
        );
        // **迂回で入り方が変わったら、接続点も決め直す**
        if routed.len() > 1 {
            let mut aim_a = detour_aim(coord(a)?, size(a)?, &routed, direction);
            let mut aim_b = detour_aim(coord(b)?, size(b)?, &routed, direction);
            routed[0] = cardinal(coord(a)?, size(a)?, ia, aim_a.unwrap_or(routed[1]), flow);
            let last = routed.len() - 1;
            routed[last] = cardinal(
                coord(b)?,
                size(b)?,
                ib,
                aim_b.unwrap_or(routed[last - 1]),
                flow,
            );
            // 横の辺から出入りするなら、まず横へ抜けてから曲がる
            let axis = usize::from(direction != "TB");
            let get = |p: Point, i: usize| if i == 0 { p.0 } else { p.1 };
            let set = |p: &mut Point, i: usize, v: f64| if i == 0 { p.0 = v } else { p.1 = v };
            // **迂回用の車線は、両端の節点からも離す**（実測 ── 4.3px しか離れず、迂回に見えなかった）
            let clear = theme::num(theme_, "size.gap-order")?;
            for (aim, node) in [(aim_a, a), (aim_b, b)] {
                let Some(aim) = aim else { continue };
                let c = get(coord(node)?, axis) + get(size(node)?, axis) / 2.0;
                let need = get(size(node)?, axis) / 2.0 + clear;
                let mut lane = get(aim, axis);
                if (lane - c).abs() < need {
                    lane = if lane >= c { c + need } else { c - need };
                    let n_pts = routed.len();
                    for pt in routed.iter_mut().take(n_pts - 1).skip(1) {
                        if (get(*pt, axis) - c) * (lane - c) > 0.0 {
                            set(pt, axis, lane);
                        }
                    }
                    if axis == 0 {
                        aim_a = aim_a.map(|p| (lane, p.1));
                        aim_b = aim_b.map(|p| (lane, p.1));
                    }
                }
            }
            if let Some(aim) = aim_a {
                let mut corner = routed[0];
                set(&mut corner, axis, get(aim, axis));
                routed.insert(1, corner);
            }
            if let Some(aim) = aim_b {
                let mut corner = routed[routed.len() - 1];
                set(&mut corner, axis, get(aim, axis));
                let at = routed.len() - 1;
                routed.insert(at, corner);
            }
        }
        edge_points.push(routed);
    }

    // 囲みのラベルを、線を避けた位置へ置く ── **動かせない線のほうを優先し、ラベルが譲る**
    let mut frame_svgs: Vec<String> = Vec::new();
    let mut top_labels: Vec<String> = Vec::new();
    let mut frame_label_areas: Vec<Rect> = Vec::new();
    // 線の太さより粗く刻むと、線を跳び越して「当たっていない」と誤判定する
    let probe = frame_style.num("size.stroke-width")?.max(0.5);
    let all_points: Vec<Point> = edge_points.iter().flat_map(|p| densify(p, probe)).collect();
    for (i, g) in groups.iter().enumerate() {
        let b = group_boxes[&format!("__g{i}")];
        let has_label = truthy(g, "label");
        let pad_x = frame_style.num("size.label-pad-x")?;
        let mut label_x = b.x + pad_x;
        if has_label {
            let label = text_of(g, "label");
            let fs = frame_style.num("font.size-small")?;
            let lw =
                text::width_with(&label, fs, frame_style.num("font.latin-width-ratio")?) + pad_x;
            // **ラベルの縦位置は、部品が実際に描く位置と同じ式から出す**
            let top = b.y + label_h - fs * frame_style.num("size.frame-label-rise")?;
            let bottom = top + fs * frame_style.num("size.frame-label-h")?;
            let step = probe;
            let n_steps = (((b.width - lw - pad_x * 2.0) / step) as i64).max(0);
            for k in 0..=n_steps {
                let cx = b.x + pad_x + step * k as f64;
                if !all_points
                    .iter()
                    .any(|(x, y)| cx < *x && *x < cx + lw && top < *y && *y < bottom)
                {
                    label_x = cx;
                    break;
                }
            }
            frame_label_areas.push((label_x, top, label_x + lw, bottom));
        }
        let lift = if has_label { label_h } else { 0.0 };
        let r = registry::render(
            &frame_style.text("parts.group")?,
            &as_props(
                &json!({"x": b.x, "y": b.y + lift, "width": b.width, "height": b.height - lift, "label": null}),
            ),
            &frame_style,
        )?;
        frame_svgs.push(r.svg);
        if has_label {
            // **ラベルは辺より後に描く** ── 避けられなかったときでも、帯が線を断って読める
            top_labels.push(
                registry::render(
                    "frame_label",
                    &as_props(
                        &json!({"x": label_x, "y": b.y + label_h, "label": text_of(g, "label")}),
                    ),
                    &frame_style,
                )?
                .svg,
            );
        }
    }
    let mut body: Vec<String> = frame_svgs;

    // ラベルどうしの重なりは、辺1本ずつでは判断できない ── **まとめて渡す**
    let labelled: Vec<(usize, Vec<Point>, String)> = edges
        .iter()
        .enumerate()
        .filter(|(_, e)| truthy(e, "label"))
        .map(|(i, e)| (i, edge_points[i].clone(), text_of(e, "label")))
        .collect();
    // 節点そのものも、ラベルが避けるべき相手（実測 ── 渡していなかったときは、3節点の鎖で4件重なった）
    let node_areas: Vec<Rect> = coords
        .iter()
        .map(|(n, p)| {
            let s = size_of[n];
            (p.0, p.1, p.0 + s.0, p.1 + s.1)
        })
        .collect();
    // 囲みの枠線が占める領域（線の太さぶんの細い帯4本）。**ラベルは枠線を隠してはいけない**
    let sw = frame_style.num("size.stroke-width")? * 2.0;
    let mut frame_line_areas: Vec<Rect> = Vec::new();
    for (x0, y0, x1, y1) in &frame_bounds {
        frame_line_areas.extend([
            (x0 - sw, y0 - sw, x1 + sw, y0 + sw),
            (x0 - sw, y1 - sw, x1 + sw, y1 + sw),
            (x0 - sw, y0 - sw, x0 + sw, y1 + sw),
            (x1 - sw, y0 - sw, x1 + sw, y1 + sw),
        ]);
    }
    let label_at: HashMap<usize, Point> = if labelled.is_empty() {
        HashMap::new()
    } else {
        let mut occupied = frame_label_areas.clone();
        occupied.extend(frame_line_areas);
        occupied.extend(node_areas);
        place_edge_labels(&labelled, &edge_style, &occupied)?
            .into_iter()
            .collect()
    };
    for (idx, e) in edges.iter().enumerate() {
        let pts: Vec<Value> = edge_points[idx]
            .iter()
            .map(|(x, y)| json!([x, y]))
            .collect();
        let at = label_at
            .get(&idx)
            .map_or(Value::Null, |(x, y)| json!([x, y]));
        let p = json!({
            "points": pts,
            "label": e.get("label").cloned().unwrap_or(Value::Null),
            "label_at": at,
            "dashed": e.get("dashed").cloned().unwrap_or(Value::Bool(false)),
            "arrow": e.get("arrow").cloned().unwrap_or_else(|| Value::from("head")),
        });
        body.push(
            registry::render(&edge_style.text("parts.edge")?, &as_props(&p), &edge_style)?.svg,
        );
    }
    body.extend(top_labels);
    for n in nodes {
        let id = text_of(n, "id");
        let (x, y) = coord(&id)?;
        let svg = &rendered.iter().find(|(k, _)| *k == id).expect("在る").1.svg;
        // **節点であることに印を付ける** ── 検査は「辺の終端が節点のインクに着いているか」を判定する
        body.push(format!(
            "<g class=\"wf-node\" transform=\"translate({x:.1},{y:.1})\">{svg}</g>"
        ));
    }

    // 画布は、節点だけでなく囲みのはみ出し ・ 迂回経路 ・ その上のラベルまで含めて取る
    let pad = frame_style.num("size.canvas-pad")?;
    let fs_small = edge_style.num("font.size-small")?;
    let lab_h = fs_small * edge_style.num("size.label-line-h")?;
    let mut edge_bounds: Vec<Rect> = Vec::new();
    for (idx, e) in edges.iter().enumerate() {
        let xs: Vec<f64> = edge_points[idx].iter().map(|p| p.0).collect();
        let ys: Vec<f64> = edge_points[idx].iter().map(|p| p.1).collect();
        let lo_x = xs.iter().copied().fold(f64::INFINITY, f64::min);
        let lo_y = ys.iter().copied().fold(f64::INFINITY, f64::min);
        let hi_x = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let hi_y = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        edge_bounds.push((lo_x, lo_y, hi_x, hi_y));
        if truthy(e, "label") {
            let (cx, cy) = label_at.get(&idx).copied().unwrap_or_else(|| {
                (
                    sum(xs.iter().copied()) / xs.len() as f64,
                    sum(ys.iter().copied()) / ys.len() as f64,
                )
            });
            let lw = text::width_with(
                &text_of(e, "label"),
                fs_small,
                edge_style.num("font.latin-width-ratio")?,
            ) + edge_style.num("size.label-pad-x")?;
            edge_bounds.push((cx - lw / 2.0, cy - lab_h, cx + lw / 2.0, cy + lab_h / 2.0));
        }
    }
    let all_b: Vec<&Rect> = frame_bounds.iter().chain(edge_bounds.iter()).collect();
    let min_x = all_b.iter().map(|b| b.0).fold(0.0_f64, f64::min);
    let min_y = all_b.iter().map(|b| b.1).fold(0.0_f64, f64::min);
    let max_x = all_b.iter().map(|b| b.2).fold(total_w, f64::max);
    let max_y = all_b.iter().map(|b| b.3).fold(total_h, f64::max);
    // **余白は四辺へ均等に取る** ── 右下にだけ足すと、いちばん幅の広い要素が左端 ・ 上端に貼り付く
    let half = pad / 2.0;
    let w = max_x - min_x + pad;
    let h = max_y - min_y + pad;
    let (dx, dy) = (half - min_x, half - min_y);
    let inner = if dx != 0.0 || dy != 0.0 {
        format!(
            "<g transform=\"translate({dx:.1},{dy:.1})\">{}</g>",
            body.concat()
        )
    } else {
        body.concat()
    };
    Ok(Fragment::own(inner, w, h))
}

/// 図を1枚の完結した SVG として描く。**ここがするのは器を被せることだけ。**
///
/// # Errors
///
/// 組み立てられないときに返す。
pub fn render_figure(
    nodes: &[Value],
    edges: &[Value],
    groups: &[Value],
    direction: &str,
    theme_: Option<&Theme>,
    layout: Option<Strategy>,
) -> Result<String, String> {
    let theme_ = theme_.unwrap_or_else(|| theme::default_theme());
    let r = figure_fragment(nodes, edges, groups, direction, theme_, layout, 0)?;
    Ok(format!(
        "<svg class=\"wf-fig\" viewBox=\"0 0 {:.0} {:.0}\" width=\"{:.0}\" height=\"{:.0}\" role=\"img\">{}</svg>",
        r.width, r.height, r.width, r.height, r.svg
    ))
}

/// 自動配置を要らない部品を、それ単独で1枚の SVG へ描く。
///
/// **値から座標が一意に決まる部品は、配置の解決を経由しない。**
///
/// # Errors
///
/// 部品が描けないときに返す。
pub fn render_chart(
    kind: &str,
    props_: &Map<String, Value>,
    role: &str,
    overrides: Option<&Map<String, Value>>,
    theme_: Option<&Theme>,
) -> Result<String, String> {
    let st = style::resolve(
        role,
        overrides,
        Some(theme_.unwrap_or_else(|| theme::default_theme())),
    )?;
    let r = registry::render(kind, props_, &st)?;
    let pad = st.num("size.canvas-pad-tight")?;
    let (w, h) = (r.width + pad, r.height + pad);
    Ok(format!("<svg class=\"wf-fig\" viewBox=\"0 0 {w:.0} {h:.0}\" width=\"{w:.0}\" height=\"{h:.0}\" role=\"img\">{}</svg>", r.svg))
}
