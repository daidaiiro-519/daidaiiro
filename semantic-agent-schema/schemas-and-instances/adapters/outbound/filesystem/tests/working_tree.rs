//! 渡された経路だけを読み書きする。スキーマの置き場所は、ファイルならそのまま、フォルダなら直下の JSON のファイルに開く。

#![allow(clippy::unwrap_used)]

use outbound_filesystem::WorkingTree;

#[test]
fn opens_schema_places_without_descending() {
    let dir = std::env::temp_dir().join(format!("sas-files-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for p in [
        "s/b.json",
        "s/a.json",
        "s/deep/x.json",
        "s/c.txt",
        "one.json",
    ] {
        let to = dir.join(p);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::write(to, "{}").unwrap();
    }
    let store = WorkingTree::new(dir.clone());
    assert_eq!(
        store
            .schema_files(&["s".into(), "one.json".into(), "s/a.json".into()])
            .unwrap(),
        ["s/a.json", "s/b.json", "one.json"]
    );
    assert!(store.schema_files(&["none".into()]).is_err());
    assert_eq!(store.read("s/b.json").unwrap(), "{}");
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
    assert_eq!(std::fs::read_dir(dir.join("new")).unwrap().count(), 1);
    for bad in ["../x.json", "/tmp/x.json", "a/./b.json", "", ".git/config"] {
        assert!(store.write(bad, "{}").is_err(), "{bad} に書けてしまった");
    }
    std::fs::remove_dir_all(dir).unwrap();
}
