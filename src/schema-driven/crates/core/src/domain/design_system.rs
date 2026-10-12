//! デザインシステム（schemas/design-system.schema.json）の検査と、トークンから CSS の変数を作ること。
//! 部品のカタログは、デザインシステムに従う。
//! 見た目そのものは具体が持ち、基盤は形と参照と直値の有無だけを見る。

use crate::domain::schema::Schema;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// 読み込んだデザインシステム。トークンの名前は CSS の変数の名前（先頭の -- を除く）。
struct Tokens {
    scope: String,
    base: BTreeMap<String, String>,
    light: BTreeMap<String, String>,
    dark: BTreeMap<String, String>,
    component: BTreeMap<String, String>,
}

fn table(value: &Value) -> BTreeMap<String, String> {
    value
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(key, value)| value.as_str().map(|text| (key.clone(), text.to_owned())))
        .collect()
}

fn read(system: &Value) -> Tokens {
    let tokens = &system["tokens"];
    let mut base = BTreeMap::new();
    for group in tokens["base"].as_object().into_iter().flatten() {
        base.extend(table(group.1));
    }
    Tokens {
        scope: system["scope"].as_str().unwrap_or(":root").to_owned(),
        base,
        light: table(&tokens["semantic"]["light"]),
        dark: table(&tokens["semantic"]["dark"]),
        component: table(&tokens["component"]),
    }
}

/// calc() などの中の var(--名前) の名前を、出てきた順に返す。
fn variables_used(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = css;
    while let Some(index) = rest.find("var(--") {
        let tail = &rest[index + "var(--".len()..];
        let name: String = tail
            .chars()
            .take_while(|character| {
                character.is_ascii_alphanumeric() || *character == '-' || *character == '_'
            })
            .collect();
        out.push(name);
        rest = tail;
    }
    out
}

/// CSS の中で定義している変数（--名前:）の名前。
fn variables_defined(css: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = css;
    while let Some(index) = rest.find("--") {
        let before = rest[..index].chars().last();
        let tail = &rest[index + 2..];
        let name: String = tail
            .chars()
            .take_while(|character| {
                character.is_ascii_alphanumeric() || *character == '-' || *character == '_'
            })
            .collect();
        let after = tail[name.len()..].trim_start();
        if !name.is_empty() && before != Some('(') && after.starts_with(':') {
            out.insert(name.clone());
        }
        rest = tail;
    }
    out
}

/// CSS のコメント（/* … */）を除く。コメントの中の色や変数の名前は、検査の相手ではない。
fn without_comments(css: &str) -> String {
    let mut out = String::new();
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        match rest[start + 2..].find("*/") {
            Some(end) => rest = &rest[start + 2 + end + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// テンプレートの中の CSS（style 要素の中身と style 属性の値）。コメントは除く。
fn css_in(parts: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = parts;
    while let Some(index) = rest.find("<style") {
        let tail = &rest[index..];
        let Some(open) = tail.find('>') else { break };
        let body = &tail[open + 1..];
        let close = body.find("</style>").unwrap_or(body.len());
        out.push(without_comments(&body[..close]));
        rest = &body[close..];
    }
    let mut rest = parts;
    while let Some(index) = rest.find("style=\"") {
        let tail = &rest[index + "style=\"".len()..];
        let close = tail.find('"').unwrap_or(tail.len());
        out.push(without_comments(&tail[..close]));
        rest = &tail[close..];
    }
    out
}

/// CSS の中の色の直値（#16進 ・ rgb() ・ rgba() ・ hsl() ・ hsla()）。
fn literal_colors(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    let characters: Vec<char> = css.chars().collect();
    let mut i = 0;
    while i < characters.len() {
        if characters[i] == '#' {
            let digits: String = characters[i + 1..]
                .iter()
                .take_while(|character| character.is_ascii_alphanumeric())
                .collect();
            if matches!(digits.len(), 3 | 4 | 6 | 8)
                && digits
                    .chars()
                    .all(|character| character.is_ascii_hexdigit())
            {
                out.push(format!("#{digits}"));
            }
        }
        i += 1;
    }
    for function in ["rgb(", "rgba(", "hsl(", "hsla("] {
        let mut rest = css;
        while let Some(index) = rest.find(function) {
            let starts_word = rest[..index]
                .chars()
                .last()
                .is_none_or(|character| !character.is_ascii_alphanumeric() && character != '-');
            if starts_word {
                out.push(function.to_owned());
            }
            rest = &rest[index + function.len()..];
        }
    }
    out
}

/// デザインシステムを検査する。parts は部品の HTML（CSS を含む）、components はカタログの部品の名前。
/// 見つかったことを、見つけた順に返す。空なら契約を満たしている。
pub fn check(system_text: &str, parts: &str, components: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let system: Value = match serde_json::from_str(system_text) {
        Ok(value) => value,
        Err(error) => return vec![format!("デザインシステムが JSON として読めない：{error}")],
    };
    let shape = Schema::new(
        "design-system.schema.json",
        serde_json::from_str(crate::embedded::schema_text(
            "schema-driven/design-system@v1",
        ))
        .unwrap_or(Value::Null),
        vec![],
    );
    match shape.validate(&system) {
        Ok(validation) => {
            out.extend(
                validation
                    .unfilled
                    .iter()
                    .map(|unfilled| format!("形：{} が無い", unfilled.property())),
            );
            out.extend(
                validation
                    .errors
                    .iter()
                    .map(|error| format!("形：{} {}", error.property(), error.reason())),
            );
        }
        Err(error) => out.push(format!("形：{}", error.0)),
    }
    if !out.is_empty() {
        return out;
    }
    let tokens = read(&system);
    for (theme, refs) in [("light", &tokens.light), ("dark", &tokens.dark)] {
        for (name, target) in refs {
            if !tokens.base.contains_key(target) {
                out.push(format!(
                    "意味のトークン {theme}.{name} が指す基礎のトークン {target} が無い"
                ));
            }
        }
    }
    let light: BTreeSet<&String> = tokens.light.keys().collect();
    let dark: BTreeSet<&String> = tokens.dark.keys().collect();
    for name in light.difference(&dark) {
        out.push(format!("意味のトークン {name} が、明にあって暗に無い"));
    }
    for name in dark.difference(&light) {
        out.push(format!("意味のトークン {name} が、暗にあって明に無い"));
    }
    let defined = |name: &str| {
        tokens.light.contains_key(name)
            || tokens.component.contains_key(name)
            || tokens.base.contains_key(name)
    };
    for (name, value) in &tokens.component {
        if value.starts_with("calc(") {
            for used in variables_used(value) {
                if !defined(&used) {
                    out.push(format!(
                        "部品のトークン {name} の calc() が使う --{used} が定義されていない"
                    ));
                }
            }
        } else if !defined(value) {
            out.push(format!(
                "部品のトークン {name} が指す {value} が、基礎にも意味にも部品にも無い"
            ));
        }
    }
    let mut stated = BTreeSet::new();
    for component in system["components"].as_array().into_iter().flatten() {
        let name = component["name"].as_str().unwrap_or_default();
        stated.insert(name.to_owned());
        for (state, properties) in component["states"].as_object().into_iter().flatten() {
            for (property, token) in properties.as_object().into_iter().flatten() {
                let token = token.as_str().unwrap_or_default();
                if !defined(token) {
                    out.push(format!("部品 {name} の状態 {state} の {property} が使う {token} が定義されていない"));
                }
            }
        }
    }
    for name in components {
        if !stated.contains(name) {
            out.push(format!(
                "部品 {name} の状態が、デザインシステムの components に無い"
            ));
        }
    }
    let css = css_in(parts);
    let mut local = BTreeSet::new();
    for block in &css {
        local.extend(variables_defined(block));
    }
    for block in &css {
        for color in literal_colors(block) {
            out.push(format!(
                "テンプレートに色の直値がある：{color}（トークンから引く）"
            ));
        }
        for used in variables_used(block) {
            if !defined(&used) && !local.contains(&used) {
                out.push(format!(
                    "テンプレートが使う変数 --{used} が定義されていない"
                ));
            }
        }
    }
    out
}

fn value_of(tokens: &Tokens, name: &str) -> String {
    if tokens.light.contains_key(name) || tokens.component.contains_key(name) {
        format!("var(--{name})")
    } else {
        tokens
            .base
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_owned())
    }
}

/// トークンから CSS の変数を作る。明 ・ OS の暗 ・ 明示の暗の3つのブロックにする。check を通ったものに使う。
pub fn css(system_text: &str) -> String {
    let Ok(system) = serde_json::from_str::<Value>(system_text) else {
        return String::new();
    };
    let tokens = read(&system);
    let semantic = |refs: &BTreeMap<String, String>| -> String {
        refs.iter()
            .map(|(name, target)| {
                format!(
                    "--{name}:{}",
                    tokens.base.get(target).cloned().unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join(";")
    };
    let component: String = tokens
        .component
        .iter()
        .map(|(name, value)| {
            let resolved = if value.starts_with("calc(") {
                value.clone()
            } else {
                value_of(&tokens, value)
            };
            format!(";--{name}:{resolved}")
        })
        .collect();
    let (dark_auto, dark_set) = if tokens.scope == ":root" {
        (
            ":root:not([data-theme=\"light\"])".to_owned(),
            ":root[data-theme=\"dark\"]".to_owned(),
        )
    } else {
        (
            format!(":root:not([data-theme=\"light\"]) {}", tokens.scope),
            format!(":root[data-theme=\"dark\"] {}", tokens.scope),
        )
    };
    let dark = format!("{};color-scheme:dark", semantic(&tokens.dark));
    // 基礎の層（余白 ・ 字の大きさ ・ 角の丸みなど）も変数にする。テンプレートが基礎の名前を使っても、変数が在る
    let base: String = tokens
        .base
        .iter()
        .map(|(name, value)| format!("--{name}:{value};"))
        .collect();
    format!(
        "{}{{{base}{}{}}}\n@media (prefers-color-scheme: dark){{{dark_auto}{{{dark}}}}}\n{dark_set}{{{dark}}}",
        tokens.scope,
        semantic(&tokens.light),
        component
    )
}

#[cfg(test)]
mod tests {
    // 部品のカタログは、デザインシステムに従う。
    // 基盤は、デザインシステム（トークンの3層と部品ごとの状態）を検査し、トークンから CSS の変数を作る。
    use super::{check, css};
    use serde_json::{json, Value};

    /// テスト用のカタログの部品。pill の style だけを差し替える。
    fn parts(pill_style: &str) -> String {
        format!(
            r#"<template id="page"><style>{{{{tokens}}}}</style><main><h1>{{{{title}}}}</h1>{{{{sections}}}}</main></template>
<template id="section"><section><h2>{{{{heading}}}}</h2>{{{{body}}}}</section></template>
<template id="pill"><b style="{pill_style}">{{{{text}}}}</b></template>"#
        )
    }

    fn system() -> Value {
        json!({
            "scope": ":root",
            "tokens": {
                "base": {"color": {"teal-700": "#0d5c55", "teal-300": "#6cc9b8", "paper-100": "#eef1ef", "night-900": "#0e1614"}},
                "semantic": {"light": {"bg": "paper-100", "key": "teal-700"}, "dark": {"bg": "night-900", "key": "teal-300"}},
                "component": {"pill-bg": "key"}
            },
            "components": [{"name": "pill", "states": {"default": {"background": "pill-bg", "text": "bg"}}}]
        })
    }

    /// テスト用のカタログ（部品は pill だけ）を検査した指摘を、1行ずつつないで返す。
    fn findings(pill_style: &str, system: Value) -> String {
        check(
            &system.to_string(),
            &parts(pill_style),
            &["pill".to_owned()],
        )
        .join("\n")
    }

    #[test]
    fn tokens_become_css_variables_for_light_and_both_dark_settings() {
        assert_eq!(findings("background:var(--pill-bg)", system()), "");
        let tokens = css(&system().to_string());
        assert!(
            tokens.contains(":root{--night-900:#0e1614;--paper-100:#eef1ef;--teal-300:#6cc9b8;--teal-700:#0d5c55;--bg:#eef1ef;--key:#0d5c55;--pill-bg:var(--key)}"),
            "{tokens}"
        );
        assert!(
            tokens.contains(r#"@media (prefers-color-scheme: dark){:root:not([data-theme="light"]){--bg:#0e1614;--key:#6cc9b8;color-scheme:dark}}"#),
            "{tokens}"
        );
        assert!(
            tokens.contains(
                r#":root[data-theme="dark"]{--bg:#0e1614;--key:#6cc9b8;color-scheme:dark}"#
            ),
            "{tokens}"
        );
    }

    #[test]
    fn a_template_may_use_base_tokens_such_as_spacing() {
        let mut system = system();
        system["tokens"]["base"]["space"] = json!({"sp-3": ".45rem"});
        assert_eq!(
            findings(
                "padding:var(--sp-3);background:var(--pill-bg)",
                system.clone()
            ),
            ""
        );
        let tokens = css(&system.to_string());
        assert!(tokens.contains("--sp-3:.45rem"), "{tokens}");
    }

    #[test]
    fn comments_in_the_css_are_not_checked() {
        assert_eq!(
            findings(
                "/* var(--名前) で引き、#ff0000 のような直値を書かない */background:var(--pill-bg)",
                system()
            ),
            ""
        );
    }

    #[test]
    fn a_literal_color_in_the_template_stops_rendering() {
        let found = findings("background:#ff0000", system());
        assert!(found.contains("色の直値"), "{found}");
    }

    #[test]
    fn a_variable_the_tokens_do_not_define_stops_rendering() {
        let found = findings("background:var(--nope)", system());
        assert!(found.contains("--nope"), "{found}");
    }

    #[test]
    fn a_semantic_token_must_point_at_a_base_token() {
        let mut broken = system();
        broken["tokens"]["semantic"]["light"]["key"] = json!("teal-999");
        let found = findings("background:var(--pill-bg)", broken);
        assert!(found.contains("teal-999"), "{found}");
    }

    #[test]
    fn light_and_dark_must_have_the_same_keys() {
        let mut broken = system();
        broken["tokens"]["semantic"]["dark"]
            .as_object_mut()
            .unwrap()
            .remove("bg");
        let found = findings("background:var(--pill-bg)", broken);
        assert!(found.contains("bg"), "{found}");
    }

    #[test]
    fn every_component_has_its_states() {
        let mut broken = system();
        broken["components"] = json!([]);
        let found = findings("background:var(--pill-bg)", broken);
        assert!(found.contains("pill"), "{found}");
    }

    #[test]
    fn a_state_must_use_a_defined_token() {
        let mut broken = system();
        broken["components"][0]["states"]["pressed"] = json!({"background": "warn"});
        let found = findings("background:var(--pill-bg)", broken);
        assert!(found.contains("warn"), "{found}");
    }
}
