// SPDX-License-Identifier: MIT
//! 見た目の解決 ── カスケードの優先順と、範囲外の値を弾くこと。

use ds_business_logic::registry::{known_kinds, render};
use ds_business_logic::style::resolve;
use ds_business_logic::theme::{default_theme, source};
use serde_json::{json, Map, Value};

fn over(v: Value) -> Map<String, Value> {
    v.as_object().expect("対応表").clone()
}

fn with(extra: Value) -> Map<String, Value> {
    let mut t = default_theme().clone();
    for (k, v) in over(extra) {
        t.insert(k, v);
    }
    t
}

#[test]
fn theme_value_is_used_by_default() {
    let s = resolve("plain", None, None).expect("解決できる");
    assert_eq!(
        s.num("font.size").ok(),
        default_theme()["font.size"].as_f64()
    );
}

#[test]
fn role_overrides_theme() {
    let focus = resolve("focus", None, None).expect("解決できる");
    let plain = resolve("plain", None, None).expect("解決できる");
    assert_ne!(focus.values, plain.values);
}

#[test]
fn inline_override_beats_role() {
    let s = resolve(
        "focus",
        Some(&over(json!({"size.stroke-width": 3.0}))),
        None,
    )
    .expect("解決できる");
    assert_eq!(s.num("size.stroke-width"), Ok(3.0));
}

#[test]
fn replaced_theme_becomes_the_base() {
    let t = with(json!({"font.size": 20.0}));
    assert_eq!(
        resolve("plain", None, Some(&t))
            .expect("解決できる")
            .num("font.size"),
        Ok(20.0)
    );
}

#[test]
fn token_reference_is_resolved() {
    let s = resolve(
        "plain",
        Some(&over(json!({"color.ink": "color.accent"}))),
        None,
    )
    .expect("解決できる");
    assert_eq!(
        s.text("color.ink").ok().as_deref(),
        default_theme()["color.accent"].as_str()
    );
}

#[test]
fn adding_to_theme_adds_a_role() {
    // 役割はテーマが持つ ── 数を増やすのにエンジンを触らせない
    let t = with(
        json!({"role.危険.color.box-fill": "#FBE9E7", "role.危険.color.box-stroke": "color.warn"}),
    );
    let s = resolve("危険", None, Some(&t)).expect("解決できる");
    assert_eq!(s.text("color.box-fill").as_deref(), Ok("#FBE9E7"));
    assert_eq!(
        s.text("color.box-stroke").ok().as_deref(),
        default_theme()["color.warn"].as_str()
    );
}

#[test]
fn unknown_role_fails_before_drawing() {
    // 黙って既定で描くと綴り違いに気づけない。使える役割を挙げて返す
    let err = resolve("知らない役割", None, None).expect_err("断る");
    assert!(err.contains("focus"), "{err}");
}

#[test]
fn role_without_overrides_always_passes() {
    assert!(resolve("plain", None, Some(default_theme())).is_ok());
}

#[test]
fn role_definition_does_not_leak_into_result() {
    let s = resolve("focus", None, None).expect("解決できる");
    assert!(!s.values.keys().any(|k| k.starts_with("role.")));
}

#[test]
fn override_with_unknown_name_fails() {
    let err = resolve("plain", Some(&over(json!({"size.box-hight": 40}))), None).expect_err("断る");
    assert!(err.contains("size.box-hight"), "{err}");
}

#[test]
fn override_with_known_name_passes() {
    let s = resolve("plain", Some(&over(json!({"size.box-h": 40}))), None).expect("解決できる");
    assert_eq!(s.num("size.box-h"), Ok(40.0));
}

#[test]
fn theme_missing_a_key_fails_before_drawing() {
    // 欠けたまま描き始めると、そのキーを参照する部品に当たった時点で組みかけの SVG を破棄する
    assert!(resolve("plain", None, Some(&over(json!({"font.size": 12})))).is_err());
}

#[test]
fn out_of_range_fails_and_in_range_passes() {
    assert!(!source().ranges.is_empty(), "範囲を持つトークンが在る");
    for (key, lo, hi) in &source().ranges {
        for bad in [lo - 1.0, hi + 1.0] {
            assert!(
                resolve("plain", Some(&over(json!({ key.as_str(): bad }))), None).is_err(),
                "{key} = {bad} を通した"
            );
        }
        let mid = (lo + hi) / 2.0;
        let s = resolve("plain", Some(&over(json!({ key.as_str(): mid }))), None)
            .expect("範囲の中は通る");
        assert_eq!(s.num(key), Ok(mid), "{key}");
    }
}

#[test]
fn default_theme_fits_its_own_bounds() {
    assert!(resolve("plain", None, None).is_ok());
}

#[test]
fn unregistered_component_is_refused_by_name() {
    let st = resolve("plain", None, None).expect("解決できる");
    let err = render("そんな部品はない", &Map::new(), &st).expect_err("断る");
    assert!(err.contains("そんな部品はない"), "{err}");
}

#[test]
fn core_components_are_registered() {
    let kinds = known_kinds();
    for k in ["box", "edge", "frame"] {
        assert!(kinds.contains(&k), "{k}");
    }
}
