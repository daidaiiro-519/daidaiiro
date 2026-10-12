//! デザインシステム（schemas/design-system.schema.json）の検査と、トークンから CSS の変数を作ること。
//! トークンの名前と値と層の分け方は concrete が持ち、基盤は形式 ・ 参照 ・ 循環しないこと ・
//! コントラストの下限（文字 4.5:1、部品の境界 3:1）と、カタログの CSS がトークンだけを使っているかを見る。

use crate::domain::schema::Schema;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// 読み込んだデザインシステム。トークンの名前は CSS の変数の名前（先頭の -- を除く）。
struct System {
    scope: String,
    light: BTreeMap<String, String>,
    dark: BTreeMap<String, String>,
    contrast: Vec<(String, String, String)>,
}

fn table(value: &Value) -> BTreeMap<String, String> {
    value
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(key, value)| value.as_str().map(|text| (key.clone(), text.to_owned())))
        .collect()
}

fn read(system: &Value) -> System {
    let text = |value: &Value, key: &str| value[key].as_str().unwrap_or_default().to_owned();
    System {
        scope: system["scope"].as_str().unwrap_or(":root").to_owned(),
        light: table(&system["tokens"]),
        dark: table(&system["dark"]),
        contrast: system["contrast"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|pair| {
                (
                    text(pair, "foreground"),
                    text(pair, "background"),
                    text(pair, "use"),
                )
            })
            .collect(),
    }
}

/// 値が {名前} なら、指すトークンの名前。
fn alias(value: &str) -> Option<&str> {
    value
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
        .filter(|name| !name.is_empty())
}

/// 明か暗の、ある名前の値。暗に無い名前は明の値を使う。
fn value_in<'system>(system: &'system System, dark: bool, name: &str) -> Option<&'system String> {
    if dark {
        system.dark.get(name).or_else(|| system.light.get(name))
    } else {
        system.light.get(name)
    }
}

/// 参照をたどった先の生の値。たどれなければ理由。
enum Resolved {
    Value(String),
    Missing,
    Cycle(Vec<String>),
}

fn resolve(system: &System, dark: bool, name: &str) -> Resolved {
    let mut path = vec![name.to_owned()];
    let mut current = name.to_owned();
    loop {
        let Some(value) = value_in(system, dark, &current) else {
            return Resolved::Missing;
        };
        match alias(value) {
            None => return Resolved::Value(value.clone()),
            Some(next) => {
                if let Some(start) = path.iter().position(|seen| seen == next) {
                    let mut cycle = path[start..].to_vec();
                    cycle.push(next.to_owned());
                    return Resolved::Cycle(cycle);
                }
                path.push(next.to_owned());
                current = next.to_owned();
            }
        }
    }
}

/// #rgb か #rrggbb の色を、0〜1 の RGB にする。
fn rgb(value: &str) -> Option<[f64; 3]> {
    let digits = value.strip_prefix('#')?;
    let full: String = match digits.len() {
        3 => digits.chars().flat_map(|digit| [digit, digit]).collect(),
        6 => digits.to_owned(),
        _ => return None,
    };
    let channel = |index: usize| {
        u8::from_str_radix(full.get(index..index + 2)?, 16)
            .ok()
            .map(|byte| f64::from(byte) / 255.0)
    };
    Some([channel(0)?, channel(2)?, channel(4)?])
}

/// WCAG 2.2 の相対輝度。
fn luminance(color: [f64; 3]) -> f64 {
    let linear = |channel: f64| {
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color[0]) + 0.7152 * linear(color[1]) + 0.0722 * linear(color[2])
}

/// WCAG 2.2 のコントラスト比。
fn contrast_ratio(first: [f64; 3], second: [f64; 3]) -> f64 {
    let (one, two) = (luminance(first), luminance(second));
    (one.max(two) + 0.05) / (one.min(two) + 0.05)
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

/// 形の検査（schemas/design-system.schema.json）。
fn shape_findings(system: &Value) -> Vec<String> {
    let shape = Schema::new(
        "design-system.schema.json",
        serde_json::from_str(crate::embedded::schema_text(
            "schema-driven/design-system@v1",
        ))
        .unwrap_or(Value::Null),
        vec![],
    );
    match shape.validate(system) {
        Ok(validation) => validation
            .unfilled
            .iter()
            .map(|unfilled| format!("形：{} が無い", unfilled.property()))
            .chain(
                validation
                    .errors
                    .iter()
                    .map(|error| format!("形：{} {}", error.property(), error.reason())),
            )
            .collect(),
        Err(error) => vec![format!("形：{}", error.0)],
    }
}

/// 参照の検査：{名前} の指す先があるか、暗の名前が明にあるか、生の値に CSS を壊す文字が無いか。
fn reference_findings(system: &System) -> Vec<String> {
    let mut out = Vec::new();
    for name in system.dark.keys() {
        if !system.light.contains_key(name) {
            out.push(format!("暗の {name} が tokens に無い"));
        }
    }
    for (theme, tokens) in [("明", &system.light), ("暗", &system.dark)] {
        for (name, value) in tokens {
            match alias(value) {
                Some(target) if !system.light.contains_key(target) => {
                    out.push(format!("{theme}の {name} が指す {target} が無い"));
                }
                Some(_) => {}
                None if value.contains(['{', '}', ';', '<', '>']) => out.push(format!(
                    "{theme}の {name} の値「{value}」に、使えない文字（{{ }} ; < >）がある"
                )),
                None => {}
            }
        }
    }
    out
}

/// 循環の検査。同じ輪は1回だけ返す。
fn cycle_findings(system: &System) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for (theme, dark) in [("明", false), ("暗", true)] {
        for name in system.light.keys() {
            if let Resolved::Cycle(cycle) = resolve(system, dark, name) {
                let members: BTreeSet<String> = cycle.iter().cloned().collect();
                if seen.insert(members) {
                    out.push(format!(
                        "{theme}の参照が循環している：{}",
                        cycle.join(" → ")
                    ));
                }
            }
        }
    }
    out
}

/// コントラストの検査。宣言された組を、明と（暗の値があれば）暗で比べる。
fn contrast_findings(system: &System) -> Vec<String> {
    let mut out = Vec::new();
    let themes: &[(&str, bool)] = if system.dark.is_empty() {
        &[("明", false)]
    } else {
        &[("明", false), ("暗", true)]
    };
    for (foreground, background, usage) in &system.contrast {
        let unknown: Vec<&String> = [foreground, background]
            .into_iter()
            .filter(|name| !system.light.contains_key(*name))
            .collect();
        if !unknown.is_empty() {
            for name in unknown {
                out.push(format!("contrast の {name} が tokens に無い"));
            }
            continue;
        }
        let (minimum, label) = if usage == "border" {
            (3.0, "部品の境界")
        } else {
            (4.5, "文字")
        };
        for (theme, dark) in themes {
            let color = |name: &str| {
                match resolve(system, *dark, name) {
                Resolved::Value(value) => rgb(&value).ok_or(format!(
                    "{theme}：contrast の {name} の値「{value}」を色として読めない（#rgb か #rrggbb で書く）"
                )),
                Resolved::Missing | Resolved::Cycle(_) => {
                    Err(format!("{theme}：contrast の {name} の値をたどれない"))
                }
            }
            };
            match (color(foreground), color(background)) {
                (Ok(front), Ok(back)) => {
                    let ratio = contrast_ratio(front, back);
                    if ratio < minimum {
                        out.push(format!(
                            "{theme}：{foreground} と {background} のコントラストが {ratio:.2}:1 で、{label}の下限 {minimum}:1 に届かない"
                        ));
                    }
                }
                (front, back) => out.extend(front.err().into_iter().chain(back.err())),
            }
        }
    }
    out
}

/// カタログの CSS の検査：色の直値が無く、使う変数がトークンかその CSS の中で定義されているか。
fn css_findings(system: &System, parts: &str, style: &str) -> Vec<String> {
    let mut css = css_in(parts);
    css.push(without_comments(style));
    let mut local = BTreeSet::new();
    for block in &css {
        local.extend(variables_defined(block));
    }
    let mut out = Vec::new();
    for block in &css {
        for color in literal_colors(block) {
            out.push(format!(
                "カタログの CSS に色の直値がある：{color}（トークンから引く）"
            ));
        }
        for used in variables_used(block) {
            if !system.light.contains_key(&used) && !local.contains(&used) {
                out.push(format!(
                    "カタログの CSS が使う変数 --{used} が定義されていない"
                ));
            }
        }
    }
    out
}

/// デザインシステムを検査する。parts は部品の HTML、style は部品の CSS。
/// 見つかったことを、見つけた順に返す。空なら契約を満たしている。
pub fn check(system_text: &str, parts: &str, style: &str) -> Vec<String> {
    let system: Value = match serde_json::from_str(system_text) {
        Ok(value) => value,
        Err(error) => return vec![format!("デザインシステムが JSON として読めない：{error}")],
    };
    let shape = shape_findings(&system);
    if !shape.is_empty() {
        return shape;
    }
    let system = read(&system);
    let references = reference_findings(&system);
    if !references.is_empty() {
        return references;
    }
    let cycles = cycle_findings(&system);
    if !cycles.is_empty() {
        return cycles;
    }
    let mut out = contrast_findings(&system);
    out.extend(css_findings(&system, parts, style));
    out
}

/// トークンから CSS の変数を作る。明と、暗の値があれば OS の暗 ・ 明示の暗の2つのブロックにする。check を通ったものに使う。
pub fn css(system_text: &str) -> String {
    let Ok(system) = serde_json::from_str::<Value>(system_text) else {
        return String::new();
    };
    let system = read(&system);
    let declarations = |tokens: &BTreeMap<String, String>| -> String {
        tokens
            .iter()
            .map(|(name, value)| match alias(value) {
                Some(target) => format!("--{name}:var(--{target})"),
                None => format!("--{name}:{value}"),
            })
            .collect::<Vec<_>>()
            .join(";")
    };
    let light = format!("{}{{{}}}", system.scope, declarations(&system.light));
    if system.dark.is_empty() {
        return light;
    }
    let (dark_auto, dark_set) = if system.scope == ":root" {
        (
            ":root:not([data-theme=\"light\"])".to_owned(),
            ":root[data-theme=\"dark\"]".to_owned(),
        )
    } else {
        (
            format!(":root:not([data-theme=\"light\"]) {}", system.scope),
            format!(":root[data-theme=\"dark\"] {}", system.scope),
        )
    };
    let dark = format!("{};color-scheme:dark", declarations(&system.dark));
    format!("{light}\n@media (prefers-color-scheme: dark){{{dark_auto}{{{dark}}}}}\n{dark_set}{{{dark}}}")
}

#[cfg(test)]
mod tests {
    // 基盤は、トークンの形式 ・ 参照 ・ 循環 ・ 宣言された組のコントラストと、カタログの CSS がトークンだけを使うかを見る。
    // トークンの名前と層の分け方は、試験のデザインシステムが自分で決める。
    use super::{check, css};
    use serde_json::{json, Value};

    /// 文字 ink（#111d1a）と背景 paper（#f7f8f7）は 16.5:1、境界 line（#7a8a85）と paper は 3.4:1。
    fn system() -> Value {
        json!({
            "scope": ":root",
            "tokens": {"paper": "#f7f8f7", "ink": "#111d1a", "line": "#7a8a85", "night": "#0e1614", "mist": "#e6eeeb",
                       "bg": "{paper}", "text": "{ink}", "space": ".5rem"},
            "dark": {"bg": "{night}", "text": "{mist}"},
            "contrast": [{"foreground": "text", "background": "bg", "use": "text"},
                         {"foreground": "line", "background": "paper", "use": "border"}]
        })
    }

    /// 試験のデザインシステムを、style の CSS とともに検査した指摘を、1行ずつつないで返す。
    fn findings(style: &str, system: Value) -> String {
        check(
            &system.to_string(),
            "<template id=\"page\"></template>",
            style,
        )
        .join("\n")
    }

    #[test]
    fn tokens_become_css_variables_for_light_and_both_dark_settings() {
        assert_eq!(findings("b{color:var(--text)}", system()), "");
        assert_eq!(
            css(&system().to_string()),
            ":root{--bg:var(--paper);--ink:#111d1a;--line:#7a8a85;--mist:#e6eeeb;--night:#0e1614;--paper:#f7f8f7;--space:.5rem;--text:var(--ink)}\n@media (prefers-color-scheme: dark){:root:not([data-theme=\"light\"]){--bg:var(--night);--text:var(--mist);color-scheme:dark}}\n:root[data-theme=\"dark\"]{--bg:var(--night);--text:var(--mist);color-scheme:dark}"
        );
    }

    #[test]
    fn without_dark_values_only_the_light_block_is_written() {
        let mut light_only = system();
        light_only.as_object_mut().unwrap().remove("dark");
        assert_eq!(findings("", light_only.clone()), "");
        assert!(!css(&light_only.to_string()).contains("dark"));
    }

    #[test]
    fn the_shape_is_checked_first() {
        let mut broken = system();
        broken.as_object_mut().unwrap().remove("contrast");
        assert!(findings("", broken).contains("contrast"));
        let mut badly_named = system();
        badly_named["tokens"]["Bad_Name"] = json!("#000000");
        assert!(findings("", badly_named).starts_with("形："));
    }

    #[test]
    fn a_reference_must_point_at_a_token() {
        let mut broken = system();
        broken["tokens"]["text"] = json!("{nope}");
        assert_eq!(findings("", broken), "明の text が指す nope が無い");
    }

    #[test]
    fn a_dark_value_must_name_a_light_token() {
        let mut broken = system();
        broken["dark"]["extra"] = json!("#000000");
        assert_eq!(findings("", broken), "暗の extra が tokens に無い");
    }

    #[test]
    fn a_raw_value_may_not_break_out_of_the_css() {
        let mut broken = system();
        broken["tokens"]["space"] = json!("1px}</style><script>");
        assert!(findings("", broken).contains("使えない文字"));
    }

    #[test]
    fn references_must_not_loop() {
        let mut broken = system();
        broken["tokens"]["paper"] = json!("{bg}");
        assert_eq!(
            findings("", broken),
            "明の参照が循環している：bg → paper → bg"
        );
    }

    #[test]
    fn text_below_four_and_a_half_to_one_is_refused() {
        let mut broken = system();
        // #767676 と #ffffff は 4.54:1（通る）、#777777 は 4.48:1（届かない）
        broken["tokens"]["paper"] = json!("#ffffff");
        broken["tokens"]["ink"] = json!("#767676");
        broken.as_object_mut().unwrap().remove("dark");
        assert_eq!(findings("", broken.clone()), "");
        broken["tokens"]["ink"] = json!("#777777");
        assert_eq!(
            findings("", broken),
            "明：text と bg のコントラストが 4.48:1 で、文字の下限 4.5:1 に届かない"
        );
    }

    #[test]
    fn a_border_below_three_to_one_is_refused_in_the_dark_too() {
        let mut broken = system();
        broken["dark"]["line"] = json!("#1c2826");
        broken["dark"]["paper"] = json!("#0e1614");
        let found = findings("", broken);
        assert!(
            found.starts_with("暗：line と paper のコントラストが 1."),
            "{found}"
        );
        assert!(found.contains("部品の境界の下限 3:1"), "{found}");
    }

    #[test]
    fn a_compared_value_must_be_a_color() {
        let mut broken = system();
        broken["contrast"][0]["foreground"] = json!("space");
        assert!(findings("", broken).contains("色として読めない"));
    }

    #[test]
    fn a_compared_token_must_exist() {
        let mut broken = system();
        broken["contrast"][0]["background"] = json!("nope");
        assert_eq!(findings("", broken), "contrast の nope が tokens に無い");
    }

    #[test]
    fn the_catalog_css_uses_tokens_only() {
        assert!(findings("b{color:#ff0000}", system()).contains("色の直値"));
        assert!(findings("b{color:var(--nope)}", system()).contains("--nope"));
        assert_eq!(
            findings(
                "/* #ff0000 は書かない */b{--local:1px;margin:var(--local)}",
                system()
            ),
            ""
        );
    }
}
