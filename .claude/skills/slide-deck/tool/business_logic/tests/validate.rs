// SPDX-License-Identifier: MIT
//! **形では書けない規則を、機械に見させる** ── 散文の規定は破れる。
//!
//!     cargo test -p sd_business_logic

use std::path::PathBuf;

use sd_business_logic::{deck, validate};
use serde_json::{json, Value};

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

/// 1枚だけのデッキを組む。**渡した欄で、基本の枚を上書きする。**
fn one(slide: Value) -> Value {
    let mut base = json!({
        "label": "ためし", "layout": "single", "heading": "断定形の主張",
        "blocks": [{"kind": "text", "body": "本文"}]
    });
    if let (Some(into), Some(from)) = (base.as_object_mut(), slide.as_object()) {
        for (k, v) in from {
            if v.is_null() {
                into.remove(k);
            } else {
                into.insert(k.clone(), v.clone());
            }
        }
    }
    json!({ "title": "題", "theme": "warm-paper", "slides": [base] })
}

fn found(value: &Value) -> Vec<String> {
    let refs = references();
    validate::check(&refs, &deck::schema_path(&refs), value)
}

fn hits(value: &Value, part: &str) -> bool {
    found(value).iter().any(|x| x.contains(part))
}

#[test]
fn the_example_passes() {
    let example = deck::load(&references().join("deck-example.json")).expect("読める");
    assert_eq!(found(&example), Vec::<String>::new());
}

#[test]
fn four_large_elements_stop_it() {
    // **4つ以上あると、どれが主張か消える**（設計規則 §2）
    let big: Vec<Value> = (0..4)
        .map(|i| json!({"kind": "stat", "value": i.to_string(), "caption": "条件"}))
        .collect();
    assert!(hits(&one(json!({"blocks": big})), "大きい要素"));
    let three: Vec<Value> = (0..3)
        .map(|i| json!({"kind": "stat", "value": i.to_string(), "caption": "条件"}))
        .collect();
    assert!(
        !hits(&one(json!({"blocks": three})), "大きい要素"),
        "3つは通る"
    );
}

#[test]
fn two_emphases_stop_it() {
    let two = json!([{"kind": "flow", "rows": [
        {"title": "甲", "mark": true}, {"title": "乙", "mark": true}]}]);
    assert!(hits(&one(json!({"blocks": two})), "強調"));
    let one_mark = json!([{"kind": "flow", "rows": [
        {"title": "甲", "mark": true}, {"title": "乙"}]}]);
    assert!(
        !hits(&one(json!({"blocks": one_mark})), "強調"),
        "1か所は通る"
    );
}

#[test]
fn a_warm_kicker_counts_as_an_emphasis() {
    // **器が違っても、読み手には同じ1つの強調として見える**
    let flow = json!([{"kind": "flow", "rows": [{"title": "甲", "mark": true}]}]);
    let value = one(json!({"blocks": flow, "kicker": {"text": "印", "tone": "warm"}}));
    assert!(hits(&value, "強調が 2 か所"), "{:?}", found(&value));
}

#[test]
fn a_question_heading_stops_it() {
    for heading in ["移行は終わったか", "移行は終わった？", "移行は終わった?"]
    {
        assert!(
            hits(&one(json!({"heading": heading})), "問いの形"),
            "{heading}"
        );
    }
    assert!(!hits(
        &one(json!({"heading": "移行は終わった"})),
        "問いの形"
    ));
}

#[test]
fn a_heading_that_takes_three_lines_stops_it() {
    let long = "あ".repeat(61);
    assert!(hits(&one(json!({"heading": long})), "見出しが 61 字"));
    let fits = "あ".repeat(60);
    assert!(
        !hits(&one(json!({"heading": fits})), "見出しが"),
        "60字は通る"
    );
}

#[test]
fn a_page_without_a_heading_stops_it() {
    // null を渡すと、その欄を落とす
    assert!(hits(&one(json!({"heading": null})), "見出しが無い"));
}

#[test]
fn a_source_in_the_body_stops_it() {
    // **出典を、要素から離さない**（設計規則 §2 ・ §8 の3）
    for body in ["出典　arXiv 0000.00000", "https://example.com"] {
        let blocks = json!([{"kind": "text", "body": body}]);
        assert!(
            hits(&one(json!({"blocks": blocks})), "出典が本文に在る"),
            "{body}"
        );
    }
}

#[test]
fn a_close_that_takes_three_lines_stops_it() {
    let value = json!({"title": "題", "theme": "warm-paper", "slides": [{
        "label": "甲", "layout": "cols", "heading": "主張", "columns": [
            {"role": "いま", "close": "あ".repeat(121),
             "blocks": [{"kind": "text", "body": "本文"}]},
            {"role": "提案", "blocks": [{"kind": "text", "body": "本文"}]}]}]});
    assert!(hits(&value, "締めが 121 字"), "{:?}", found(&value));
}

#[test]
fn column_roles_conflicting_across_pages_stop_it() {
    let column = |role: &str| json!({"role": role, "blocks": [{"kind": "text", "body": "本文"}]});
    let value = json!({"title": "題", "theme": "warm-paper", "slides": [
        {"label": "甲", "layout": "cols", "heading": "主張",
         "columns": [column("いま"), column("提案")]},
        {"label": "乙", "layout": "cols", "heading": "主張",
         "columns": [column("提案"), column("いま")]}]});
    assert!(hits(&value, "対応づけを崩さない"), "{:?}", found(&value));
}

#[test]
fn a_page_without_roles_is_not_a_conflict() {
    // **役割を与えた枚どうしだけを比べる** ── 与えていない枚は、役割を持たない枚で
    // あって、違反ではない
    let column = |role: &str| json!({"role": role, "blocks": [{"kind": "text", "body": "本文"}]});
    let bare = json!({"blocks": [{"kind": "text", "body": "本文"}]});
    let value = json!({"title": "題", "theme": "warm-paper", "slides": [
        {"label": "甲", "layout": "cols", "heading": "主張",
         "columns": [column("いま"), column("提案")]},
        {"label": "乙", "layout": "cols", "heading": "主張",
         "columns": [bare.clone(), bare]}]});
    assert_eq!(found(&value), Vec::<String>::new());
}

#[test]
fn the_number_of_columns_must_match_the_layout() {
    // **本数は、並べ方ごとに決まっている** ── 器の名前と中身の数が食い違うと、
    // 出来上がりの段が崩れる
    let column = json!({"blocks": [{"kind": "text", "body": "本文"}]});
    let value = json!({"title": "題", "theme": "warm-paper", "slides": [
        {"label": "甲", "layout": "cols", "heading": "主張",
         "columns": [column.clone(), column.clone(), column.clone()]}]});
    assert!(
        hits(&value, "cols の列が 3 本である"),
        "{:?}",
        found(&value)
    );
    let value = json!({"title": "題", "theme": "warm-paper", "slides": [
        {"label": "甲", "layout": "cols-3", "heading": "主張",
         "columns": [column.clone(), column.clone()]}]});
    assert!(
        hits(&value, "cols-3 の列が 2 本である"),
        "{:?}",
        found(&value)
    );
    // **下限は形が持つ** ── 同じことを2か所で見ない
    let value = json!({"title": "題", "theme": "warm-paper", "slides": [
        {"label": "甲", "layout": "cols", "heading": "主張", "columns": [column]}]});
    let bad = found(&value);
    assert!(bad.iter().all(|x| x.starts_with("形:")), "{bad:?}");
}

#[test]
fn both_containers_at_once_stop_it() {
    let column = json!({"blocks": [{"kind": "text", "body": "本文"}]});
    let value = json!({"title": "題", "theme": "warm-paper", "slides": [
        {"label": "甲", "layout": "cols", "heading": "主張",
         "blocks": [{"kind": "text", "body": "本文"}],
         "columns": [column.clone(), column]}]});
    assert!(hits(&value, "どちらか1つにする"), "{:?}", found(&value));
}

#[test]
fn a_cover_with_elements_stops_it() {
    let value = json!({"title": "題", "theme": "warm-paper", "slides": [
        {"label": "表紙", "layout": "cover", "heading": "題",
         "blocks": [{"kind": "text", "body": "本文"}]}]});
    assert!(hits(&value, "表紙は要素を持たない"), "{:?}", found(&value));
}

#[test]
fn an_unknown_theme_stops_it() {
    let mut value = one(json!({}));
    value["theme"] = json!("無いテーマ");
    assert!(hits(&value, "themes/ に無い"), "{:?}", found(&value));
}

#[test]
fn an_unknown_element_stops_it_by_shape() {
    // **形で止まるものは、形が止める** ── 同じことを2か所で見ない
    let value = one(json!({"blocks": [{"kind": "無い要素"}]}));
    let bad = found(&value);
    assert!(!bad.is_empty());
    assert!(bad.iter().all(|x| x.starts_with("形:")), "{bad:?}");
}

#[test]
fn what_the_shape_stops_is_not_looked_at_further() {
    // **形が通らなければ、その先は見ない** ── 通っていない入力へ後段の規則を当てると、
    // 欄が無いことを違反として報告することになる
    let value = json!({"slides": [{"kind": "壊れている"}]});
    let bad = found(&value);
    assert!(!bad.is_empty());
    assert!(bad.iter().all(|x| x.starts_with("形:")), "{bad:?}");
}
