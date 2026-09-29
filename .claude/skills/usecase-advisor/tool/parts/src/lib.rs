// SPDX-License-Identifier: MIT
//! usecase-advisor の部品。**入口と宣言を参照しない。**
//!
//! 許可辺は `Cargo.toml` が宣言する ── この crate は同じ workspace の crate を参照しないので、
//! 外側の層を参照した時点でコンパイルが通らない。

pub mod refs;
