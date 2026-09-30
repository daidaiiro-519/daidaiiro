// SPDX-License-Identifier: MIT
//! acdr の業務ロジック層。**サービス層とプレゼンテーション層を参照しない。**
//!
//! 依存の向きは `Cargo.toml` が宣言する ── この crate は入口も宣言も参照しないので、
//! 外側の層を参照した時点でコンパイルが通らない。

pub mod code;
pub mod markdown;
pub mod panes;
pub mod record;
pub mod refs;
pub mod shape;
pub mod style;
pub mod template;
pub mod tokens;
pub mod validate;
