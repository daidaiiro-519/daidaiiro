// SPDX-License-Identifier: MIT
//! narration の部品。**入口と宣言を参照しない。**
//!
//! 許可辺は `Cargo.toml` が宣言する ── この crate は依存を1件も持たないので、
//! 外側の層を参照した時点でコンパイルが通らない。

pub mod mp3;
pub mod voice;
