// SPDX-License-Identifier: MIT
//! no-more-spaghetti のデータアクセス層。**ファイル ・ 外部の道具 ・ 通信の入出力だけを持つ。**
//!
//! 依存の向きは `Cargo.toml` が宣言する ── この crate は同じ workspace の crate を参照しない。
//! **判定を置かない** ── 何を読み、読んだものをどう扱うかは、業務ロジック層が決める。
//! `files` と `process` は雛形の複製である。この Skill に固有の入出力は、別の module に置く。

pub mod files;
// **この Skill に固有の入出力である** ── 規則の道具の出力をファイルへ流す起動と、そのファイルの読み書き
pub mod logfile;
pub mod process;
