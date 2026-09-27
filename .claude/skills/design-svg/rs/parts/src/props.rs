// SPDX-License-Identifier: MIT
//! 部品の入力を読む。**入力は JSON の値のまま受け、読むときに型を選ぶ。**
//!
//! 整数で渡されたものは整数のまま、小数は小数のまま持つ ── 素のまま SVG へ出す値は、書き方
//! がそのまま出力に現れる。

use serde_json::Value;

use crate::registry::Props;

/// 数として読む。**無ければ `None`。**
#[must_use]
pub fn num(props: &Props, key: &str) -> Option<f64> {
    props.get(key).and_then(Value::as_f64)
}

/// 数として読む。**無ければ誤り。**
///
/// # Errors
///
/// 無いときと、数でないときに返す。
pub fn need_num(props: &Props, key: &str) -> Result<f64, String> {
    match props.get(key) {
        None => Err(crate::py::quote(key).to_string()),
        Some(v) => v
            .as_f64()
            .ok_or_else(|| format!("'{key}' は数でなければならない: {}", crate::py::repr(v))),
    }
}

/// 在れば真偽として読む。**移す前と同じ真偽の決め方をする**（0 ・ 空 ・ null は偽）。
#[must_use]
pub fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// 鍵を真偽として読む。**無ければ偽。**
#[must_use]
pub fn flag(props: &Props, key: &str) -> bool {
    props.get(key).is_some_and(truthy)
}

/// 文字として読む。**文字でない値は、移す前の `str()` と同じ書き方で文字にする。**
#[must_use]
pub fn text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => crate::py::num(other),
    }
}

/// 鍵を文字として読む。**無ければ既定値。**
#[must_use]
pub fn text_or(props: &Props, key: &str, default: &str) -> String {
    props.get(key).map_or_else(|| default.to_owned(), text)
}

/// 鍵を文字として読む。**無ければ誤り。**
///
/// # Errors
///
/// 無いときに返す。
pub fn need_text(props: &Props, key: &str) -> Result<String, String> {
    props
        .get(key)
        .map(text)
        .ok_or_else(|| crate::py::quote(key))
}

/// 鍵を並びとして読む。**無ければ空。**
#[must_use]
pub fn list<'a>(props: &'a Props, key: &str) -> &'a [Value] {
    props
        .get(key)
        .and_then(|x| x.as_array())
        .map_or(&[], |x| x.as_slice())
}

/// 点を読む。`[x, y]` の形である。
#[must_use]
pub fn point(v: &Value) -> Option<(f64, f64)> {
    let a = v.as_array()?;
    Some((a.first()?.as_f64()?, a.get(1)?.as_f64()?))
}

/// 点の並びを読む。
#[must_use]
pub fn points(v: &Value) -> Vec<(f64, f64)> {
    v.as_array()
        .map(|a| a.iter().filter_map(point).collect())
        .unwrap_or_default()
}

/// 文字列を、HTML の中へそのまま置ける形にする。**引用符も逃がす。**
#[must_use]
pub fn esc(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// 鍵の値が、整数で書かれているか。
#[must_use]
pub fn is_int(props: &Props, key: &str) -> bool {
    props.get(key).is_some_and(|v| v.is_i64() || v.is_u64())
}
