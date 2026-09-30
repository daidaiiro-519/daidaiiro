// SPDX-License-Identifier: MIT
//! brainstorming-board の業務ロジック層。**サービス層とプレゼンテーション層を参照しない。**
//!
//! 依存の向きは `Cargo.toml` が宣言する ── この crate は入口も宣言も参照しないので、外側の層を
//! 参照した時点でコンパイルが通らない。
//!
//! **3つで組む。** 入力の契約が何を書けるかを、型が何処へ並ぶかを、トークンが色を持つ。

pub mod audit;
pub mod blocks;
pub mod cell;
pub mod deck;
pub mod figcheck;
pub mod init;
pub mod panel;
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
