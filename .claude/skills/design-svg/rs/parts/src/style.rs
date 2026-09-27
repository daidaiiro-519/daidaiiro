// SPDX-License-Identifier: MIT
//! スタイルの解決 ── CSS のカスケードに相当する。
//!
//! 出どころは3層 ── テーマの既定値 → 役割による上書き → その場限りの上書き。後のものが前の
//! ものに勝つ。役割の中身もテーマが持つので（`role.focus.color.box-fill` のような平らな
//! 名前）、テーマを差し替えれば「どんな役割があるか」ごと入れ替わる。
//!
//! 値に妥当な範囲があるトークンは、解決のたびにその範囲へ照らして検査する。**範囲外の値で
//! 描いて破綻してから気づくのではなく、その場で止める。**

use serde_json::{Map, Value};

use crate::theme::{self, PLAIN, ROLE_PREFIX};

/// 足りないキーを、報告に何件まで並べるか。
const SHOWN_MISSING: usize = 5;

/// 解決済みの見た目。**型は土台（`theme`）が持つ** ── 部品の台帳が参照するため、方針の層に置かない。
pub use crate::theme::Style;

/// 値がトークン名を指していたら、指し先の値へたどる。**1層だけ。**
fn deref(value: &Value, theme: &Map<String, Value>) -> Value {
    if let Value::String(s) = value {
        if let Some(v) = theme.get(s) {
            return v.clone();
        }
    }
    value.clone()
}

fn check_ranges(resolved: &Map<String, Value>) -> Result<(), String> {
    for (key, lo, hi) in &theme::source().ranges {
        let Some(Value::Number(n)) = resolved.get(key) else {
            continue;
        };
        let Some(v) = n.as_f64() else { continue };
        if !(*lo <= v && v <= *hi) {
            return Err(format!(
                "トークン '{key}' の値 {} が妥当な範囲 [{}, {}] の外にある",
                crate::py::num(&Value::Number(n.clone())),
                crate::py::float(*lo),
                crate::py::float(*hi)
            ));
        }
    }
    Ok(())
}

/// 役割とその場の上書きから、実際に使う値を組み立てる。
///
/// # Errors
///
/// テーマが既定のキーを欠いているとき ・ テーマが知らない役割のとき ・ 上書きがテーマに無い
/// 名前を指したとき ・ 範囲外の値のときに返す。
pub fn resolve(
    role: &str,
    overrides: Option<&Map<String, Value>>,
    theme: Option<&Map<String, Value>>,
) -> Result<Style, String> {
    let base = theme::default_theme();
    let theme = theme.unwrap_or(base);
    let mut missing: Vec<&String> = base.keys().filter(|k| !theme.contains_key(*k)).collect();
    if !missing.is_empty() {
        missing.sort();
        let shown: Vec<String> = missing
            .iter()
            .take(SHOWN_MISSING)
            .map(|k| crate::py::quote(k))
            .collect();
        return Err(format!(
            "テーマに足りないキーがある: [{}]{}（全{}件）。テーマは差し替えであって作り直しではないので、既定のテーマを土台にする",
            shown.join(", "),
            if missing.len() > SHOWN_MISSING { "…" } else { "" },
            missing.len()
        ));
    }
    let prefix = format!("{ROLE_PREFIX}{role}.");
    let role_over: Vec<(String, Value)> = theme
        .iter()
        .filter_map(|(k, v)| {
            k.strip_prefix(&prefix)
                .map(|rest| (rest.to_owned(), v.clone()))
        })
        .collect();
    if role != PLAIN && role_over.is_empty() {
        let mut known: Vec<String> = theme
            .keys()
            .filter_map(|k| k.strip_prefix(ROLE_PREFIX))
            .filter_map(|rest| rest.split('.').next())
            .map(str::to_owned)
            .collect();
        known.push(PLAIN.to_owned());
        known.sort();
        known.dedup();
        let shown: Vec<String> = known.iter().map(|k| crate::py::quote(k)).collect();
        return Err(format!(
            "テーマが知らない役割 '{role}' が渡された。使えるのは [{}]。新しい役割は、テーマへ 'role.{role}.<トークン名>' を足すと増える",
            shown.join(", ")
        ));
    }
    let mut merged: Map<String, Value> = theme
        .iter()
        .filter(|(k, _)| !k.starts_with(ROLE_PREFIX))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    for (k, v) in role_over {
        merged.insert(k, v);
    }
    if let Some(over) = overrides {
        let mut unknown: Vec<&String> = over.keys().filter(|k| !merged.contains_key(*k)).collect();
        if !unknown.is_empty() {
            unknown.sort();
            let shown: Vec<String> = unknown.iter().map(|k| crate::py::quote(k)).collect();
            return Err(format!(
                "テーマが知らないトークン名が上書きに渡された: [{}]。新しいトークンはテーマへ足す ── その場の上書きから増やさない",
                shown.join(", ")
            ));
        }
        for (k, v) in over {
            merged.insert(k.clone(), v.clone());
        }
    }
    let resolved: Map<String, Value> = merged
        .iter()
        .map(|(k, v)| (k.clone(), deref(v, theme)))
        .collect();
    check_ranges(&resolved)?;
    Ok(Style { values: resolved })
}
