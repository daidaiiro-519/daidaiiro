// SPDX-License-Identifier: MIT
//! 向きの判定が課す義務のうち、**言語に依存しない2つ**を事例で固定する。
//!
//! - 作業領域の中を指すのに、どの層にも属さない参照を報告する
//! - 層を指すのに、その層にモジュールが無い参照を報告する
//!
//! **判定は言語を認知しない。** 受け取るのは層の並び ・ 依存 ・ モジュールの一覧 ・
//! 名前の区切り方だけである。
//!
//!     cargo test -p nms_parts --test obligations

use nms_parts::inward::judge::{unresolved, Edge, Layer, Names, Order, Unresolved};

fn order() -> Order {
    Order::inner_to_outer(vec![
        Layer::new("core".to_owned(), "src/core".to_owned()),
        Layer::new("gen".to_owned(), "src/gen".to_owned()),
        Layer::new("adapter".to_owned(), "src/adapter".to_owned()),
    ])
}

fn edge(from: &str, to: &str) -> Edge {
    Edge::new(from.to_owned(), to.to_owned(), format!("{from}:1"))
}

/// 経路で書く言語の区切り方。
fn paths() -> Names {
    Names::paths(&["ts"])
}

fn points() -> Vec<String> {
    ["src/core/name.ts", "src/stray/x.ts", "src/adapter/use.ts"]
        .iter()
        .map(|x| (*x).to_owned())
        .collect()
}

#[test]
fn a_reference_to_a_module_outside_every_layer_is_reported() {
    let got = unresolved(
        &order(),
        &[edge("src/adapter/use.ts", "src/stray/x")],
        &points(),
        &paths(),
    );
    assert!(
        got.iter()
            .any(|u| matches!(u, Unresolved::Unlayered { to, .. } if to == "src/stray/x")),
        "{got:?}"
    );
}

#[test]
fn a_reference_into_a_layer_without_modules_is_reported() {
    let got = unresolved(
        &order(),
        &[edge("src/adapter/use.ts", "src/gen/made")],
        &points(),
        &paths(),
    );
    assert!(
        got.iter()
            .any(|u| matches!(u, Unresolved::Missing { to, .. } if to == "src/gen/made")),
        "{got:?}"
    );
}

#[test]
fn a_reference_to_an_existing_module_in_a_layer_is_not_reported() {
    let got = unresolved(
        &order(),
        &[edge("src/adapter/use.ts", "src/core/name")],
        &points(),
        &paths(),
    );
    assert!(got.is_empty(), "{got:?}");
}

#[test]
fn a_reference_outside_the_workspace_is_not_reported() {
    // 外の部品（vitest）は、根のファイル（vitest.config.ts）と名前の頭が同じでも、作業領域の外である
    let mut pts = points();
    pts.push("vitest.config.ts".to_owned());
    let got = unresolved(
        &order(),
        &[edge("src/adapter/use.ts", "vitest")],
        &pts,
        &paths(),
    );
    assert!(got.is_empty(), "{got:?}");
}

#[test]
fn a_version_number_is_not_taken_as_an_extension() {
    // `v1.43.0` の `.0` は拡張子ではない ── その言語のソースの拡張子だけを外す
    let order = Order::inner_to_outer(vec![Layer::new("semconv".to_owned(), "semconv".to_owned())]);
    let pts = vec![
        "semconv/v1.43.0/attr.go".to_owned(),
        "adapter/use.go".to_owned(),
    ];
    let order2 = Order::inner_to_outer(vec![
        order.layers()[0].clone(),
        Layer::new("adapter".to_owned(), "adapter".to_owned()),
    ]);
    let got = unresolved(
        &order2,
        &[edge("adapter/use.go", "semconv/v1.43.0")],
        &pts,
        &Names::paths(&["go"]),
    );
    assert!(got.is_empty(), "{got:?}");
}

#[test]
fn dotted_names_use_dots_and_rust_use_trees() {
    // . 区切りの名前で書く言語は、. と :: で区切る。Rust の `a::{b, c}` は `{` の手前までを名前とする
    let order = Order::inner_to_outer(vec![
        Layer::new("parts".to_owned(), "parts".to_owned()),
        Layer::new("declare".to_owned(), "declare".to_owned()),
    ]);
    let pts = vec!["parts.src.record".to_owned(), "declare.src".to_owned()];
    let got = unresolved(
        &order,
        &[edge("declare.src", "parts.src.{record, tokens}")],
        &pts,
        &Names::dotted(),
    );
    assert!(got.is_empty(), "{got:?}");
}
