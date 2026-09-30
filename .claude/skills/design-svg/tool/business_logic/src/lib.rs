// SPDX-License-Identifier: MIT
//! design-svg の部品 ── **構造化データから SVG を組み立てる、独立した描画エンジン。**
//!
//! 構造（節点 ・ 辺 ・ 囲み）とスタイル（色 ・ 寸法 ・ 角丸など）を分けて持つ。新しい描画部品は
//! 台帳へ登録するだけで足せる。テーマを差し替えれば、宣言を一切変えずに見た目だけを変えられる。
//!
//! **このエンジンの契約は「構造化データを受け取る」ことだけである。** 受け取るのは節点 ・ 辺 ・
//! 囲みという、どんなグラフ図にも共通する一般名詞で、そのデータが何を意味するかは知らない。
//!
//! サービス層とプレゼンテーション層を参照しない ── 依存の向きは `Cargo.toml` が宣言する。
//! **入出力を直接扱わない** ── ファイルの読み書きは、公開しない別名 `data_access` を通す（ACDR 0058）。

// **別名を公開しない** ── 公開すると、上の層がこの crate を経由してデータアクセス層へ届く
use ds_data_access as data_access;

pub mod boolean;
pub mod canvas;
pub mod catalog;
pub mod compose;
pub mod files;
pub mod geometry;
pub mod grid;
pub mod ids;
pub mod intset;
pub mod labels;
pub mod layout_contract;
pub mod lint;
pub mod nesting;
pub mod props;
pub mod py;
pub mod radial;
pub mod refs;
pub mod registry;
pub mod shapes;
pub mod shapes_decor;
pub mod shapes_freeform;
pub mod shapes_hex;
pub mod shapes_interaction;
pub mod shapes_quantity;
pub mod shapes_table;
pub mod shapes_text;
pub mod shapes_titled;
pub mod style;
pub mod sugiyama;
pub mod text;
pub mod theme;
pub mod tree;
pub mod verify;
pub mod xml;

/// 部品の一覧。**部品のファイルを足したら、ここへ1行足す** ── 足さないと台帳に載らない。
///
/// 台帳（`registry`）は土台の層に在り、部品の層を呼べない。だから一覧は、全部の層を
/// 知ってよい crate の根が持つ。
#[must_use]
pub fn components() -> Vec<(&'static str, registry::Component)> {
    let mut all = Vec::new();
    all.extend(shapes::register());
    all.extend(shapes_decor::register());
    all.extend(shapes_freeform::register());
    all.extend(shapes_hex::register());
    all.extend(shapes_interaction::register());
    all.extend(shapes_quantity::register());
    all.extend(shapes_table::register());
    all.extend(shapes_text::register());
    all.extend(shapes_titled::register());
    all
}
