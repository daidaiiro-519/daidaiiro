// SPDX-License-Identifier: MIT
//! narration の業務ロジック層。**サービス層とプレゼンテーション層を参照しない。**
//!
//! 依存の向きは `Cargo.toml` が宣言する ── この crate は依存を1件も持たないので、
//! 外側の層を参照した時点でコンパイルが通らない。

pub mod mp3;
pub mod voice;
