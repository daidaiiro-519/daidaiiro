// SPDX-License-Identifier: MIT
//! 型が満たすこと。**形はコードではなく型が持つ。**
//!
//!     cargo test -p sd_business_logic

use std::path::{Path, PathBuf};

use sd_business_logic::deck;
use sd_business_logic::template::Parts;

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

fn written(name: &str, body: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("sd-template");
    std::fs::create_dir_all(&dir).expect("作れる");
    let path = dir.join(name);
    std::fs::write(&path, body).expect("書ける");
    path
}

fn load(path: &Path) -> Result<Parts, String> {
    Parts::load(path)
}

#[test]
fn the_real_template_loads() {
    let parts = Parts::load(&deck::template_path(&references())).expect("読める");
    // **部品の名前は型が決める** ── こちらに一覧を書くと、型を直した瞬間に食い違う
    assert!(parts.names().contains(&"page"), "{:?}", parts.names());
    assert!(parts.names().len() > 40, "{} 件", parts.names().len());
}

#[test]
fn a_part_is_filled() {
    let path = written("one.html", r#"<template data-part="a">[{{x}}]</template>"#);
    let parts = load(&path).expect("読める");
    assert_eq!(
        parts.part("a", &[("x", "甲".to_owned())]).expect("組める"),
        "[甲]"
    );
}

#[test]
fn the_same_slot_is_filled_everywhere_it_appears() {
    let path = written(
        "twice.html",
        r#"<template data-part="a">{{x}}-{{x}}</template>"#,
    );
    let parts = load(&path).expect("読める");
    assert_eq!(
        parts.part("a", &[("x", "甲".to_owned())]).expect("組める"),
        "甲-甲"
    );
}

#[test]
fn a_missing_value_is_refused() {
    // **埋め忘れを、出てから気づく形にしない**
    let path = written(
        "need.html",
        r#"<template data-part="a">{{x}}{{y}}</template>"#,
    );
    let parts = load(&path).expect("読める");
    let why = parts
        .part("a", &[("x", "甲".to_owned())])
        .expect_err("断る");
    assert!(why.contains("足りない") && why.contains('y'), "{why}");
}

#[test]
fn a_value_the_template_does_not_hold_is_refused() {
    // **余分な値も、出てから気づく形にしない**
    let path = written("extra.html", r#"<template data-part="a">{{x}}</template>"#);
    let parts = load(&path).expect("読める");
    let why = parts
        .part("a", &[("x", "甲".to_owned()), ("z", "乙".to_owned())])
        .expect_err("断る");
    assert!(why.contains("型に無い値") && why.contains('z'), "{why}");
}

#[test]
fn a_part_the_template_does_not_hold_is_refused() {
    let path = written("none.html", r#"<template data-part="a">{{x}}</template>"#);
    let parts = load(&path).expect("読める");
    let why = parts.part("b", &[]).expect_err("断る");
    assert!(why.contains("型に無い部品"), "{why}");
}

#[test]
fn a_nested_template_is_not_the_seam() {
    // **面の型は中に <template> を持つ。** 最後の閉じだけが切れ目である
    let path = written(
        "nested.html",
        "<template data-part=\"a\">前<template>中</template>後</template>\
         <template data-part=\"b\">乙</template>",
    );
    let parts = load(&path).expect("読める");
    assert_eq!(parts.names(), vec!["a", "b"]);
    assert_eq!(
        parts.part("a", &[]).expect("組める"),
        "前<template>中</template>後"
    );
}

#[test]
fn a_template_without_parts_is_refused() {
    let path = written("empty.html", "<html></html>");
    let why = load(&path).expect_err("断る");
    assert!(why.contains("部品が1つも無い"), "{why}");
}

#[test]
fn a_duplicated_name_is_refused() {
    // **同じ名前が2つ在ると、どちらが使われるかを誰も言えない**
    let path = written(
        "dup.html",
        "<template data-part=\"a\">甲</template><template data-part=\"a\">乙</template>",
    );
    let why = load(&path).expect_err("断る");
    assert!(why.contains("重複"), "{why}");
}

#[test]
fn an_unclosed_part_is_refused() {
    let path = written("open.html", "<template data-part=\"a\">甲");
    let why = load(&path).expect_err("断る");
    assert!(why.contains("閉じていない"), "{why}");
}

#[test]
fn a_value_that_looks_like_a_slot_is_not_filled_again() {
    // **差し込んだ値の中の `{{…}}` を、差し込む場所として読まない** ── 読むと、
    // 入力に書いた文字列が、型の一部として解釈されることになる
    let path = written("again.html", r#"<template data-part="a">{{x}}</template>"#);
    let parts = load(&path).expect("読める");
    assert_eq!(
        parts
            .part("a", &[("x", "{{x}}".to_owned())])
            .expect("組める"),
        "{{x}}"
    );
}
