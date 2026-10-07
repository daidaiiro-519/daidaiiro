// SPDX-License-Identifier: MIT
//! 形の演算と、自由な形。
//!
//! **受けると宣言した文法は、実際に受けられるかを機械で照合する** ── 宣言しただけの契約は
//! 守られない。

use ds_business_logic::boolean::{
    boolean_op, circle_polygon, point_in_polygon, rect_polygon, CIRCLE_FACETS,
};
use ds_business_logic::registry::render;
use ds_business_logic::shapes_freeform::{path_bounds, path_points};
use ds_business_logic::style::resolve;
use ds_business_logic::theme::Style;
use serde_json::{json, Map, Value};

fn area(poly: &[(f64, f64)]) -> f64 {
    let n = poly.len();
    (0..n)
        .map(|i| {
            let (x1, y1) = poly[i];
            let (x2, y2) = poly[(i + 1) % n];
            x1 * y2 - x2 * y1
        })
        .sum::<f64>()
        .abs()
        / 2.0
}

fn circle(cx: f64, cy: f64, r: f64) -> Vec<(f64, f64)> {
    circle_polygon(cx, cy, r, CIRCLE_FACETS)
}

#[test]
fn subtracting_inner_shape_yields_a_hole() {
    let res = boolean_op(
        &[circle(70.0, 70.0, 55.0), circle(70.0, 70.0, 25.0)],
        "subtract",
    )
    .expect("演算できる");
    assert_eq!(res.len(), 2);
    // 穴の中心は塗りの外 ── 偶奇則で数える
    assert_eq!(
        res.iter()
            .filter(|p| point_in_polygon((70.0, 70.0), p))
            .count()
            % 2,
        0
    );
}

#[test]
fn subtracting_overlap_reduces_area() {
    let a = circle(60.0, 60.0, 45.0);
    let res = boolean_op(&[a.clone(), circle(100.0, 60.0, 45.0)], "subtract").expect("演算できる");
    assert_eq!(res.len(), 1);
    assert!(area(&res[0]) < area(&a));
}

#[test]
fn subtracting_the_container_leaves_nothing() {
    assert!(boolean_op(
        &[circle(70.0, 70.0, 25.0), circle(70.0, 70.0, 55.0)],
        "subtract"
    )
    .expect("演算できる")
    .is_empty());
}

#[test]
fn subtracting_a_disjoint_shape_changes_nothing() {
    let a = rect_polygon(0.0, 0.0, 20.0, 20.0);
    let res = boolean_op(
        &[a.clone(), rect_polygon(100.0, 100.0, 20.0, 20.0)],
        "subtract",
    )
    .expect("演算できる");
    assert_eq!(res.len(), 1);
    assert!((area(&res[0]) - area(&a)).abs() < 1e-9);
}

#[test]
fn union_and_intersection_of_overlapping_shapes() {
    let a = circle(60.0, 60.0, 45.0);
    let b = circle(100.0, 60.0, 45.0);
    let u: f64 = boolean_op(&[a.clone(), b.clone()], "union")
        .expect("演算できる")
        .iter()
        .map(|p| area(p))
        .sum();
    let i: f64 = boolean_op(&[a.clone(), b], "intersect")
        .expect("演算できる")
        .iter()
        .map(|p| area(p))
        .sum();
    assert!(u > area(&a));
    assert!(i < area(&a));
}

#[test]
fn disjoint_shapes() {
    let a = rect_polygon(0.0, 0.0, 20.0, 20.0);
    let b = rect_polygon(100.0, 0.0, 20.0, 20.0);
    assert!(boolean_op(&[a.clone(), b.clone()], "intersect")
        .expect("演算できる")
        .is_empty());
    assert_eq!(boolean_op(&[a, b], "union").expect("演算できる").len(), 2);
}

#[test]
fn guards() {
    assert!(
        boolean_op(&[rect_polygon(0.0, 0.0, 10.0, 10.0)], "union").is_err(),
        "形1つは断る"
    );
    assert!(
        boolean_op(
            &[
                rect_polygon(0.0, 0.0, 10.0, 10.0),
                rect_polygon(5.0, 5.0, 10.0, 10.0)
            ],
            "xor"
        )
        .is_err(),
        "知らない演算は断る"
    );
}

#[test]
fn path_syntax_parses() {
    for d in [
        "M0,40 Q30,0 60,40 Q90,80 120,40",             // 絶対の2次曲線
        "M-24 28 v-6 c0 -27 48 -27 48 0 v6",           // 相対の3次曲線
        "M35 17 a17 17 0 1 1 -34 0 a17 17 0 1 1 34 0", // 円弧
        "M0 0 H56 V36 H0 Z",                           // 水平垂直と閉じ
        "M0,0 L10,0 S20,0 20,10 T30,20",               // 省略形
    ] {
        let (_, _, w, h) = path_bounds(d).unwrap_or_else(|e| panic!("{d}: {e}"));
        assert!(w > 0.0 && h > 0.0, "{d}");
    }
}

#[test]
fn arc_includes_bulge_beyond_endpoints() {
    let (_, _, w, h) = path_bounds("M0,0 a10 10 0 0 1 0,20").expect("読める");
    assert!((w - 10.0).abs() < 0.5 && (h - 20.0).abs() < 0.5, "{w} {h}");
}

#[test]
fn close_returns_to_start() {
    assert_eq!(
        path_points("M10,10 L20,10 Z").expect("読める").last(),
        Some(&(10.0, 10.0))
    );
}

#[test]
fn unparsable_path_is_refused() {
    for d in ["M0,0 L", "M0,0 X10,10"] {
        assert!(path_bounds(d).is_err(), "{d}");
    }
}

#[test]
fn coordinates_are_not_rewritten() {
    let d = "M-24 28 v-6 c0 -27 48 -27 48 0 v6";
    let svg = render("path", &props(json!({"d": d, "filled": false})), &style())
        .expect("描ける")
        .svg;
    assert!(svg.contains(d));
}

fn style() -> Style {
    resolve("plain", None, None).expect("解決できる")
}

fn props(v: Value) -> Map<String, Value> {
    v.as_object().expect("対応表").clone()
}

/// 辺1本を描く。
fn edge_svg(extra: Value) -> String {
    let mut p = json!({"points": [[0, 100], [0, 0]]});
    for (k, v) in extra.as_object().expect("対応表") {
        p[k] = v.clone();
    }
    render("edge", &props(p), &style()).expect("描ける").svg
}

#[test]
fn a_hook_is_an_open_curve_not_a_filled_head() {
    let svg = edge_svg(json!({"arrowhead": "hook"}));
    assert!(!svg.contains("<polygon"), "かぎは塗った三角ではない: {svg}");
    assert!(svg.contains(" A"), "かぎは弧で描く: {svg}");
}

#[test]
fn an_unknown_arrowhead_is_refused() {
    let p = props(json!({"points": [[0, 100], [0, 0]], "arrowhead": "star"}));
    assert!(render("edge", &p, &style()).is_err());
}
