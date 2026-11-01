//! 描いたページの書き出し：全部書けたときだけ置き換え、途中で失敗したら前のページに戻す。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use outbound_filesystem::OutDir;
use rendering_application::ports::{PageRefusal, PageWriter};
use std::path::{Path, PathBuf};

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sas-out-dir-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn leftovers(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap() {
        let e = e.unwrap();
        let name = e.file_name().to_string_lossy().to_string();
        if e.path().is_file() && (name.ends_with(".sas-tmp") || name.ends_with(".sas-bak")) {
            out.push(name);
        }
    }
    out
}

#[test]
fn a_failure_while_replacing_restores_the_pages_already_replaced() {
    let root = temp("restore");
    let site = root.join("site");
    std::fs::create_dir_all(&site).unwrap();
    std::fs::write(site.join("a.html"), "前の a").unwrap();
    std::fs::write(site.join("b.html"), "前の b").unwrap();
    // b.html の退避先に中身のあるフォルダを置き、2つ目の置き換えだけが失敗するようにする
    std::fs::create_dir_all(site.join(".b.html.sas-bak").join("x")).unwrap();
    let pages = vec![
        ("a.html".to_string(), "新しい a".to_string()),
        ("b.html".to_string(), "新しい b".to_string()),
    ];
    let r = OutDir::new(root.clone()).write_pages("site", &pages, &[]);
    assert!(matches!(r, Err(PageRefusal::Failed(_))), "{r:?}");
    assert_eq!(
        std::fs::read_to_string(site.join("a.html")).unwrap(),
        "前の a",
        "置き換えた a を戻していない"
    );
    assert_eq!(
        std::fs::read_to_string(site.join("b.html")).unwrap(),
        "前の b"
    );
    assert!(
        leftovers(&site).is_empty(),
        "作業用のファイルが残った：{:?}",
        leftovers(&site)
    );
}

#[test]
fn a_page_on_a_folder_is_refused_before_writing() {
    let root = temp("folder");
    let site = root.join("site");
    std::fs::create_dir_all(site.join("b.html").join("x")).unwrap();
    std::fs::write(site.join("a.html"), "前の a").unwrap();
    let pages = vec![
        ("a.html".to_string(), "新しい a".to_string()),
        ("b.html".to_string(), "新しい b".to_string()),
    ];
    let r = OutDir::new(root.clone()).write_pages("site", &pages, &[]);
    assert!(matches!(r, Err(PageRefusal::Failed(_))), "{r:?}");
    assert_eq!(
        std::fs::read_to_string(site.join("a.html")).unwrap(),
        "前の a"
    );
    assert!(site.join("b.html").join("x").is_dir());
    assert!(leftovers(&site).is_empty());
}

#[test]
fn all_pages_are_replaced_when_every_write_succeeds() {
    let root = temp("ok");
    let site = root.join("site");
    std::fs::create_dir_all(&site).unwrap();
    std::fs::write(site.join("a.html"), "前の a").unwrap();
    let pages = vec![
        ("a.html".to_string(), "新しい a".to_string()),
        ("sub/b.html".to_string(), "新しい b".to_string()),
    ];
    OutDir::new(root.clone())
        .write_pages("site", &pages, &[])
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(site.join("a.html")).unwrap(),
        "新しい a"
    );
    assert_eq!(
        std::fs::read_to_string(site.join("sub/b.html")).unwrap(),
        "新しい b"
    );
    assert!(leftovers(&site).is_empty());
}

#[test]
fn a_page_on_an_input_file_writes_nothing() {
    let root = temp("overlap");
    std::fs::create_dir_all(root.join("design")).unwrap();
    std::fs::write(root.join("design/page.html"), "テンプレート").unwrap();
    let pages = vec![("design/page.html".to_string(), "ページ".to_string())];
    let r = OutDir::new(root.clone()).write_pages(
        "design/..",
        &pages,
        &["design/page.html".to_string()],
    );
    assert!(matches!(r, Err(PageRefusal::Overlaps { .. })), "{r:?}");
    assert_eq!(
        std::fs::read_to_string(root.join("design/page.html")).unwrap(),
        "テンプレート"
    );
}

#[test]
fn rewriting_after_an_interrupted_write_leaves_no_backup() {
    let root = temp("interrupted");
    let site = root.join("site");
    std::fs::create_dir_all(&site).unwrap();
    // 前の書き出しが置き換えの途中で止められたあと：a は前のページを退避したまま新しいページが無く、b は置き換え済みで退避が残る
    std::fs::write(site.join(".a.html.sas-bak"), "前の a").unwrap();
    std::fs::write(site.join("b.html"), "新しい b").unwrap();
    std::fs::write(site.join(".b.html.sas-bak"), "前の b").unwrap();
    let pages = vec![
        ("a.html".to_string(), "新しい a".to_string()),
        ("b.html".to_string(), "新しい b".to_string()),
    ];
    OutDir::new(root.clone())
        .write_pages("site", &pages, &[])
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(site.join("a.html")).unwrap(),
        "新しい a"
    );
    assert_eq!(
        std::fs::read_to_string(site.join("b.html")).unwrap(),
        "新しい b"
    );
    assert!(
        leftovers(&site).is_empty(),
        "打ち直したのに作業用のファイルが残った：{:?}",
        leftovers(&site)
    );
}
