// SPDX-License-Identifier: MIT
//! class の解決 ── **1 class ＝ 1事例の表を theme.json から作り、class ごとに確かめる。**
//!
//! 各事例で ①属性の値がライトの値である ②class が残る ③style にダークの規則が在る
//! ④座標 ・ d ・ テキストが入力と同じ、を確かめる。加えて ⑤2回解決しても同じ（冪等）
//! ⑥上書きの有無 × ダークの値の有無のデシジョンテーブル。

use ds_business_logic::classes::{self, Class, Kind};
use ds_business_logic::resolve::{resolve, ARROW_PREFIX, ID_HEAD, ROOT_CLASS};
use ds_business_logic::theme;
use ds_business_logic::xml::{self, Element};
use serde_json::{Map, Value};

/// その class を試す要素。**修飾は、値を持つ欄の土台の class に重ねる。**
fn sample(c: &Class) -> (String, String) {
    let slot = match c.kind {
        Kind::Modifier => ["shape", "line", "text"]
            .into_iter()
            .find(|s| c.values.contains_key(*s))
            .unwrap_or("shape"),
        k => k.slot(),
    };
    let base = match (c.kind, slot) {
        (Kind::Modifier, "shape") => "box ",
        (Kind::Modifier, "line") => "flow ",
        (Kind::Modifier, _) => "label ",
        _ => "",
    };
    let cls = format!("{base}{}", c.name);
    let el = match slot {
        "line" => format!(r#"<path class="{cls}" d="M10,10 L90,10"/>"#),
        "text" => format!(r#"<text class="{cls}" x="10" y="40">注文</text>"#),
        _ => format!(r#"<rect class="{cls}" x="10" y="10" width="80" height="30"/>"#),
    };
    (cls, format!(r#"<svg viewBox="0 0 100 60">{el}</svg>"#))
}

fn first_drawn(root: &Element) -> &Element {
    root.elements()
        .find(|e| e.get("data-dsvg").is_none())
        .expect("描く要素が在る")
}

fn token(name: &str) -> String {
    let t = theme::default_theme();
    let v = t.get(name).expect("トークンが在る");
    let v = match v {
        Value::String(s) => t.get(s).unwrap_or(v),
        o => o,
    };
    match v {
        Value::String(s) => s.clone(),
        o => ds_business_logic::py::num(o),
    }
}

#[test]
fn every_class_resolves_to_its_light_values_and_keeps_the_class() {
    let mut bad = Vec::new();
    for c in &classes::notation().classes {
        let (cls, svg) = sample(c);
        let out = resolve(&svg, None, true).expect("解決できる");
        let root = xml::parse(&out).expect("読める");
        let el = first_drawn(&root);
        // ② class が残る
        if el.get("class") != Some(cls.as_str()) {
            bad.push(format!("{}: class が残らない", c.name));
        }
        // ① 属性の値がライトの値
        let names: Vec<&str> = cls.split(' ').collect();
        for (attr, tok) in classes::values_of(&names) {
            if attr == "rx" && el.tag != "rect" {
                continue;
            }
            let want = match tok.as_str() {
                "none" => "none".to_owned(),
                "arrow" => String::new(),
                t => token(t),
            };
            let got = el.get(&attr).unwrap_or_default();
            if tok == "arrow" {
                if !(got.starts_with(&format!("url(#{ID_HEAD}")) && got.contains(ARROW_PREFIX)) {
                    bad.push(format!("{}: 矢じりが付かない", c.name));
                }
            } else if got != want {
                bad.push(format!("{}: {attr} = {got}（期待 {want}）", c.name));
            }
        }
        // ③ 色を持つ class は、style にダークの規則を持つ
        let has_colour = classes::values_of(&names)
            .iter()
            .any(|(a, t)| (a == "fill" || a == "stroke") && t != "none");
        let selector: String = classes::ordered(&names)
            .iter()
            .map(|k| format!(".{}", k.name))
            .collect();
        if has_colour && !out.contains(&format!(".{ROOT_CLASS} {selector}{{")) {
            bad.push(format!("{}: ダークの規則が無い", c.name));
        }
        // ④ 座標とテキストは変わらない
        let before = xml::parse(&svg).expect("読める");
        let src = first_drawn(&before);
        for (k, v) in &src.attrs {
            if el.get(k) != Some(v.as_str()) {
                bad.push(format!("{}: {k} が変わった", c.name));
            }
        }
        if src.text() != el.text() {
            bad.push(format!("{}: テキストが変わった", c.name));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn resolving_twice_gives_the_same_svg() {
    let mut bad = Vec::new();
    for c in &classes::notation().classes {
        let (_, svg) = sample(c);
        let once = resolve(&svg, None, true).expect("解決できる");
        let twice = resolve(&once, None, true).expect("解決できる");
        if once != twice {
            bad.push(c.name.clone());
        }
    }
    assert!(bad.is_empty(), "冪等でない: {bad:?}");
}

#[test]
fn a_flow_gets_one_arrowhead_per_colour() {
    let svg = r#"<svg viewBox="0 0 100 60"><path class="flow" d="M0,10 L90,10"/><path class="flow" d="M0,20 L90,20"/><path class="flow focus" d="M0,30 L90,30"/></svg>"#;
    let out = resolve(svg, None, true).expect("解決できる");
    assert_eq!(out.matches("<marker ").count(), 2, "{out}");
    assert!(out.contains("orient=\"auto-start-reverse\""));
}

fn colour_override() -> Map<String, Value> {
    let mut t = theme::default_theme().clone();
    t.insert("color.ink".to_owned(), Value::from("#000000"));
    t
}

// ⑥ デシジョンテーブル ── 上書き（有 ・ 無）× ダークの値（有 ・ 無）
// 「ダークの値が無い」は、色を持たない class（swatch）だけの図で作る

#[test]
fn no_override_with_dark_values_emits_the_style() {
    let svg = r#"<svg viewBox="0 0 100 60"><text class="label" x="0" y="20">甲</text></svg>"#;
    let out = resolve(svg, None, true).expect("解決できる");
    assert!(out.contains("<style data-dsvg=\"dark\">"), "{out}");
    assert!(out.contains("--dsvg-ink:#"), "{out}");
    assert!(out.contains(&format!("class=\"{ROOT_CLASS}\"")), "{out}");
}

#[test]
fn no_override_without_dark_values_emits_no_style() {
    let svg = r##"<svg viewBox="0 0 100 60"><rect class="swatch" x="0" y="0" width="10" height="10" fill="#123456"/></svg>"##;
    let out = resolve(svg, None, true).expect("解決できる");
    assert!(!out.contains("<style"), "{out}");
    assert!(out.contains("fill=\"#123456\""), "swatch の色はそのまま");
}

#[test]
fn an_override_with_dark_values_emits_no_style() {
    let svg = r#"<svg viewBox="0 0 100 60"><text class="label" x="0" y="20">甲</text></svg>"#;
    let t = colour_override();
    let out = resolve(svg, Some(&t), false).expect("解決できる");
    assert!(!out.contains("<style"), "{out}");
    assert!(
        !out.contains(ROOT_CLASS),
        "上書きした図は dsvg を持たない: {out}"
    );
    assert!(
        out.contains("fill=\"#000000\""),
        "上書きした値が出る: {out}"
    );
}

#[test]
fn an_override_without_dark_values_emits_no_style() {
    let svg = r##"<svg viewBox="0 0 100 60"><rect class="swatch" x="0" y="0" width="10" height="10" fill="#123456"/></svg>"##;
    let t = colour_override();
    let out = resolve(svg, Some(&t), false).expect("解決できる");
    assert!(!out.contains("<style"), "{out}");
}

#[test]
fn an_unreadable_svg_is_refused() {
    assert!(resolve("<svg><rect></svg>", None, true).is_err());
    assert!(resolve("<g/>", None, true).is_err());
}

#[test]
fn an_ellipse_keeps_its_own_radii() {
    // 楕円の rx は形そのもので、角丸の半径ではない
    let svg =
        r#"<svg viewBox="0 0 100 60"><ellipse class="box" cx="50" cy="30" rx="40" ry="20"/></svg>"#;
    let out = resolve(svg, None, true).expect("解決できる");
    assert!(out.contains("rx=\"40\""), "{out}");
}

/// SVG の中の id を全部集める。
fn ids(svg: &str) -> Vec<String> {
    let re = regex::Regex::new(r#" id="([^"]+)""#).expect("式");
    re.captures_iter(svg).map(|c| c[1].to_owned()).collect()
}

#[test]
fn two_resolved_figures_on_one_page_share_no_id() {
    // **1つのページに並べても id が衝突しない** ── 衝突すると、隠れた節の中の marker を別の図が参照し、矢じりが消える
    let a = r#"<svg viewBox="0 0 100 60"><path class="flow" d="M0,10 L90,10"/><rect class="swatch" id="g" x="0" y="20" width="10" height="10" fill="url(#g)"/></svg>"#;
    let b = r#"<svg viewBox="0 0 100 60"><path class="flow" d="M0,30 L90,30"/><rect class="swatch" id="g" x="0" y="40" width="10" height="10" fill="url(#g)"/></svg>"#;
    let (ra, rb) = (
        resolve(a, None, true).expect("解決できる"),
        resolve(b, None, true).expect("解決できる"),
    );
    let page = format!("<html><body>{ra}{rb}</body></html>");
    let all = ids(&page);
    let mut uniq = all.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(all.len(), uniq.len(), "id が重なる: {all:?}");
    assert_eq!(all.len(), 4, "矢じり2つと、作成者の id 2つ: {all:?}");
}

#[test]
fn author_ids_get_the_prefix_and_references_follow() {
    let svg = r##"<svg viewBox="0 0 100 60"><defs><linearGradient id="g"><stop class="swatch" offset="0" stop-color="#123456"/></linearGradient></defs><rect class="swatch" x="0" y="0" width="10" height="10" fill="url(#g)"/></svg>"##;
    let out = resolve(svg, None, true).expect("解決できる");
    let id = ids(&out)
        .into_iter()
        .find(|i| i.ends_with("-g"))
        .expect("付け直した id");
    assert!(id.starts_with(ID_HEAD), "{id}");
    assert!(
        out.contains(&format!("url(#{id})")),
        "参照が付け直した id を指す: {out}"
    );
    assert_eq!(
        out,
        resolve(&out, None, true).expect("解決できる"),
        "2回解決しても同じ"
    );
}

#[test]
fn the_style_also_holds_the_text_size_against_page_css() {
    // 埋め込み先のページに .note{font-size:…} が在っても、注記の大きさが変わらない
    let svg = r#"<svg viewBox="0 0 100 60"><text class="note" x="0" y="20">注記</text></svg>"#;
    let out = resolve(svg, None, true).expect("解決できる");
    assert!(
        out.contains(".dsvg .note{fill:var(--dsvg-ink-soft,"),
        "{out}"
    );
    assert!(out.contains(";font-size:11px}"), "{out}");
}
