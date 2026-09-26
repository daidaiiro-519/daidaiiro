// SPDX-License-Identifier: MIT
//! 10言語すべてで、辺が取れて向きが判定できることを事例で検証する。
//!
//!     cargo test -p nms_parts
//!
//! **外の道具を1つも呼ばない。** 文法は binary へ焼き込まれているので、検査する側に
//! その言語の道具が入っていなくても測れる ── この事例がそれを固定する。
//!
//! 事例は `tests/fixtures/<言語>/{core,adapter}` に置く。`adapter` が `core` を
//! 参照する形だけを置き、**並びを逆にすると違反になること**を確かめる。

use std::path::PathBuf;

use nms_parts::inward::judge::{judge, Layer, Order};
use nms_parts::inward::syntax::Tree;
use nms_parts::inward::Extractor as _;

/// 言語ごとの、層の識別子。**識別子の空間は言語が決める** ── 経路か、点の名前か、
/// 名前空間かは、その言語の参照の書き方で決まる。
const CASES: [(&str, &str, &str); 10] = [
    ("python", "core", "adapter"),
    ("rust", "core", "adapter"),
    ("typescript", "core", "adapter"),
    ("java", "core", "adapter"),
    ("kotlin", "core", "adapter"),
    ("csharp", "core", "adapter"),
    ("php", "core", "adapter"),
    ("go", "core", "adapter"),
    ("ruby", "core", "adapter"),
    ("cpp", "core", "adapter"),
];

fn fixture(language: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(language)
}

fn order(inner: &str, outer: &str) -> Order {
    Order::inner_to_outer(vec![
        Layer::new("inner".to_owned(), inner.to_owned()),
        Layer::new("outer".to_owned(), outer.to_owned()),
    ])
}

#[test]
fn every_language_yields_edges() {
    for (language, inner, _) in CASES {
        let tree = Tree::of(language).unwrap_or_else(|| panic!("{language} の抽出器が無い"));
        let got = tree
            .extract(&fixture(language))
            .unwrap_or_else(|e| panic!("{language} で抽出できない ── {e}"));
        assert!(
            got.undecided.is_empty(),
            "{language} に判定できていない範囲が在る ── {:?}",
            got.undecided
        );
        assert!(!got.edges.is_empty(), "{language} で辺が1件も取れていない");
        assert!(
            got.edges.iter().any(|e| e.to.contains(inner)),
            "{language} で内側への参照が取れていない ── {:?}",
            got.edges
        );
    }
}

#[test]
fn the_declared_direction_is_kept() {
    for (language, inner, outer) in CASES {
        let tree = Tree::of(language).expect("抽出器が在る");
        let got = tree.extract(&fixture(language)).expect("抽出できる");
        let bad = judge(&order(inner, outer), &got.edges);
        assert!(
            bad.is_empty(),
            "{language} で正しい並びが不合格になった ── {bad:?}"
        );
    }
}

#[test]
fn reversing_the_order_makes_the_same_edge_a_violation() {
    for (language, inner, outer) in CASES {
        let tree = Tree::of(language).expect("抽出器が在る");
        let got = tree.extract(&fixture(language)).expect("抽出できる");
        // **並びを逆にすると、同じ辺が違反になる** ── 判定が言語を認知していない証しである
        let bad = judge(&order(outer, inner), &got.edges);
        assert!(!bad.is_empty(), "{language} で逆の並びを検出していない");
    }
}

#[test]
fn the_limit_of_the_method_is_always_declared() {
    for (language, _, _) in CASES {
        let tree = Tree::of(language).expect("抽出器が在る");
        let got = tree.extract(&fixture(language)).expect("抽出できる");
        // **取りこぼす範囲を黙らせない。** 申告であって、検出ではない
        assert!(
            !got.limits.is_empty(),
            "{language} が測り方の限界を申告していない"
        );
    }
}

#[test]
fn an_empty_place_is_not_a_pass() {
    let dir = std::env::temp_dir().join("inward-empty");
    std::fs::create_dir_all(&dir).expect("作れる");
    for (language, _, _) in CASES {
        let tree = Tree::of(language).expect("抽出器が在る");
        let got = tree.extract(&dir).expect("抽出できる");
        assert!(got.edges.is_empty());
        // **空振りを合格に寄せない。**
        assert!(
            !got.undecided.is_empty(),
            "{language} が0件を合格にしている"
        );
    }
}

#[test]
fn every_declared_language_has_an_extractor() {
    for language in Tree::languages() {
        assert!(
            Tree::of(language).is_some(),
            "{language} の抽出器が組めない"
        );
        assert!(
            CASES.iter().any(|(x, _, _)| *x == language),
            "{language} の事例が無い"
        );
    }
    assert_eq!(
        Tree::languages().len(),
        CASES.len(),
        "宣言した言語と事例の数が合わない"
    );
}

#[test]
fn an_unknown_language_is_refused() {
    assert!(Tree::of("cobol").is_none(), "知らない言語は返さない");
}
