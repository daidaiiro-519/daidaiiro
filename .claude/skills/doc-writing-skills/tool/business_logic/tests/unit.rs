// SPDX-License-Identifier: MIT
//! 文書を単位へ切る操作を、事例で検証する。
//!
//!     cargo test -p dws_business_logic

use dws_business_logic::unit::{split, Kind};

#[test]
fn each_shape_becomes_its_own_kind() {
    let doc = "# 題\n\n本文である。\n\n- 項目\n\n> 引用\n\n| 甲 | 乙 |\n|---|---|\n| 1 | 2 |\n";
    let got = split(doc);
    let kinds: Vec<Kind> = got.iter().map(|u| u.kind).collect();
    assert_eq!(
        kinds,
        vec![
            Kind::Heading,
            Kind::Body,
            Kind::Item,
            Kind::Quote,
            Kind::Cell,
            Kind::Cell,
            Kind::Cell,
            Kind::Cell
        ],
        "表の区切りの行は数えない"
    );
}

#[test]
fn the_line_of_the_original_is_kept() {
    let doc = "# 題\n\n本文である。\n";
    let got = split(doc);
    assert_eq!(got[0].line, 1);
    assert_eq!(got[1].line, 3, "空の行を飛ばしても、行は原文のままである");
}

#[test]
fn the_inside_of_a_fence_is_not_prose() {
    let doc = "本文である。\n\n```\n# これは見出しではない\n```\n\n後の文である。\n";
    let got = split(doc);
    let code: Vec<&str> = got
        .iter()
        .filter(|u| u.kind == Kind::Code)
        .map(|u| u.text.as_str())
        .collect();
    assert_eq!(code, vec!["# これは見出しではない"]);
    assert_eq!(got.iter().filter(|u| u.kind == Kind::Heading).count(), 0);
}

#[test]
fn a_fence_closes_only_with_the_same_mark() {
    let doc = "```\n~~~\nまだ囲みの中である\n```\n外に出た。\n";
    let got = split(doc);
    // **印の行そのものは単位にしない。** 印が違えば閉じないので、中身は囲みのままである
    let code: Vec<&str> = got
        .iter()
        .filter(|u| u.kind == Kind::Code)
        .map(|u| u.text.as_str())
        .collect();
    assert_eq!(code, vec!["まだ囲みの中である"], "印が違えば閉じない");
    assert_eq!(got.last().expect("在る").kind, Kind::Body);
}

#[test]
fn the_front_matter_is_not_prose() {
    let doc = "---\nname: x\n---\n\n本文である。\n";
    let got = split(doc);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].text, "本文である。");
}

#[test]
fn the_notation_is_removed_from_the_text() {
    let doc = "**強調**と`記号`と[名前](先)である。\n";
    let got = split(doc);
    assert_eq!(
        got[0].text, "強調と記号と名前である。",
        "記法は人が読む文ではない"
    );
}

#[test]
fn the_depth_of_a_heading_is_counted() {
    let got = split("# 一\n### 三\n");
    assert_eq!(got[0].level, 1);
    assert_eq!(got[1].level, 3);
}

#[test]
fn the_indent_of_an_item_is_counted() {
    let got = split("- 浅い\n  - 深い\n");
    assert_eq!(got[0].indent, 0);
    assert_eq!(got[1].indent, 2);
}

use dws_business_logic::unit::split_html;

#[test]
fn an_html_page_is_split_by_its_elements_not_by_its_lines() {
    // **タグを本文として読まない** ── 1行に並んだ要素も、要素ごとの単位になる
    let page =
        "<h2>題</h2><p>本文である。</p><ul><li>項目</li></ul><table><tr><td>欄</td></tr></table>";
    let got = split_html(page);
    let kinds: Vec<Kind> = got.iter().map(|u| u.kind).collect();
    assert_eq!(
        kinds,
        vec![Kind::Heading, Kind::Body, Kind::Item, Kind::Cell]
    );
    assert_eq!(got[0].level, 2);
    assert_eq!(got[1].text, "本文である。");
    assert!(!got[1].raw.contains('<'), "{}", got[1].raw);
}

#[test]
fn code_in_an_html_page_is_not_prose() {
    // **コードの要素の中は、書き手の文ではない** ── 欄の中身が全部コードなら、その欄はコードである
    let page = "<table><tr><td class=\"cd\"><code>/// 規則を捨てる。**強調**</code></td></tr></table><pre>形を揃える</pre>";
    let got = split_html(page);
    assert!(got.iter().all(|u| u.kind == Kind::Code), "{got:?}");
}

#[test]
fn scripts_and_styles_are_not_units() {
    let page = "<style>.a{} /* 効く */</style><script>let x = 1;</script><p>本文である。</p>";
    let got = split_html(page);
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(got[0].text, "本文である。");
}

#[test]
fn entities_are_decoded_and_the_line_is_kept() {
    let page = "<p>一行目</p>\n<p>A &amp; B &lt;C&gt;</p>";
    let got = split_html(page);
    assert_eq!(got[1].text, "A & B <C>");
    assert_eq!(got[1].line, 2, "行は原文のままである");
}
