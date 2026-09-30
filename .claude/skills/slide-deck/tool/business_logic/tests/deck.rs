// SPDX-License-Identifier: MIT
//! **3つで組む** ── 型が形を、入力が中身を、テーマが配色を持つ。
//!
//!     cargo test -p sd_business_logic

use std::path::PathBuf;

use sd_business_logic::{deck, theme};
use serde_json::Value;

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

fn example(name: &str, title: &str) -> Value {
    let mut deck = deck::load(&references().join("deck-example.json")).expect("読める");
    if let Some(map) = deck.as_object_mut() {
        map.insert("title".to_owned(), Value::String(title.to_owned()));
        map.insert("theme".to_owned(), Value::String(name.to_owned()));
    }
    deck
}

fn built(name: &str) -> String {
    deck::build(&references(), &example(name, "ためし")).expect("組める")
}

#[test]
fn a_deck_is_built_with_the_theme_applied() {
    let refs = references();
    let body = built("warm-paper");
    assert!(body.contains("<title>ためし</title>"));
    assert!(body.contains("▼ テーマ") && body.contains("▲ テーマここまで"));
    let colors = theme::tokens(&theme::read(&refs, "warm-paper").expect("読める"));
    assert!(body.contains(&colors["--ground"]), "配色が貼られていない");
}

#[test]
fn the_built_page_passes_the_check() {
    // **区切りが無いと、テーマの外の直書きを判定できない**
    let dir = std::env::temp_dir().join("sd-deck");
    std::fs::create_dir_all(&dir).expect("作れる");
    for name in theme::theme_names(&references()) {
        let path = dir.join(format!("{name}.html"));
        std::fs::write(&path, built(&name)).expect("書ける");
        assert_eq!(
            theme::findings(&references(), &[path.display().to_string()]),
            Vec::<String>::new(),
            "{name} で組んだ1枚が検査を通らない"
        );
    }
}

#[test]
fn the_same_input_gives_the_same_page() {
    // **冪等である** ── 日付も乱数も読まない
    assert_eq!(built("warm-paper"), built("warm-paper"));
}

#[test]
fn the_page_count_is_preserved() {
    let body = built("warm-paper");
    assert_eq!(body.matches("<section class=\"slide").count(), 2);
    assert!(body.contains("<span id=\"total\">2</span>"));
}

#[test]
fn the_labels_follow_the_page_order() {
    // **0から数える** ── 先頭に空を足すと、全部の枚が1つ前のラベルを出す
    let value = example("warm-paper", "ためし");
    let first = value["slides"][0]["label"].as_str().expect("在る");
    assert!(
        deck::build(&references(), &value)
            .expect("組める")
            .contains(&format!("const LABELS = [\"{first}\"")),
        "先頭のラベルが、1枚目のものになっていない"
    );
}

#[test]
fn an_input_that_does_not_pass_writes_nothing() {
    // **入力が通っていなければ、1バイトも出さない**
    let dir = std::env::temp_dir().join("sd-deck");
    std::fs::create_dir_all(&dir).expect("作れる");
    let source = dir.join("bad.json");
    std::fs::write(
        &source,
        r#"{"title":"題","theme":"warm-paper","slides":[]}"#,
    )
    .expect("書ける");
    let out = dir.join("bad.html");
    let _ = std::fs::remove_file(&out);
    let why = deck::build_deck(&references(), &source, &out, false).expect_err("断る");
    assert!(why.contains("HTML は書き出さない"), "{why}");
    assert!(!out.exists(), "通っていない入力で、出力が残っている");
}

#[test]
fn checking_only_does_not_write() {
    let dir = std::env::temp_dir().join("sd-deck");
    std::fs::create_dir_all(&dir).expect("作れる");
    let source = dir.join("ok.json");
    std::fs::write(
        &source,
        serde_json::to_string(&example("warm-paper", "ためし")).expect("組める"),
    )
    .expect("書ける");
    let out = dir.join("only-check.html");
    let _ = std::fs::remove_file(&out);
    let got = deck::build_deck(&references(), &source, &out, true).expect("検査できる");
    assert!(!got.same && got.note == "差が在る", "{got:?}");
    assert!(!out.exists(), "検査だけのときに書き出している");
    // 書いてから検査すると、同一になる
    deck::build_deck(&references(), &source, &out, false).expect("書ける");
    let got = deck::build_deck(&references(), &source, &out, true).expect("検査できる");
    assert!(got.same && got.note == "同一", "{got:?}");
}

#[test]
fn emphasis_is_kept_and_everything_else_becomes_text() {
    // **太字と強調だけは通す** ── 枚の中で主従を付けるために要る
    assert_eq!(deck::esc("<b>甲</b>"), "<b>甲</b>");
    assert_eq!(deck::esc("甲<br>乙"), "甲<br>乙");
    assert_eq!(deck::esc("<mark>甲</mark>"), "<mark>甲</mark>");
    // **それ以外の印は文字として出る** ── 通すと、入力が出来上がりの構造を変えられる
    assert_eq!(
        deck::esc("<script>x</script>"),
        "&lt;script&gt;x&lt;/script&gt;"
    );
    assert_eq!(deck::esc("<div>"), "&lt;div&gt;");
    assert_eq!(deck::esc("甲 & 乙"), "甲 &amp; 乙");
}

#[test]
fn a_figure_is_placed_as_it_came() {
    // **この Skill は図を描かない。** 受け取るのは SVG の文字列だけである
    let refs = references();
    let parts =
        sd_business_logic::template::Parts::load(&deck::template_path(&refs)).expect("読める");
    let svg = "<svg><circle r=\"1\"/></svg>";
    let got =
        deck::block(&parts, &serde_json::json!({"kind": "figure", "svg": svg})).expect("組める");
    assert!(got.contains(svg), "SVG が書き換わっている ── {got}");
}

#[test]
fn an_unknown_element_is_refused() {
    let refs = references();
    let parts =
        sd_business_logic::template::Parts::load(&deck::template_path(&refs)).expect("読める");
    let why = deck::block(&parts, &serde_json::json!({"kind": "無い要素"})).expect_err("断る");
    assert!(why.contains("知らない要素"), "{why}");
}

#[test]
fn every_declared_kind_can_be_built() {
    // **種類を足すときは、型にも部品を足す** ── 片方だけに足すと、入力が通って
    // 出力が空になる
    let refs = references();
    let parts =
        sd_business_logic::template::Parts::load(&deck::template_path(&refs)).expect("読める");
    for kind in deck::KINDS {
        let got = deck::block(&parts, &serde_json::json!({"kind": kind}))
            .unwrap_or_else(|why| panic!("{kind} を組めない ── {why}"));
        assert!(!got.is_empty(), "{kind} が空を返した");
    }
}
