// SPDX-License-Identifier: MIT
//! 節の構成が、対応する雛形を満たすかを事例で検証する。
//!
//!     cargo test -p sc_business_logic

use sc_business_logic::sections::{headings, missing};

fn doc(names: &[&str]) -> String {
    names
        .iter()
        .map(|n| format!("## {n}\n\n本文\n\n"))
        .collect()
}

#[test]
fn headings_are_read_in_order() {
    assert_eq!(
        headings(&doc(&["目的", "役割", "参照"])),
        ["目的", "役割", "参照"]
    );
}

#[test]
fn a_heading_needs_a_space() {
    assert!(
        headings("##目的\n").is_empty(),
        "区切りの無い見出しは節ではない"
    );
    assert!(headings("### 目的\n").is_empty(), "深い見出しは節ではない");
    assert!(headings("# 目的\n").is_empty(), "題は節ではない");
}

#[test]
fn missing_sections_are_reported() {
    let template = doc(&["目的", "役割", "出力形式"]);
    assert_eq!(missing(&doc(&["目的", "役割"]), &template), ["出力形式"]);
    assert!(
        missing(&template, &template).is_empty(),
        "満たしていれば0件"
    );
}

#[test]
fn placeholders_in_the_template_are_ignored() {
    let template = "## 目的\n\n本文\n\n## {{節の名前}}\n\n本文\n";
    assert!(
        missing(&doc(&["目的"]), template).is_empty(),
        "差し込む場所を要求しない"
    );
}

#[test]
fn a_broken_placeholder_is_a_name() {
    let template = "## {{閉じていない\n\n本文\n";
    assert_eq!(
        missing("", template),
        ["{{閉じていない"],
        "閉じていなければ名前である"
    );
}

#[test]
fn the_order_is_not_required() {
    let template = doc(&["目的", "役割"]);
    assert!(
        missing(&doc(&["役割", "目的"]), &template).is_empty(),
        "並びが違っても0件"
    );
}

#[test]
fn trailing_spaces_do_not_change_the_name() {
    assert_eq!(headings("## 目的   \n"), ["目的"]);
}
