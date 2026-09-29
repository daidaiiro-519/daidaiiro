// SPDX-License-Identifier: MIT
//! 照合の3つの種類を、事例で検証する。
//!
//!     cargo test -p fc_parts

use fc_parts::find::{self, How};

#[test]
fn an_identifier_matches_as_a_word() {
    // **前後が語の文字なら、それは別の名前である**
    assert_eq!(
        find::identifier("use compact_summary here", "compact_summary").len(),
        1
    );
    assert!(find::identifier("xcompact_summary", "compact_summary").is_empty());
    assert!(find::identifier("compact_summaryx", "compact_summary").is_empty());
}

#[test]
fn a_word_carries_its_own_separators() {
    // **語は、自分の記法を自分の中に持っている** ── `.` を含む語は、`.` を内側とする
    assert_eq!(
        find::identifier("github.copilot.otel.enabled", "otel.enabled").len(),
        0
    );
    assert_eq!(
        find::identifier("set otel.enabled here", "otel.enabled").len(),
        1
    );
    // 語が `.` を含まなければ、`.` は区切りである
    assert_eq!(
        find::identifier("compact_summary. end", "compact_summary").len(),
        1
    );
}

#[test]
fn an_identifier_matches_at_the_end_of_a_sentence() {
    assert_eq!(
        find::identifier("the field is compact_summary.", "compact_summary").len(),
        1
    );
}

#[test]
fn a_quote_keeps_every_word() {
    let text = "these are\n  the words   here";
    assert_eq!(
        find::quote(text, "these are the words here").len(),
        1,
        "行をまたいでも一致する"
    );
    assert!(
        find::quote(text, "these are the word here").is_empty(),
        "語は1文字も変えない"
    );
}

#[test]
fn japanese_folds_without_a_space() {
    // **日本語は、行の折り返しに空白を持たない**
    let text = "これは折り返した\n文である。";
    assert_eq!(find::quote(text, "これは折り返した文である。").len(), 1);
    // 英語は語の切れ目に空白を持つので、逆に空白1つが要る
    let text = "this is a wrapped\nsentence.";
    assert_eq!(find::quote(text, "this is a wrapped sentence.").len(), 1);
}

#[test]
fn text_finds_anything() {
    // **探索のための種類である。** 照合した証しにはならない
    assert_eq!(find::text("xcompact_summaryx", "compact_summary").len(), 1);
    assert_eq!(find::text("aaa", "a").len(), 3, "重なりも数える");
}

#[test]
fn an_empty_needle_matches_nothing() {
    for how in How::all() {
        assert!(
            find::find(how, "本文である", "").is_empty(),
            "{} が空を一致させた",
            how.key()
        );
    }
}

#[test]
fn the_position_is_counted_in_characters() {
    // **バイトで数えると、日本語を含む原文で読み手の見る位置と食い違う**
    let text = "あいう compact_summary";
    assert_eq!(find::identifier(text, "compact_summary"), vec![4]);
}

#[test]
fn the_kind_must_be_named() {
    assert!(How::of("identifier").is_some());
    assert!(How::of("quote").is_some());
    assert!(How::of("text").is_some());
    // **道具の側で「たぶん識別子だろう」と決めない**
    assert!(How::of("").is_none());
    assert!(How::of("word").is_none());
}

#[test]
fn every_kind_has_a_machine_readable_name() {
    for how in How::all() {
        assert!(how.key().is_ascii(), "機械が分岐する値は ASCII である");
    }
}
