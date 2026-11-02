//! ファイルとリンクを読む出口：先の無いリンクを見つけ、リンクの先のフォルダの中へは降りず、同じ親フォルダのほかのフォルダを返す。

#![allow(clippy::unwrap_used)]

use outbound_filesystem::WorkingTree;
use skills_application::ports::SkillFiles;
use std::path::PathBuf;

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("asc-fs-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[cfg(unix)]
#[test]
fn dangling_links_are_found_and_linked_folders_are_not_walked() {
    use std::os::unix::fs::symlink;
    let root = temp("links");
    std::fs::create_dir_all(root.join("skills/fact-check/scripts")).unwrap();
    std::fs::create_dir_all(root.join("shared/docs")).unwrap();
    // 先のあるリンク（フォルダ）と、先の無いリンク（入れ子の中にも1つ）
    symlink("../../shared/docs", root.join("skills/fact-check/docs")).unwrap();
    symlink(
        "../../shared/references",
        root.join("skills/fact-check/references"),
    )
    .unwrap();
    symlink("../gone.py", root.join("skills/fact-check/scripts/run.py")).unwrap();
    // リンクの先のフォルダの中の先の無いリンクは、その Skill のものではないので数えない
    symlink("nowhere", root.join("shared/docs/broken")).unwrap();
    let tree = WorkingTree::new(root.clone());
    assert_eq!(
        tree.dangling_links("skills/fact-check").unwrap(),
        [
            "skills/fact-check/references",
            "skills/fact-check/scripts/run.py"
        ]
    );
    assert!(tree.exists("skills/fact-check/docs"));
    assert!(!tree.exists("skills/fact-check/references"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn siblings_are_the_other_folders_of_the_same_parent() {
    let root = temp("siblings");
    for d in ["skills/a", "skills/b", "skills/c"] {
        std::fs::create_dir_all(root.join(d)).unwrap();
    }
    std::fs::write(root.join("skills/readme.md"), "x").unwrap();
    let tree = WorkingTree::new(root.clone());
    assert_eq!(tree.siblings("skills/b").unwrap(), ["skills/a", "skills/c"]);
    assert_eq!(
        tree.folders("skills").unwrap(),
        ["skills/a", "skills/b", "skills/c"]
    );
    std::fs::remove_dir_all(root).unwrap();
}
