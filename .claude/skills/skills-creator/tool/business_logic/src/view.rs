// SPDX-License-Identifier: MIT
//! 見た目の複製の検査（ボード view-design-tokens）。**見つけるが、直さない。**
//!
//! 見た目の正本は skills-creator の `references/view/` に1組だけ在り、scaffold が各 Skill の
//! `references/` へ複製する。ここは、複製が正本と一致するか、規則に色の直値が無いか、規則が参照する
//! 変数がすべて定まっているか、文字と地の色の比が足りるかを検査する。**Skill 固有の2つ
//! （`view.skill.tokens.json` ・ `view.skill.css`）は正本と比べない。**

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use crate::data_access::files;

/// 正本と一致させる複製。
pub const COPIES: [&str; 4] = [
    "view.tokens.json",
    "view.tokens.schema.json",
    "view.css",
    "view.template.html",
];

/// Skill 固有のトークン。
const SKILL_TOKENS: &str = "view.skill.tokens.json";
/// Skill 固有の規則。
const SKILL_CSS: &str = "view.skill.css";

/// 文字と地の色の比の下限（WCAG 2.x の本文の AA）。
const MIN_CONTRAST: f64 = 4.5;

/// 比を測る組（文字の役割、地の役割）。
const PAIRS: [(&str, &str); 8] = [
    ("ink", "paper"),
    ("ink", "surface"),
    ("muted", "paper"),
    ("muted", "surface"),
    ("accent", "paper"),
    ("accent", "accent-soft"),
    ("paper", "accent"),
    ("warn", "warn-soft"),
];

fn read_json(path: &Path) -> Option<Value> {
    files::read_to_string(path)
        .ok()
        .and_then(|b| serde_json::from_str(&b).ok())
}

/// `#rrggbb` の相対輝度。
fn luminance(hex: &str) -> Option<f64> {
    let h = hex.strip_prefix('#')?;
    if h.len() != 6 {
        return None;
    }
    let ch = |i: usize| -> Option<f64> {
        let c = f64::from(u8::from_str_radix(h.get(i..i + 2)?, 16).ok()?) / 255.0;
        Some(if c <= 0.039_28 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        })
    };
    Some(0.0722_f64.mul_add(ch(4)?, 0.2126_f64.mul_add(ch(0)?, 0.7152 * ch(2)?)))
}

/// 2つの色の比（WCAG 2.x の式）。
#[must_use]
pub fn contrast(a: &str, b: &str) -> Option<f64> {
    let (x, y) = (luminance(a)?, luminance(b)?);
    let (hi, lo) = if x >= y { (x, y) } else { (y, x) };
    Some((hi + 0.05) / (lo + 0.05))
}

/// 規則の中の色の直値（`#…` ・ `rgb(` ・ `hsl(`）を取り出す。
fn literals(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    let chars: Vec<char> = css.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if *c == '#' {
            let hex: String = chars[i + 1..]
                .iter()
                .take_while(|d| d.is_ascii_hexdigit())
                .collect();
            let next = chars.get(i + 1 + hex.len());
            let ends = next.is_none_or(|d| !d.is_ascii_alphanumeric() && *d != '-' && *d != '_');
            if matches!(hex.len(), 3 | 4 | 6 | 8) && ends {
                out.push(format!("#{hex}"));
            }
        }
    }
    let lower = css.to_ascii_lowercase();
    for f in ["rgb(", "rgba(", "hsl(", "hsla("] {
        if lower.contains(f) {
            out.push(f.trim_end_matches('(').to_owned());
        }
    }
    out
}

/// 規則が参照する変数（`var(--名前)`）の名前を取り出す。
fn used_vars(css: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = css;
    while let Some(i) = rest.find("var(--") {
        rest = &rest[i + 6..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        out.insert(name);
    }
    out
}

/// 注記（`/* … */`）を外す。**注記の中の例（var(--役割) など）を、規則として数えない。**
fn without_comments(css: &str) -> String {
    let mut out = String::new();
    let mut rest = css;
    while let Some(i) = rest.find("/*") {
        out.push_str(&rest[..i]);
        rest = rest[i + 2..]
            .find("*/")
            .map_or("", |j| &rest[i + 2 + j + 2..]);
    }
    out.push_str(rest);
    out
}

/// 型の中の `<style>` の中身。
fn template_style(html: &str) -> String {
    html.split("<style>")
        .skip(1)
        .filter_map(|s| s.split("</style>").next())
        .collect::<Vec<_>>()
        .join("")
}

/// 複製が正本と一致するか。**トークンの palette（どのパレットを使うか）は Skill の選択なので比べない。**
fn copies(refs: &Path, canon: &Path, out: &mut Vec<String>) {
    for f in COPIES {
        let mine = refs.join(f);
        if !files::is_file(&mine) {
            out.push(format!(
                "references/{f} が無い ── 見た目の正本の複製である。skills-creator の references/view/{f} を複製する"
            ));
            continue;
        }
        let same = if f == "view.tokens.json" {
            let strip = |v: Option<Value>| {
                v.map(|mut v| {
                    if let Some(m) = v.as_object_mut() {
                        m.remove("palette");
                    }
                    v
                })
            };
            strip(read_json(&mine)) == strip(read_json(&canon.join(f)))
        } else {
            files::read_to_string(&mine).ok() == files::read_to_string(canon.join(f)).ok()
        };
        if !same {
            out.push(format!(
                "references/{f} が正本と違う ── skills-creator の references/view/{f} に合わせる（Skill 固有の見た目は view.skill.tokens.json ・ view.skill.css に置く）"
            ));
        }
    }
}

/// 使うパレットの役割の名前と、Skill 固有のトークンの名前。
fn defined(tokens: &Value, skill: Option<&Value>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let name = tokens
        .get("palette")
        .and_then(Value::as_str)
        .unwrap_or_default();
    for source in [tokens.get("palettes").and_then(|p| p.get(name)), skill]
        .into_iter()
        .flatten()
    {
        for scheme in ["light", "dark"] {
            for (k, _) in source
                .get(scheme)
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
            {
                out.insert(k.clone());
            }
        }
    }
    out
}

/// パレットごとの文字と地の比。**使っていないパレットも測る** ── Skill が選び直したときに比が不足する。
fn contrasts(tokens: &Value, out: &mut Vec<String>) {
    for (name, palette) in tokens
        .get("palettes")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        for scheme in ["light", "dark"] {
            let Some(colors) = palette.get(scheme) else {
                continue;
            };
            for (fg, bg) in PAIRS {
                let (Some(a), Some(b)) = (
                    colors.get(fg).and_then(Value::as_str),
                    colors.get(bg).and_then(Value::as_str),
                ) else {
                    continue;
                };
                let Some(r) = contrast(a, b) else { continue };
                if r < MIN_CONTRAST {
                    out.push(format!(
                        "view.tokens.json: {name} の{}で、{fg} と {bg} の比が {r:.2} ── {MIN_CONTRAST} 以上にする",
                        if scheme == "light" { "明" } else { "暗" }
                    ));
                }
            }
        }
    }
}

/// Skill の見た目の複製を検査する。`canon` は正本の置き場所（skills-creator の `references/view/`）。
#[must_use]
pub fn findings(root: &Path, canon: &Path) -> Vec<String> {
    let refs = root.join("references");
    let mut out = Vec::new();
    copies(&refs, canon, &mut out);
    let Some(tokens) = read_json(&refs.join("view.tokens.json")) else {
        return out;
    };
    let name = tokens
        .get("palette")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if tokens.get("palettes").and_then(|p| p.get(name)).is_none() {
        out.push(format!(
            "view.tokens.json: palette の {name} が palettes に無い"
        ));
    }
    let skill = read_json(&refs.join(SKILL_TOKENS));
    let known = defined(&tokens, skill.as_ref());
    let sources = [
        (
            "view.css",
            files::read_to_string(refs.join("view.css")).unwrap_or_default(),
        ),
        (
            SKILL_CSS,
            files::read_to_string(refs.join(SKILL_CSS)).unwrap_or_default(),
        ),
        (
            "view.template.html",
            template_style(
                &files::read_to_string(refs.join("view.template.html")).unwrap_or_default(),
            ),
        ),
    ];
    for (file, css) in &sources {
        let css = without_comments(css);
        let css = css.as_str();
        for lit in literals(css) {
            out.push(format!(
                "{file}: 色の直値 {lit} が在る ── 色は var(--役割) で参照し、値はトークンに置く"
            ));
        }
        for v in used_vars(css) {
            if !known.contains(&v) {
                out.push(format!(
                    "{file}: 変数 --{v} がトークンに無い ── 共通の役割か view.skill.tokens.json で定める"
                ));
            }
        }
    }
    contrasts(&tokens, &mut out);
    out
}
