// SPDX-License-Identifier: MIT
//! 置く操作を事例で検証する。
//!
//!     cargo test -p sc_parts

use std::path::PathBuf;

use sc_parts::scaffold::{place, Item};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sc-scaffold-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("作れる");
    dir
}

#[test]
fn the_items_are_written_where_they_are_told() {
    let root = scratch("write");
    let items = vec![
        Item::keep(PathBuf::from("rs/parts/src/lib.rs"), "部品".to_owned()),
        Item::keep(PathBuf::from("mcp.json"), "登録".to_owned()),
    ];
    let placed = place(&root, &items).expect("置ける");
    assert_eq!(placed.written.len(), 2);
    assert!(placed.kept.is_empty());
    // **深い場所でも、包みごと作る**
    assert_eq!(
        std::fs::read_to_string(root.join("rs/parts/src/lib.rs")).expect("読める"),
        "部品"
    );
}

#[test]
fn what_is_already_there_is_kept() {
    let root = scratch("keep");
    let items = vec![Item::keep(PathBuf::from("mcp.json"), "はじめ".to_owned())];
    place(&root, &items).expect("置ける");
    let again = vec![Item::keep(PathBuf::from("mcp.json"), "あと".to_owned())];
    let placed = place(&root, &again).expect("置ける");
    // **既に在るものを上書きしない** ── 書いたものが消える
    assert!(placed.written.is_empty());
    assert_eq!(placed.kept.len(), 1);
    assert_eq!(
        std::fs::read_to_string(root.join("mcp.json")).expect("読める"),
        "はじめ"
    );
}
