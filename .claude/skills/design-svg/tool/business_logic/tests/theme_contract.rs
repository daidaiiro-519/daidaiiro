// SPDX-License-Identifier: MIT
//! theme.json の契約 ── **図のデザインシステムが、ブレストボード design-svg-rework の論点3で決めた
//! 形を持つこと。**
//!
//! 段階（フォントサイズ ・ ストローク幅 ・ 角丸の半径 ・ 破線のパターン）は、部分集合ではなく
//! 等しいことを1つずつ確かめる。しきい値と禁止した組み合わせは theme.json から読み、テストに
//! 値を書かない ── 値を決め直しても、テストを書き換えずに済む。

use std::path::PathBuf;

use ds_business_logic::classes::{self, Kind};
use ds_business_logic::theme;
use serde_json::Value;

fn refs() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

/// トークンの名前を、1層だけたどった値。
fn value(tok: &str) -> Option<Value> {
    let t = theme::default_theme();
    let v = t.get(tok)?;
    Some(match v {
        Value::String(s) => t.get(s).cloned().unwrap_or_else(|| v.clone()),
        other => other.clone(),
    })
}

#[test]
fn the_theme_passes_its_schema() {
    let found = ds_business_logic::refs::validate(&refs()).expect("検査できる");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn the_font_sizes_are_exactly_the_decided_set() {
    assert_eq!(
        classes::notation().scale.font_size,
        vec![10.0, 11.0, 12.0, 13.0]
    );
}

#[test]
fn the_stroke_widths_are_exactly_the_decided_set() {
    assert_eq!(classes::notation().scale.stroke_width, vec![1.0, 1.5, 2.0]);
}

#[test]
fn the_radii_are_exactly_the_decided_set() {
    assert_eq!(classes::notation().scale.radius, vec![3.0, 5.0, 8.0]);
}

#[test]
fn the_dash_patterns_are_exactly_the_decided_set() {
    assert_eq!(classes::notation().scale.dash, vec!["4 3".to_owned()]);
}

#[test]
fn the_vocabulary_holds_every_class_the_board_listed() {
    // 論点3の答え（図形 ・ 線 ・ テキスト）。ここに無い class を足すのは ACDR で決める
    let board = [
        "box", "focus", "warn", "kind-1", "kind-2", "kind-3", "boundary", "area", "badge",
        "swatch", "series-1", "series-2", "series-3", "flow", "async", "link", "label", "note",
    ];
    let missing: Vec<&str> = board
        .iter()
        .copied()
        .filter(|n| !classes::known(n))
        .collect();
    assert!(missing.is_empty(), "一覧に無い: {missing:?}");
}

#[test]
fn every_class_has_one_meaning_and_values_for_its_kind() {
    let mut bad = Vec::new();
    for c in &classes::notation().classes {
        if c.meaning.trim().is_empty() {
            bad.push(format!("{}: 意味が無い", c.name));
        }
        match c.kind {
            Kind::Modifier => {
                if c.values.is_empty() {
                    bad.push(format!("{}: 修飾なのに値が無い", c.name));
                }
            }
            k => {
                if !c.values.contains_key(k.slot()) {
                    bad.push(format!("{}: {} の値が無い", c.name, k.slot()));
                }
            }
        }
    }
    assert!(bad.is_empty(), "{bad:?}");
}

/// class の値をすべて並べる。`(class, 欄, 属性, トークン)`
fn every_value() -> Vec<(String, String, String, String)> {
    classes::notation()
        .classes
        .iter()
        .flat_map(|c| {
            c.values.iter().flat_map(move |(slot, attrs)| {
                attrs
                    .iter()
                    .map(move |(a, t)| (c.name.clone(), slot.clone(), a.clone(), t.clone()))
            })
        })
        .collect()
}

#[test]
fn every_token_a_class_points_to_exists() {
    let bad: Vec<String> = every_value()
        .into_iter()
        .filter(|(.., t)| t != "none" && t != "arrow" && value(t).is_none())
        .map(|(c, s, a, t)| format!("{c}.{s}.{a} → {t}"))
        .collect();
    assert!(bad.is_empty(), "テーマに無いトークン: {bad:?}");
}

#[test]
fn every_size_a_class_resolves_to_is_on_the_scale() {
    let s = &classes::notation().scale;
    let mut bad = Vec::new();
    for (c, slot, a, t) in every_value() {
        let Some(v) = value(&t) else { continue };
        let ok = match a.as_str() {
            "font-size" => v.as_f64().is_some_and(|x| s.font_size.contains(&x)),
            "stroke-width" => v.as_f64().is_some_and(|x| s.stroke_width.contains(&x)),
            "rx" => v.as_f64().is_some_and(|x| s.radius.contains(&x)),
            "stroke-dasharray" => v.as_str().is_some_and(|x| s.dash.iter().any(|d| d == x)),
            _ => true,
        };
        if !ok {
            bad.push(format!("{c}.{slot}.{a} = {v}"));
        }
    }
    assert!(bad.is_empty(), "段階に無い値: {bad:?}");
}

#[test]
fn every_colour_a_class_uses_has_a_light_and_a_dark_value() {
    let dark = &classes::notation().dark;
    let mut bad = Vec::new();
    for (c, slot, a, t) in every_value() {
        if a != "fill" && a != "stroke" || t == "none" {
            continue;
        }
        let light = value(&t).and_then(|v| v.as_str().map(str::to_owned));
        if !light.is_some_and(|l| l.starts_with('#')) {
            bad.push(format!("{c}.{slot}.{a}: ライトの値が色でない"));
        }
        if !dark.contains_key(&t) {
            bad.push(format!("{c}.{slot}.{a}: {t} のダークの値が無い"));
        }
    }
    assert!(bad.is_empty(), "{bad:?}");
}

#[test]
fn the_forbidden_pairs_name_known_classes() {
    let bad: Vec<String> = classes::notation()
        .forbidden
        .iter()
        .filter(|(a, b, why)| !classes::known(a) || !classes::known(b) || why.is_empty())
        .map(|(a, b, _)| format!("{a} × {b}"))
        .collect();
    assert!(bad.is_empty(), "{bad:?}");
    // 論点3が例に挙げた組は、表に在る
    for (a, b) in [("focus", "boundary"), ("warn", "focus"), ("box", "flow")] {
        assert!(
            classes::conflict(a, b).is_some(),
            "{a} × {b} が禁止されていない"
        );
    }
}

#[test]
fn the_thresholds_live_in_the_theme() {
    let c = classes::notation().checks;
    assert!(c.phone_width > 0.0, "スマホ幅が無い");
    assert!(c.min_font_px > 0.0, "しきい値 T が無い");
    assert!(c.precision > 0.0, "比較の精度が無い");
    let m = &classes::notation().marker;
    assert!(!m.d.is_empty() && m.width > 0.0, "矢じりの形が無い");
}

#[test]
fn the_template_classes_are_in_the_vocabulary() {
    // テンプレート（ACDR 2）が要る class ── 開始状態 ・ 入力欄 ・ 集合
    for n in ["start", "field", "set"] {
        assert!(classes::known(n), "一覧に無い: {n}");
    }
}

#[test]
fn a_set_takes_its_category_colour_at_the_set_opacity() {
    let v = classes::values_of(&["set", "kind-2"]);
    let get = |a: &str| v.iter().find(|(k, _)| k == a).map(|(_, t)| t.clone());
    assert_eq!(
        get("fill").as_deref(),
        Some("color.kind-2"),
        "背景色ではなくカテゴリ色で塗る"
    );
    assert_eq!(get("fill-opacity").as_deref(), Some("opacity.set"));
    assert!(value("opacity.set")
        .and_then(|x| x.as_f64())
        .is_some_and(|o| o > 0.0 && o < 1.0));
}

#[test]
fn the_template_classes_refuse_the_wrong_modifiers() {
    for (a, b) in [
        ("start", "focus"),
        ("set", "warn"),
        ("set", "focus"),
        ("field", "kind-1"),
    ] {
        assert!(
            classes::conflict(a, b).is_some(),
            "{a} × {b} が禁止されていない"
        );
    }
    for (a, b) in [("set", "kind-1"), ("field", "warn"), ("field", "focus")] {
        assert!(classes::conflict(a, b).is_none(), "{a} × {b} は使える");
    }
}
