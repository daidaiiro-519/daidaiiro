// SPDX-License-Identifier: MIT
//! デザイントークンを読み、CSS のカスタムプロパティへ組む。
//!
//! **トークンの正本は1つである。** この道具が読むのは `references/tokens.json` だけで、
//! 色はここから出る。2か所に書くと、どちらが正しいかを毎回確認することになる。
//!
//! **層を跨いだ参照を、形の層で弾く。** 意味の層は基礎のキーだけを参照し、部品の層は
//! 意味か基礎のキーだけを参照する。散文の規定では破れる。

use crate::data_access::files;

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

/// 明暗の3つの選択子。**1つの表から生成する** ── 手で3か所へ書くと、1つのキーが
/// 片側から脱落しても誰も検出しない。
const LIGHT: &str = ":root";
const DARK: (&str, &str) = (
    "@media (prefers-color-scheme:dark){:root:not([data-theme=\"light\"])",
    ":root[data-theme=\"dark\"]",
);
/// iframe と Shadow の中へも同じ表を流し込む。**Shadow の中に `:root` は無い** ──
/// 寄せないと、そこで定めたキーが1つも解決せず、印が色を失う。
const HOST_LIGHT: &str = ":root,:host";
const HOST_DARK: (&str, &str) = (
    "@media (prefers-color-scheme:dark){:root:not([data-theme=\"light\"]),\
     :host:not([data-theme=\"light\"])",
    ":root[data-theme=\"dark\"],:host[data-theme=\"dark\"]",
);

/// 名前のまま出す層。**色は意味の層を経由するが、寸法 ・ 字寸 ・ 字送り ・ 角丸 ・ 枠は、
/// 役割ではなく層そのものが意味である。**
const SCALES: [&str; 5] = ["space", "font_size", "tracking", "radius", "border"];

/// 文字と地の組み合わせ。**読める比を、機械が確かめる** ── 目で見て薄いと気づくのは、
/// 出したあとである（実測 ── 実際にそうなった）。
const CONTRAST: [(&str, f64); 4] = [("ink", 4.5), ("sub", 4.5), ("muted", 4.5), ("dim", 4.5)];

/// 文字を載せる面。
const SURFACES: [&str; 3] = ["paper", "card", "panel"];

/// 重ねる面どうし。**同じ濃さだと、載っているものが地の一部に見える** ── 実測 ── 表が地と
/// 同じ色で「どこからが表か」が読めなくなった。
const LAYERED: [(&str, &str, f64); 3] = [
    ("card", "sunk", 1.15),
    ("card", "panel", 1.10),
    ("rule", "card", 1.5),
];

/// 直値か。**部品の層は、寸法の式をそのまま持てる** ── 寸法は役割ではなく、層そのものが
/// 意味である。
fn literal(value: &str) -> bool {
    value.starts_with("calc(")
        || value
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit() || c == '.')
}

/// 相対輝度（WCAG 2.1 の定義）。
fn luminance(hex: &str) -> f64 {
    let body = hex.trim_start_matches('#');
    let at = |i: usize| -> f64 {
        let Some(pair) = body.get(i..i + 2) else {
            return 0.0;
        };
        let value = f64::from(u8::from_str_radix(pair, 16).unwrap_or(0)) / 255.0;
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * at(0) + 0.7152 * at(2) + 0.0722 * at(4)
}

/// 2色の比。**1（同じ）から 21（黒と白）まで。**
#[must_use]
pub fn contrast(a: &str, b: &str) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// トークンの正本の場所。
#[must_use]
pub fn path(references: &Path) -> PathBuf {
    references.join("tokens.json")
}

/// 形の契約の場所。
#[must_use]
pub fn schema_path(references: &Path) -> PathBuf {
    references.join("tokens.schema.json")
}

/// 正本を読む。
///
/// # Errors
///
/// 読めないときと、JSON として読めないときに返す。
pub fn load(path: &Path) -> Result<Value, String> {
    let body =
        files::read_to_string(path).map_err(|e| format!("{}: 読めない ── {e}", path.display()))?;
    serde_json::from_str(&body)
        .map_err(|e| format!("{}: JSON として読めない ── {e}", path.display()))
}

fn table<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Map<String, Value>> {
    let mut at = value;
    for key in keys {
        at = at.get(*key)?;
    }
    at.as_object()
}

fn text(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| value.to_string(), std::borrow::ToOwned::to_owned)
}

/// 基礎の層を、1つの表へ畳む。**キーの重複はそこで判明する。**
///
/// # Errors
///
/// 基礎のキーが重複しているときに返す。
pub fn raw(value: &Value) -> Result<Vec<(String, String)>, String> {
    let Some(base) = table(value, &["base"]) else {
        return Err("base が無い".to_owned());
    };
    let mut out: Vec<(String, String)> = Vec::new();
    for (kind, group) in base {
        let Some(group) = group.as_object() else {
            continue;
        };
        for (key, value) in group {
            if out.iter().any(|(k, _)| k == key) {
                return Err(format!("基礎のキーが重複している: {key}（{kind}）"));
            }
            out.push((key.clone(), text(value)));
        }
    }
    Ok(out)
}

fn lookup(pairs: &[(String, String)], key: &str) -> Option<String> {
    pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

/// 形と、層を跨ぐ参照を検査する。**0件になるものだけを検査する。**
#[must_use]
pub fn validate(references: &Path, value: &Value) -> Vec<String> {
    let mut err = crate::shape::against(&schema_path(references), value, "形");
    let base = match raw(value) {
        Ok(base) => base,
        Err(why) => {
            err.push(why);
            return err;
        }
    };
    let empty = Map::new();
    let light = table(value, &["semantic", "light"]).unwrap_or(&empty);
    let dark = table(value, &["semantic", "dark"]).unwrap_or(&empty);
    let mut only: Vec<(&String, &str)> = light
        .keys()
        .filter(|k| !dark.contains_key(*k))
        .map(|k| (k, "light"))
        .chain(
            dark.keys()
                .filter(|k| !light.contains_key(*k))
                .map(|k| (k, "dark")),
        )
        .collect();
    only.sort_by(|a, b| a.0.cmp(b.0));
    for (key, side) in only {
        err.push(format!(
            "意味のキー --{key} が {side} にしか無い ── 明暗で同じキー集合にする"
        ));
    }
    for (side, table) in [("light", light), ("dark", dark)] {
        for (key, want) in table {
            let want = text(want);
            if lookup(&base, &want).is_none() {
                err.push(format!(
                    "意味 {side} の --{key} が、基礎に無いキー「{want}」を参照している"
                ));
            }
        }
    }
    err.extend(contrast_errors(value, &base));
    if let Some(parts) = table(value, &["component"]) {
        for (key, want) in parts {
            let want = text(want);
            if literal(&want) {
                continue;
            }
            if lookup(&base, &want).is_none() && !light.contains_key(&want) {
                err.push(format!(
                    "部品 --{key} が、意味にも基礎に無いキー「{want}」を参照している"
                ));
            }
        }
    }
    err
}

/// 文字と地の比、重ねる面どうしの差を検査する。
fn contrast_errors(value: &Value, base: &[(String, String)]) -> Vec<String> {
    let mut err = Vec::new();
    for side in ["light", "dark"] {
        let Some(table) = table(value, &["semantic", side]) else {
            continue;
        };
        let color = |role: &str| -> Option<String> { lookup(base, &text(table.get(role)?)) };
        for (fg, need) in CONTRAST {
            if !table.contains_key(fg) {
                continue;
            }
            for bg in SURFACES {
                if !table.contains_key(bg) {
                    continue;
                }
                let (Some(a), Some(b)) = (color(fg), color(bg)) else {
                    continue;
                };
                let got = contrast(&a, &b);
                if got < need {
                    err.push(format!(
                        "{side}: --{fg} が --{bg} の上で {got:.2}（要 {need}）── {a} / {b}"
                    ));
                }
            }
        }
        for (top, under, need) in LAYERED {
            if !table.contains_key(top) || !table.contains_key(under) {
                continue;
            }
            let (Some(a), Some(b)) = (color(top), color(under)) else {
                continue;
            };
            let got = contrast(&a, &b);
            if got < need {
                err.push(format!(
                    "{side}: --{top} と --{under} の差が {got:.3}（要 {need}）── 重ねても分かれて見えない"
                ));
            }
        }
    }
    err
}

fn lines(table: &Map<String, Value>, base: &[(String, String)]) -> String {
    table
        .iter()
        .map(|(key, want)| {
            let want = text(want);
            format!("--{key}:{};", lookup(base, &want).unwrap_or_default())
        })
        .collect()
}

/// 3つの選択子と、部品の層を組む。**明暗で変わらない層は、明の側に1回だけ出す。**
///
/// `host` を渡すと、Shadow の中でも解決する選択子で組む。
///
/// # Errors
///
/// 基礎のキーが重複しているときに返す。
pub fn css(value: &Value, host: bool) -> Result<String, String> {
    let (light_sel, dark_sel) = if host {
        (HOST_LIGHT, HOST_DARK)
    } else {
        (LIGHT, DARK)
    };
    let base = raw(value)?;
    let empty = Map::new();
    let parts: String = table(value, &["component"])
        .unwrap_or(&empty)
        .iter()
        .map(|(key, want)| {
            let want = text(want);
            let value = if literal(&want) {
                want
            } else {
                format!("var(--{want})")
            };
            format!("--{key}:{value};")
        })
        .collect();
    let mut scale = String::new();
    for group in SCALES {
        if let Some(rows) = table(value, &["base", group]) {
            for (key, value) in rows {
                scale.push_str(&format!("--{key}:{};", text(value)));
            }
        }
    }
    let light = lines(
        table(value, &["semantic", "light"]).unwrap_or(&empty),
        &base,
    ) + &scale
        + &parts;
    let dark = lines(table(value, &["semantic", "dark"]).unwrap_or(&empty), &base);
    Ok(format!(
        "{light_sel}{{{light}}}{}{{{dark}}}}}{}{{{dark}}}",
        dark_sel.0, dark_sel.1
    ))
}

/// 生成物から、参照したキーと定義したキーを抜く。**差を0件にするために参照する。**
///
/// 代替値を持つ参照（`var(--x,#fff)`）は、定義が無くても崩壊しない ── 別に数える。
#[must_use]
pub fn refs_and_defs(built: &str) -> (Vec<String>, Vec<String>) {
    let mut refs = Vec::new();
    let mut defs = Vec::new();
    // 参照 ── `var(` の直後の名前で、閉じ括弧で終わるものだけを数える
    let mut from = 0;
    while let Some(at) = built[from..].find("var(").map(|x| from + x) {
        let head = trim_left(built, at + 4);
        if let Some((name, end)) = name_at(built, head) {
            if built[trim_left(built, end)..].starts_with(')') && !refs.contains(&name) {
                refs.push(name);
            }
        }
        from = at + 4;
    }
    // 定義 ── 名前のあとに `:` が続くもの
    let mut from = 0;
    while let Some(at) = built[from..].find("--").map(|x| from + x) {
        if let Some((name, end)) = name_at(built, at) {
            if built[trim_left(built, end)..].starts_with(':') && !defs.contains(&name) {
                defs.push(name.clone());
            }
            from = end;
        } else {
            from = at + 2;
        }
    }
    refs.sort();
    defs.sort();
    (refs, defs)
}

/// 空白を飛ばした位置を返す。
fn trim_left(body: &str, from: usize) -> usize {
    from + body[from..].len() - body[from..].trim_start().len()
}

/// その位置から始まるキーの名前を読む。
fn name_at(body: &str, from: usize) -> Option<(String, usize)> {
    if !body[from..].starts_with("--") {
        return None;
    }
    let end = body[from + 2..]
        .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
        .map_or(body.len(), |x| from + 2 + x);
    if end == from + 2 {
        return None;
    }
    Some((body[from..end].to_owned(), end))
}
