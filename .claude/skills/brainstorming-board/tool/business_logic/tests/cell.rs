// SPDX-License-Identifier: MIT
//! 欄の整形を、事例で検証する ── **冪等であること**と、**引用を変えないこと**。
//!
//!     cargo test -p bb_business_logic

use std::path::PathBuf;

use bb_business_logic::cell::{self, mark, pairs};
use bb_business_logic::template::Parts;

fn parts() -> Parts {
    let references = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references");
    Parts::load(&references.join("board.template.html")).expect("読める")
}

fn strip(body: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for c in body.chars() {
        match c {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(c),
            _ => {}
        }
    }
    out
}

#[test]
fn a_cell_with_two_things_becomes_two_rows() {
    let parts = parts();
    let once = cell::cell(&parts, "<b>主張である</b> ── これは説明である").expect("組める");
    assert!(once.contains("class=\"lead-s\""), "{once}");
    assert!(once.contains("class=\"sub-s\""), "{once}");
    // **二重の太字にならない**
    assert_eq!(once.matches("<b").count(), 1, "{once}");
}

#[test]
fn applying_it_again_changes_nothing() {
    // **何度当てても同じものが出る（冪等）**
    let parts = parts();
    let x = "<b>主張である</b> ── これは説明である";
    let once = cell::cell(&parts, x).expect("組める");
    assert_eq!(cell::cell(&parts, &once).expect("組める"), once);
    assert_eq!(
        cell::cell(&parts, &cell::cell(&parts, &once).expect("組める")).expect("組める"),
        once
    );
    assert_eq!(cell::cell(&parts, x).expect("組める"), once);
}

#[test]
fn a_quotation_is_not_split() {
    // **原文の形を変えない** ── 割った時点で、それは引用ではなくなる
    let parts = parts();
    let quoted = "「原文のことば ── これも原文」";
    assert_eq!(cell::cell(&parts, quoted).expect("組める"), quoted);
}

#[test]
fn without_a_separator_nothing_happens() {
    let parts = parts();
    assert_eq!(
        cell::cell(&parts, "ただの一文である").expect("組める"),
        "ただの一文である"
    );
    assert_eq!(cell::cell(&parts, "").expect("組める"), "");
}

#[test]
fn the_separator_inside_a_mark_is_not_a_separator() {
    // **印を付けたあとの文字列には、属性の中にも区切りが入る** ── そこで割ると、属性が
    // 本文へ漏れる（実測 ── 実際に漏れた）
    let parts = parts();
    let marked = mark(
        &parts,
        "主張である ── 説明である",
        "前の中身 ── その続き",
        "この回で変わった",
        false,
        None,
    )
    .expect("組める");
    let split = cell::cell(&parts, &marked).expect("組める");
    assert!(
        !strip(&split).contains("data-w="),
        "属性が本文へ漏れた ── {split}"
    );
}

#[test]
fn a_pair_becomes_a_two_column_table() {
    let parts = parts();
    let rows = pairs(
        &parts,
        &["甲 ── 乙".to_owned(), "丙".to_owned()],
        "左",
        "右",
    )
    .expect("組める");
    assert!(rows.contains("<table"), "{rows}");
    assert!(rows.contains("<th>左</th>"), "{rows}");
    // **区切りが無い行は、右を空にする**
    assert!(rows.contains("<td></td>"), "{rows}");
}

#[test]
fn a_mark_in_a_table_cell_does_not_leak() {
    let parts = parts();
    let marked = mark(
        &parts,
        "甲 ── 乙",
        "丙 ── 丁",
        "この回で変わった",
        false,
        None,
    )
    .expect("組める");
    let rows = pairs(&parts, &[marked], "左", "右").expect("組める");
    assert!(
        !strip(&rows).contains("data-b="),
        "属性が欄へ漏れた ── {rows}"
    );
}

#[test]
fn a_table_without_a_head_shows_no_empty_band() {
    // **空の `<th>` を並べると、中身の無い帯が表の上に1本出る**
    let parts = parts();
    let rows = vec![vec!["甲".to_owned(), "乙".to_owned()]];
    let bare = cell::table(&parts, &[String::new(), String::new()], &rows).expect("組める");
    assert!(!bare.contains("<th"), "{bare}");
    let named = cell::table(&parts, &["名".to_owned(), "値".to_owned()], &rows).expect("組める");
    assert!(named.contains("<th"), "{named}");
}

#[test]
fn the_excerpt_is_counted_in_characters() {
    // **バイトで切ると、日本語が割れる**
    let long = "あ".repeat(60);
    let taken = cell::plain(&long, 46);
    assert_eq!(taken.chars().count(), 47, "46字と、続きの印 ── {taken}");
    assert!(taken.ends_with('…'), "{taken}");
    assert_eq!(cell::plain("<b>甲</b>", 46), "甲", "印は落とす");
}

#[test]
fn the_key_does_not_make_a_stray_space() {
    let parts = parts();
    let plain = cell::key(&parts, "A", "").expect("組める");
    assert!(!plain.contains("\" \""), "{plain}");
    let toned = cell::key(&parts, "×", "out").expect("組める");
    assert!(toned.contains("out"), "{toned}");
}
