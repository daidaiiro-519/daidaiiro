// SPDX-License-Identifier: MIT
//! slide-deck の業務ロジック層。**サービス層とプレゼンテーション層を参照しない。**
//!
//! 依存の向きは `Cargo.toml` が宣言する ── この crate は入口も宣言も参照しないので、
//! 外側の層を参照した時点でコンパイルが通らない。
//!
//! **3つで組む。** 入力の形は契約が、出来上がりの形は型が、配色はテーマが持つ。

pub mod deck;
pub mod review;
pub mod template;
pub mod theme;
pub mod validate;
