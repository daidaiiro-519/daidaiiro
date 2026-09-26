// SPDX-License-Identifier: MIT
//! 層の並びから禁じる辺を導く3つの規則を、事例で検証する。
//!
//!     cargo test -p inward_core

use inward_core::{forbidden, judge, Because, Edge, Layer, Order};

fn order(names: &[&str]) -> Order {
    Order::inner_to_outer(
        names
            .iter()
            .map(|n| Layer::new((*n).to_owned(), (*n).to_owned()))
            .collect(),
    )
}

fn pairs(o: &Order) -> Vec<(String, String, Because)> {
    forbidden(o)
        .into_iter()
        .map(|f| (f.from, f.to, f.because))
        .collect()
}

#[test]
fn inner_must_not_reach_outer() {
    let o = order(&["core", "app", "adapter"]);
    let got = pairs(&o);
    assert!(got.contains(&("core".into(), "app".into(), Because::InnerReachesOuter)));
    assert!(
        got.contains(&("core".into(), "adapter".into(), Because::InnerReachesOuter)),
        "1段飛ばしも禁じる"
    );
    assert!(got.contains(&("app".into(), "adapter".into(), Because::InnerReachesOuter)));
    assert_eq!(got.len(), 3, "外から内は禁じない");
}

#[test]
fn one_layer_forbids_nothing() {
    assert!(forbidden(&order(&["core"])).is_empty());
    assert!(forbidden(&Order::default()).is_empty(), "層が無ければ0件");
}

#[test]
fn siblings_are_forbidden_only_when_independent() {
    let plain = order(&["core", "app"]);
    assert!(
        !pairs(&plain).iter().any(|(f, t, _)| f == t),
        "既定では兄弟を禁じない"
    );
    let o = Order::inner_to_outer(vec![
        Layer::new("core".into(), "core".into()).independent(),
        Layer::new("app".into(), "app".into()),
    ]);
    assert!(pairs(&o).contains(&(
        "core".into(),
        "core".into(),
        Because::SiblingsAreIndependent
    )));
}

#[test]
fn a_closed_layer_blocks_the_outer_ones() {
    let o = Order::inner_to_outer(vec![
        Layer::new("core".into(), "core".into()),
        Layer::new("app".into(), "app".into()).closed(),
        Layer::new("adapter".into(), "adapter".into()),
    ]);
    let got = pairs(&o);
    assert!(
        got.contains(&(
            "adapter".into(),
            "core".into(),
            Because::CrossesAClosedLayer
        )),
        "閉じた層を越えて内側へ到達できない"
    );
    assert!(
        !got.contains(&("app".into(), "core".into(), Because::CrossesAClosedLayer)),
        "隣は越えていない"
    );
}

#[test]
fn a_violation_carries_where_it_is_written() {
    let o = order(&["core", "app"]);
    let edges = vec![Edge::new("core".into(), "app".into(), "core/x.py:3".into())];
    let bad = judge(&o, &edges);
    assert_eq!(bad.len(), 1);
    assert_eq!(bad[0].at, "core/x.py:3");
    assert_eq!(bad[0].because, Because::InnerReachesOuter);
}

#[test]
fn the_allowed_direction_is_not_a_violation() {
    let o = order(&["core", "app"]);
    let edges = vec![Edge::new("app".into(), "core".into(), "app/y.py:1".into())];
    assert!(judge(&o, &edges).is_empty(), "外から内は許す");
}

#[test]
fn inside_one_layer_is_free() {
    let o = order(&["core", "app"]);
    let edges = vec![Edge::new(
        "core.a".into(),
        "core.b".into(),
        "core/a.py:1".into(),
    )];
    assert!(judge(&o, &edges).is_empty());
}

#[test]
fn outside_every_layer_is_not_judged() {
    let o = order(&["core", "app"]);
    let edges = vec![Edge::new(
        "core.a".into(),
        "std.io".into(),
        "core/a.py:1".into(),
    )];
    assert!(judge(&o, &edges).is_empty(), "層の外は対象外である");
}

#[test]
fn the_longest_matching_layer_wins() {
    // **短いほうへ吸われると違反が消える。** core と core.deep が並ぶとき、
    // core.deep.x は core.deep に属する
    let o = Order::inner_to_outer(vec![
        Layer::new("deep".into(), "core.deep".into()),
        Layer::new("core".into(), "core".into()),
    ]);
    let edges = vec![Edge::new(
        "core.deep.x".into(),
        "core.y".into(),
        "a:1".into(),
    )];
    let bad = judge(&o, &edges);
    assert_eq!(bad.len(), 1, "deep から core は、内から外である");
    assert_eq!(bad[0].from, "deep");
}

#[test]
fn separators_of_three_kinds_are_read() {
    let o = order(&["core", "app"]);
    for id in ["core.a", "core/a", "core::a"] {
        let edges = vec![Edge::new((*id).to_owned(), "app".into(), "a:1".into())];
        assert_eq!(judge(&o, &edges).len(), 1, "{id} が層に属していない");
    }
    let edges = vec![Edge::new("corex".into(), "app".into(), "a:1".into())];
    assert!(judge(&o, &edges).is_empty(), "前方一致だけでは層に属さない");
}
