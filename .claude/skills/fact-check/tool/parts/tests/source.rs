// SPDX-License-Identifier: MIT
//! 原文の読み取りと照合を、事例で検証する。
//!
//!     cargo test -p fc_parts

use std::path::{Path, PathBuf};

use fc_parts::find::How;
use fc_parts::source::{self, Doc};

/// この事例だけの置き場所を作る。
fn place(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("fc-source").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("作れる");
    dir
}

fn put(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).expect("書ける");
    path
}

#[test]
fn a_line_is_counted_from_one() {
    let doc = Doc::new("x".to_owned(), "一\n二\n三\n".to_owned());
    assert_eq!(doc.line_of(0), 1);
    assert_eq!(doc.line_of(2), 2);
    assert_eq!(doc.line_of(4), 3);
    assert_eq!(doc.line_text(2), "二");
}

#[test]
fn a_lone_cr_is_a_line_break() {
    // **実測** ── PDF から起こした原文が CR を 11,526 件持っていた
    assert_eq!(source::normalize("一\r二\r\n三"), "一\n二\n三");
    let doc = Doc::new("x".to_owned(), source::normalize("一\r二"));
    assert_eq!(
        doc.line_of(2),
        2,
        "統一しないと、報告する行が読み手の行と食い違う"
    );
}

#[test]
fn a_hit_carries_the_document_and_the_line() {
    let dir = place("hit");
    let path = put(&dir, "a.md", "頭\ncompact_summary が在る\n尾\n");
    let scan = source::scan(
        &path,
        &["compact_summary".to_owned()],
        How::Identifier,
        None,
        40,
        source::MAX_BYTES,
    );
    assert_eq!(scan.results.len(), 1);
    let hit = &scan.results[0].hits[0];
    assert_eq!(hit.doc, "a.md");
    assert_eq!(hit.line, 2);
    assert_eq!(hit.excerpt, "compact_summary が在る");
}

#[test]
fn the_documents_are_not_joined() {
    // **連結すると、どの原文のどこで一致したかが報告できない**
    let dir = place("many");
    put(&dir, "a.md", "compact_summary\n");
    put(&dir, "b.md", "compact_summary\n");
    let scan = source::scan(
        &dir,
        &["compact_summary".to_owned()],
        How::Identifier,
        None,
        40,
        source::MAX_BYTES,
    );
    assert_eq!(scan.docs_read, 2);
    let docs: Vec<&str> = scan.results[0]
        .hits
        .iter()
        .map(|h| h.doc.as_str())
        .collect();
    assert_eq!(docs, vec!["a.md", "b.md"]);
    // 行はそれぞれの原文の中で数える
    assert!(scan.results[0].hits.iter().all(|h| h.line == 1));
}

#[test]
fn an_anchor_limits_the_range() {
    // **名前が在ることと、その名前がそこで使われることは別である**
    let dir = place("anchor");
    let mut body = String::from("PostCompact input\ntrigger\n");
    body.push_str(&"埋め\n".repeat(60));
    body.push_str("trigger\n");
    let path = put(&dir, "a.md", &body);
    let all = source::scan(
        &path,
        &["trigger".to_owned()],
        How::Identifier,
        None,
        40,
        source::MAX_BYTES,
    );
    assert_eq!(all.results[0].hits.len(), 2);
    let near = source::scan(
        &path,
        &["trigger".to_owned()],
        How::Identifier,
        Some("PostCompact input"),
        40,
        source::MAX_BYTES,
    );
    assert_eq!(
        near.results[0].hits.len(),
        1,
        "別の事象の項目を、その事象のものとして数えない"
    );
}

#[test]
fn what_could_not_be_read_is_returned() {
    // **黙って除外すると、0件が「無い」なのか「読めなかっただけ」なのかを区別できない**
    let dir = place("big");
    put(&dir, "a.md", &"あ".repeat(100));
    let scan = source::scan(&dir, &["あ".to_owned()], How::Text, None, 40, 10);
    assert_eq!(scan.docs_read, 0);
    assert_eq!(scan.unreadable.len(), 1);
    assert!(scan.unreadable[0].1.starts_with("大きすぎる"));
    assert!(
        !scan.can_conclude_absent(),
        "0件を「無い」と結論してはいけない"
    );
    assert_eq!(scan.missing().len(), 1);
}

#[test]
fn a_missing_path_is_not_an_empty_result() {
    let scan = source::scan(
        Path::new("/在らない/原文"),
        &["x".to_owned()],
        How::Text,
        None,
        40,
        source::MAX_BYTES,
    );
    assert_eq!(scan.unreadable.len(), 1);
    assert!(!scan.can_conclude_absent());
}

#[test]
fn a_record_of_the_fetch_is_not_read_as_a_source() {
    let dir = place("meta");
    put(&dir, "a.md", "compact_summary\n");
    put(&dir, "a.md.meta.json", r#"{"url": "compact_summary"}"#);
    let scan = source::scan(
        &dir,
        &["compact_summary".to_owned()],
        How::Identifier,
        None,
        40,
        source::MAX_BYTES,
    );
    assert_eq!(scan.docs_read, 1, "取得の記録は原文ではない");
}

#[test]
fn everything_that_can_conclude_absent_was_read() {
    let dir = place("clean");
    put(&dir, "a.md", "本文\n");
    let scan = source::scan(
        &dir,
        &["無い語".to_owned()],
        How::Text,
        None,
        40,
        source::MAX_BYTES,
    );
    assert!(
        scan.can_conclude_absent(),
        "全部読めたなら、0件は「無い」である"
    );
    assert_eq!(scan.missing().len(), 1);
}

#[test]
fn a_name_is_made_from_the_source() {
    assert_eq!(source::slug("https://example.com/a/b"), "example.com_a_b");
    assert_eq!(
        source::slug("https://example.com/a?x=1&y=2"),
        "example.com_a_x_1_y_2"
    );
    assert!(
        source::slug(&format!("https://e.com/{}", "a".repeat(300)))
            .chars()
            .count()
            <= 120
    );
}

#[test]
fn only_what_was_actually_received_is_kept() {
    assert!(source::acceptable(200, 1));
    assert!(!source::acceptable(200, 0), "空を原文として受け取らない");
    assert!(
        !source::acceptable(404, 900),
        "誤りの頁を原文として受け取らない"
    );
    assert!(!source::acceptable(301, 900));
}

#[test]
fn the_listing_reads_the_records() {
    let dir = place("listing");
    put(&dir, "a.md", "本文\n");
    put(
        &dir,
        "a.md.meta.json",
        r#"{"url": "https://example.com/a", "fetched_at": "2026-09-27T10:00:00+0900", "lines": 12}"#,
    );
    let rows = source::listing(&dir);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "2026-09-27", "取得した日だけを出す");
    assert_eq!(rows[0].1, 12);
    assert_eq!(rows[0].2, "https://example.com/a");
}
