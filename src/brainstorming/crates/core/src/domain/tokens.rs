//! Design Tokens 形式（2025.10）の明と暗のファイルを、基盤のデザインシステム（schema-driven/design-system@v1）へ移す。
//! 移す規則は ux-advisor の回答（2026-10-16）のとおりである。
//! パスの . を - に ・ 色は hex の小文字 ・ {a.b} の参照は {a-b} ・ dimension は値と単位をつなぐ ・
//! typography は5つのトークンに分ける ・ $extensions の works.daidaiiro.contrast を contrast へ。

use serde_json::{json, Map, Value};

/// コントラストの組を置く拡張の名前。
pub const CONTRAST: &str = "works.daidaiiro.contrast";

fn dashed(path: &str) -> String {
    path.replace('.', "-")
}

/// {a.b} の参照を {a-b} に直す。
fn reference(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        match rest[open..].find('}') {
            Some(close) => {
                out.push('{');
                out.push_str(&dashed(&rest[open + 1..open + close]));
                out.push('}');
                rest = &rest[open + close + 1..];
            }
            None => {
                out.push_str(&rest[open..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

fn dimension(value: &Value) -> Result<String, String> {
    match value {
        Value::String(text) => Ok(reference(text)),
        Value::Object(object) => {
            let number = object
                .get("value")
                .and_then(Value::as_f64)
                .ok_or("dimension に value が無い")?;
            let unit = object
                .get("unit")
                .and_then(Value::as_str)
                .ok_or("dimension に unit が無い")?;
            Ok(format!("{}{unit}", number_text(number)))
        }
        other => Err(format!("dimension の形が違う：{other}")),
    }
}

fn number_text(number: f64) -> String {
    if number.fract() == 0.0 {
        format!("{}", number as i64)
    } else {
        format!("{number}")
    }
}

fn family(value: &Value) -> Result<String, String> {
    match value {
        Value::String(text) => Ok(reference(text)),
        Value::Array(names) => Ok(names
            .iter()
            .filter_map(Value::as_str)
            .map(|name| {
                if name.contains(' ') {
                    format!("\"{name}\"")
                } else {
                    name.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")),
        other => Err(format!("fontFamily の形が違う：{other}")),
    }
}

/// トークン1つを、名前と値の組（typography は5つ）にする。
fn convert(name: &str, kind: &str, value: &Value) -> Result<Vec<(String, String)>, String> {
    if let Value::String(text) = value {
        return Ok(vec![(name.to_owned(), reference(text))]);
    }
    Ok(match kind {
        "color" => {
            let hex = value
                .get("hex")
                .and_then(Value::as_str)
                .ok_or(format!("{name}：色に hex が無い"))?;
            vec![(name.to_owned(), hex.to_lowercase())]
        }
        "dimension" => vec![(name.to_owned(), dimension(value)?)],
        "fontFamily" => vec![(name.to_owned(), family(value)?)],
        "typography" => {
            let parts = [
                ("fontFamily", "font-family"),
                ("fontSize", "font-size"),
                ("fontWeight", "font-weight"),
                ("letterSpacing", "letter-spacing"),
                ("lineHeight", "line-height"),
            ];
            let mut out = Vec::new();
            for (key, css) in parts {
                let part = value
                    .get(key)
                    .ok_or(format!("{name}：typography に {key} が無い"))?;
                let text = match part {
                    Value::String(text) => reference(text),
                    Value::Object(_) => dimension(part)?,
                    Value::Number(number) => number.to_string(),
                    other => return Err(format!("{name}：{key} の形が違う：{other}")),
                };
                out.push((format!("{name}-{css}"), text));
            }
            out
        }
        "number" | "fontWeight" => vec![(name.to_owned(), value.to_string())],
        other => return Err(format!("{name}：知らない型 {other}")),
    })
}

/// 群をたどってトークンを集める。$ で始まるキーは群の情報なのでたどらない。
fn walk(
    node: &Value,
    path: &mut Vec<String>,
    kind: &str,
    out: &mut Vec<(String, String)>,
) -> Result<(), String> {
    let object = node
        .as_object()
        .ok_or(format!("{}：オブジェクトでない", path.join(".")))?;
    let kind = object
        .get("$type")
        .and_then(Value::as_str)
        .unwrap_or(kind)
        .to_owned();
    if let Some(value) = object.get("$value") {
        return convert(&path.join("-"), &kind, value).map(|pairs| out.extend(pairs));
    }
    for (key, child) in object {
        if key.starts_with('$') {
            continue;
        }
        path.push(key.clone());
        walk(child, path, &kind, out)?;
        path.pop();
    }
    Ok(())
}

fn tokens(file: &Value) -> Result<Map<String, Value>, String> {
    let mut pairs = Vec::new();
    walk(file, &mut Vec::new(), "", &mut pairs)?;
    Ok(pairs
        .into_iter()
        .map(|(name, value)| (name, Value::String(value)))
        .collect())
}

/// 明 light と暗 dark のファイルから、design-system@v1 の値を作る。
pub fn design_system(light: &Value, dark: &Value) -> Result<Value, String> {
    let base = tokens(light)?;
    let night = tokens(dark)?;
    if let Some(unknown) = night.keys().find(|name| !base.contains_key(*name)) {
        return Err(format!("暗のトークン {unknown} が明に無い"));
    }
    let pairs = light
        .pointer(&format!("/$extensions/{CONTRAST}"))
        .and_then(Value::as_array)
        .ok_or(format!("$extensions の {CONTRAST} が無い"))?;
    let contrast: Vec<Value> = pairs
        .iter()
        .map(|pair| {
            json!({
                "foreground": dashed(pair.get("foreground").and_then(Value::as_str).unwrap_or("")),
                "background": dashed(pair.get("background").and_then(Value::as_str).unwrap_or("")),
                "use": pair.get("use").cloned().unwrap_or(Value::Null),
            })
        })
        .collect();
    Ok(json!({
        "$schema": "schema-driven/design-system@v1",
        "scope": ":root",
        "tokens": base,
        "dark": night,
        "contrast": contrast,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn light() -> Value {
        json!({
            "$schema": "brainstorming/tokens@v1",
            "base": {"$type": "color", "teal": {"700": {"$value": {"colorSpace": "srgb", "components": [0, 0, 0], "hex": "#0D5C55"}}}},
            "color": {"$type": "color", "key": {"$value": "{base.teal.700}"}},
            "size": {"$type": "dimension", "body": {"$value": {"value": 15, "unit": "px"}}, "radius": {"$value": {"value": 0.6, "unit": "rem"}}},
            "font": {"$type": "fontFamily", "body": {"$value": ["Zen Kaku Gothic New", "sans-serif"]}},
            "type": {"$type": "typography", "body": {"$value": {"fontFamily": "{font.body}", "fontSize": "{size.body}", "fontWeight": 400, "letterSpacing": {"value": 0, "unit": "px"}, "lineHeight": 1.85}}},
            "$extensions": {CONTRAST: [{"foreground": "color.key", "background": "base.teal.700", "use": "text"}]}
        })
    }

    #[test]
    fn the_rules_of_the_move_are_applied() {
        let dark = json!({"color": {"$type": "color", "key": {"$value": "{base.teal.700}"}}});
        let made = design_system(&light(), &dark).unwrap();
        assert_eq!(made["tokens"]["base-teal-700"], "#0d5c55");
        assert_eq!(made["tokens"]["color-key"], "{base-teal-700}");
        assert_eq!(made["tokens"]["size-body"], "15px");
        assert_eq!(made["tokens"]["size-radius"], "0.6rem");
        assert_eq!(
            made["tokens"]["font-body"],
            "\"Zen Kaku Gothic New\", sans-serif"
        );
        assert_eq!(made["tokens"]["type-body-font-family"], "{font-body}");
        assert_eq!(made["tokens"]["type-body-letter-spacing"], "0px");
        assert_eq!(made["tokens"]["type-body-line-height"], "1.85");
        assert_eq!(made["dark"]["color-key"], "{base-teal-700}");
        assert_eq!(made["contrast"][0]["foreground"], "color-key");
    }

    #[test]
    fn a_dark_token_missing_from_light_is_refused() {
        let dark = json!({"color": {"$type": "color", "other": {"$value": "{base.teal.700}"}}});
        assert_eq!(
            design_system(&light(), &dark),
            Err("暗のトークン color-other が明に無い".to_owned())
        );
    }

    #[test]
    fn missing_contrast_pairs_are_refused() {
        let mut no_pairs = light();
        no_pairs.as_object_mut().map(|o| o.remove("$extensions"));
        assert!(design_system(&no_pairs, &json!({})).is_err());
    }
}
