// SPDX-License-Identifier: MIT
//! 1枚の組み立てと、出す前の検査を、事例で検証する。
//!
//!     cargo test -p bb_business_logic

use std::path::PathBuf;

use bb_business_logic::audit::audit;
use bb_business_logic::blocks;
use bb_business_logic::deck::{self, Deck};
use bb_business_logic::template::Parts;
use bb_business_logic::topic::{Option_, Topic};
use serde_json::json;

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

fn parts() -> Parts {
    Parts::load(&references().join("board.template.html")).expect("読める")
}

/// 承認へ出せる論点。**ここから欠かせて、検査を確かめる。**
fn sound(no: usize) -> Topic {
    Topic {
        no,
        label: format!("試し{no}"),
        question: "何を決めるか".to_owned(),
        status: "open".to_owned(),
        answer: "残った答え".to_owned(),
        pick: Some(("A".to_owned(), "決めたこと".to_owned())),
        example: "<b>できあがるもの</b>".to_owned(),
        kept: vec![
            Option_::new("案甲".to_owned(), "中身".to_owned(), "代償".to_owned()),
            Option_::new("案乙".to_owned(), "中身".to_owned(), "代償".to_owned()),
        ],
        weaknesses: vec![("外の作業で試す".to_owned(), "後続で決定".to_owned())],
        grounds: vec![(
            "支える先".to_owned(),
            "もとにしたこと".to_owned(),
            "measured".to_owned(),
            "出どころ".to_owned(),
        )],
        ..Topic::default()
    }
}

fn build(topics: &[Topic], deck: Deck) -> Result<deck::Made, String> {
    deck::build(&parts(), topics, &deck)
}

fn one(topics: &[Topic]) -> deck::Made {
    build(
        topics,
        Deck {
            theme: "試し".to_owned(),
            board: "t".to_owned(),
            round: 1,
            ..Deck::default()
        },
    )
    .expect("組める")
}

#[test]
fn a_single_surviving_option_is_refused() {
    // **1つしか残らないなら、それは選択ではない**
    let mut t = sound(1);
    t.kept.truncate(1);
    let why = build(
        std::slice::from_ref(&t),
        Deck {
            theme: "試し".to_owned(),
            ..Deck::default()
        },
    )
    .expect_err("断る");
    assert!(why.contains("反証を通過した案が1つしかない"), "{why}");
}

#[test]
fn the_finished_image_is_not_folded() {
    // **畳むと、答えは文章だけで届く**（実測 ── 「図と完成イメージから全くイメージが
    // わかない」と差し戻された）
    let mut t = sound(1);
    t.figures = vec![("<svg></svg>".to_owned(), "図の説明".to_owned())];
    let html = one(std::slice::from_ref(&t)).body;
    let opened = |mark: &str| -> bool {
        let at = html.find(mark).expect("在る");
        let head = html[..at].rfind("<details").expect("在る");
        html[head..at].contains("open")
    };
    assert!(
        opened("この答えの完成イメージ"),
        "完成イメージが畳まれている"
    );
    assert!(!opened("この答えの前提"), "裏づけまで開いている");
}

#[test]
fn the_section_headings_are_made_by_the_tool() {
    let html = one(&[sound(1)]).body;
    for heading in [
        "この答えの完成イメージ",
        "この答えが残った理由",
        "この答えの前提",
        "この答えが扱わない範囲",
    ] {
        assert!(html.contains(heading), "{heading} が出ていない");
    }
}

#[test]
fn a_settled_topic_has_no_answer_form() {
    // **決まったことを、もう一度聞かない**
    let open = one(&[sound(1)]).body;
    assert!(open.contains("class=\"form\""), "回答欄が無い");
    let mut t = sound(1);
    t.status = "settled".to_owned();
    let done = one(std::slice::from_ref(&t)).body;
    assert!(
        !done.contains("class=\"form\""),
        "決着した論点に回答欄が在る"
    );
    assert!(done.contains("決着"), "札が出ていない");
}

#[test]
fn only_the_topic_in_view_has_a_form() {
    // **どれに答えるかを利用者に判定させない**
    let topics = vec![sound(1), sound(2)];
    let made = build(
        &topics,
        Deck {
            theme: "試し".to_owned(),
            board: "t".to_owned(),
            round: 1,
            queue: vec![(1, "前提が片付いた".to_owned())],
            ..Deck::default()
        },
    )
    .expect("組める");
    assert_eq!(
        made.body.matches("class=\"form\"").count(),
        1,
        "回答欄が1つでない"
    );
    assert!(made.body.contains("いま見る"), "札が出ていない");
    assert!(made.body.contains("待ち"), "待ちの札が出ていない");
}

#[test]
fn an_unknown_ground_kind_is_refused() {
    let mut t = sound(1);
    t.grounds[0].2 = "たぶん実測".to_owned();
    let why = build(
        std::slice::from_ref(&t),
        Deck {
            theme: "試し".to_owned(),
            ..Deck::default()
        },
    )
    .expect_err("断る");
    assert!(why.contains("出どころの種類が「たぶん実測」"), "{why}");
}

#[test]
fn a_ground_without_a_support_or_a_source_is_refused() {
    for at in [0usize, 3] {
        let mut t = sound(1);
        t.grounds[0].0 = if at == 0 {
            String::new()
        } else {
            t.grounds[0].0.clone()
        };
        t.grounds[0].3 = if at == 3 {
            String::new()
        } else {
            t.grounds[0].3.clone()
        };
        let why = build(
            std::slice::from_ref(&t),
            Deck {
                theme: "試し".to_owned(),
                ..Deck::default()
            },
        )
        .expect_err("断る");
        assert!(why.contains("支える先か出どころが無い"), "{why}");
    }
}

#[test]
fn too_many_open_topics_are_found() {
    let topics: Vec<Topic> = (1..=6).map(sound).collect();
    let found = audit(&topics, &[], &[]);
    assert!(
        found
            .iter()
            .any(|x| x.contains("一度に開いている論点が6件ある")),
        "{found:?}"
    );
}

#[test]
fn a_missing_now_showing_is_found() {
    let topics: Vec<Topic> = (1..=4).map(sound).collect();
    let found = audit(&topics, &[], &[]);
    assert!(
        found.iter().any(|x| x.contains("いま見る論点")),
        "{found:?}"
    );
    // **道具が箱を組む回は、面にも同じ表を置かない**
    let queued = audit(&topics, &[], &[(1, "開いた".to_owned())]);
    assert!(
        !queued.iter().any(|x| x.contains("いま見る論点が無い")),
        "{queued:?}"
    );
}

#[test]
fn an_answer_without_a_finished_image_is_found() {
    let mut t = sound(1);
    t.example = String::new();
    let found = audit(std::slice::from_ref(&t), &[], &[]);
    assert!(
        found
            .iter()
            .any(|x| x.contains("完成イメージ（図か実例）が無い")),
        "{found:?}"
    );
    // **待ちの論点は、承認へ出していない**
    t.status = "waiting".to_owned();
    let waiting = audit(std::slice::from_ref(&t), &[], &[]);
    assert!(
        !waiting.iter().any(|x| x.contains("完成イメージ")),
        "{waiting:?}"
    );
}

#[test]
fn an_undeclared_dependency_is_found() {
    // **答えの本文だけが他の論点を前提にし、根拠の欄に出てこなかった**
    let mut a = sound(1);
    a.pick = Some(("A".to_owned(), "論点2 の決着に依る".to_owned()));
    let b = sound(2);
    let found = audit(&[a, b], &[], &[]);
    assert!(
        found.iter().any(|x| x.contains("宣言されていない依存")),
        "{found:?}"
    );
}

#[test]
fn a_declared_dependency_is_not_found() {
    let mut a = sound(1);
    a.pick = Some(("A".to_owned(), "論点2 の決着に依る".to_owned()));
    a.grounds[0].3 = "論点2".to_owned();
    let b = sound(2);
    let found = audit(&[a, b], &[], &[]);
    assert!(
        !found.iter().any(|x| x.contains("宣言されていない依存")),
        "{found:?}"
    );
}

#[test]
fn an_answer_nothing_tries_is_found() {
    // **下流にも外の作業にも使われない答えを、承認へ出そうとした**
    let mut t = sound(1);
    t.weaknesses = vec![("ここでは決めない".to_owned(), "対象外".to_owned())];
    let found = audit(std::slice::from_ref(&t), &[], &[]);
    assert!(
        found.iter().any(|x| x.contains("試す相手が無い")),
        "{found:?}"
    );
    // 外の作業で試す行き先が書かれていれば、出ない
    let outer = audit(&[sound(1)], &[], &[]);
    assert!(
        !outer.iter().any(|x| x.contains("試す相手が無い")),
        "{outer:?}"
    );
}

#[test]
fn a_scope_item_without_a_treatment_is_found_when_in_view() {
    let mut t = sound(1);
    t.weaknesses = vec![("扱いを書いていない".to_owned(), String::new())];
    let found = audit(std::slice::from_ref(&t), &[], &[(1, "開いた".to_owned())]);
    assert!(found.iter().any(|x| x.contains("扱いが無い")), "{found:?}");
}

#[test]
fn a_figure_is_placed_with_both_limits() {
    // **上限は2つ同時に当てる** ── 元の幅と、入れ物の幅である。片方だけを行内の style へ
    // 書くと、それがブレストボードの CSS に勝ち、狭い入れ物からはみ出す（実測 ── はみ出した）
    let dir = std::env::temp_dir().join("bb-figure");
    std::fs::create_dir_all(dir.join("figures")).expect("作れる");
    std::fs::write(
        dir.join("figures").join("w.svg"),
        "<svg viewBox=\"0 0 968 340\" width=\"968\" height=\"340\"></svg>",
    )
    .expect("書ける");
    let got = blocks::build(
        &parts(),
        &[json!({"kind": "figure", "name": "w", "caption": "図"})],
        &dir.join("figures"),
    )
    .expect("組める");
    assert!(
        got.contains("min(100%,968px)"),
        "上限が1つしか当たっていない ── {got}"
    );
    assert!(
        !got.contains(" width=\"968\""),
        "固定幅が残っている ── {got}"
    );
}

#[test]
fn every_declared_kind_can_be_built() {
    // **種類ごとに1つの形だけを持つ** ── 片方だけに足すと、入力が通って出力が空になる
    let parts = parts();
    let dir = std::env::temp_dir().join("bb-figure").join("figures");
    let cases = [
        json!({"kind": "para", "text": "本文"}),
        json!({"kind": "note", "text": "注記"}),
        json!({"kind": "heading", "level": 3, "text": "見出し"}),
        json!({"kind": "html", "text": "<b>そのまま</b>"}),
        json!({"kind": "list", "items": [{"text": "項目"}]}),
        json!({"kind": "fold", "heading": "畳み", "body": [{"kind": "para", "text": "中身"}]}),
        json!({"kind": "card", "heading": "札", "events": [{"tag": "finding", "text": "甲 ── 乙"}]}),
        json!({"kind": "table", "cols": ["列"], "rows": [["行", ["値"]]]}),
        json!({"kind": "grid", "cols": ["列"], "rows": [["値"]]}),
        json!({"kind": "previous", "letter": "A", "body": "前の答え", "why": "理由"}),
    ];
    for case in &cases {
        let got = blocks::build(&parts, std::slice::from_ref(case), &dir)
            .unwrap_or_else(|why| panic!("{case} を組めない ── {why}"));
        assert!(!got.is_empty(), "{case} が空を返した");
    }
    let why = blocks::build(&parts, &[json!({"kind": "無い種類"})], &dir).expect_err("断る");
    assert!(why.contains("知らない宣言の種類"), "{why}");
}

#[test]
fn the_same_kind_of_event_is_bundled_under_one_heading() {
    // **同じ種別が複数あるなら、1つの見出しの下へ束ねて番号を付与する**
    let got = blocks::build(
        &parts(),
        &[json!({"kind": "card", "heading": "札", "events": [
            {"tag": "returned", "text": "甲"},
            {"tag": "returned", "text": "乙"},
            {"tag": "finding", "text": "丙"}]})],
        &std::env::temp_dir(),
    )
    .expect("組める");
    assert_eq!(
        got.matches("差し戻し").count(),
        1,
        "見出しが束ねられていない"
    );
    assert!(got.contains("判明事項"), "2つ目の種別が無い");
}
