// SPDX-License-Identifier: MIT
//! brainstorming-board の業務ロジック層。**入出力を持たず、判定と変換だけを置く。**
//!
//! 依存の向きは `Cargo.toml` が宣言する ── この crate はデータアクセス層だけを参照するので、
//! 上の層を参照した時点でコンパイルが通らない。ファイル ・ 通信は `data_access` を通す（ACDR 0058）。
//!
//! **3つで組む。** 入力の契約が何を書けるかを、型が何処へ並ぶかを、トークンが色を持つ。

// **公開しない別名である** ── 雛形の複製が Skill の接頭辞に依存せずに呼べるようにする。
// 公開すると、上の層がこの crate を経由してデータアクセス層へ届き、層を飛ばせてしまう
use bb_data_access as data_access;

pub mod audit;
pub mod blocks;
pub mod cell;
pub mod deck;
pub mod figcheck;
pub mod init;
pub mod panel;
pub mod refs;
pub mod render;
pub mod seq;
pub mod serve;
pub mod shape;
pub mod snapshot;
pub mod style;
pub mod template;
pub mod tokens;
pub mod topic;
pub mod validate;
