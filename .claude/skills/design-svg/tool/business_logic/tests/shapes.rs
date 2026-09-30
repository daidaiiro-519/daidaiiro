// SPDX-License-Identifier: MIT
//! 形の演算と、自由な形 ・ 文字 ・ 絵記号。
//!
//! **受けると宣言した文法は、実際に受けられるかを機械で照合する** ── 宣言しただけの契約は
//! 守られない。

use ds_business_logic::boolean::{
    boolean_op, circle_polygon, point_in_polygon, rect_polygon, CIRCLE_FACETS,
};
use ds_business_logic::canvas::render_canvas;
use ds_business_logic::registry::render;
use ds_business_logic::shapes_decor::icon_names;
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

fn canvas(w: f64, h: f64, layers: Value) -> String {
    render_canvas(w, h, layers.as_array().expect("並び"), None, None).expect("描ける")
}

#[test]
fn coordinates_are_not_rewritten() {
    let d = "M-24 28 v-6 c0 -27 48 -27 48 0 v6";
    let svg = canvas(
        200.0,
        120.0,
        json!([{"kind": "path", "x": 10, "y": 10, "props": {"d": d, "filled": false}}]),
    );
    assert!(svg.contains(d));
}

#[test]
fn text_accepts_a_sequence_of_lines() {
    let svg = canvas(
        300.0,
        120.0,
        json!([{"kind": "text", "x": 10, "y": 10, "props": {"text": ["1行目", "2行目"]}}]),
    );
    assert_eq!(svg.matches("<text").count(), 2);
}

fn style() -> Style {
    resolve("plain", None, None).expect("解決できる")
}

fn props(v: Value) -> Map<String, Value> {
    v.as_object().expect("対応表").clone()
}

#[test]
fn alignment_does_not_move_the_bounding_box() {
    let st = style();
    let base = render("text", &props(json!({"text": "あいうえお"})), &st).expect("描ける");
    for align in ["start", "middle", "end"] {
        let other = render(
            "text",
            &props(json!({"text": "あいうえお", "align": align})),
            &st,
        )
        .expect("描ける");
        assert_eq!(
            (base.width, base.height),
            (other.width, other.height),
            "{align}"
        );
    }
}

#[test]
fn glyphs_share_their_size_and_come_from_the_registry() {
    let st = style();
    let person =
        render("icon", &props(json!({"name": "person", "size": 40})), &st).expect("描ける");
    let doc = render("icon", &props(json!({"name": "doc", "size": 40})), &st).expect("描ける");
    assert_eq!((person.width, person.height), (40.0, 40.0));
    assert_eq!((doc.width, doc.height), (40.0, 40.0));
    for n in ["person", "doc", "spark"] {
        assert!(icon_names().contains(&n), "{n}");
    }
}

#[test]
fn scaling_does_not_thicken_strokes() {
    let st = style();
    let effective = |size: i64| {
        let svg = render("icon", &props(json!({"name": "doc", "size": size})), &st)
            .expect("描ける")
            .svg;
        let grab = |pre: &str, stop: char| -> f64 {
            let at = svg.find(pre).expect("在る") + pre.len();
            svg[at..]
                .split(stop)
                .next()
                .expect("在る")
                .parse()
                .expect("数")
        };
        (grab("scale(", ')') * grab("stroke-width=\"", '"') * 1000.0).round()
    };
    assert_eq!(effective(24), effective(96));
}

#[test]
fn unknown_glyph_name_is_refused() {
    assert!(render("icon", &props(json!({"name": "無い絵"})), &style()).is_err());
}

#[test]
fn geometry_stays_sound_when_tiled() {
    let mut layers: Vec<Value> = icon_names().iter().enumerate().map(|(i, n)| json!({"kind": "icon", "x": 10 + i * 40, "y": 10, "props": {"name": n, "size": 32}})).collect();
    layers.push(json!({"kind": "icon", "x": 10, "y": 60, "props": {"name": "person", "size": 48}}));
    layers.push(json!({"kind": "text", "x": 70, "y": 60, "props": {"text": "人と絵", "size": 16, "weight": "bold"}}));
    let svg = canvas(560.0, 130.0, Value::Array(layers));
    assert!(ds_business_logic::verify::check(&svg).is_empty());
    assert!(ds_business_logic::verify::check_shapes(&svg).is_empty());
}
