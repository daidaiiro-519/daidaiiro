//! JSON Schema の部分集合 ── **7語だけを持つ。** 外の crate を引き込まない
//! （`jsonschema` は78件、`boon` は72件を引き込む。実測 2026-09-26）。
use serde_json::Value;

/// 形を検査する。検出したものを `out` へ並べる。
pub fn validate(schema: &Value, v: &Value, at: &str, out: &mut Vec<String>) {
    let Some(s) = schema.as_object() else { return };
    if let Some(t) = s.get("type").and_then(|x| x.as_str()) {
        let 合う = match t {
            "object" => v.is_object(),
            "array" => v.is_array(),
            "string" => v.is_string(),
            "number" => v.is_number(),
            "integer" => v.is_i64() || v.is_u64(),
            "boolean" => v.is_boolean(),
            "null" => v.is_null(),
            _ => true,
        };
        if !合う {
            out.push(format!("{at}: 型が {t} ではない"));
            return;
        }
    }
    if let Some(e) = s.get("enum").and_then(|x| x.as_array()) {
        if !e.contains(v) {
            out.push(format!("{at}: 値が一覧に無い"));
        }
    }
    if let Some(req) = s.get("required").and_then(|x| x.as_array()) {
        for k in req.iter().filter_map(|x| x.as_str()) {
            if v.get(k).is_none() {
                out.push(format!("{at}: 欄 {k} が無い"));
            }
        }
    }
    if let Some(p) = s.get("properties").and_then(|x| x.as_object()) {
        for (k, sub) in p {
            if let Some(child) = v.get(k) {
                validate(sub, child, &format!("{at}/{k}"), out);
            }
        }
        if s.get("additionalProperties") == Some(&Value::Bool(false)) {
            if let Some(o) = v.as_object() {
                for k in o.keys().filter(|k| !p.contains_key(*k)) {
                    out.push(format!("{at}: 知らない欄 {k}"));
                }
            }
        }
    }
    if let Some(n) = s.get("minItems").and_then(|x| x.as_u64()) {
        if v.as_array().is_some_and(|a| (a.len() as u64) < n) {
            out.push(format!("{at}: 並びが {n} 件に満たない"));
        }
    }
    if let (Some(sub), Some(a)) = (s.get("items"), v.as_array()) {
        for (i, x) in a.iter().enumerate() {
            validate(sub, x, &format!("{at}[{i}]"), out);
        }
    }
}
