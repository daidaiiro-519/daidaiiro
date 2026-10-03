// SPDX-License-Identifier: MIT
//! ux-advisor のサービス層。道具の一覧を持ち、**能力の正本はここである。**
//!
//! 助言型の道具は、references の4つ（get ・ validate ・ view ・ import）だけである（ACDR 0061）。
//! 依存の向きは `Cargo.toml` が宣言する ── この crate は業務ロジック層だけを参照する。

pub mod contract;
mod refs;

pub use contract::{catalog, Arg, Given, Outcome, Tool};

/// この Skill が持つ道具の一覧。**能力の正本である。**
#[must_use]
pub fn tools() -> Vec<Tool> {
    refs::tools()
}
