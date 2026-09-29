// SPDX-License-Identifier: MIT
//! 面の描画と、印の付き方を事例で検証する。
//!
//! **この道具の壊れ方は、出来上がった面を開くまで見えない。** だから検証するのは、印が
//! 付くことだけではない ── **印が、別の印の属性の中へ入らないこと**を、同じ重さで検証する。
//!
//!     cargo test -p acd_parts

use std::path::PathBuf;

use acd_parts::markdown::{self, Marked};
use acd_parts::template::Parts;
use serde_json::{json, Value};

fn parts() -> Parts {
    let references = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references");
    Parts::load(&references.join("acdr.template.html")).expect("読める")
}

fn change(find: &str, before: &str, why: &str) -> Value {
    json!({"find": find, "before": before, "why": why})
}

fn mark(body: &str, marks: &[Value]) -> Marked {
    markdown::mark(&parts(), body, marks).expect("付けられる")
}

/// 付いた印の属性を並べる。
fn attrs(body: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(at) = body[from..].find("<mark class=\"chg\"").map(|x| from + x) {
        let end = body[at..].find('>').map_or(body.len(), |x| at + x);
        let tag = &body[at..end];
        let pick = |key: &str| -> String {
            tag.find(key).map_or_else(String::new, |i| {
                let head = i + key.len();
                tag[head..]
                    .find('"')
                    .map_or_else(String::new, |j| tag[head..head + j].to_owned())
            })
        };
        out.push((pick("data-b=\""), pick("data-w=\"")));
        from = end;
    }
    out
}

fn stripped(body: &str) -> String {
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
fn the_found_word_is_marked() {
    let got = mark("<p>あいうえお</p>", &[change("いう", "旧", "理由")]);
    assert!(got.body.contains("data-b=\"旧\""), "{}", got.body);
    assert!(got.body.contains(">いう</mark>"), "{}", got.body);
}

#[test]
fn the_body_text_is_unchanged() {
    let got = mark("<p>あいうえお</p>", &[change("いう", "旧", "理由")]);
    assert_eq!(stripped(&got.body), "あいうえお");
}

#[test]
fn an_unmatched_word_is_reported() {
    // **黙って除外しない**
    let got = mark("<p>あ</p>", &[change("無い語", "x", "y")]);
    assert!(
        got.dropped.iter().any(|x| x.contains("一致せず")),
        "{:?}",
        got.dropped
    );
    assert_eq!(got.kept, 0);
}

#[test]
fn a_word_only_inside_code_is_not_marked() {
    // **コードと図の中に印を差し込むと、その中身が壊れる**
    let got = mark(
        "<pre>いう</pre><p>あお</p>",
        &[change("いう", "旧", "理由")],
    );
    assert!(!got.body.contains("<mark"), "{}", got.body);
    assert!(
        got.dropped
            .iter()
            .any(|x| x.contains("コードか図の中にしかない")),
        "{:?}",
        got.dropped
    );
}

#[test]
fn a_reason_containing_the_next_word_stays_out_of_the_attributes() {
    // **今日、実際に壊れた形である。**
    let got = mark(
        "<p>先の箇所と、後の箇所がある。</p>",
        &[
            change("先の箇所", "旧1", "ここに 後の箇所 という語が入っている"),
            change("後の箇所", "旧2", "理由2"),
        ],
    );
    let found = attrs(&got.body);
    assert_eq!(found.len(), 2, "{}", got.body);
    for (before, why) in found {
        assert!(!before.contains("<mark"), "{before}");
        assert!(!why.contains("<mark"), "{why}");
    }
}

#[test]
fn a_before_text_containing_the_next_word_stays_out_of_the_attributes() {
    let got = mark(
        "<p>甲と乙がある。</p>",
        &[
            change("甲", "むかしは 乙 と書いていた", "理由1"),
            change("乙", "旧2", "理由2"),
        ],
    );
    assert_eq!(attrs(&got.body).len(), 2, "{}", got.body);
    assert!(
        !got.body.contains("data-b=\"むかしは <mark"),
        "{}",
        got.body
    );
}

#[test]
fn the_count_matches_the_report() {
    let got = mark(
        "<p>甲と乙がある。</p>",
        &[change("甲", "乙", "理由1"), change("乙", "旧2", "理由2")],
    );
    assert_eq!((got.kept, got.asked), (2, 2));
    assert_eq!(got.body.matches("<mark class=\"chg\"").count(), 2);
}

#[test]
fn the_same_word_twice_is_reported_and_dropped() {
    let got = mark(
        "<p>あいうえお</p>",
        &[
            change("いう", "旧1", "理由1"),
            change("いう", "旧2", "理由2"),
        ],
    );
    assert_eq!(got.body.matches("<mark class=\"chg\"").count(), 1);
    assert!(
        got.dropped.iter().any(|x| x.contains("重なる")),
        "{:?}",
        got.dropped
    );
}

#[test]
fn a_containing_mark_is_reported_and_dropped() {
    // **同じ場所へ2つ差し込むと、片方が他方の中へ入る**
    let got = mark(
        "<p>あいうえお</p>",
        &[
            change("あいうえ", "旧1", "理由1"),
            change("いう", "旧2", "理由2"),
        ],
    );
    assert_eq!(got.body.matches("<mark class=\"chg\"").count(), 1);
    assert!(
        got.dropped.iter().any(|x| x.contains("重なる")),
        "{:?}",
        got.dropped
    );
}

#[test]
fn the_input_order_does_not_change_the_result() {
    let a = change("甲", "旧1", "理由1");
    let b = change("乙", "旧2", "理由2");
    let body = "<p>甲と乙がある。</p>";
    assert_eq!(
        mark(body, &[a.clone(), b.clone()]).body,
        mark(body, &[b, a]).body
    );
}

#[test]
fn a_mark_without_a_reason_is_reported() {
    let got = mark("<p>あいう</p>", &[json!({"find": "いう", "before": "旧"})]);
    assert!(
        got.dropped.iter().any(|x| x.contains("なぜが無い")),
        "{:?}",
        got.dropped
    );
    assert_eq!(got.kept, 1, "報告はするが、印は付ける");
}

#[test]
fn a_heading_becomes_a_heading() {
    let parts = parts();
    let got = markdown::render(&parts, "# 題\n\n本文である。\n").expect("描ける");
    assert!(got.contains("<h1>題</h1>"), "{got}");
    assert!(got.contains("本文である。"), "{got}");
}

#[test]
fn a_comment_does_not_come_out_as_text() {
    let got = markdown::render(&parts(), "<!-- 見えてはいけない -->\n本文\n").expect("描ける");
    assert!(!got.contains("見えてはいけない"), "{got}");
}

#[test]
fn a_table_without_a_head_shows_no_empty_band() {
    // **空の `<th>` を並べると、中身の無い帯が表の上に1本出る**
    let got = markdown::render(&parts(), "|  |  |\n|---|---|\n| 甲 | 乙 |\n").expect("描ける");
    assert!(!got.contains("<th"), "{got}");
    assert!(got.contains("甲"), "{got}");
}

#[test]
fn a_table_with_a_head_keeps_it() {
    let got = markdown::render(&parts(), "| 名 | 値 |\n|---|---|\n| 甲 | 乙 |\n").expect("描ける");
    assert!(got.contains("<th"), "{got}");
}

#[test]
fn an_escaped_bar_stays_inside_its_cell() {
    // **`\|` は、セルの区切りではなく縦線の文字である**（GitHub の表の書き方）
    // ── 区切りとして扱うと、セルが途中で切れ、その先の文字が消える
    let got = markdown::render(
        &parts(),
        "| 環境 | 実行する1行 |\n|---|---|\n| Linux | `curl -fsSL https://x/install.sh \\| bash` |\n",
    )
    .expect("描ける");
    assert!(got.contains("install.sh | bash"), "{got}");
    assert_eq!(got.matches("<td").count(), 2, "{got}");
}

#[test]
fn a_table_scrolls_inside_its_own_frame() {
    // **表は、横に溢れたら表の枠の中で横に送る** ── ページ全体の外枠（wrap）で囲むと、
    // 狭い画面でページごと横にはみ出す
    let got = markdown::render(&parts(), "| 名 | 値 |\n|---|---|\n| 甲 | 乙 |\n").expect("描ける");
    assert!(got.contains("<div class=\"scroll\"><table"), "{got}");
}

#[test]
fn a_fence_keeps_its_content_as_text() {
    let got = markdown::render(&parts(), "```\n<b>甲</b>\n```\n").expect("描ける");
    assert!(got.contains("&lt;b&gt;甲&lt;/b&gt;"), "{got}");
}

#[test]
fn a_break_is_kept_but_other_tags_are_not() {
    let got = markdown::render(&parts(), "甲<br>乙<b>丙</b>\n").expect("描ける");
    assert!(got.contains("甲<br>乙"), "{got}");
    assert!(got.contains("&lt;b&gt;丙&lt;/b&gt;"), "{got}");
}

#[test]
fn the_range_outside_pre_is_what_may_be_marked() {
    let body = "あ<pre>い</pre>う";
    let spans = markdown::outside_pre(body);
    let inside: String = spans.iter().filter_map(|(a, b)| body.get(*a..*b)).collect();
    assert!(!inside.contains('い'), "{inside:?}");
    assert!(inside.contains('あ') && inside.contains('う'), "{inside:?}");
}
