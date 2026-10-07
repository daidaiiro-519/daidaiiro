// SPDX-License-Identifier: MIT
//! 検査1〜4 ── **事例は qa-advisor の方針で作る。**
//!
//! - 検査1 ・ 2 ── 同値分割（イーチチョイス）。無効パーティションは1事例に1つだけ入れる
//! - 検査3 ── class の全ての組のデシジョンテーブル。期待値は theme.json の表から作る
//! - 検査4 ── 3値の境界値分析。しきい値 T は theme.json から読む
//!
//! 失敗したときは、事例の名前を集めて1つのメッセージで出す ── どの事例が落ちたかが分かる。

use ds_business_logic::checks::{
    check_classes, check_combinations, check_layout, check_values, labels, min_font_px,
    rendered_width,
};
use ds_business_logic::classes::{self, Kind};
use ds_business_logic::resolve::resolve;
use ds_business_logic::xml;

fn svg(body: &str) -> String {
    format!(r#"<svg viewBox="0 0 400 300">{body}</svg>"#)
}

/// 事例の表を回し、期待と違った事例の名前を集める。
fn table(cases: &[(&str, String, usize)], f: fn(&str) -> Result<Vec<String>, String>) {
    let mut bad = Vec::new();
    for (name, body, want) in cases {
        let got = f(body).expect("読める");
        if got.len() != *want {
            bad.push(format!("{name}: {} 件（期待 {want}）{got:?}", got.len()));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

// ── 検査1 ── パーティションセット1：要素の種類 ／ パーティションセット2：class 属性の値

#[test]
fn check1_every_drawing_element_needs_a_class() {
    let drawn = [
        r#"<rect x="0" y="0" width="10" height="10"/>"#,
        r#"<circle cx="5" cy="5" r="5"/>"#,
        r#"<ellipse cx="5" cy="5" rx="5" ry="3"/>"#,
        r#"<line x1="0" y1="0" x2="10" y2="0"/>"#,
        r#"<polyline points="0,0 10,0"/>"#,
        r#"<polygon points="0,0 10,0 5,5"/>"#,
        r#"<path d="M0,0 L10,0"/>"#,
        r#"<text x="0" y="10">甲</text>"#,
        r#"<image href="a.png" x="0" y="0" width="10" height="10"/>"#,
    ];
    let cases: Vec<(&str, String, usize)> = drawn.iter().map(|e| (*e, svg(e), 1)).collect();
    table(&cases, check_classes);
}

#[test]
fn check1_structural_elements_need_no_class() {
    let cases = [
        (
            "svg と g",
            svg(r#"<g><rect class="box" x="0" y="0" width="1" height="1"/></g>"#),
            0,
        ),
        (
            "defs",
            svg(r#"<defs><linearGradient id="g"><stop offset="0"/></linearGradient></defs>"#),
            0,
        ),
        (
            "title と desc",
            svg("<title>図</title><desc>説明</desc>"),
            0,
        ),
        (
            "tspan は親の text に従う",
            svg(r#"<text class="label" x="0" y="10"><tspan>甲</tspan></text>"#),
            0,
        ),
    ];
    table(&cases, check_classes);
}

#[test]
fn check1_a_class_on_the_parent_g_does_not_cover_its_children() {
    let cases = [(
        "親の g の class",
        svg(r#"<g class="box"><rect x="0" y="0" width="10" height="10"/></g>"#),
        1,
    )];
    table(&cases, check_classes);
}

#[test]
fn check1_an_author_written_marker_is_found() {
    let cases = [(
        "marker",
        svg(r#"<defs><marker id="a"><path d="M0,0 L10,5 L0,10 z"/></marker></defs>"#),
        1,
    )];
    table(&cases, check_classes);
}

#[test]
fn check1_partitions_of_the_class_attribute() {
    let rect = |cls: &str| {
        svg(&format!(
            r#"<rect{cls} x="0" y="0" width="10" height="10"/>"#
        ))
    };
    let cases = [
        ("無い", rect(""), 1),
        ("空文字", rect(r#" class="""#), 1),
        ("一覧の1つ", rect(r#" class="box""#), 0),
        ("一覧の複数", rect(r#" class="box focus""#), 0),
        ("一覧に無い1つ", rect(r#" class="card""#), 1),
        ("一覧の1つと一覧に無い1つ", rect(r#" class="box card""#), 1),
    ];
    table(&cases, check_classes);
}

// ── 検査2 ── パーティションセット1：属性 ／ 2：書き方 ／ 3：値の形

#[test]
fn check2_each_attribute_is_found() {
    let cases = [
        (
            "fill",
            svg(r##"<rect class="box" fill="#fff" x="0" y="0" width="1" height="1"/>"##),
            1,
        ),
        (
            "stroke",
            svg(r##"<rect class="box" stroke="#ffffff" x="0" y="0" width="1" height="1"/>"##),
            1,
        ),
        (
            "stroke-width",
            svg(r#"<rect class="box" stroke-width="1.3" x="0" y="0" width="1" height="1"/>"#),
            1,
        ),
        (
            "rx",
            svg(r#"<rect class="box" rx="10" x="0" y="0" width="1" height="1"/>"#),
            1,
        ),
        (
            "ry",
            svg(r#"<rect class="box" ry="10" x="0" y="0" width="1" height="1"/>"#),
            1,
        ),
        (
            "stroke-dasharray",
            svg(r#"<path class="flow" stroke-dasharray="5 4" d="M0,0 L1,1"/>"#),
            1,
        ),
        (
            "font-size",
            svg(r#"<text class="label" font-size="12" x="0" y="10">甲</text>"#),
            1,
        ),
        (
            "楕円の rx は値ではない",
            svg(r#"<ellipse class="box" cx="5" cy="5" rx="5" ry="3"/>"#),
            0,
        ),
        (
            "何も書かない",
            svg(r#"<rect class="box" x="0" y="0" width="1" height="1"/>"#),
            0,
        ),
    ];
    table(&cases, check_values);
}

#[test]
fn check2_each_way_of_writing_is_found() {
    let cases = [
        (
            "表示属性",
            svg(r#"<rect class="box" fill="red" x="0" y="0" width="1" height="1"/>"#),
            1,
        ),
        (
            "style 属性",
            svg(r#"<rect class="box" style="fill:red" x="0" y="0" width="1" height="1"/>"#),
            1,
        ),
        (
            "作成者の style 要素",
            svg("<style>.box{fill:red}</style>"),
            1,
        ),
        (
            "style 属性の幅は値ではない",
            svg(r#"<rect class="box" style="min-width:560px" x="0" y="0" width="1" height="1"/>"#),
            0,
        ),
    ];
    table(&cases, check_values);
}

#[test]
fn check2_each_value_form_is_found_except_none() {
    let fill = |v: &str| {
        svg(&format!(
            r#"<rect class="box" fill="{v}" x="0" y="0" width="1" height="1"/>"#
        ))
    };
    let cases = [
        ("#rgb", fill("#fff"), 1),
        ("#rrggbb", fill("#ffffff"), 1),
        ("rgb()", fill("rgb(1,2,3)"), 1),
        ("色名", fill("white"), 1),
        ("var(--x)", fill("var(--accent)"), 1),
        ("currentColor", fill("currentColor"), 1),
        ("none は値ではない", fill("none"), 0),
        (
            "stroke の none",
            svg(r#"<rect class="box" stroke="none" x="0" y="0" width="1" height="1"/>"#),
            0,
        ),
    ];
    table(&cases, check_values);
}

#[test]
fn check2_a_value_on_a_g_is_found() {
    let cases = [(
        "g の fill",
        svg(r##"<g fill="#000"><rect class="box" x="0" y="0" width="1" height="1"/></g>"##),
        1,
    )];
    table(&cases, check_values);
}

#[test]
fn check2_the_swatch_exception_covers_only_its_own_colours() {
    let cases = [
        (
            "① swatch の fill",
            svg(r##"<rect class="swatch" fill="#0d5c55" x="0" y="0" width="1" height="1"/>"##),
            0,
        ),
        (
            "① swatch の stroke と stop-color",
            svg(r##"<stop class="swatch" stop-color="#0d5c55" stroke="#000"/>"##),
            0,
        ),
        (
            "② swatch でない要素の fill",
            svg(r##"<rect class="box" fill="#0d5c55" x="0" y="0" width="1" height="1"/>"##),
            1,
        ),
        (
            "③ swatch の font-size",
            svg(r#"<text class="swatch" font-size="12" x="0" y="10">甲</text>"#),
            1,
        ),
        (
            "④ 親の g が swatch で、子の fill",
            svg(
                r##"<g class="swatch"><rect class="box" fill="#0d5c55" x="0" y="0" width="1" height="1"/></g>"##,
            ),
            1,
        ),
    ];
    table(&cases, check_values);
}

#[test]
fn check2_is_applied_before_resolving_not_after() {
    // 解決の前は0件。解決のあとは値が属性に在るので、検査2の対象にしない
    let before = svg(r#"<rect class="box" x="0" y="0" width="10" height="10"/>"#);
    assert!(check_values(&before).expect("読める").is_empty());
    let after = resolve(&before, None, true).expect("解決できる");
    assert!(
        !check_values(&after).expect("読める").is_empty(),
        "解決のあとに適用すると検出になる ── だから適用しない"
    );
}

// ── 検査3 ── デシジョンテーブル

/// 期待値。**theme.json の表と、種類の規則から作る。**
fn expected_conflict(a: &classes::Class, b: &classes::Class) -> bool {
    let n = classes::notation();
    if n.forbidden
        .iter()
        .any(|(p, q, _)| (*p == a.name && *q == b.name) || (*p == b.name && *q == a.name))
    {
        return true;
    }
    match (a.kind, b.kind) {
        (Kind::Modifier, Kind::Modifier) => false,
        (Kind::Modifier, _) => {
            !b.modifiers
                || !(a.values.contains_key(b.kind.slot()) || a.values.contains_key(&b.name))
        }
        (_, Kind::Modifier) => {
            !a.modifiers
                || !(b.values.contains_key(a.kind.slot()) || b.values.contains_key(&a.name))
        }
        _ => true,
    }
}

#[test]
fn check3_every_pair_of_classes_matches_the_table() {
    let all = &classes::notation().classes;
    let mut bad = Vec::new();
    let mut columns = 0;
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            columns += 1;
            let want = expected_conflict(a, b);
            let got = classes::conflict(&a.name, &b.name).is_some();
            if want != got {
                bad.push(format!("{} × {}: {got}（期待 {want}）", a.name, b.name));
            }
        }
    }
    let n = all.len();
    assert_eq!(columns, n * (n - 1) / 2, "列が欠けている");
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn check3_the_examples_from_the_board_are_found_once() {
    let rect = |cls: &str| {
        svg(&format!(
            r#"<rect class="{cls}" x="0" y="0" width="1" height="1"/>"#
        ))
    };
    let cases = [
        ("focus と boundary", rect("boundary focus"), 1),
        ("warn と focus", rect("box warn focus"), 1),
        (
            "box に指定した flow",
            svg(r#"<path class="box flow" d="M0,0 L1,1"/>"#),
            1,
        ),
        ("禁止の組を含む3つの組", rect("box focus warn"), 1),
        ("box 単独", rect("box"), 0),
        ("boundary 単独", rect("boundary"), 0),
        ("focus だけ", rect("focus"), 1),
        ("テキストの class を矩形に", rect("label"), 1),
        (
            "図形の class をテキストに",
            svg(r#"<text class="box" x="0" y="10">甲</text>"#),
            1,
        ),
    ];
    table(&cases, check_combinations);
}

#[test]
fn check3_each_class_alone_is_fine_on_a_fitting_element() {
    let mut bad = Vec::new();
    for c in &classes::notation().classes {
        let body = match c.kind {
            Kind::Modifier => continue,
            Kind::Text => format!(r#"<text class="{}" x="0" y="10">甲</text>"#, c.name),
            Kind::Line => format!(r#"<path class="{}" d="M0,0 L1,1"/>"#, c.name),
            Kind::Shape => format!(
                r#"<rect class="{}" x="0" y="0" width="1" height="1"/>"#,
                c.name
            ),
        };
        let got = check_combinations(&svg(&body)).expect("読める");
        if !got.is_empty() {
            bad.push(format!("{}: {got:?}", c.name));
        }
    }
    assert!(bad.is_empty(), "{bad:?}");
}

// ── 検査4 ── 境界値分析

fn t() -> f64 {
    classes::notation().checks.min_font_px
}

fn precision() -> f64 {
    classes::notation().checks.precision
}

#[test]
fn check4_the_rendering_width_has_its_boundaries_at_the_phone_width() {
    let phone = classes::notation().checks.phone_width;
    let width = |style: &str| {
        let s = format!(r#"<svg viewBox="0 0 400 300"{style}></svg>"#);
        rendered_width(&xml::parse(&s).expect("読める"))
    };
    assert!((width("") - phone).abs() < f64::EPSILON, "min-width 無し");
    for (min, want) in [
        (phone - 1.0, phone),
        (phone, phone),
        (phone + 1.0, phone + 1.0),
    ] {
        let got = width(&format!(r#" style="min-width:{min}px""#));
        assert!((got - want).abs() < f64::EPSILON, "min-width {min}: {got}");
    }
}

/// 実効のフォントサイズが T ちょうどになる図。**viewBox の幅 1000 ・ font-size 10 ・ min-width で
/// 描画の幅を決める** ── 描画の幅 1px が実効の 0.01px になり、精度と一致する。
fn at_threshold(step: f64) -> String {
    let w = (t() * 100.0 + step).round();
    format!(
        r#"<svg viewBox="0 0 1000 300" style="min-width:{w}px"><text x="10" y="100" font-size="10">甲</text></svg>"#
    )
}

#[test]
fn check4_the_smallest_font_has_its_boundary_at_t() {
    let step = precision() * 100.0;
    let found = |s: &str| {
        check_layout(s)
            .expect("読める")
            .iter()
            .any(|f| f.contains("最小のフォントサイズ"))
    };
    assert!(
        !found(&at_threshold(0.0)),
        "T ちょうどは合格: {:?}",
        min_font_px(&at_threshold(0.0))
    );
    assert!(!found(&at_threshold(step)), "T を超える側は合格");
    assert!(
        found(&at_threshold(-step)),
        "T を下回る側は検出: {:?}",
        min_font_px(&at_threshold(-step))
    );
}

/// 幅 w の文字列を、ボックスの左端から dx の位置に置く。
fn in_box(dx: f64) -> String {
    let fs = 12.0;
    let w = ds_business_logic::text::width("注文", fs);
    let x = 10.0 + dx;
    let r = format!(
        r#"<rect class="box" x="10" y="10" width="{w}" height="40"/><text class="label" x="{x}" y="34" font-size="{fs}">注文</text>"#
    );
    svg(&r)
}

#[test]
fn check4_text_leaving_its_box_has_its_boundary_at_the_edge() {
    let found = |s: &str| {
        check_layout(s)
            .expect("読める")
            .iter()
            .filter(|f| f.contains("ボックスからはみ出す"))
            .count()
    };
    assert_eq!(found(&in_box(-0.0)), 0, "辺にちょうど接する");
    assert_eq!(found(&in_box(-1.0)), 1, "1単位はみ出す");
    let inside = svg(
        r#"<rect class="box" x="10" y="10" width="200" height="40"/><text class="label" x="20" y="34" font-size="12">注文</text>"#,
    );
    assert_eq!(found(&inside), 0, "内側");
}

#[test]
fn check4_overlapping_text_is_still_found() {
    let s = svg(
        r#"<text class="label" x="10" y="20" font-size="12">注文</text><text class="label" x="12" y="22" font-size="12">注文</text>"#,
    );
    assert!(check_layout(&s)
        .expect("読める")
        .iter()
        .any(|f| f.contains("文字が重なる")));
}

#[test]
fn multiline_labels_are_listed_from_two_lines() {
    let one = svg(
        r#"<rect class="box" x="0" y="0" width="200" height="60"/><text class="label" x="10" y="20">支払い期限</text>"#,
    );
    assert!(
        labels(&one).expect("読める").lines.is_empty(),
        "1行は載らない"
    );
    let two = svg(
        r#"<rect class="box" x="0" y="0" width="200" height="60"/><text class="label" x="10" y="20">支払い期限</text><text class="note" x="10" y="40">（48時間）切れ</text>"#,
    );
    assert_eq!(
        labels(&two).expect("読める").lines,
        vec!["支払い期限｜（48時間）切れ".to_owned()]
    );
}

#[test]
fn labels_are_listed_by_class() {
    let s = svg(
        r#"<rect class="box kind-2" x="0" y="0" width="200" height="60"/><text class="label" x="10" y="20">注文が確定された</text><text class="note warn" x="10" y="200">拒否する</text>"#,
    );
    let l = labels(&s).expect("読める");
    assert_eq!(
        l.by_class.get("kind-2"),
        Some(&vec!["注文が確定された".to_owned()])
    );
    assert_eq!(l.by_class.get("warn"), Some(&vec!["拒否する".to_owned()]));
}

// ── 検査4 ── 線の上の文字（境界値）

/// 縦線 x=100 の左に、右端が x=100+dx の文字を置く。
fn text_by_line(dx: f64) -> String {
    let fs = 12.0;
    let w = ds_business_logic::text::width("七月中旬", fs);
    let x = 100.0 + dx - w;
    svg(&format!(
        r#"<line class="grid" x1="100" y1="0" x2="100" y2="200"/><text class="label" x="{x}" y="100" font-size="{fs}">七月中旬</text>"#
    ))
}

fn over_stroke(s: &str) -> usize {
    let s = resolve(s, None, true).expect("解決できる");
    check_layout(&s)
        .expect("読める")
        .iter()
        .filter(|f| f.contains("線の上に文字"))
        .count()
}

#[test]
fn check4_text_crossing_a_stroke_has_its_boundary_at_the_line() {
    assert_eq!(over_stroke(&text_by_line(-1.0)), 0, "線の手前で終わる");
    assert_eq!(over_stroke(&text_by_line(0.0)), 0, "線にちょうど接する");
    assert_eq!(over_stroke(&text_by_line(1.0)), 1, "線を1単位越える");
}

#[test]
fn check4_text_over_a_circle_outline_is_found() {
    let s = svg(
        r#"<circle class="area" cx="200" cy="150" r="100"/><text class="title" x="200" y="54" text-anchor="middle">設計</text>"#,
    );
    assert_eq!(over_stroke(&s), 1);
}

#[test]
fn check4_a_badge_on_a_line_and_a_label_in_a_box_are_allowed() {
    let badge = svg(
        r#"<path class="link" d="M0,100 H300"/><rect class="badge" x="100" y="90" width="80" height="20"/><text class="note small" x="140" y="104" text-anchor="middle">共用</text>"#,
    );
    assert_eq!(over_stroke(&badge), 0, "線の上のバッジ");
    let boxed = svg(
        r#"<rect class="box" x="10" y="10" width="200" height="40"/><text class="label" x="20" y="34">箱の中の名前</text>"#,
    );
    assert_eq!(over_stroke(&boxed), 0, "箱の中のラベル");
}

#[test]
fn check4_a_line_hidden_under_a_filled_box_is_allowed() {
    // 2つの領域にまたがる箱の名前 ── 箱の塗りが、先に描かれた領域の線を隠す
    let s = svg(
        r#"<rect class="area" x="10" y="10" width="150" height="200"/><rect class="area" x="160" y="10" width="150" height="200"/><rect class="box" x="60" y="80" width="200" height="40"/><text class="label" x="160" y="104" text-anchor="middle">2つの領域の振り分け</text>"#,
    );
    assert_eq!(over_stroke(&s), 0);
    // 線が箱より後に描かれると、箱の中の名前の上を通る
    let late = svg(
        r#"<rect class="box" x="60" y="80" width="200" height="40"/><text class="label" x="160" y="104" text-anchor="middle">2つの領域の振り分け</text><line class="grid" x1="160" y1="10" x2="160" y2="200"/>"#,
    );
    assert_eq!(over_stroke(&late), 1);
}
