// SPDX-License-Identifier: MIT
//! skills-creator の業務ロジック層。**入出力を持たず、判定と変換だけを置く。**
//!
//! 依存の向きは `Cargo.toml` が宣言する ── この crate はデータアクセス層だけを参照するので、
//! 上の層を参照した時点でコンパイルが通らない。ファイル ・ 外部の道具 ・ 通信は
//! `data_access` を通す（ACDR 0058）。

// **公開しない別名である** ── 雛形の複製（refs）が Skill の接頭辞に依存せずに呼べるようにする。
// 公開すると、上の層がこの crate を経由してデータアクセス層へ届き、層を飛ばせてしまう
use sc_data_access as data_access;

pub mod accept;
pub mod behavior;
pub mod check;
pub mod conform;
pub mod profile;
pub mod provider;
pub mod refs;
pub mod scaffold;
pub mod sections;
pub mod view;
