// SPDX-License-Identifier: MIT
//! 8つの判定を事例で検証する。
//!
//!     cargo test -p dws_parts

use std::path::PathBuf;

use dws_parts::checks::{Pair, Words};
use dws_parts::gate;

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

fn words() -> Words {
    Words::new(
        gate::load_predicates(&references().join("predicates.json")).expect("読める"),
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
fn a_wago_predicate_is_found() {
    assert!(hit("wago.md", "形を揃える。\n", "述部が和語である"));
    assert!(!hit("kango.md", "形を統一する。\n", "述部が和語である"));
}

#[test]
fn a_quotation_is_not_checked_for_the_predicate() {
    // **原文の形を変えない。** 言い換えた時点で、それは引用ではなくなる
    let body = "原典は「形を揃える」と述べる。\n";
    assert!(!hit("wagoq.md", body, "述部が和語である"));
}

#[test]
fn a_lookbehind_keeps_the_replacement_itself() {
    // 「折り畳む」は言い換える先そのものである ── 前に「折り」が在れば検出しない
    assert!(!hit("fold.md", "節を折り畳む。\n", "述部が和語である"));
    assert!(hit("tatamu.md", "布を畳む。\n", "述部が和語である"));
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
fn the_inside_of_a_fence_is_not_judged_as_prose() {
    let body = "# 題\n\n```\n形を揃える\n```\n";
    assert!(!hit("fence.md", body, "述部が和語である"));
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
    let body = r#"{"$schema": "x", "answer": "形を揃える。"}"#;
    assert!(hit("a.json", body, "述部が和語である"));
    let body = r#"{"$schema": "形を揃える。"}"#;
    assert!(
        !hit("b.json", body, "述部が和語である"),
        "印で始まる名前は見ない"
    );
}

#[test]
fn the_same_finding_is_not_reported_twice() {
    let body = "形を揃える。\n";
    let found = inspect("dup.md", body);
    let wago = found.iter().filter(|x| *x == "述部が和語である").count();
    assert_eq!(wago, 1, "読み手が同じ場所を2回開くことになる");
}

#[test]
fn the_declared_checks_are_the_ones_that_run() {
    let names: Vec<&str> = gate::all().iter().map(|c| c.name).collect();
    assert_eq!(names.len(), 8, "判定の一覧が正本である");
    let found = inspect("all.md", "# 一\n\n### 三\n\n形を揃える。\n");
    for name in &found {
        assert!(
            names.contains(&name.as_str()),
            "宣言していない判定が出た ── {name}"
        );
    }
}

#[test]
fn every_example_of_a_predicate_is_found() {
    // **常体と敬体の両方を、一覧の事例で当てる。** 事例は predicates.json が持つ ──
    // この側に語を書かない。1件でも通過すれば、その行は活用を取りこぼしている
    let body = std::fs::read_to_string(references().join("predicates.json")).expect("読める");
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("JSON である");
    let mut missed = Vec::new();
    for (i, item) in parsed["predicates"]
        .as_array()
        .expect("配列である")
        .iter()
        .enumerate()
    {
        for ex in item["examples"].as_array().expect("事例を持つ") {
            let text = ex.as_str().expect("文字列である");
            if !hit(
                &format!("ex{i}.md"),
                &format!("{text}。\n"),
                "述部が和語である",
            ) {
                missed.push(format!("{i}: {text}"));
            }
        }
    }
    assert!(missed.is_empty(), "検出しなかった事例: {missed:?}");
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
        gate::load_predicates(&references().join("predicates.json")).expect("読める"),
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
