// SPDX-License-Identifier: MIT
//! 7つの判定を事例で検証する。
//!
//!     cargo test -p dws_business_logic

use dws_business_logic::checks::{Pair, Words};
use dws_business_logic::gate;

fn words() -> Words {
    Words::new(
        vec![("ブレストボード".to_owned(), "板".to_owned())],
        vec![Pair::new(
            "盤面".to_owned(),
            "ブレストボード".to_owned(),
            "決定".to_owned(),
        )],
    )
}

fn inspect(name: &str, body: &str) -> Vec<String> {
    let dir = std::env::temp_dir().join("dws-checks");
    std::fs::create_dir_all(&dir).expect("作れる");
    let path = dir.join(name);
    std::fs::write(&path, body).expect("書ける");
    gate::inspect(&path, &words())
        .expect("検査できる")
        .into_iter()
        .map(|f| f.check)
        .collect()
}

fn hit(name: &str, body: &str, check: &str) -> bool {
    inspect(name, body).iter().any(|x| x == check)
}

#[test]
fn a_skipped_heading_is_found() {
    assert!(hit(
        "skip.md",
        "# 一\n\n### 三\n",
        "見出しの階層が飛んでいる"
    ));
    assert!(!hit(
        "noskip.md",
        "# 一\n\n## 二\n\n### 三\n",
        "見出しの階層が飛んでいる"
    ));
}

#[test]
fn a_mixed_style_is_found() {
    assert!(hit(
        "mixed.md",
        "これは本文である。\n\nこれは本文です。\n",
        "文体が混ざっている"
    ));
    assert!(!hit(
        "plain.md",
        "これは本文である。\n\nこれも本文である。\n",
        "文体が混ざっている"
    ));
}

#[test]
fn a_quotation_is_not_counted_as_the_writers_style() {
    // **引用は書き手の文体ではない。** 統一しようとすれば原文を書き換えることになる
    let body = "これは本文である。\n\n> これは引用です。\n";
    assert!(!hit("quote.md", body, "文体が混ざっている"));
    // **鉤括弧は、文の頭に在るときだけ引用として扱う。** 文の途中から始まるものは、
    // 書き手の文の一部として数える（実測 ── 移す前の実装も同じ判定である）
    let body = "これは本文である。\n\n「これは引用です。」\n";
    assert!(
        !hit("kagi.md", body, "文体が混ざっている"),
        "文頭の鉤括弧は他人の言葉である"
    );
}

#[test]
fn a_noun_ending_is_not_counted() {
    // **体言止めと常体は、品詞を判定しないと分けられない** ── 分けたことにしない
    let body = "これは本文である。\n\n対象は、その扱い。\n";
    assert!(!hit("noun.md", body, "文体が混ざっている"));
}

#[test]
fn unparallel_items_are_found() {
    assert!(hit(
        "items.md",
        "- これは項目である\n- これは項目です\n",
        "並んだ項目の語尾が統一されていない"
    ));
    assert!(!hit(
        "same.md",
        "- これは項目である\n- これも項目である\n",
        "並んだ項目の語尾が統一されていない"
    ));
}

#[test]
fn items_at_different_depths_are_separate_groups() {
    let body = "- これは項目である\n  - これは項目です\n";
    assert!(
        !hit("depth.md", body, "並んだ項目の語尾が統一されていない"),
        "字下げが違えば別の並びである"
    );
}

#[test]
fn a_figure_drawn_with_characters_is_found() {
    assert!(hit(
        "box.md",
        "┌──┐\n│甲│\n└──┘\n",
        "文字で図や表を描いている"
    ));
    assert!(hit(
        "rule.md",
        "── 区切り ──────\n",
        "文字で図や表を描いている"
    ));
    assert!(!hit(
        "dash.md",
        "これは本文である ── 二倍ダッシュは図ではない。\n",
        "文字で図や表を描いている"
    ));
}

#[test]
fn two_words_for_one_meaning_are_found() {
    let body = "ブレストボードを組む。板を組む。\n";
    assert!(hit("syn.md", body, "同じ意味の語が2つある"));
    assert!(!hit(
        "one.md",
        "ブレストボードを組む。\n",
        "同じ意味の語が2つある"
    ));
}

#[test]
fn an_identifier_in_backticks_is_not_a_word_choice() {
    // **記号で囲んだ中は検査しない。** 言い換えると、その名前で参照するものが壊れる
    let body = "ブレストボードを組む。`板` という名前で参照する。\n";
    assert!(!hit("code.md", body, "同じ意味の語が2つある"));
}

#[test]
fn a_retired_word_is_found() {
    assert!(hit("retired.md", "盤面を組む。\n", "廃語を使用している"));
    assert!(!hit(
        "fine.md",
        "ブレストボードを組む。\n",
        "廃語を使用している"
    ));
}

#[test]
fn a_broken_emphasis_is_found() {
    // 閉じの印が約物の直後に在ると、CommonMark では閉じ記号にならない
    assert!(hit(
        "emph.md",
        "**これは強調である。**続きの文。\n",
        "強調が描画されない"
    ));
    assert!(!hit(
        "ok.md",
        "**これは強調である** 続きの文。\n",
        "強調が描画されない"
    ));
}

#[test]
fn an_emphasis_inside_code_is_not_judged() {
    // **記号で囲んだ中の ** は記法ではない**
    assert!(!hit(
        "code.md",
        "例は `**強調する。**続き` である。\n",
        "強調が描画されない"
    ));
}

#[test]
fn a_tag_next_to_the_mark_is_not_judged_in_html() {
    // HTML では、閉じの印の直後に在るのはタグであり、文字ではない
    assert!(!hit(
        "tag.html",
        "<p><b>強調</b>の後に **これは強調である。**</p><p>次の段落</p>",
        "強調が描画されない"
    ));
    // 表に並べたコードの行は、判定しない
    assert!(!hit(
        "diff.html",
        "<table><tr><td class=\"cd\"><code>/// 規則を捨てる。**強調する。**続き</code></td></tr></table>",
        "強調が描画されない"
    ));
    // 本文の廃語は、HTML でも検出する
    assert!(hit("prose.html", "<p>盤面を組む。</p>", "廃語を使用している"));
}

#[test]
fn the_inside_of_a_fence_is_not_judged_as_prose() {
    let body = "# 題\n\n```\n盤面を組む\n```\n";
    assert!(!hit("fence.md", body, "廃語を使用している"));
}

#[test]
fn the_exempt_mark_turns_everything_off() {
    let body = "<!-- doc-writing-skills: exempt -->\n\n# 一\n\n### 三\n\n形を揃える。\n";
    assert!(
        inspect("exempt.md", body).is_empty(),
        "外す印が在れば、何も出ない"
    );
}

#[test]
fn the_mark_only_works_at_the_head() {
    let body = format!("{}\n\n# 一\n\n### 三\n", "あ".repeat(500));
    let with = format!("{body}\n<!-- doc-writing-skills: exempt -->\n");
    assert!(
        !inspect("late.md", &with).is_empty(),
        "途中に書いても、全体は外れない"
    );
}

#[test]
fn json_is_read_as_the_strings_it_holds() {
    // **構文を本文として読まない** ── 名前も括弧も書き手の文ではない
    let body = r#"{"$schema": "x", "answer": "盤面を組む。"}"#;
    assert!(hit("a.json", body, "廃語を使用している"));
    let body = r#"{"$schema": "盤面を組む。"}"#;
    assert!(
        !hit("b.json", body, "廃語を使用している"),
        "印で始まる名前は見ない"
    );
}

#[test]
fn the_same_finding_is_not_reported_twice() {
    let body = "盤面を組む。\n";
    let found = inspect("dup.md", body);
    let retired = found.iter().filter(|x| *x == "廃語を使用している").count();
    assert_eq!(retired, 1, "読み手が同じ場所を2回開くことになる");
}

#[test]
fn the_declared_checks_are_the_ones_that_run() {
    let names: Vec<&str> = gate::all().iter().map(|c| c.name).collect();
    assert_eq!(names.len(), 7, "判定の一覧が正本である");
    let found = inspect("all.md", "# 一\n\n### 三\n\n形を揃える。\n");
    for name in &found {
        assert!(
            names.contains(&name.as_str()),
            "宣言していない判定が出た ── {name}"
        );
    }
}

#[test]
fn a_polite_past_is_counted_as_polite() {
    // 「〜ました。」は敬体である ── 常体に数えると、敬体だけの文書が文体の混在になる
    assert!(!hit(
        "past.md",
        "検査を適用しました。\n結果は2件です。\n",
        "文体が混ざっている"
    ));
}

#[test]
fn a_full_stop_inside_brackets_does_not_end_the_sentence() {
    // 鉤括弧の中の句点で文を切ると、引用の文末（〜ました。）を書き手の文末として数える
    let body = "判定漏れは「〜ました。」の分類である。\n\n規則は1つである。\n";
    assert!(!hit("inner.md", body, "文体が混ざっている"));
}

#[test]
fn a_retired_word_inside_brackets_is_not_found() {
    // 廃語を引用して説明する文は、廃語を使用していない
    assert!(!hit(
        "quoted.md",
        "「盤面」は破棄した。\n",
        "廃語を使用している"
    ));
}

#[test]
fn a_pattern_finds_the_conjugated_forms_of_a_retired_verb() {
    // 終止形の文字列だけでは、〜ます の形が通過する
    let dir = std::env::temp_dir().join("dws-retired");
    std::fs::create_dir_all(&dir).expect("作れる");
    let list = dir.join("retired-words.json");
    std::fs::write(
        &list,
        r#"{"retired":[{"word":"要る","use_instead":"必要とする","pattern":"要[るりらっれ]"}]}"#,
    )
    .expect("書ける");
    let words = Words::new(
        Vec::new(),
        gate::load_retired(&list).expect("読める"),
    );
    let found = |body: &str| {
        let path = dir.join("a.md");
        std::fs::write(&path, body).expect("書ける");
        gate::inspect(&path, &words)
            .expect("検査できる")
            .iter()
            .any(|f| f.check == "廃語を使用している")
    };
    assert!(found("道具が要ります。\n"));
    assert!(found("道具は要らない。\n"));
    assert!(!found("道具が必要です。\n"));
}

#[test]
fn a_pattern_that_cannot_compile_is_an_error() {
    // 黙って語の文字列へ戻すと、活用形を検出しないまま通過する
    let dir = std::env::temp_dir().join("dws-retired-bad");
    std::fs::create_dir_all(&dir).expect("作れる");
    let list = dir.join("retired-words.json");
    std::fs::write(
        &list,
        r#"{"retired":[{"word":"要る","use_instead":"必要とする","pattern":"要["}]}"#,
    )
    .expect("書ける");
    assert!(gate::load_retired(&list).is_err());
}
