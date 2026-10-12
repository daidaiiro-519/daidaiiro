//! 試験で共有するもの：リポジトリの根と、Skill の置き場所。リポジトリの中の場所は、ここからの名前で指す。
#![allow(dead_code)]

use std::path::PathBuf;

/// リポジトリの根。この crate は src/<Skill>/ に置く決まりなので、2つ上である。
pub fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// Skill を置く場所。
pub fn skills() -> PathBuf {
    repo().join(".claude/skills")
}
