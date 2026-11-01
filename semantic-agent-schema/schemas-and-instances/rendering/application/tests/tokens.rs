//! デザイントークンの種類と CSS（Design Tokens Format Module 2025.10 ・ Color module 2025.10）。

#![allow(clippy::unwrap_used)]

use rendering_application::use_cases::tokens::{css_of, flatten, type_errors};
use serde_json::json;

#[test]
fn values_follow_their_types() {
    let t = flatten(Some(&json!({
        "w": { "$type": "fontWeight", "a": { "$value": "bold" }, "b": { "$value": 450 }, "c": { "$value": "Bold" } },
        "e": { "$type": "cubicBezier", "ok": { "$value": [0.5, 0, 1, 1] }, "bad": { "$value": [2, 0, 1, 1] } },
        "d": { "$type": "duration", "ok": { "$value": { "value": 200, "unit": "ms" } }, "bad": { "$value": { "value": 1, "unit": "min" } } },
        "untyped": { "$value": 3 }
    })));
    let errors: Vec<String> = type_errors(&t).into_iter().map(|(n, _)| n).collect();
    assert_eq!(errors, vec!["d.bad", "e.bad", "untyped", "w.c"]);
}

#[test]
fn an_alias_takes_the_type_of_its_target() {
    let t = flatten(Some(&json!({
        "base": { "$type": "number", "$value": 1.5 },
        "line": { "$value": "{base}" }
    })));
    assert!(type_errors(&t).is_empty());
    assert_eq!(css_of(&t), ":root { --base: 1.5; --line: var(--base); }");
}

#[test]
fn colors_use_color_or_the_named_function() {
    let t = flatten(Some(&json!({
        "c": {
            "$type": "color",
            "a": { "$value": { "colorSpace": "srgb", "components": [1, 0, 1], "hex": "#ff00ff" } },
            "b": { "$value": { "colorSpace": "oklch", "components": [0.7, 0.1, "none"], "alpha": 0.5 } }
        }
    })));
    assert!(type_errors(&t).is_empty());
    assert_eq!(
        css_of(&t),
        ":root { --c-a: color(srgb 1 0 1); --c-b: oklch(0.7 0.1 none / 0.5); }"
    );
}

#[test]
fn composite_tokens_are_not_css() {
    let t = flatten(Some(&json!({
        "shadow": { "$type": "shadow", "$value": { "color": "{c}", "offsetX": { "value": 1, "unit": "px" } } }
    })));
    assert_eq!(css_of(&t), ":root {  }");
}
