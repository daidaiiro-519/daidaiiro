// SPDX-License-Identifier: MIT
//! qa-advisor の業務ロジック層。**助言型は、references の実装（refs）だけを持つ**（ACDR 0061）。
//!
//! 判断基準と回答の形は references が持ち、道具は取り出す ・ 検査する ・ 描画する ・ 取り込むの4つである。
//! 依存の向きは `Cargo.toml` が宣言する ── この crate はデータアクセス層だけを参照する。

// **公開しない別名である** ── 雛形の複製（refs）が Skill の接頭辞に依存せずに呼べるようにする。
// 公開すると、上の層がこの crate を経由してデータアクセス層へ届き、層を飛ばせてしまう
use qa_data_access as data_access;

pub mod refs;
