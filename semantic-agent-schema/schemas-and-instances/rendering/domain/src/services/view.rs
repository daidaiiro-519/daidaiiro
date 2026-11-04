//! 描くときの決まり：テンプレートの種類の選び方（Concrete の根の x-view。無ければ page）・ ページの経路 ・
//! テンプレートに渡す値の形（トークンの値の木 ・ JS へ渡す JSON の文字列）。

use serde_json::{Map, Value};

/// x-view が無いときのテンプレートの種類。
pub const DEFAULT_VIEW: &str = "page";

/// Concrete の根の x-view で選ぶテンプレートの種類（無ければ page）。x-view は根だけに書く。
#[must_use]
pub fn view_kind(concrete: &Value) -> &str {
    concrete
        .get("x-view")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_VIEW)
}

/// トークンの木から、値だけの木（$ で始まる名前を除き、トークンは $value の値）。テンプレートは tokens.color.accent で引く。
#[must_use]
pub fn token_values(v: Option<&Value>) -> Value {
    let Some(m) = v.and_then(Value::as_object) else {
        return Value::Object(Map::new());
    };
    Value::Object(
        m.iter()
            .filter(|(k, _)| !k.starts_with('$'))
            .map(|(k, child)| {
                let value = child
                    .get("$value")
                    .cloned()
                    .unwrap_or_else(|| token_values(Some(child)));
                (k.clone(), value)
            })
            .collect(),
    )
}

/// JS へ渡す JSON の文字列。<script> に入れても閉じないよう、< > & ' と行の区切り（U+2028 ・ U+2029）を \u の形にする。
#[must_use]
pub fn script_json(v: &Value) -> String {
    let mut out = String::new();
    for c in v.to_string().chars() {
        match c {
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\'' => out.push_str("\\u0027"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c => out.push(c),
        }
    }
    out
}

/// 対象の経路から、ページの経路（拡張子を .html に）。
#[must_use]
pub fn page_path_of(target: &str) -> String {
    target
        .strip_suffix(".json")
        .map_or_else(|| format!("{target}.html"), |s| format!("{s}.html"))
}
