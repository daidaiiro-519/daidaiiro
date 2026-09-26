// SPDX-License-Identifier: MIT
//! doc-writing-skills の部品。**入口と宣言を参照しない。**
//!
//! 許可辺は `Cargo.toml` が宣言する ── この crate は依存を1件も持たないので、
//! 外側の層を参照した時点でコンパイルが通らない。

pub mod checks;
pub mod ending;
pub mod finding;
pub mod gate;
pub mod tails;
pub mod unit;
