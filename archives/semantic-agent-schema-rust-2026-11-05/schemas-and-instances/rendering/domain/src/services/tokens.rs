//! デザイントークン（Design Tokens Format Module 2025.10 と Color module 2025.10）。
//!
//! - 種類（$type）は、トークンに無ければ近い親の group から受け継ぐ。どこにも無ければ、そのトークンは正しくない
//! - 値が種類の形に合うかを見る（color ・ dimension ・ fontFamily ・ fontWeight ・ duration ・ cubicBezier ・ number。
//!   合成の種類 strokeStyle ・ border ・ transition ・ shadow ・ gradient ・ typography は形の大枠だけ）
//! - 別名（{group.token}）は、先のトークンが在り、輪になっていないこと
//! - CSS のカスタムプロパティにする（合成の種類は CSS にせず、テンプレートが tokens から値として使う）

use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// トークン1つ（名前は group.token）。
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// 種類（受け継いだものを含む）。決まらなければ None
    pub kind: Option<String>,
    /// 値（$value）
    pub value: Value,
}

/// トークンを、名前（group.token）→ トークンの並びにする。
#[must_use]
pub fn flatten(tokens: Option<&Value>) -> BTreeMap<String, Token> {
    fn walk(
        prefix: &str,
        inherited: Option<&str>,
        v: &Map<String, Value>,
        out: &mut BTreeMap<String, Token>,
    ) {
        let here = v.get("$type").and_then(Value::as_str).or(inherited);
        for (k, child) in v {
            if k.starts_with('$') {
                continue;
            }
            let name = if prefix.is_empty() {
                k.clone()
            } else {
                format!("{prefix}.{k}")
            };
            if let Some(obj) = child.as_object() {
                if let Some(value) = obj.get("$value") {
                    let kind = obj
                        .get("$type")
                        .and_then(Value::as_str)
                        .or(here)
                        .map(ToString::to_string);
                    out.insert(
                        name,
                        Token {
                            kind,
                            value: value.clone(),
                        },
                    );
                } else {
                    walk(&name, here, obj, out);
                }
            }
        }
    }
    let mut out = BTreeMap::new();
    if let Some(m) = tokens.and_then(Value::as_object) {
        walk("", None, m, &mut out);
    }
    out
}

/// 別名（{group.token}）の先の名前。別名でなければ None。
#[must_use]
pub fn alias_of(v: &Value) -> Option<&str> {
    v.as_str()
        .and_then(|s| s.strip_prefix('{'))
        .and_then(|s| s.strip_suffix('}'))
}

/// 解けない別名と、輪になった別名（トークンの名前と理由）。
#[must_use]
pub fn alias_errors(tokens: &BTreeMap<String, Token>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (name, t) in tokens {
        let mut seen = BTreeSet::from([name.clone()]);
        let mut cur = &t.value;
        while let Some(target) = alias_of(cur) {
            match tokens.get(target) {
                None => {
                    out.push((
                        name.clone(),
                        format!("別名 {{{target}}} の先のトークンが無い"),
                    ));
                    break;
                }
                Some(_) if !seen.insert(target.to_string()) => {
                    out.push((name.clone(), format!("別名が輪になっている（{target}）")));
                    break;
                }
                Some(next) => cur = &next.value,
            }
        }
    }
    out
}

/// Color module 2025.10 の色空間。
const COLOR_SPACES: [&str; 14] = [
    "srgb",
    "srgb-linear",
    "hsl",
    "hwb",
    "lab",
    "lch",
    "oklab",
    "oklch",
    "display-p3",
    "a98-rgb",
    "prophoto-rgb",
    "rec2020",
    "xyz-d65",
    "xyz-d50",
];

/// CSS で関数の名前で書く色空間（ほかは color() のキーワード）。
const NAMED_FUNCTIONS: [&str; 6] = ["hsl", "hwb", "lab", "lch", "oklab", "oklch"];

const FONT_WEIGHTS: [&str; 18] = [
    "thin",
    "hairline",
    "extra-light",
    "ultra-light",
    "light",
    "normal",
    "regular",
    "book",
    "medium",
    "semi-bold",
    "demi-bold",
    "bold",
    "extra-bold",
    "ultra-bold",
    "black",
    "heavy",
    "extra-black",
    "ultra-black",
];

const COMPOSITES: [&str; 6] = [
    "strokeStyle",
    "border",
    "transition",
    "shadow",
    "gradient",
    "typography",
];

fn number(v: &Value) -> Option<f64> {
    v.as_f64()
}

fn unit_value(v: &Value, units: &[&str]) -> Result<(), String> {
    let o = v.as_object().ok_or("値は {value, unit} のオブジェクト")?;
    number(o.get("value").unwrap_or(&Value::Null)).ok_or("value は数")?;
    let u = o.get("unit").and_then(Value::as_str).unwrap_or_default();
    if !units.contains(&u) {
        return Err(format!("unit は {} のどれか", units.join(" ・ ")));
    }
    Ok(())
}

/// 値が種類の形に合うか（別名は別に確かめるので見ない）。
fn value_error(kind: &str, v: &Value) -> Option<String> {
    if alias_of(v).is_some() {
        return None;
    }
    let r: Result<(), String> = match kind {
        "color" => (|| {
            let o = v
                .as_object()
                .ok_or("値は {colorSpace, components} のオブジェクト（Color module）")?;
            let space = o
                .get("colorSpace")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !COLOR_SPACES.contains(&space) {
                return Err(format!(
                    "colorSpace は {} のどれか",
                    COLOR_SPACES.join(" ・ ")
                ));
            }
            let c = o
                .get("components")
                .and_then(Value::as_array)
                .ok_or("components は並び")?;
            if c.len() != 3
                || !c
                    .iter()
                    .all(|x| x.is_number() || x.as_str() == Some("none"))
            {
                return Err("components は数か \"none\" の3つ".into());
            }
            if let Some(a) = o.get("alpha")
                && !number(a).is_some_and(|a| (0.0..=1.0).contains(&a))
            {
                return Err("alpha は 0 から 1 の数".into());
            }
            if let Some(h) = o.get("hex") {
                let ok = h.as_str().is_some_and(|h| {
                    h.len() == 7
                        && h.starts_with('#')
                        && h[1..].chars().all(|c| c.is_ascii_hexdigit())
                });
                if !ok {
                    return Err("hex は 6桁の #rrggbb".into());
                }
            }
            Ok(())
        })(),
        "dimension" => unit_value(v, &["px", "rem"]),
        "duration" => unit_value(v, &["ms", "s"]),
        "fontFamily" => {
            if v.is_string()
                || v.as_array()
                    .is_some_and(|a| !a.is_empty() && a.iter().all(Value::is_string))
            {
                Ok(())
            } else {
                Err("文字列か、文字列の並び".into())
            }
        }
        "fontWeight" => {
            if number(v).is_some_and(|n| (1.0..=1000.0).contains(&n))
                || v.as_str().is_some_and(|s| FONT_WEIGHTS.contains(&s))
            {
                Ok(())
            } else {
                Err("1 から 1000 の数か、決まった名前（bold など）".into())
            }
        }
        "cubicBezier" => {
            let ok = v.as_array().is_some_and(|a| {
                a.len() == 4
                    && a.iter().all(Value::is_number)
                    && [0, 2]
                        .iter()
                        .all(|i| number(&a[*i]).is_some_and(|x| (0.0..=1.0).contains(&x)))
            });
            if ok {
                Ok(())
            } else {
                Err("4つの数 [P1x, P1y, P2x, P2y]。x は 0 から 1".into())
            }
        }
        "number" => {
            if v.is_number() {
                Ok(())
            } else {
                Err("数".into())
            }
        }
        k if COMPOSITES.contains(&k) => {
            if v.is_object()
                || (k == "gradient" || k == "shadow" || k == "strokeStyle")
                    && (v.is_array() || v.is_string())
            {
                Ok(())
            } else {
                Err("合成の種類の値はオブジェクト（gradient ・ shadow は並びも）".into())
            }
        }
        other => Err(format!(
            "{other} は Design Tokens Format Module 2025.10 の種類に無い"
        )),
    };
    r.err()
}

/// 種類が決まらない ・ 値が種類の形に合わないトークン（名前と理由）。
#[must_use]
pub fn type_errors(tokens: &BTreeMap<String, Token>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (name, t) in tokens {
        // 別名は、先のトークンの種類を受け継ぐ
        let kind = t.kind.clone().or_else(|| {
            let mut cur = &t.value;
            let mut seen = BTreeSet::new();
            while let Some(target) = alias_of(cur) {
                if !seen.insert(target.to_string()) {
                    break;
                }
                let next = tokens.get(target)?;
                if next.kind.is_some() {
                    return next.kind.clone();
                }
                cur = &next.value;
            }
            None
        });
        match kind {
            None => out.push((
                name.clone(),
                "種類（$type）が決まらない。トークンか親の group に $type を書く".to_string(),
            )),
            Some(k) => {
                if let Some(why) = value_error(&k, &t.value) {
                    out.push((name.clone(), format!("{k}：{why}")));
                }
            }
        }
    }
    out
}

/// トークンの名前を CSS のカスタムプロパティの名前に。
#[must_use]
pub fn var_name(token: &str) -> String {
    format!("--{}", token.replace('.', "-"))
}

fn num_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// 1つのトークンの CSS の値。合成の種類と、形に合わない値は None（CSS にしない）。
fn css_value(t: &Token, tokens: &BTreeMap<String, Token>) -> Option<String> {
    if let Some(target) = alias_of(&t.value) {
        return tokens
            .contains_key(target)
            .then(|| format!("var({})", var_name(target)));
    }
    let kind = t.kind.as_deref()?;
    if value_error(kind, &t.value).is_some() {
        return None;
    }
    let v = &t.value;
    match kind {
        "color" => {
            let space = v["colorSpace"].as_str()?;
            let comps: Vec<String> = v["components"].as_array()?.iter().map(num_text).collect();
            let alpha = v
                .get("alpha")
                .filter(|a| a.as_f64() != Some(1.0))
                .map(|a| format!(" / {a}"))
                .unwrap_or_default();
            Some(if NAMED_FUNCTIONS.contains(&space) {
                format!("{space}({}{alpha})", comps.join(" "))
            } else {
                format!("color({space} {}{alpha})", comps.join(" "))
            })
        }
        "dimension" | "duration" => {
            Some(format!("{}{}", num_text(&v["value"]), v["unit"].as_str()?))
        }
        "fontFamily" => {
            let generic = [
                "serif",
                "sans-serif",
                "monospace",
                "cursive",
                "fantasy",
                "system-ui",
                "ui-serif",
                "ui-sans-serif",
                "ui-monospace",
                "ui-rounded",
                "math",
                "emoji",
                "fangsong",
            ];
            let names: Vec<String> = match v {
                Value::String(s) => vec![s.clone()],
                Value::Array(a) => a
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect(),
                _ => return None,
            };
            Some(
                names
                    .iter()
                    .map(|n| {
                        if generic.contains(&n.as_str()) {
                            n.clone()
                        } else {
                            format!("\"{}\"", n.replace('"', "\\\""))
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        }
        "fontWeight" => Some(match v {
            Value::String(s) => match s.as_str() {
                "thin" | "hairline" => "100",
                "extra-light" | "ultra-light" => "200",
                "light" => "300",
                "normal" | "regular" | "book" => "400",
                "medium" => "500",
                "semi-bold" | "demi-bold" => "600",
                "bold" => "700",
                "extra-bold" | "ultra-bold" => "800",
                "black" | "heavy" => "900",
                _ => "950",
            }
            .to_string(),
            other => num_text(other),
        }),
        "cubicBezier" => Some(format!(
            "cubic-bezier({})",
            v.as_array()?
                .iter()
                .map(num_text)
                .collect::<Vec<_>>()
                .join(", ")
        )),
        "number" => Some(num_text(v)),
        _ => None,
    }
}

/// トークンを :root のカスタムプロパティにする（名前の順）。合成の種類は CSS にしない。
#[must_use]
pub fn css_of(tokens: &BTreeMap<String, Token>) -> String {
    let lines: Vec<String> = tokens
        .iter()
        .filter_map(|(name, t)| css_value(t, tokens).map(|v| format!("{}: {v};", var_name(name))))
        .collect();
    format!(":root {{ {} }}", lines.join(" "))
}
