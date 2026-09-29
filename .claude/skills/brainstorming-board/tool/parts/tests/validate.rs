// SPDX-License-Identifier: MIT
//! 入力の検査 ・ 型 ・ トークン ・ 図の検査を、事例で検証する。
//!
//!     cargo test -p bb_parts

use std::path::PathBuf;

use bb_parts::template::Parts;
use bb_parts::{figcheck, tokens, validate};
use serde_json::{json, Value};

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

/// 検査を通る入力。
fn sound() -> Value {
    json!({
        "title": "試し", "board": "tameshi", "round": 1,
        "topics": [{
            "no": 1, "name": "試し", "status": "open",
            "question": "何を決めるか", "answer": "残った答え"
        }]
    })
}

fn found(d: &Value) -> Vec<String> {
    let refs = references();
    validate::inspect(&refs, &refs, d)
}

fn hits(d: &Value, part: &str) -> bool {
    found(d).iter().any(|x| x.contains(part))
}

#[test]
fn a_sound_input_passes() {
    assert_eq!(found(&sound()), Vec::<String>::new());
}

#[test]
fn a_board_name_outside_the_shape_stops_it() {
    let mut d = sound();
    d["board"] = json!("ためし");
    let bad = found(&d);
    assert!(bad.iter().all(|x| x.starts_with("形:")), "{bad:?}");
    assert!(!bad.is_empty());
}

#[test]
fn two_claims_in_one_line_stop_it() {
    // **1つの器に、1つのことだけを入れる**
    let mut d = sound();
    d["topics"][0]["answer"] = json!("甲 ── 乙 ── 丙");
    assert!(hits(&d, "1つの行に区切り"), "{:?}", found(&d));
    d["topics"][0]["answer"] = json!("甲 ── 乙");
    assert!(!hits(&d, "1つの行に区切り"), "1つは通る");
}

#[test]
fn a_line_break_divides_the_lines_in_a_cell() {
    // **改行は欄の中の行を分ける** ── 行ごとに1つの主張を数える
    let mut d = sound();
    d["topics"][0]["answer"] = json!("甲 ── 乙<br>丙 ── 丁");
    assert!(!hits(&d, "1つの行に区切り"), "{:?}", found(&d));
}

#[test]
fn a_quotation_is_not_counted() {
    let mut d = sound();
    d["topics"][0]["answer"] = json!("原典は「甲 ── 乙 ── 丙」と述べる。");
    assert!(!hits(&d, "1つの行に区切り"), "{:?}", found(&d));
}

#[test]
fn a_sentence_too_long_stops_it() {
    let mut d = sound();
    d["topics"][0]["answer"] = json!(format!("{}。", "あ".repeat(121)));
    assert!(hits(&d, "1文が 122 字ある"), "{:?}", found(&d));
}

#[test]
fn a_heading_or_table_inside_a_cell_stops_it() {
    // **入力は宣言だけを保持する**
    for tag in [
        "<h2>甲</h2>",
        "<ul><li>甲</li></ul>",
        "<table><tr><td>甲</td></tr></table>",
    ] {
        let mut d = sound();
        d["topics"][0]["answer"] = json!(tag);
        assert!(hits(&d, "欄の中に在る"), "{tag}");
    }
    let mut d = sound();
    d["topics"][0]["answer"] = json!("<b>甲</b>である");
    assert!(!hits(&d, "欄の中に在る"), "行の中の印は通る");
}

#[test]
fn an_authored_section_heading_stops_it() {
    // **節の見出しは道具が作る** ── 手で書くと、ブレストボードごとに違う形になる
    let mut d = sound();
    d["topics"][0]["example"] = json!([{
        "kind": "fold", "heading": "この答えの完成イメージ",
        "body": [{"kind": "para", "text": "中身"}]
    }]);
    assert!(hits(&d, "節の見出しを手で書いている"), "{:?}", found(&d));
}

#[test]
fn a_figure_that_does_not_resolve_stops_it() {
    let dir = std::env::temp_dir().join("bb-validate");
    std::fs::create_dir_all(dir.join("figures")).expect("作れる");
    let mut d = sound();
    d["topics"][0]["figures"] = json!([{"kind": "figure", "name": "在らない"}]);
    let bad = validate::refs(&dir, &d);
    assert!(
        bad.iter()
            .any(|x| x.contains("figures/在らない.svg が無い")),
        "{bad:?}"
    );
    // 在る図は、検出しない
    std::fs::write(dir.join("figures").join("在る.svg"), "<svg/>").expect("書ける");
    d["topics"][0]["figures"] = json!([{"kind": "figure", "name": "在る"}]);
    assert_eq!(validate::refs(&dir, &d), Vec::<String>::new());
}

#[test]
fn every_cell_is_looked_at() {
    // **取りこぼしを作らない** ── 見ていない欄には、どんな散文でも置ける
    let mut d = sound();
    let t = &mut d["topics"][0];
    t["intro"] = json!("甲 ── 乙 ── 丙");
    t["passed"] = json!([{"name": "甲 ── 乙 ── 丙", "body": "中身", "cost": "代償"}]);
    t["dropped"] = json!([{"body": "甲 ── 乙 ── 丙", "reason": "理由"}]);
    t["grounds"] = json!([{"supports": "支える先", "basis": "甲 ── 乙 ── 丙",
                           "tag": "measured", "source": "出どころ"}]);
    t["requirements"] = json!(["甲 ── 乙 ── 丙"]);
    t["findings"] = json!(["甲 ── 乙 ── 丙"]);
    t["out_of_scope"] = json!([{"item": "甲 ── 乙 ── 丙"}]);
    let places = found(&d)
        .iter()
        .filter(|x| x.contains("1つの行に区切り"))
        .count();
    assert_eq!(places, 7, "{:?}", found(&d));
}

#[test]
fn the_tokens_pass_their_own_contract() {
    let refs = references();
    let value = tokens::load(&tokens::path(&refs)).expect("読める");
    assert_eq!(tokens::validate(&refs, &value), Vec::<String>::new());
}

#[test]
fn a_thin_contrast_is_found() {
    // **目で見て薄いと気づくのは、出したあとである**
    let refs = references();
    let mut value = tokens::load(&tokens::path(&refs)).expect("読める");
    // 本文の色を、地とほぼ同じにする
    let ground = value["semantic"]["light"]["paper"].clone();
    value["semantic"]["light"]["ink"] = ground.clone();
    value["semantic"]["dark"]["ink"] = value["semantic"]["dark"]["paper"].clone();
    let bad = tokens::validate(&refs, &value);
    assert!(
        bad.iter().any(|x| x.contains("--ink が --paper の上で")),
        "{bad:?}"
    );
}

#[test]
fn a_literal_in_the_component_tier_is_allowed() {
    // **部品の層は、寸法の式をそのまま持てる** ── 寸法は役割ではなく、層そのものが意味である
    let refs = references();
    let mut value = tokens::load(&tokens::path(&refs)).expect("読める");
    value["component"]["just-literal"] = json!("7.4rem");
    value["component"]["just-calc"] = json!("calc(var(--sp-1) * 2)");
    let bad = tokens::validate(&refs, &value);
    assert!(!bad.iter().any(|x| x.contains("just-")), "{bad:?}");
    // 名前を間違えたものは検出する
    value["component"]["just-wrong"] = json!("在らないキー");
    let bad = tokens::validate(&refs, &value);
    assert!(bad.iter().any(|x| x.contains("just-wrong")), "{bad:?}");
}

#[test]
fn what_the_css_references_is_what_it_defines() {
    // **未定義の参照は無言で初期値へ縮退し、誰も検出できない**
    let refs = references();
    let value = tokens::load(&tokens::path(&refs)).expect("読める");
    let built = tokens::css(&value, false).expect("組める");
    let (used, defs) = tokens::refs_and_defs(&built);
    let short: Vec<&String> = used.iter().filter(|k| !defs.contains(k)).collect();
    assert!(short.is_empty(), "定義の無い参照が在る ── {short:?}");
}

#[test]
fn the_parts_the_build_needs_are_present() {
    let parts = Parts::load(&references().join("board.template.html")).expect("読める");
    let names = parts.names();
    for want in [
        "page", "board", "front", "topic", "tab", "pane", "mark", "cell", "table", "form",
        "drawer", "queue",
    ] {
        assert!(names.contains(&want), "{want} が型に無い");
    }
}

#[test]
fn an_overlapping_label_is_found() {
    let svg = "<svg viewBox=\"0 0 200 100\">\
        <text x=\"10\" y=\"20\" font-size=\"12\">かさなる文字</text>\
        <text x=\"20\" y=\"22\" font-size=\"12\">こちらも文字</text></svg>";
    let got = figcheck::check("試し", svg);
    assert_eq!(got.counts.0, 1, "{:?}", got.lines);
    assert!(got.total() > 0);
}

#[test]
fn a_label_outside_the_canvas_is_found() {
    let svg = "<svg viewBox=\"0 0 50 100\">\
        <text x=\"10\" y=\"20\" font-size=\"12\">はみ出す長い文字列である</text></svg>";
    let got = figcheck::check("試し", svg);
    assert_eq!(got.counts.1, 1, "{:?}", got.lines);
}

#[test]
fn a_line_piercing_a_box_is_found() {
    let svg = "<svg viewBox=\"0 0 200 200\">\
        <rect x=\"50\" y=\"50\" width=\"100\" height=\"50\"/>\
        <line x1=\"100\" y1=\"10\" x2=\"100\" y2=\"190\"/></svg>";
    let got = figcheck::check("試し", svg);
    assert_eq!(got.counts.2, 1, "{:?}", got.lines);
}

#[test]
fn an_indent_made_of_spaces_is_found() {
    // **SVG は行頭の空白を除去するので、階層が潰れる**
    let svg = "<svg viewBox=\"0 0 200 100\">\
        <text x=\"10\" y=\"20\" font-size=\"12\">　　字下げした文字</text></svg>";
    let got = figcheck::check("試し", svg);
    assert_eq!(got.counts.3, 1, "{:?}", got.lines);
}

#[test]
fn a_label_across_lines_is_not_measured() {
    // **幅の見積もりは「1行ぶんの字数 × 字寸」なので、中に改行を持つものへ当てると、
    // 実際の3倍の幅を主張して重なりを捏造する**
    let svg = "<svg viewBox=\"0 0 200 100\">\
        <text x=\"10\" y=\"20\" font-size=\"12\">\n1行目\n2行目\n</text>\
        <text x=\"20\" y=\"22\" font-size=\"12\">\n甲\n乙\n</text></svg>";
    let got = figcheck::check("試し", svg);
    assert_eq!(got.items, 0, "{:?}", got.lines);
    assert_eq!(got.total(), 0);
}

#[test]
fn a_sound_figure_is_not_found() {
    // **0件にできるものだけを検査している**
    let svg = "<svg viewBox=\"0 0 200 100\">\
        <text x=\"10\" y=\"20\" font-size=\"12\">甲</text>\
        <text x=\"10\" y=\"60\" font-size=\"12\">乙</text></svg>";
    let got = figcheck::check("試し", svg);
    assert_eq!(got.total(), 0, "{:?}", got.lines);
    assert_eq!(got.items, 2);
}
