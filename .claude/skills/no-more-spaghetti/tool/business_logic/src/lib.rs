// SPDX-License-Identifier: MIT
//! skills の業務ロジック層。**サービス層とプレゼンテーション層を参照しない。**
//!
//! 依存の向きは `Cargo.toml` が宣言する ── この crate はプレゼンテーション層を1件も参照できない。

pub mod init;
pub mod inward;
pub mod label;
pub mod rules;
pub mod run;
pub mod validate;
