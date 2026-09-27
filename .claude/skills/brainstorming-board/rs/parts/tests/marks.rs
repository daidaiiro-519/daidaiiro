// SPDX-License-Identifier: MIT
//! この回で変わったところの印を、事例で検証する ── **欄ごとに当たること。**
//!
//! **毎回どこを直したかを、書き手が文章で言うのは仕組みではない。**
//!
//!     cargo test -p bb_parts

use std::path::PathBuf;

use bb_parts::deck::{self, Deck};
use bb_parts::snapshot::{snapshot, Diff};
use bb_parts::template::Parts;
use bb_parts::topic::{Option_, Topic};
use serde_json::{json, Value};

fn parts() -> Parts {
    let references = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references");
    Parts::load(&references.join("board.template.html")).expect("読める")
}

/// 印を確かめるための論点。
fn topic(answer: &str, pick: &str, found: &[&str]) -> Topic {
    Topic {
        no: 1,
        label: "試し".to_owned(),
        question: "問い".to_owned(),
        status: "open".to_owned(),
        answer: answer.to_owned(),
        pick: Some(("A".to_owned(), pick.to_owned())),
        example: "<b>できあがるもの</b>".to_owned(),
        found: found.iter().map(|x| (*x).to_owned()).collect(),
        grounds: vec![(
            "支える先".to_owned(),
            "もとにしたこと".to_owned(),
            "measured".to_owned(),
            "出どころ".to_owned(),
        )],
        ..Topic::default()
    }
}

fn board(topics: &[Topic], prev: Option<Value>, round: usize) -> String {
    let parts = parts();
    deck::build(
        &parts,
        topics,
        &Deck {
            theme: "試し".to_owned(),
            board: "t".to_owned(),
            round,
            prev,
            ..Deck::default()
        },
    )
    .expect("組める")
    .body
}

#[test]
fn the_record_holds_a_row_as_a_list_of_cells() {
    let a = topic("はじめの答え", "A ── はじめの答え", &["甲 ── 乙"]);
    let snap = snapshot(std::slice::from_ref(&a));
    assert_eq!(
        snap["1"]["grounds"],
        json!([["支える先", "もとにしたこと", "measured", "出どころ"]])
    );
}

#[test]
fn without_a_previous_round_no_mark_is_put() {
    let a = topic("はじめの答え", "A ── はじめの答え", &["甲 ── 乙"]);
    assert_eq!(Diff::new(&a, None).n, 0);
}

#[test]
fn the_same_input_gives_no_change() {
    let a = topic("はじめの答え", "A ── はじめの答え", &["甲 ── 乙"]);
    let snap = snapshot(std::slice::from_ref(&a));
    assert_eq!(Diff::new(&a, Some(&snap)).n, 0);
}

#[test]
fn only_the_changed_cells_are_counted() {
    let a = topic("はじめの答え", "A ── はじめの答え", &["甲 ── 乙"]);
    let snap = snapshot(std::slice::from_ref(&a));
    // **答えを1つ変え、行を1つ足す** ── 変えていない欄は数えない
    let b = topic(
        "はじめの答え",
        "A′ ── 直した答え",
        &["甲 ── 乙", "丙 ── 丁"],
    );
    let mut d = Diff::new(&b, Some(&snap));
    assert_eq!(d.n, 2, "答え1 ・ 足した行1");
    let parts = parts();
    let put = d.one(&parts, "pick", "A′ ── 直した答え").expect("組める");
    assert!(
        put.contains("はじめの答え"),
        "前の中身が印に入っていない ── {put}"
    );
    assert_eq!(
        d.mark(&parts, "found", 0, 0, "甲 ── 乙").expect("組める"),
        "甲 ── 乙",
        "変えていない欄は、素のまま"
    );
    let added = d.mark(&parts, "found", 1, 0, "丙 ── 丁").expect("組める");
    assert!(added.contains("この回で足した"), "{added}");
}

#[test]
fn a_change_in_one_column_marks_only_that_column() {
    let a = topic("答え", "A ── 答え", &[]);
    let snap = snapshot(std::slice::from_ref(&a));
    let mut b = a.clone();
    b.grounds[0].3 = "別の出どころ".to_owned();
    let mut d = Diff::new(&b, Some(&snap));
    assert_eq!(d.n, 1);
    let parts = parts();
    assert_eq!(
        d.mark(&parts, "grounds", 0, 1, "もとにしたこと")
            .expect("組める"),
        "もとにしたこと",
        "中身の欄には印が付かない"
    );
    let put = d
        .mark(&parts, "grounds", 0, 3, "別の出どころ")
        .expect("組める");
    assert!(put.contains("chg"), "{put}");
}

#[test]
fn inserting_a_row_does_not_mark_everything_after_it() {
    // **位置だけで比べると、1行足しただけで以降が全部「変わった」と出る**（実測）
    let a = topic("答え", "A ── 答え", &["甲", "乙", "丙", "丁"]);
    let snap = snapshot(std::slice::from_ref(&a));
    let b = topic("答え", "A ── 答え", &["甲", "新しい行", "乙", "丙", "丁"]);
    assert_eq!(
        Diff::new(&b, Some(&snap)).n,
        1,
        "足した1行だけが変わっている"
    );
}

#[test]
fn the_count_appears_on_the_front_page() {
    let a = topic("はじめの答え", "A ── はじめの答え", &["甲 ── 乙"]);
    let snap = snapshot(std::slice::from_ref(&a));
    let b = topic(
        "はじめの答え",
        "A′ ── 直した答え",
        &["甲 ── 乙", "丙 ── 丁"],
    );
    let html = board(std::slice::from_ref(&b), Some(snap.clone()), 2);
    assert!(html.contains("class=\"chg\""), "印が本文に付いていない");
    assert!(
        html.contains("この回で変わったところ（2 か所）"),
        "件数が出ていない"
    );
    // 変わっていなければ 0 と出る
    let same = board(std::slice::from_ref(&a), Some(snap.clone()), 2);
    assert!(same.contains("（0 か所）"), "0 と出ていない");
    // 渡さなければ、節ごと出ない
    let none = board(std::slice::from_ref(&b), None, 2);
    assert!(
        !none.contains("この回で変わったところ"),
        "渡していないのに出た"
    );
    // **2回組んでも同じ（冪等）**
    assert_eq!(board(std::slice::from_ref(&b), Some(snap), 2), html);
}

#[test]
fn the_drawer_reaches_the_mark_from_any_page() {
    // **現在地の節だけに置くと、他の論点を見ているあいだは何も見えない**
    let a = topic("はじめの答え", "A ── はじめの答え", &["甲 ── 乙"]);
    let snap = snapshot(std::slice::from_ref(&a));
    let b = topic("はじめの答え", "A′ ── 直した答え", &["甲 ── 乙"]);
    let html = board(std::slice::from_ref(&b), Some(snap), 2);
    assert!(html.contains("id=\"drawer\""), "引き出しが無い");
    assert!(html.contains("dgo"), "跳ぶ手が無い");
    assert!(html.contains("id=\"c1-0\""), "印に番号が無い ── 跳べない");
}

#[test]
fn the_fields_that_are_marked_are_the_fields_that_are_counted() {
    // **片方にしか無い欄は、変わっても画面に出ない**（実測 ── 完成イメージと案の変更が
    // 1つも出なかった）
    let a = topic("答え", "A ── 答え", &[]);
    let snap = snapshot(std::slice::from_ref(&a));
    let mut b = a.clone();
    b.kept = vec![
        Option_::new("案甲".to_owned(), "中身".to_owned(), "代償".to_owned()),
        Option_::new("案乙".to_owned(), "中身".to_owned(), "代償".to_owned()),
    ];
    b.example = "<b>別のできあがり</b>".to_owned();
    let d = Diff::new(&b, Some(&snap));
    assert!(
        d.n >= 7,
        "案6欄と完成イメージ1欄が数えられていない ── {}",
        d.n
    );
    let html = board(std::slice::from_ref(&b), Some(snap), 2);
    assert!(html.contains("案甲"), "案が出ていない");
    assert!(html.contains("class=\"chg\""), "印が付いていない");
}

#[test]
fn the_list_shows_one_entry_per_row() {
    // **欄ごとに出すと、「持つ」だけの行が並ぶ**（実測 ── 実際に並んだ）
    let a = topic("答え", "A ── 答え", &[]);
    let snap = snapshot(std::slice::from_ref(&a));
    let mut b = a.clone();
    b.grounds[0].1 = "別のもとにしたこと".to_owned();
    b.grounds[0].3 = "別の出どころ".to_owned();
    let mut d = Diff::new(&b, Some(&snap));
    assert_eq!(d.n, 2, "2欄が変わっている");
    let parts = parts();
    let _ = d.mark(&parts, "grounds", 0, 1, "別のもとにしたこと");
    let _ = d.mark(&parts, "grounds", 0, 3, "別の出どころ");
    assert_eq!(d.items.len(), 1, "一覧は行ごとに1件 ── {:?}", d.items);
}

#[test]
fn a_figure_redrawn_is_a_change() {
    // 図は中身が長いので、要約で比べる ── **描き直しも変更である**
    let mut a = topic("答え", "A ── 答え", &[]);
    a.figures = vec![("<svg>甲</svg>".to_owned(), "図の説明".to_owned())];
    let snap = snapshot(std::slice::from_ref(&a));
    let mut b = a.clone();
    b.figures = vec![("<svg>乙</svg>".to_owned(), "図の説明".to_owned())];
    assert_eq!(
        Diff::new(&b, Some(&snap)).n,
        1,
        "描き直しが数えられていない"
    );
}
