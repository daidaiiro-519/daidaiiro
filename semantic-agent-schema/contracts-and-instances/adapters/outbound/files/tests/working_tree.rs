//! 作業ディレクトリの JSON を経路の順に並べ、記録とビルドの出力には降りない。

#![allow(clippy::unwrap_used)]

use contracts_and_instances_adapters_outbound_files::WorkingTree;
use contracts_and_instances_conformance_application::ports::DocumentStore;

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
