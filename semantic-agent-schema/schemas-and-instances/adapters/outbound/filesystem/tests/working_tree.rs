//! 作業ディレクトリの JSON を経路の順に並べ、記録とビルドの出力には降りない。

#![allow(clippy::unwrap_used)]

use outbound_filesystem::WorkingTree;

#[test]
fn lists_json_and_skips_records() {
    let dir = std::env::temp_dir().join(format!("sas-files-{}", std::process::id()));
    for p in [
        "b/x.json",
        "a.json",
        ".sas/r.json",
        "target/t.json",
        "c.txt",
    ] {
        let to = dir.join(p);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::write(to, "{}").unwrap();
    }
    let store = WorkingTree::new(dir.clone());
    assert_eq!(store.list().unwrap(), ["a.json", "b/x.json"]);
    assert_eq!(store.read("b/x.json").unwrap(), "{}");
    assert!(store.read("none.json").is_err());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn writes_inside_the_tree_only() {
    let dir = std::env::temp_dir().join(format!("sas-files-write-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = WorkingTree::new(dir.clone());
    store.write("new/u2.json", "{\"a\":1}").unwrap();
    assert_eq!(store.read("new/u2.json").unwrap(), "{\"a\":1}");
    store.write("new/u2.json", "{}").unwrap();
    assert_eq!(store.read("new/u2.json").unwrap(), "{}");
    // 一時ファイルは残らない
    assert_eq!(store.list().unwrap(), ["new/u2.json"]);
    assert_eq!(std::fs::read_dir(dir.join("new")).unwrap().count(), 1);
    for bad in [
        "../x.json",
        "/tmp/x.json",
        ".sas/x.json",
        "a/./b.json",
        "",
        ".git/config",
    ] {
        assert!(store.write(bad, "{}").is_err(), "{bad} に書けてしまった");
    }
    std::fs::remove_dir_all(dir).unwrap();
}
