//! デザインシステム（references/design-system.schema.json）の検査と、トークンから CSS の変数を作ること。
//! デザインテンプレートを持つものは、デザインシステムを持つ（ボード board-on-schema-driven の論点5）。
//! 見た目そのものは具体が持ち、基盤は形と参照と直値の有無だけを見る（ACDR 0132）。

use crate::domain::schema::Schema;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const DESIGN_SYSTEM: &str = include_str!("../../../../references/design-system.schema.json");

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

/// デザインシステムを検査する。parts はデザインテンプレート（CSS を含む）、components は Design が登録した部品の名前。
/// 見つかったことを、見つけた順に返す。空なら契約を満たしている。
pub fn check(system_text: &str, parts: &str, components: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let system: Value = match serde_json::from_str(system_text) {
        Ok(value) => value,
        Err(error) => return vec![format!("デザインシステムが JSON として読めない：{error}")],
    };
    let shape = Schema::new(
        "design-system.schema.json",
        serde_json::from_str(DESIGN_SYSTEM).unwrap_or(Value::Null),
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
