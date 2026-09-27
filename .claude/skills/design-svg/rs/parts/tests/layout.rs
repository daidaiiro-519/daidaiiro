// SPDX-License-Identifier: MIT
//! 配置の計算そのもの。**SVG を1文字も作らずに、座標と個数を検証する。**

use ds_parts::grid::{layout_grid, Grid, Key};
use ds_parts::layout_contract::{LayoutResult, Unsupported};
use ds_parts::nesting::layout_nested;
use ds_parts::radial::layout_radial;
use ds_parts::sugiyama::layout_graph;
use ds_parts::tree::layout_tree;
use serde_json::{json, Value};

const SIZE: (f64, f64) = (80.0, 40.0);

fn sizes(ids: &[&str]) -> Vec<(String, (f64, f64))> {
    ids.iter().map(|i| ((*i).to_owned(), SIZE)).collect()
}

fn edges(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
        .collect()
}

fn at(r: &LayoutResult, id: &str) -> (f64, f64) {
    r.position(id)
        .unwrap_or_else(|| panic!("{id} が置かれていない"))
}

fn no_overlap(r: &LayoutResult, sizes: &[(String, (f64, f64))]) -> bool {
    for (i, (a, (aw, ah))) in sizes.iter().enumerate() {
        let (ax, ay) = at(r, a);
        for (b, (bw, bh)) in &sizes[i + 1..] {
            let (bx, by) = at(r, b);
            if ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by {
                return false;
            }
        }
    }
    true
}

/// 層の番号を、座標から読み戻す。**同じ高さの箱は、同じ層なら同じ y を持つ。**
fn ranks(r: &LayoutResult) -> Vec<(String, usize)> {
    let mut ys: Vec<f64> = r.positions.iter().map(|(_, p)| p.1).collect();
    ys.sort_by(f64::total_cmp);
    ys.dedup();
    r.positions
        .iter()
        .map(|(k, p)| (k.clone(), ys.iter().position(|y| *y == p.1).unwrap_or(0)))
        .collect()
}

fn rank_of(rs: &[(String, usize)], id: &str) -> usize {
    rs.iter()
        .find(|(k, _)| k == id)
        .map(|(_, r)| *r)
        .unwrap_or(usize::MAX)
}

#[test]
fn layers_descend_along_edge_direction() {
    let r = layout_graph(
        &sizes(&["a", "b", "c"]),
        &edges(&[("a", "b"), ("b", "c")]),
        40.0,
        30.0,
        "TB",
    )
    .expect("解ける");
    assert!(at(&r, "a").1 < at(&r, "b").1 && at(&r, "b").1 < at(&r, "c").1);
}

#[test]
fn horizontal_mode_descends_sideways() {
    let r =
        layout_graph(&sizes(&["a", "b"]), &edges(&[("a", "b")]), 40.0, 30.0, "LR").expect("解ける");
    assert!(at(&r, "a").0 < at(&r, "b").0);
    assert!((at(&r, "a").1 - at(&r, "b").1).abs() < 1e-9);
}

#[test]
fn layers_minimize_total_edge_length() {
    let pairs = [
        ("長1", "長2"),
        ("長2", "長3"),
        ("長3", "長4"),
        ("長4", "合"),
        ("短1", "短2"),
        ("短2", "合"),
        ("中1", "中2"),
        ("中2", "中3"),
        ("中3", "合"),
    ];
    let ids = [
        "長1", "長2", "長3", "長4", "短1", "短2", "中1", "中2", "中3", "合",
    ];
    let r = layout_graph(&sizes(&ids), &edges(&pairs), 40.0, 30.0, "TB").expect("解ける");
    let rs = ranks(&r);
    let total: usize = pairs
        .iter()
        .map(|(a, b)| rank_of(&rs, b) - rank_of(&rs, a))
        .sum();
    assert_eq!(total, pairs.len());
}

#[test]
fn disconnected_components_get_their_own_layers() {
    let r = layout_graph(
        &sizes(&["a", "b", "x", "y"]),
        &edges(&[("a", "b"), ("x", "y")]),
        40.0,
        30.0,
        "TB",
    )
    .expect("解ける");
    let rs = ranks(&r);
    assert_eq!(rank_of(&rs, "a"), 0);
    assert_eq!(rank_of(&rs, "x"), 0);
    assert_eq!(rank_of(&rs, "b"), 1);
    assert_eq!(rank_of(&rs, "y"), 1);
}

#[test]
fn cycles_resolve_without_overlapping_nodes() {
    let s = sizes(&["a", "b", "c"]);
    let r = layout_graph(
        &s,
        &edges(&[("a", "b"), ("b", "c"), ("c", "a")]),
        40.0,
        30.0,
        "TB",
    )
    .expect("解ける");
    assert_eq!(r.positions.len(), 3);
    assert!(no_overlap(&r, &s));
}

#[test]
fn self_edge_still_resolves() {
    let r = layout_graph(&sizes(&["a"]), &edges(&[("a", "a")]), 40.0, 30.0, "TB").expect("解ける");
    assert!(r.position("a").is_some());
}

#[test]
fn layer_skipping_edge_gets_a_bend() {
    // 仮節点を経由するから、2点の直線ではなくなる
    let r = layout_graph(
        &sizes(&["a", "b", "c"]),
        &edges(&[("a", "b"), ("b", "c"), ("a", "c")]),
        40.0,
        30.0,
        "TB",
    )
    .expect("解ける");
    assert!(r.path(2).expect("在る").len() > 2);
}

#[test]
fn nested_group_fully_contains_the_inner_one() {
    let groups = vec![
        (
            Some("外".to_owned()),
            vec!["a", "b", "c", "d"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
        ),
        (Some("内".to_owned()), vec!["c".to_owned(), "d".to_owned()]),
    ];
    let n = layout_nested(
        &sizes(&["a", "b", "c", "d"]),
        &edges(&[("a", "b"), ("c", "d")]),
        &groups,
        40.0,
        30.0,
        "TB",
        12.0,
        16.0,
        layout_graph,
    )
    .expect("解ける");
    let mut boxes: Vec<_> = n.group_boxes.values().copied().collect();
    boxes.sort_by(|a, b| (a.width * a.height).total_cmp(&(b.width * b.height)));
    let (inner, outer) = (boxes[0], boxes[boxes.len() - 1]);
    assert!(outer.x <= inner.x && outer.y <= inner.y);
    assert!(outer.x + outer.width >= inner.x + inner.width);
    assert!(outer.y + outer.height >= inner.y + inner.height);
}

#[test]
fn non_nested_overlap_is_refused() {
    // 1つの節点が2つの群に半端に属す形は、この戦略では解けない
    let groups = vec![
        (Some("甲".to_owned()), vec!["a".to_owned(), "b".to_owned()]),
        (Some("乙".to_owned()), vec!["b".to_owned(), "c".to_owned()]),
    ];
    let r = layout_nested(
        &sizes(&["a", "b", "c"]),
        &[],
        &groups,
        40.0,
        30.0,
        "TB",
        12.0,
        16.0,
        layout_graph,
    );
    assert!(matches!(r, Err(Unsupported::ByStrategy(_))));
}

fn centres(r: &LayoutResult, ids: &[String]) -> Vec<(f64, f64)> {
    ids.iter()
        .map(|k| (at(r, k).0 + SIZE.0 / 2.0, at(r, k).1 + SIZE.1 / 2.0))
        .collect()
}

#[test]
fn radial_nodes_are_evenly_spaced_on_one_circle() {
    let s = sizes(&["a", "b", "c", "d"]);
    let r = layout_radial(
        &s,
        &edges(&[("a", "b"), ("b", "c"), ("c", "d"), ("d", "a")]),
        40.0,
        30.0,
        "TB",
    )
    .expect("解ける");
    let ids: Vec<String> = s.iter().map(|(k, _)| k.clone()).collect();
    let c = centres(&r, &ids);
    let (ox, oy) = (
        c.iter().map(|p| p.0).sum::<f64>() / 4.0,
        c.iter().map(|p| p.1).sum::<f64>() / 4.0,
    );
    let radii: Vec<f64> = c.iter().map(|(x, y)| (x - ox).hypot(y - oy)).collect();
    let spread = radii.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        - radii.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(spread < 1.0);
}

fn same(ids: &str, w: f64, h: f64) -> Vec<(String, (f64, f64))> {
    ids.chars().map(|c| (c.to_string(), (w, h))).collect()
}

#[test]
fn radial_bigger_nodes_give_a_bigger_circle() {
    // 半径を決め打ちしていないことを、大きさを変えて検証する
    let small = layout_radial(&same("abcd", 40.0, 20.0), &[], 40.0, 30.0, "TB").expect("解ける");
    let big = layout_radial(&same("abcd", 200.0, 100.0), &[], 40.0, 30.0, "TB").expect("解ける");
    assert!(big.width > small.width);
}

#[test]
fn radial_nodes_never_overlap_at_any_size() {
    for (w, h) in [(40.0, 20.0), (200.0, 100.0), (300.0, 30.0)] {
        let s = same("abcde", w, h);
        let r = layout_radial(&s, &[], 40.0, 30.0, "TB").expect("解ける");
        assert!(no_overlap(&r, &s), "{w}x{h}");
    }
}

#[test]
fn strategies_return_the_same_shape() {
    // 戦略を差し替えられる、という根拠 ── 同じ型で、同じ節点を置く
    let s = sizes(&["a", "b"]);
    let e = edges(&[("a", "b")]);
    let keys = |r: LayoutResult| -> Vec<String> {
        let mut k: Vec<String> = r.positions.into_iter().map(|(k, _)| k).collect();
        k.sort();
        k
    };
    let g = keys(layout_graph(&s, &e, 40.0, 30.0, "TB").expect("解ける"));
    assert_eq!(
        g,
        keys(layout_radial(&s, &e, 40.0, 30.0, "TB").expect("解ける"))
    );
    assert_eq!(
        g,
        keys(layout_tree(&s, &e, 40.0, 30.0, "TB").expect("解ける"))
    );
}

#[test]
fn radial_circle_order_comes_from_edges_not_declaration() {
    // 宣言順に従うと、隣り合うべき節点が輪の反対側へ行き、絵はもつれた星になる
    let keys: Vec<String> = (0..8).map(|i| format!("n{i}")).collect();
    let cyc: Vec<(String, String)> = (0..8)
        .map(|i| (keys[i].clone(), keys[(i + 1) % 8].clone()))
        .collect();
    let shuffled: Vec<(String, (f64, f64))> = keys
        .iter()
        .step_by(2)
        .chain(keys.iter().skip(1).step_by(2))
        .map(|k| (k.clone(), SIZE))
        .collect();
    let r = layout_radial(&shuffled, &cyc, 40.0, 30.0, "TB").expect("解ける");
    let c = centres(&r, &keys);
    let (ox, oy) = (
        c.iter().map(|p| p.0).sum::<f64>() / 8.0,
        c.iter().map(|p| p.1).sum::<f64>() / 8.0,
    );
    let angle = |k: &str| {
        let i = keys.iter().position(|x| x == k).expect("在る");
        (c[i].1 - oy).atan2(c[i].0 - ox)
    };
    let step = std::f64::consts::TAU / 8.0;
    for (a, b) in &cyc {
        let d = (angle(a) - angle(b)).abs() % std::f64::consts::TAU;
        let d = d.min(std::f64::consts::TAU - d);
        assert!((d - step).abs() < step * 0.1, "{a}-{b}: {d}");
    }
}

/// 節点の大きさと辺。
type Graph = (Vec<(String, (f64, f64))>, Vec<(String, String)>);

fn mindmap(branches: usize) -> Graph {
    let mut ids = vec![("root".to_owned(), SIZE)];
    let mut es = Vec::new();
    for i in 0..branches {
        let a = format!("a{i}");
        ids.push((a.clone(), SIZE));
        es.push(("root".to_owned(), a.clone()));
        for j in 0..3 {
            let b = format!("b{i}{j}");
            ids.push((b.clone(), SIZE));
            es.push((a.clone(), b));
        }
    }
    (ids, es)
}

#[test]
fn radial_shape_that_cannot_fit_the_circle_is_refused() {
    // 1つの節点から3方向以上へ分かれる木は、輪の上に並べきれない
    let (s, e) = mindmap(3);
    assert!(matches!(
        layout_radial(&s, &e, 40.0, 30.0, "TB"),
        Err(Unsupported::ByStrategy(_))
    ));
}

#[test]
fn tree_root_is_at_the_center() {
    let (s, e) = mindmap(4);
    let r = layout_tree(&s, &e, 40.0, 30.0, "TB").expect("解ける");
    let ids: Vec<String> = s.iter().map(|(k, _)| k.clone()).collect();
    let c = centres(&r, &ids);
    let n = c.len() as f64;
    let root = centres(&r, &["root".to_owned()])[0];
    assert!((root.0 - c.iter().map(|p| p.0).sum::<f64>() / n).abs() < SIZE.0);
    assert!((root.1 - c.iter().map(|p| p.1).sum::<f64>() / n).abs() < SIZE.1);
}

#[test]
fn tree_deeper_nodes_are_farther_from_center() {
    let (s, e) = mindmap(4);
    let r = layout_tree(&s, &e, 40.0, 30.0, "TB").expect("解ける");
    let o = centres(&r, &["root".to_owned()])[0];
    let dist = |k: String| {
        let p = centres(&r, &[k])[0];
        (p.0 - o.0).hypot(p.1 - o.1)
    };
    let near = (0..4)
        .map(|i| dist(format!("a{i}")))
        .fold(f64::NEG_INFINITY, f64::max);
    let far = (0..4)
        .flat_map(|i| (0..3).map(move |j| format!("b{i}{j}")))
        .map(dist)
        .fold(f64::INFINITY, f64::min);
    assert!(near < far);
}

#[test]
fn tree_nodes_do_not_overlap() {
    for br in [2, 4, 10] {
        let (s, e) = mindmap(br);
        assert!(
            no_overlap(&layout_tree(&s, &e, 40.0, 30.0, "TB").expect("解ける"), &s),
            "{br}"
        );
    }
}

#[test]
fn tree_more_branches_do_not_form_a_band() {
    // 比べるのは形（縦横比）と大きさの向きだけで、何倍という数は置かない
    let (s, e) = mindmap(10);
    let t = layout_tree(&s, &e, 40.0, 30.0, "TB").expect("解ける");
    let g = layout_graph(&s, &e, 40.0, 30.0, "TB").expect("解ける");
    let ratio = |r: &LayoutResult| r.width.max(r.height) / r.width.min(r.height);
    assert!(ratio(&t) < 2.0);
    assert!(ratio(&g) > ratio(&t) * 2.0);
    assert!(t.width.max(t.height) < g.width.max(g.height));
}

#[test]
fn tree_cycles_and_unreachable_nodes_are_placed() {
    let r = layout_tree(
        &sizes(&["a", "b", "c"]),
        &edges(&[("a", "b"), ("b", "c"), ("c", "a")]),
        40.0,
        30.0,
        "TB",
    )
    .expect("解ける");
    assert_eq!(r.positions.len(), 3);
    let r = layout_tree(
        &sizes(&["a", "b", "x"]),
        &edges(&[("a", "b")]),
        40.0,
        30.0,
        "TB",
    )
    .expect("解ける");
    assert!(["a", "b", "x"].iter().all(|k| r.position(k).is_some()));
}

fn v(x: &str) -> Value {
    serde_json::from_str(x).expect("JSON")
}

fn faults(svg: &str) -> Vec<String> {
    ds_parts::verify::all(svg)
}

#[test]
fn edge_labels_get_room_between_layers() {
    // 層の間隔は、その間を通る辺のラベルが収まるだけ空ける
    let nodes = [
        v(r#"{"id":"a","label":"概念"}"#),
        v(r#"{"id":"b","label":"満たすべき性質"}"#),
        v(r#"{"id":"c","label":"アルゴリズム"}"#),
    ];
    for (label, dir) in [
        ("保証", "LR"),
        ("何が成り立てばその概念かを言う", "LR"),
        ("何が成り立てばその概念かを言う", "TB"),
    ] {
        let es = [
            json!({"from": "a", "to": "b", "label": label}),
            json!({"from": "b", "to": "c", "label": label}),
        ];
        let svg =
            ds_parts::compose::render_figure(&nodes, &es, &[], dir, None, None).expect("描ける");
        assert!(faults(&svg).is_empty(), "{label} {dir}: {:?}", faults(&svg));
    }
}

#[test]
fn figure_assembly_adds_no_wrapper_and_contains_its_ink() {
    use ds_parts::compose::figure_fragment;
    use ds_parts::theme::default_theme;
    let r = figure_fragment(
        &[v(r#"{"id":"a","label":"A"}"#)],
        &[],
        &[],
        "TB",
        default_theme(),
        None,
        0,
    )
    .expect("描ける");
    assert!(!r.svg.contains("<svg") && r.width > 0.0 && r.height > 0.0);
    let r = figure_fragment(
        &[
            v(r#"{"id":"a","label":"A"}"#),
            v(r#"{"id":"b","label":"B"}"#),
        ],
        &[v(r#"{"from":"a","to":"b","label":"渡す"}"#)],
        &[],
        "TB",
        default_theme(),
        None,
        0,
    )
    .expect("描ける");
    let pts: Vec<(f64, f64)> =
        ds_parts::geometry::sample_ink(&r.svg, r.width.min(r.height).max(1.0) / 32.0)
            .into_iter()
            .map(|(p, _)| p)
            .collect();
    assert!(!pts.is_empty());
    assert!(pts
        .iter()
        .all(|p| p.0 >= -1.0 && p.1 >= -1.0 && p.0 <= r.width + 1.0 && p.1 <= r.height + 1.0));
}

#[test]
fn child_figure_can_be_placed_as_a_node() {
    let child = json!({"nodes": [{"id": "x", "label": "子1"}, {"id": "y", "label": "子2"}], "edges": [{"from": "x", "to": "y"}]});
    let svg = ds_parts::compose::render_figure(
        &[
            v(r#"{"id":"a","label":"親"}"#),
            json!({"id": "b", "figure": child}),
        ],
        &[v(r#"{"from":"a","to":"b"}"#)],
        &[],
        "TB",
        None,
        None,
    )
    .expect("描ける");
    assert!(faults(&svg).is_empty(), "{:?}", faults(&svg));
    assert!(svg.contains("子1") && svg.contains("子2"));
}

#[test]
fn nesting_depth_is_bounded() {
    let limit = ds_parts::theme::num(ds_parts::theme::default_theme(), "size.figure-depth-limit")
        .expect("在る") as usize;
    let mut deep = json!({"nodes": [{"id": "leaf", "label": "葉"}]});
    for _ in 0..=limit {
        deep = json!({"nodes": [{"id": "n", "figure": deep}]});
    }
    let err = ds_parts::compose::render_figure(
        &[json!({"id": "a", "figure": deep})],
        &[],
        &[],
        "TB",
        None,
        None,
    )
    .expect_err("断る");
    assert!(err.contains("深すぎる"), "{err}");
}

/// 節点と、その `(列, 行)`。
type Cell<'a> = (&'a str, Value, Value);

fn grid(at: &[Cell]) -> Grid {
    Grid {
        at: at
            .iter()
            .map(|(k, c, r)| ((*k).to_owned(), (Key::of(c), Key::of(r))))
            .collect(),
        ..Grid::default()
    }
}

#[test]
fn grid_order_can_be_supplied() {
    let s = vec![
        ("h".to_owned(), (40.0, 20.0)),
        ("a".to_owned(), (40.0, 20.0)),
    ];
    let mut g = grid(&[
        ("h", json!("見出し"), json!("上")),
        ("a", json!("左"), json!("上")),
    ]);
    g.rows = vec![Key::of(&json!("上"))];
    g.cols = vec![Key::of(&json!("見出し")), Key::of(&json!("左"))];
    let left = layout_grid(&s, &[], 10.0, 10.0, &g).expect("解ける");
    g.cols.reverse();
    let right = layout_grid(&s, &[], 10.0, 10.0, &g).expect("解ける");
    assert!(at(&left, "h").0 < at(&left, "a").0);
    assert!(at(&right, "h").0 > at(&right, "a").0);
}

#[test]
fn grid_node_without_coordinates_is_refused() {
    assert!(layout_grid(
        &[("a".to_owned(), (10.0, 10.0))],
        &[],
        10.0,
        10.0,
        &Grid::default()
    )
    .is_err());
}

#[test]
fn grid_elbow_edge_has_one_corner() {
    let s = vec![
        ("a".to_owned(), (100.0, 32.0)),
        ("b".to_owned(), (80.0, 32.0)),
    ];
    let e = edges(&[("a", "b")]);
    let mut g = grid(&[("a", json!(0), json!(0)), ("b", json!(1), json!(1))]);
    let straight = layout_grid(&s, &e, 20.0, 20.0, &g).expect("解ける");
    assert_eq!(straight.path(0).expect("在る").len(), 2);
    g.elbow = vec![(("a".to_owned(), "b".to_owned()), "vertical".to_owned())];
    let down = layout_grid(&s, &e, 20.0, 20.0, &g).expect("解ける");
    let p = down.path(0).expect("在る");
    assert!((p[1].0 - p[0].0).abs() < 1e-9);
    g.elbow = vec![(("a".to_owned(), "b".to_owned()), "horizontal".to_owned())];
    let across = layout_grid(&s, &e, 20.0, 20.0, &g).expect("解ける");
    let p = across.path(0).expect("在る");
    assert!((p[1].1 - p[0].1).abs() < 1e-9);
}

#[test]
fn grid_no_corner_when_it_coincides_with_an_end() {
    let s = vec![
        ("a".to_owned(), (80.0, 32.0)),
        ("b".to_owned(), (80.0, 32.0)),
    ];
    let mut g = grid(&[("a", json!(0), json!(0)), ("b", json!(0), json!(1))]);
    g.elbow = vec![(("a".to_owned(), "b".to_owned()), "vertical".to_owned())];
    assert_eq!(
        layout_grid(&s, &edges(&[("a", "b")]), 20.0, 20.0, &g)
            .expect("解ける")
            .path(0)
            .expect("在る")
            .len(),
        2
    );
}

#[test]
fn grid_elbow_must_name_an_edge_and_a_known_orientation() {
    let s = vec![
        ("a".to_owned(), (10.0, 10.0)),
        ("b".to_owned(), (10.0, 10.0)),
    ];
    let e = edges(&[("a", "b")]);
    let mut g = grid(&[("a", json!(0), json!(0)), ("b", json!(1), json!(1))]);
    g.elbow = vec![(("a".to_owned(), "c".to_owned()), "vertical".to_owned())];
    assert!(
        layout_grid(&s, &e, 10.0, 10.0, &g).is_err(),
        "辺に無いものを鍵線にできない"
    );
    g.elbow = vec![(("a".to_owned(), "b".to_owned()), "ななめ".to_owned())];
    assert!(
        layout_grid(&s, &e, 10.0, 10.0, &g).is_err(),
        "角の置き方は2つだけ"
    );
}
