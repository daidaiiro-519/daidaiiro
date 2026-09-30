// SPDX-License-Identifier: MIT
//! slide-deck の業務ロジック層。**入出力を持たず、判定と変換だけを置く。**
//!
//! 依存の向きは `Cargo.toml` が宣言する ── この crate はデータアクセス層だけを参照するので、
//! 上の層を参照した時点でコンパイルが通らない。ファイル ・ 外部の道具は `data_access` を通す。
//!
//! **3つで組む。** 入力の形は契約が、出来上がりの形は型が、配色はテーマが持つ。

// **公開しない別名である** ── 公開すると、上の層がこの crate を経由してデータアクセス層へ届き、
// 層を飛ばせてしまう
use sd_data_access as data_access;

pub mod deck;
pub mod review;
pub mod template;
pub mod theme;
pub mod validate;
