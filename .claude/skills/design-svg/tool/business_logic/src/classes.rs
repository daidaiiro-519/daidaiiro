// SPDX-License-Identifier: MIT
//! 図の表記法 ── **class の一覧と、その値の引き方。**
//!
//! 正本は `references/theme.json` の `classes` ・ `forbidden` ・ `scale` ・ `dark` ・ `marker` ・
//! `checks` である。ここは読むだけで、class の名前も値も1つも持たない ── 2か所に書くと、
//! 片方だけ直したときに誰も気づけない。
//!
//! class は4種類に分かれる ── 図形（`shape`）・ 線（`line`）・ テキスト（`text`）と、それらに
//! 重ねる修飾（`modifier`）。1つの要素は、図形 ・ 線 ・ テキストの class を1つだけ持ち、修飾を
//! 0個以上持つ。値は「要素の種類ごと」に持つ ── 同じ `focus` でも、図形なら塗りと枠線、テキスト
//! なら文字の色を変える。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde_json::{Map, Value};

use crate::theme;

/// class の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// 図形（矩形 ・ 円 ・ 多角形 ・ 塗りのある path）
    Shape,
    /// 線（line ・ polyline ・ 塗りの無い path）
    Line,
    /// テキスト
    Text,
    /// 図形 ・ 線 ・ テキストに重ねる修飾
    Modifier,
}

impl Kind {
    /// theme.json の書き方から読む。
    fn of(s: &str) -> Option<Self> {
        match s {
            "shape" => Some(Self::Shape),
            "line" => Some(Self::Line),
            "text" => Some(Self::Text),
            "modifier" => Some(Self::Modifier),
            _ => None,
        }
    }

    /// 値の欄の名前。**修飾は欄を持たない**（修飾の値は、重ねる先の種類の欄に在る）。
    #[must_use]
    pub const fn slot(self) -> &'static str {
        match self {
            Self::Shape => "shape",
            Self::Line => "line",
            Self::Text => "text",
            Self::Modifier => "",
        }
    }
}

/// class 1つ。
#[derive(Debug, Clone)]
pub struct Class {
    /// 名前。
    pub name: String,
    /// 意味。**1つだけ。**
    pub meaning: String,
    /// 種類。
    pub kind: Kind,
    /// 修飾の class を受けるか。**色をコンテンツとして持つもの（swatch ・ series）は受けない。**
    pub modifiers: bool,
    /// 要素の種類の欄から、`(属性, トークンの名前か none ・ arrow)` の並びへ。**書いた順に持つ。**
    pub values: BTreeMap<String, Vec<(String, String)>>,
}

/// 禁止した組み合わせ1つ。`(class, class, 理由)`
pub type Forbidden = (String, String, String);

/// 段階 ── class の値が取ってよい集合。
#[derive(Debug, Clone, Default)]
pub struct Scale {
    /// フォントサイズ。
    pub font_size: Vec<f64>,
    /// ストローク幅。
    pub stroke_width: Vec<f64>,
    /// 角丸の半径。
    pub radius: Vec<f64>,
    /// 破線のパターン。
    pub dash: Vec<String>,
}

/// 矢じりの形。
#[derive(Debug, Clone, Default)]
pub struct Marker {
    /// 形の path。
    pub d: String,
    /// marker の viewBox。
    pub view_box: String,
    /// 線の終点に合わせる点。
    pub ref_x: f64,
    /// 同上。
    pub ref_y: f64,
    /// 線の太さを1とした大きさ。
    pub width: f64,
    /// 同上。
    pub height: f64,
}

/// 検査4が使う値。
#[derive(Debug, Clone, Copy, Default)]
pub struct Checks {
    /// スマホ幅（px）。
    pub phone_width: f64,
    /// 描画したときの最小のフォントサイズ（px）。**これを下回れば検出する。**
    pub min_font_px: f64,
    /// 比較の精度。
    pub precision: f64,
}

/// 正本を読んだもの。
#[derive(Debug, Default)]
pub struct Notation {
    /// class の一覧。**theme.json の順に持つ** ── 修飾を重ねる順でもある。
    pub classes: Vec<Class>,
    /// 禁止した組み合わせ。
    pub forbidden: Vec<Forbidden>,
    /// 段階。
    pub scale: Scale,
    /// 色のトークンから、ダークモードの値へ。
    pub dark: Map<String, Value>,
    /// 矢じり。
    pub marker: Marker,
    /// 検査のしきい値。
    pub checks: Checks,
}

fn nums(v: Option<&Value>) -> Vec<f64> {
    v.and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_default()
}

fn strs(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn num(m: Option<&Value>, key: &str) -> f64 {
    m.and_then(|m| m.get(key))
        .and_then(Value::as_f64)
        .unwrap_or_default()
}

fn load() -> Notation {
    let src = &theme::source().notation;
    let classes = src
        .get("classes")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .filter_map(|(name, c)| {
                    let kind = Kind::of(c.get("kind")?.as_str()?)?;
                    let values = c
                        .get("values")
                        .and_then(Value::as_object)
                        .map(|vs| {
                            vs.iter()
                                .map(|(slot, attrs)| {
                                    let pairs = attrs
                                        .as_object()
                                        .map(|a| {
                                            a.iter()
                                                .filter_map(|(k, v)| {
                                                    Some((k.clone(), v.as_str()?.to_owned()))
                                                })
                                                .collect()
                                        })
                                        .unwrap_or_default();
                                    (slot.clone(), pairs)
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    Some(Class {
                        name: name.clone(),
                        meaning: c
                            .get("meaning")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                        kind,
                        modifiers: c.get("modifiers").and_then(Value::as_bool).unwrap_or(true),
                        values,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let forbidden = src
        .get("forbidden")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|f| {
                    let pair = strs(f.get("classes"));
                    let [x, y] = &pair[..] else { return None };
                    Some((
                        x.clone(),
                        y.clone(),
                        f.get("why")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    let scale_src = src.get("scale");
    let scale = Scale {
        font_size: nums(scale_src.and_then(|s| s.get("font-size"))),
        stroke_width: nums(scale_src.and_then(|s| s.get("stroke-width"))),
        radius: nums(scale_src.and_then(|s| s.get("radius"))),
        dash: strs(scale_src.and_then(|s| s.get("dash"))),
    };
    let m = src.get("marker");
    let marker = Marker {
        d: m.and_then(|m| m.get("d"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        view_box: m
            .and_then(|m| m.get("view-box"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        ref_x: num(m, "ref-x"),
        ref_y: num(m, "ref-y"),
        width: num(m, "width"),
        height: num(m, "height"),
    };
    let c = src.get("checks");
    let checks = Checks {
        phone_width: num(c, "phone-width"),
        min_font_px: num(c, "min-font-px"),
        precision: num(c, "precision"),
    };
    Notation {
        classes,
        forbidden,
        scale,
        dark: src
            .get("dark")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default(),
        marker,
        checks,
    }
}

/// 正本。**1度だけ読む。**
pub fn notation() -> &'static Notation {
    static ONCE: OnceLock<Notation> = OnceLock::new();
    ONCE.get_or_init(load)
}

/// 名前から class を引く。
#[must_use]
pub fn get(name: &str) -> Option<&'static Class> {
    notation().classes.iter().find(|c| c.name == name)
}

/// 名前が一覧に在るか。
#[must_use]
pub fn known(name: &str) -> bool {
    get(name).is_some()
}

/// class 属性の値を、一覧の順に並べ直した名前の並びにする。**一覧に無い名前は除く。**
#[must_use]
pub fn ordered(names: &[&str]) -> Vec<&'static Class> {
    notation()
        .classes
        .iter()
        .filter(|c| names.contains(&c.name.as_str()))
        .collect()
}

/// 2つの class を1つの要素に同時に指定してよいか。**よくなければ理由を返す。**
///
/// 禁止は4つの規則から決まる ── 表に在る組 ・ 図形 ・ 線 ・ テキストの class を2つ持つ組 ・
/// 修飾を受けない class と修飾の組 ・ 重ねる先の種類の値を持たない修飾の組。
#[must_use]
pub fn conflict(a: &str, b: &str) -> Option<String> {
    let (x, y) = (get(a)?, get(b)?);
    if a == b {
        return None;
    }
    if let Some((.., why)) = notation()
        .forbidden
        .iter()
        .find(|(p, q, _)| (p == a && q == b) || (p == b && q == a))
    {
        return Some(why.clone());
    }
    match (x.kind, y.kind) {
        (Kind::Modifier, Kind::Modifier) => None,
        (Kind::Modifier, _) => modifier_on(x, y),
        (_, Kind::Modifier) => modifier_on(y, x),
        _ => Some(format!(
            "図形 ・ 線 ・ テキストの class は1つの要素に1つだけ（{a} と {b}）"
        )),
    }
}

fn modifier_on(modifier: &Class, base: &Class) -> Option<String> {
    if !base.modifiers {
        return Some(format!(
            "{} は色をコンテンツとして持つので、修飾（{}）を受けない",
            base.name, modifier.name
        ));
    }
    if !modifier.values.contains_key(base.kind.slot()) {
        return Some(format!(
            "{} は {} の種類（{}）に値を持たない",
            modifier.name,
            base.name,
            base.kind.slot()
        ));
    }
    None
}

/// 要素の class の並びから、属性の値（トークンの名前か none ・ arrow）を決める。
/// **図形 ・ 線 ・ テキストの class が先、修飾が後**（一覧の順）に重ねる。
#[must_use]
pub fn values_of(names: &[&str]) -> Vec<(String, String)> {
    let list = ordered(names);
    let Some(base) = list.iter().find(|c| c.kind != Kind::Modifier) else {
        return Vec::new();
    };
    let slot = base.kind.slot();
    let mut out: Vec<(String, String)> = Vec::new();
    for c in &list {
        for (k, v) in c.values.get(slot).map_or(&[][..], Vec::as_slice) {
            match out.iter_mut().find(|(a, _)| a == k) {
                Some(slot) => slot.1.clone_from(v),
                None => out.push((k.clone(), v.clone())),
            }
        }
    }
    out
}

/// 色のトークンのうち、注記（`note`）として描くもの。
const SOFT_INKS: [&str; 2] = ["color.ink-soft", "color.ink-faint"];
/// 系列の class の接頭辞。
const SERIES: &str = "series-";

/// 部品が描くテキストの class ── **色のトークンと、小さい文字かどうかから決める。**
/// 部品は値ではなく意味で色を選んでいるので、その意味をそのまま class にする。
#[must_use]
pub fn text_class(color: &str, small: bool) -> String {
    let mut out = if SOFT_INKS.contains(&color) {
        "note".to_owned()
    } else {
        "label".to_owned()
    };
    if small {
        out.push_str(" small");
    }
    if let Some(m) = modifier_of_colour(color) {
        out.push(' ');
        out.push_str(m);
    }
    out
}

/// 色のトークンが表す修飾。**強調 ・ 問題 ・ カテゴリだけ。**
fn modifier_of_colour(color: &str) -> Option<&'static str> {
    notation()
        .classes
        .iter()
        .filter(|c| c.kind == Kind::Modifier)
        .find(|c| {
            c.values
                .get(Kind::Text.slot())
                .is_some_and(|v| v.iter().any(|(a, t)| a == "fill" && t == color))
        })
        .map(|c| c.name.as_str())
}

/// 系列の数。**一覧に在る `series-*` の数である。**
#[must_use]
pub fn series_count() -> usize {
    notation()
        .classes
        .iter()
        .filter(|c| c.name.starts_with(SERIES))
        .count()
        .max(1)
}

/// i 番目（0から）の系列の class。**系列の数を超えたら先頭から繰り返す。**
#[must_use]
pub fn series(i: usize) -> String {
    format!("{SERIES}{}", i % series_count() + 1)
}

/// 節点の役割（role）から、箱とテキストの class を決める。`(箱, テキスト)`
///
/// 役割の名前が修飾の class と同じなら、その修飾を重ねる。`muted`（控えめ）は外部 ・ 境界として描く。
#[must_use]
pub fn role_classes(role: &str) -> (String, String) {
    if role == "muted" {
        return ("boundary".to_owned(), "label".to_owned());
    }
    match get(role).filter(|c| c.kind == Kind::Modifier) {
        Some(m) => (format!("box {}", m.name), format!("label {}", m.name)),
        None => ("box".to_owned(), "label".to_owned()),
    }
}
