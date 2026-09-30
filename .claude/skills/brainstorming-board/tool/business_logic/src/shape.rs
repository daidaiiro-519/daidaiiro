// SPDX-License-Identifier: MIT
//! 形の契約を当てる。**契約は1か所からしか読まない。**
//!
//! **並びは経路の順である** ── 検出の順が実行ごとに変わると、差分が毎回出る。

use std::path::Path;

use serde_json::Value;

/// 契約を当て、食い違いを並べる。
#[must_use]
pub fn against(schema_path: &Path, instance: &Value, head: &str) -> Vec<String> {
    let Ok(body) = std::fs::read_to_string(schema_path) else {
        return vec![format!(
            "{head}: 契約を読めない ── {}",
            schema_path.display()
        )];
    };
    let Ok(schema) = serde_json::from_str::<Value>(&body) else {
        return vec![format!("{head}: 契約が JSON として読めない")];
    };
    let built = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&schema);
    let Ok(validator) = built else {
        return vec![format!("{head}: 契約がスキーマとして無効である")];
    };
    let mut found: Vec<String> = validator
        .iter_errors(instance)
        .map(|e| {
            let at = e.instance_path.to_string();
            format!("{head}: {} ── {e}", at.trim_start_matches('/'))
        })
        .collect();
    found.sort();
    found
}
