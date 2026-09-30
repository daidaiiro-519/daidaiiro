// SPDX-License-Identifier: MIT
//! ddd-advisor のサービス層。道具の一覧を持ち、**能力の正本はここである。**
//!
//! プレゼンテーション層（CLI ・ MCP）はこの一覧から組む ── 能力を2回書くと、片方だけが古くなる。
//! 依存の向きは `Cargo.toml` が宣言する ── この crate は業務ロジック層だけを参照する。
//!
//! **固有の道具を持たない。** advisor が持つのは references（判断基準 ・ 回答の形）だけで、
//! 道具は契約の版2 の4つ（get ・ validate ・ view ・ import）である（ACDR 0043）。

pub mod contract;
mod refs;

pub use contract::{catalog, Arg, Given, Outcome, Tool};

/// この Skill が持つ道具の一覧。**能力の正本である。**
#[must_use]
pub fn tools() -> Vec<Tool> {
    refs::tools()
}
