// SPDX-License-Identifier: MIT
//! doc-writing-skills の業務ロジック層。**入出力を持たず、判定と変換だけを置く。**
//!
//! 依存の向きは `Cargo.toml` が宣言する ── この crate はデータアクセス層だけを参照するので、
//! 上の層を参照した時点でコンパイルが通らない。ファイルと標準入力は `data_access` を通す（ACDR 0058）。

// **公開しない別名である** ── 雛形の複製が Skill の接頭辞に依存せずに呼べるようにする。
// 公開すると、上の層がこの crate を経由してデータアクセス層へ届き、層を飛ばせてしまう
use dws_data_access as data_access;

pub mod checks;
pub mod ending;
pub mod finding;
pub mod gate;
pub mod input;
pub mod instruction;
pub mod refs;
pub mod unit;
