// SPDX-License-Identifier: MIT
//! class の解決 ── **作成者が class で指定したスタイルを、図のデザインシステムの値へ置き換える。**
//!
//! 作成者は座標と class だけを記述する。ここは class を theme.json の値へ解決して属性に出力し、
//! **class は削除せずに残す**。返す SVG は値を持つので、どこに埋め込んでも同じに表示される。
//!
//! 3つのものを足す ── 属性の値 ・ flow の矢じり（`marker`）・ ダークモードの `style`。
//! 呼び出し元がトークンを上書きしたときは、ダークモードの `style` を出さない ── 配色は呼び出し元の
//! テーマが決める（ブレストボード design-svg-rework の論点5）。
//!
//! **2回解決しても結果が同じ** ── 自分が足した `marker` と `style` は、解決の前に外す。

use serde_json::{Map, Value};

use crate::classes::{self, Kind};
use crate::theme;
use crate::xml::{self, Element, Node};

/// 解決した SVG のルートに付ける class。**ダークモードの `style` はこの class の中だけに効く** ──
/// 呼び出し元がトークンを上書きした SVG は持たないので、同じページに並んでも影響を受けない。
pub const ROOT_CLASS: &str = "dsvg";
/// 解決が足した要素の印。**2回目の解決は、これを外してから足し直す。**
pub const GENERATED: &str = "data-dsvg";
/// 図ごとの id の接頭辞の頭。**`dsvg-<要約8桁>-` の形で、すべての id に付ける** ── 解決した図を
/// 1つのページに並べても id が衝突しない。衝突すると、隠れた節の中の marker を別の図が参照し、
/// 矢じりが描かれない（実測 ── テンプレートの下見のページで、2つ目以降の図の矢じりが消えた）。
pub const ID_HEAD: &str = "dsvg-";
/// 矢じりの id の、図ごとの接頭辞のあとに続く部分。
pub const ARROW_PREFIX: &str = "arrow-";
/// 図ごとの要約の桁数。
const FINGERPRINT: usize = 8;
/// 要約に使う属性 ── **解決が変えないもの（座標 ・ class ・ 変換）だけ** ── 解決したあとの SVG から
/// 求めても同じ要約になり、2回解決しても同じ接頭辞になる。
const STABLE_ATTRS: [&str; 18] = [
    "viewBox",
    "class",
    "x",
    "y",
    "width",
    "height",
    "cx",
    "cy",
    "r",
    "x1",
    "y1",
    "x2",
    "y2",
    "d",
    "points",
    "transform",
    "text-anchor",
    "href",
];
/// 矢じりを付けることを表す値。
const ARROW: &str = "arrow";
/// ダークモードの値を持つ属性。**色だけである。**
const COLOR_ATTRS: [&str; 2] = ["fill", "stroke"];
/// 角丸の半径を持てる要素。**楕円の rx は形そのものなので、解決しない。**
const ROUNDED: &str = "rect";

/// ダークモードの規則1つの宣言 ── `(属性, 色のトークン, ライトの値)`
type Colour = (String, String, String);

/// 解決のときに集めたもの。
#[derive(Default)]
struct Gathered {
    /// 矢じり ── `(id, 色の値, 色のトークン)`
    arrows: Vec<(String, String, String)>,
    /// ダークモードの規則 ── `(セレクタ, [(属性, 色のトークン, ライトの値)])`
    rules: Vec<(String, Vec<Colour>)>,
}

/// トークンの名前から値を引く。**値がトークン名を指していたら、1層だけたどる。**
fn token(theme: &Map<String, Value>, name: &str) -> Option<String> {
    let v = theme.get(name)?;
    let v = match v {
        Value::String(s) => theme.get(s).unwrap_or(v),
        other => other,
    };
    Some(match v {
        Value::String(s) => s.clone(),
        other => crate::py::num(other),
    })
}

/// 色の値から、矢じりの id を作る。**値が違えば id も違う** ── 同じページに別の配色の図が並んでも、
/// 矢じりの色が入れ替わらない。
fn arrow_id(head: &str, color: &str) -> String {
    let body: String = color
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    format!("{head}{ARROW_PREFIX}{}", body.trim_matches('-'))
}

/// 図ごとの接頭辞が付いた id から、元の id を取り出す。**付いていなければそのまま。**
fn unprefixed(id: &str) -> &str {
    let Some(rest) = id.strip_prefix(ID_HEAD) else {
        return id;
    };
    match rest.split_once('-') {
        Some((h, tail)) if h.len() == FINGERPRINT && h.chars().all(|c| c.is_ascii_hexdigit()) => {
            tail
        }
        _ => id,
    }
}

/// 要約の材料を、生成した要素を除いて文書の順に集める。id は接頭辞を外して数える。
fn material(el: &Element, out: &mut Vec<String>) {
    if el.get(GENERATED).is_some() {
        return;
    }
    out.push(el.tag.clone());
    for k in STABLE_ATTRS {
        if let Some(v) = el.get(k) {
            // ルートの class は解決が dsvg を足すので、それを除いて数える
            let v: String = if k == "class" {
                v.split_whitespace()
                    .filter(|c| *c != ROOT_CLASS)
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                v.to_owned()
            };
            if !v.is_empty() {
                out.push(format!("{k}={v}"));
            }
        }
    }
    if let Some(id) = el.get("id") {
        out.push(format!("id={}", unprefixed(id)));
    }
    for n in &el.children {
        match n {
            Node::Element(e) => material(e, out),
            Node::Text(t) if !t.trim().is_empty() => out.push(t.trim().to_owned()),
            Node::Text(_) => {}
        }
    }
}

/// 作成者が記述した id に、図ごとの接頭辞を付ける。`(元の id, 付けた id)` を集める。
fn prefix_ids(el: &mut Element, head: &str, renamed: &mut Vec<(String, String)>) {
    if let Some(id) = el.get("id").map(str::to_owned) {
        let new = format!("{head}{}", unprefixed(&id));
        renamed.push((unprefixed(&id).to_owned(), new.clone()));
        el.set("id", &new);
    }
    for ch in el.elements_mut() {
        prefix_ids(ch, head, renamed);
    }
}

/// 参照先の id を、付け直した id へ引く。**前の解決で付けた接頭辞は外して照らす。**
fn renamed_to<'a>(target: &str, renamed: &'a [(String, String)]) -> Option<&'a str> {
    let bare = unprefixed(target);
    renamed
        .iter()
        .find(|(old, _)| old == bare)
        .map(|(_, new)| new.as_str())
}

/// `url(#…)` と `href="#…"` の参照を、付け直した id へ書き換える。
fn rewrite_refs(el: &mut Element, renamed: &[(String, String)]) {
    const OPEN: &str = "url(#";
    for (k, v) in &mut el.attrs {
        if (k == "href" || k == "xlink:href") && v.starts_with('#') {
            if let Some(new) = renamed_to(&v[1..], renamed) {
                *v = format!("#{new}");
            }
            continue;
        }
        let mut out = String::new();
        let mut rest = v.as_str();
        while let Some(at) = rest.find(OPEN) {
            let after = &rest[at + OPEN.len()..];
            let Some(end) = after.find(')') else { break };
            let target = &after[..end];
            out.push_str(&rest[..at + OPEN.len()]);
            out.push_str(renamed_to(target, renamed).unwrap_or(target));
            rest = &after[end..];
        }
        out.push_str(rest);
        *v = out;
    }
    for ch in el.elements_mut() {
        rewrite_refs(ch, renamed);
    }
}

/// 1つの要素を解決する。**一覧に在る class を持たない要素は、そのまま残す。**
fn resolve_element(
    el: &mut Element,
    theme: &Map<String, Value>,
    head: &str,
    got: &mut Gathered,
) -> Result<(), String> {
    let names: Vec<String> = el.classes().iter().map(|s| (*s).to_owned()).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let list = classes::ordered(&refs);
    if list.iter().any(|c| c.kind != Kind::Modifier) {
        let values = classes::values_of(&refs);
        let mut colors = Vec::new();
        let stroke_token = values
            .iter()
            .find(|(k, _)| k == "stroke")
            .map(|(_, v)| v.clone());
        for (attr, v) in &values {
            if attr == "rx" && el.tag != ROUNDED {
                continue;
            }
            if v == "none" {
                el.set(attr, "none");
                continue;
            }
            if v == ARROW {
                let tok = stroke_token.clone().unwrap_or_default();
                let color = token(theme, &tok)
                    .ok_or_else(|| format!("矢じりの色のトークン '{tok}' がテーマに無い"))?;
                let id = arrow_id(head, &color);
                if !got.arrows.iter().any(|(i, ..)| *i == id) {
                    got.arrows.push((id.clone(), color, tok));
                }
                el.set(attr, &format!("url(#{id})"));
                continue;
            }
            let value = token(theme, v).ok_or_else(|| format!("トークン '{v}' がテーマに無い"))?;
            if COLOR_ATTRS.contains(&attr.as_str()) {
                colors.push((attr.clone(), v.clone(), value.clone()));
            }
            el.set(attr, &value);
        }
        let selector: String = list.iter().map(|c| format!(".{}", c.name)).collect();
        if !colors.is_empty() && !got.rules.iter().any(|(s, _)| *s == selector) {
            got.rules.push((selector, colors));
        }
    }
    for ch in el.elements_mut() {
        resolve_element(ch, theme, head, got)?;
    }
    Ok(())
}

/// 矢じりの `defs` を作る。
fn arrows(got: &Gathered) -> Option<Element> {
    if got.arrows.is_empty() {
        return None;
    }
    let m = &classes::notation().marker;
    let num = crate::py::float;
    let children = got
        .arrows
        .iter()
        .map(|(id, color, _)| {
            Node::Element(Element {
                tag: "marker".to_owned(),
                attrs: vec![
                    ("id".to_owned(), id.clone()),
                    ("viewBox".to_owned(), m.view_box.clone()),
                    ("refX".to_owned(), trim(&num(m.ref_x))),
                    ("refY".to_owned(), trim(&num(m.ref_y))),
                    ("markerWidth".to_owned(), trim(&num(m.width))),
                    ("markerHeight".to_owned(), trim(&num(m.height))),
                    ("orient".to_owned(), "auto-start-reverse".to_owned()),
                ],
                children: vec![Node::Element(Element {
                    tag: "path".to_owned(),
                    attrs: vec![
                        ("d".to_owned(), m.d.clone()),
                        ("fill".to_owned(), color.clone()),
                    ],
                    children: Vec::new(),
                })],
            })
        })
        .collect();
    Some(Element {
        tag: "defs".to_owned(),
        attrs: vec![(GENERATED.to_owned(), "marker".to_owned())],
        children,
    })
}

/// 整数として書ける小数は、整数で書く。
fn trim(s: &str) -> String {
    s.strip_suffix(".0").unwrap_or(s).to_owned()
}

/// CSS の変数の名前。**色のトークンの名前から `color.` を外す。**
fn var_name(tok: &str) -> String {
    format!(
        "--{ROOT_CLASS}-{}",
        tok.strip_prefix("color.").unwrap_or(tok)
    )
}

/// ダークモードの `style` を作る。**規則はライトの値を既定にして常に効き、ダークモードのときだけ
/// 変数がダークの値になる** ── 埋め込み先のページの切り替え（`data-theme`）にも従う。
fn dark_style(got: &Gathered) -> Option<Element> {
    let dark = &classes::notation().dark;
    let mut used: Vec<String> = Vec::new();
    let mut css = String::new();
    let decl = |attr: &str, tok: &str, light: &str, used: &mut Vec<String>| -> Option<String> {
        dark.get(tok)?;
        if !used.iter().any(|u| u == tok) {
            used.push(tok.to_owned());
        }
        Some(format!("{attr}:var({},{light})", var_name(tok)))
    };
    for (selector, colors) in &got.rules {
        let body: Vec<String> = colors
            .iter()
            .filter_map(|(a, t, l)| decl(a, t, l, &mut used))
            .collect();
        if !body.is_empty() {
            css.push_str(&format!(".{ROOT_CLASS} {selector}{{{}}}", body.join(";")));
        }
    }
    for (id, color, tok) in &got.arrows {
        if let Some(d) = decl("fill", tok, color, &mut used) {
            css.push_str(&format!(".{ROOT_CLASS} #{id} path{{{d}}}"));
        }
    }
    if used.is_empty() {
        return None;
    }
    let vars: Vec<String> = used
        .iter()
        .filter_map(|t| Some(format!("{}:{}", var_name(t), dark.get(t)?.as_str()?)))
        .collect();
    let vars = vars.join(";");
    css.push_str(&format!(
        "@media (prefers-color-scheme:dark){{:root:not([data-theme=\"light\"]) .{ROOT_CLASS},.{ROOT_CLASS}:root:not([data-theme=\"light\"]){{{vars}}}}}:root[data-theme=\"dark\"] .{ROOT_CLASS}{{{vars}}}"
    ));
    Some(Element {
        tag: "style".to_owned(),
        attrs: vec![(GENERATED.to_owned(), "dark".to_owned())],
        children: vec![Node::Text(css)],
    })
}

/// class を値へ解決した SVG を返す。
///
/// `theme` は既定のテーマに上書きを重ねたもの（`None` なら既定）。`dark` が偽なら、ダークモードの
/// `style` を出さず、ルートに [`ROOT_CLASS`] を付けない。
///
/// # Errors
///
/// SVG として読めないとき ・ ルートが svg でないとき ・ class が指すトークンがテーマに無いときに返す。
pub fn resolve(
    svg: &str,
    theme_: Option<&Map<String, Value>>,
    dark: bool,
) -> Result<String, String> {
    let mut root = xml::parse(svg).ok_or("SVG として読めない")?;
    if root.tag != "svg" {
        return Err(format!("ルートが svg でない: {}", root.tag));
    }
    root.children.retain(|n| match n {
        Node::Element(e) => e.get(GENERATED).is_none(),
        Node::Text(_) => true,
    });
    let theme_ = theme_.unwrap_or_else(|| theme::default_theme());
    let mut parts = Vec::new();
    material(&root, &mut parts);
    let head = crate::ids::stable(ID_HEAD, &parts) + "-";
    let mut renamed = Vec::new();
    prefix_ids(&mut root, &head, &mut renamed);
    rewrite_refs(&mut root, &renamed);
    let mut got = Gathered::default();
    for ch in root.elements_mut() {
        resolve_element(ch, theme_, &head, &mut got)?;
    }
    let mut head = Vec::new();
    if dark {
        if let Some(style) = dark_style(&got) {
            head.push(Node::Element(style));
        }
    }
    if let Some(defs) = arrows(&got) {
        head.push(Node::Element(defs));
    }
    let mut names: Vec<String> = root.classes().iter().map(|s| (*s).to_owned()).collect();
    names.retain(|n| n != ROOT_CLASS);
    if dark {
        names.push(ROOT_CLASS.to_owned());
    }
    if names.is_empty() {
        root.remove("class");
    } else {
        root.set("class", &names.join(" "));
    }
    head.append(&mut root.children);
    root.children = head;
    Ok(xml::write(&root))
}
