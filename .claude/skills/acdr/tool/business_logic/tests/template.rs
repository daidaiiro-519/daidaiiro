// SPDX-License-Identifier: MIT
//! 型が満たすこと。
//!
//! **形は型が持ち、組み立ては値を差し込むだけである** ── 形をコードの中の文字列に
//! 散らすと、記録ごとに違う形が出る。
//!
//!     cargo test -p acd_business_logic

use std::path::PathBuf;

use acd_business_logic::template::Parts;

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

fn parts() -> Parts {
    Parts::load(&references().join("acdr.template.html")).expect("読める")
}

/// 差し込む1つ。**事例の側でも、型と同じ形で渡す。**
fn slot(key: &'static str, value: &str) -> (&'static str, String) {
    (key, value.to_owned())
}

#[test]
fn a_part_is_built() {
    assert_eq!(
        parts()
            .part("sec", &[slot("heading", "見出し"), slot("body", "中身")])
            .expect("組める"),
        "<div class=\"sec\"><h4>見出し</h4>中身</div>"
    );
}

#[test]
fn a_missing_or_extra_value_is_refused() {
    // **埋め忘れも、余分な値も、出てから気づく形にしない**
    let parts = parts();
    assert!(
        parts.part("sec", &[slot("heading", "見")]).is_err(),
        "足りない"
    );
    assert!(
        parts
            .part(
                "sec",
                &[
                    slot("heading", "見"),
                    slot("body", "中"),
                    slot("extra", "余")
                ]
            )
            .is_err(),
        "余分"
    );
    assert!(parts.part("無い部品", &[]).is_err(), "知らない部品");
}

#[test]
fn the_parts_the_build_needs_are_present() {
    let parts = parts();
    let names = parts.names();
    for want in [
        "acdr",
        "sec",
        "page",
        "page-bare",
        "pane-md",
        "pane-code",
        "pane-html",
        "mark",
        "code-block",
        "diff-block",
        "md-table",
    ] {
        assert!(names.contains(&want), "{want} が型に無い");
    }
}

#[test]
fn a_nested_template_stays_inside_the_part() {
    // **面の型は中に `<template>` を持つ。** 最後の閉じだけが切れ目である
    let frag = parts()
        .part(
            "pane-html",
            &[
                slot("key", "k"),
                slot("tab", "面"),
                slot("index", ""),
                slot("shadowcss", ""),
                slot("body", "中"),
            ],
        )
        .expect("組める");
    assert!(
        frag.ends_with("中</template></section>"),
        "面の型が部品の中に残っていない ── {}",
        &frag[frag.len().saturating_sub(60)..]
    );
}

#[test]
fn the_mark_has_one_shape() {
    // **印は、3つの面のどれでも同じ形である**
    assert_eq!(
        parts()
            .part(
                "mark",
                &[
                    slot("before", "前"),
                    slot("why", "なぜ"),
                    slot("body", "後")
                ]
            )
            .expect("組める"),
        "<mark class=\"chg\" data-acdr=\"1\" tabindex=\"0\" role=\"button\" aria-expanded=\"false\"\
         \u{20}data-b=\"前\" data-w=\"なぜ\">後</mark>"
    );
}

#[test]
fn the_build_holds_no_structure() {
    // **組み立ての側に、構造を作る文字列を残さない** ── 残すと、型を直しても
    // そこだけ前の形のまま出る
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    for (name, tags) in [
        (
            "record.rs",
            vec!["<div class=\"sec\"", "<ul>", "<figure", "<table"],
        ),
        ("code.rs", vec!["<tr", "<span class=", "<div class="]),
        ("markdown.rs", vec!["<blockquote>", "<td>", "<hr>"]),
    ] {
        let body = std::fs::read_to_string(src.join(name)).expect("読める");
        let code: String = body
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for tag in tags {
            assert!(
                !code.contains(&format!("\"{tag}")),
                "{name} が {tag} を組み立てに書いている"
            );
        }
    }
}
