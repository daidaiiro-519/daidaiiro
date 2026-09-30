// SPDX-License-Identifier: MIT
//! 入力の検査を、事例で検証する。**生成物ではなく、入力を検査する。**
//!
//!     cargo test -p acd_business_logic

use std::path::PathBuf;

use acd_business_logic::{tokens, validate};
use serde_json::{json, Value};

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

/// 検査を通る入力。**ここから欄を欠かせて、検出を確かめる。**
fn sound() -> Value {
    json!({
        "no": "ACDR 0001", "title": "題", "date": "2026-09-27", "status": "proposed",
        "decision": "こうする。", "why": "こういう理由である。",
        "applies_to": "ここへ適用する。",
        "alternatives": [{"option": "案甲", "why_not": "これが壊れる。"}]
    })
}

fn found(spec: &Value) -> Vec<String> {
    validate::inspect(&references(), spec, None, None)
}

fn hits(spec: &Value, part: &str) -> bool {
    found(spec).iter().any(|x| x.contains(part))
}

#[test]
fn a_sound_input_passes() {
    assert_eq!(found(&sound()), Vec::<String>::new());
}

#[test]
fn a_missing_section_stops_it() {
    // **欠けた記録は、あとから誰にも補えない**
    for key in validate::SECTIONS {
        let mut spec = sound();
        spec.as_object_mut().expect("表である").remove(key);
        assert!(
            hits(&spec, &format!("必須の欄が欠けている: {key}")),
            "{key} が欠けても通っている"
        );
    }
}

#[test]
fn a_blank_section_is_the_same_as_a_missing_one() {
    let mut spec = sound();
    spec["decision"] = json!("   ");
    assert!(hits(&spec, "必須の欄が欠けている: decision"));
}

#[test]
fn a_status_outside_the_three_stops_it() {
    // **状態が自由文になると、承認を得ているかを読む側が判定することになる**
    let mut spec = sound();
    spec["status"] = json!("たぶん承認");
    assert!(hits(&spec, "状態が「たぶん承認」"));
    for good in validate::STATUS {
        let mut spec = sound();
        spec["status"] = json!(good);
        assert!(!hits(&spec, "状態が"), "{good} が通らない");
    }
}

#[test]
fn an_incomplete_triple_stops_it() {
    // **3つ組が完備していない変更を、記録に包含しない**
    for key in ["find", "before", "why"] {
        let mut change = json!({"find": "甲", "before": "旧", "why": "理由"});
        change.as_object_mut().expect("表である").remove(key);
        let mut spec = sound();
        spec["docs"] = json!([{"key": "m", "tab": "面", "file": "x.md", "marks": [change]}]);
        assert!(hits(&spec, &format!("変更[0] に {key} が無い")), "{key}");
    }
}

#[test]
fn an_incomplete_shift_or_alternative_stops_it() {
    let mut spec = sound();
    spec["shift"] = json!([{"what": "甲"}]);
    assert!(hits(&spec, "shift[0] に from が無い"));
    let mut spec = sound();
    spec["alternatives"] = json!([{"option": "案"}]);
    assert!(hits(&spec, "alternatives[0] に why_not が無い"));
}

#[test]
fn two_claims_in_one_cell_stop_it() {
    // **1つの器に、1つのことだけを入れる**
    let mut spec = sound();
    spec["decision"] = json!("甲にする ── 乙だからである ── 丙でもある");
    assert!(hits(&spec, "区切り"), "{:?}", found(&spec));
    let mut spec = sound();
    spec["decision"] = json!("甲にする ── 乙だからである");
    assert!(!hits(&spec, "区切り"), "1つは通る");
}

#[test]
fn a_quotation_is_not_counted() {
    // **引用は除外する** ── 原文の形を変えないと決めているためである
    let mut spec = sound();
    spec["decision"] = json!("原典は「甲 ── 乙 ── 丙」と述べる。");
    assert!(!hits(&spec, "区切り"), "{:?}", found(&spec));
}

#[test]
fn a_sentence_too_long_stops_it() {
    let mut spec = sound();
    spec["why"] = json!(format!("{}。", "あ".repeat(121)));
    assert!(hits(&spec, "1文が 122 字ある"), "{:?}", found(&spec));
    let mut spec = sound();
    spec["why"] = json!(format!("{}。", "あ".repeat(119)));
    assert!(!hits(&spec, "1文が"), "120 字は通る");
}

#[test]
fn the_length_is_counted_per_sentence() {
    // **「。」で割る** ── 欄の全体で数えると、短い文を並べた欄が落ちる
    let mut spec = sound();
    spec["why"] = json!("あ".repeat(80) + "。" + &"い".repeat(80) + "。");
    assert!(!hits(&spec, "1文が"), "{:?}", found(&spec));
}

#[test]
fn a_heading_or_table_inside_a_cell_stops_it() {
    // **入力は宣言だけを保持する**
    for tag in [
        "<h2>甲</h2>",
        "<table><tr><td>甲</td></tr></table>",
        "<ul><li>甲</li></ul>",
    ] {
        let mut spec = sound();
        spec["how"] = json!(tag);
        assert!(hits(&spec, "欄の中に在る"), "{tag}");
    }
    let mut spec = sound();
    spec["how"] = json!("<b>甲</b>である。");
    assert!(!hits(&spec, "欄の中に在る"), "行の中の印は通る");
}

#[test]
fn every_cell_is_looked_at() {
    // **取りこぼしを作らない** ── 見ていない欄には、どんな散文でも置ける
    let mut spec = sound();
    spec["shift"] = json!([{"what": "甲 ── 乙 ── 丙", "from": "旧", "to": "新"}]);
    spec["alternatives"] = json!([{"option": "案", "why_not": "甲 ── 乙 ── 丙"}]);
    spec["after_approval"] = json!(["甲 ── 乙 ── 丙"]);
    spec["docs"] = json!([{"key": "m", "tab": "面", "file": "x.md",
                           "marks": [{"find": "甲", "before": "旧", "why": "甲 ── 乙 ── 丙"}]}]);
    let places: Vec<String> = found(&spec)
        .iter()
        .filter(|x| x.contains("区切り"))
        .cloned()
        .collect();
    assert_eq!(places.len(), 4, "{places:?}");
}

#[test]
fn the_original_text_is_not_looked_at() {
    // **変更前（before）は原典であり、形を変えない。** 照合する文字列も散文ではない
    let mut spec = sound();
    spec["docs"] = json!([{"key": "m", "tab": "面", "file": "x.md", "marks": [
        {"find": "甲 ── 乙 ── 丙", "before": "旧 ── い ── ろ", "why": "理由"}]}]);
    assert!(!hits(&spec, "区切り"), "{:?}", found(&spec));
}

#[test]
fn a_reference_that_does_not_resolve_stops_it() {
    let folder = std::env::temp_dir().join("acd-validate");
    std::fs::create_dir_all(&folder).expect("作れる");
    let mut spec = sound();
    spec["figure"] = json!("在らない.svg");
    spec["docs"] = json!([{"key": "m", "tab": "面", "file": "在らない.md"}]);
    let bad = validate::refs(&folder, &spec, None);
    assert!(bad.iter().any(|x| x.contains("図が無い")), "{bad:?}");
    assert!(
        bad.iter().any(|x| x.contains("対象の文書が無い")),
        "{bad:?}"
    );
}

#[test]
fn an_empty_figure_field_is_not_a_reference() {
    // **空の欄は、図が無いことである** ── 名前として扱うと、フォルダを指す
    let folder = std::env::temp_dir().join("acd-validate");
    std::fs::create_dir_all(&folder).expect("作れる");
    let mut spec = sound();
    spec["figure"] = json!("");
    assert_eq!(validate::refs(&folder, &spec, None), Vec::<String>::new());
}

#[test]
fn the_tokens_pass_their_own_contract() {
    // **0件にできるものだけを検査している**
    let refs = references();
    let value = tokens::load(&tokens::path(&refs)).expect("読める");
    assert_eq!(tokens::validate(&refs, &value), Vec::<String>::new());
}

#[test]
fn a_semantic_key_only_on_one_side_stops_it() {
    // **手で3か所へ書くと、1つのキーが片側から脱落しても誰も検出しない**
    let refs = references();
    let mut value = tokens::load(&tokens::path(&refs)).expect("読める");
    value["semantic"]["light"]["只の片側"] = json!("ink-900");
    assert!(
        tokens::validate(&refs, &value)
            .iter()
            .any(|x| x.contains("light にしか無い")),
        "{:?}",
        tokens::validate(&refs, &value)
    );
}

#[test]
fn a_reference_across_the_tiers_is_refused() {
    // **意味の層は基礎のキーだけを参照する**
    let refs = references();
    let mut value = tokens::load(&tokens::path(&refs)).expect("読める");
    let key = value["semantic"]["light"]
        .as_object()
        .and_then(|m| m.keys().next().cloned())
        .expect("1件は在る");
    value["semantic"]["light"][&key] = json!("在らないキー");
    value["semantic"]["dark"][&key] = json!("在らないキー");
    assert!(
        tokens::validate(&refs, &value)
            .iter()
            .any(|x| x.contains("基礎に無いキー")),
        "{:?}",
        tokens::validate(&refs, &value)
    );
}

#[test]
fn what_the_css_references_is_what_it_defines() {
    // **未定義の参照は無言で初期値へ縮退し、誰も検出できない**
    let refs = references();
    let value = tokens::load(&tokens::path(&refs)).expect("読める");
    let built = tokens::css(&value, false).expect("組める");
    let (refs_used, defs) = tokens::refs_and_defs(&built);
    let short: Vec<&String> = refs_used.iter().filter(|k| !defs.contains(k)).collect();
    assert!(short.is_empty(), "定義の無い参照が在る ── {short:?}");
}

#[test]
fn the_three_selectors_come_from_one_table() {
    let refs = references();
    let value = tokens::load(&tokens::path(&refs)).expect("読める");
    let built = tokens::css(&value, false).expect("組める");
    assert!(built.starts_with(":root{"), "{}", &built[..40]);
    assert!(
        built.contains("@media (prefers-color-scheme:dark)"),
        "暗の側が無い"
    );
    assert!(
        built.contains(":root[data-theme=\"dark\"]"),
        "明示の指定が無い"
    );
    // **Shadow の中に `:root` は無い** ── 寄せないと、印が色を失う
    let host = tokens::css(&value, true).expect("組める");
    assert!(host.starts_with(":root,:host{"), "{}", &host[..40]);
}
