//! 参照：Concrete の x-relationship の to と、インスタンスの中で別のインスタンスを指している所（参照元）。
//! 削除のときに、対象を指している所を探すのに使う。

use crate::aggregates::escape;
use serde_json::Value;

/// Concrete の中の x-relationship の to（欄の所と、指す先の Concrete の $id）。
/// Concrete を消すとき、その Concrete を指している別の Concrete を探すのに使う。
#[must_use]
pub fn relationship_targets(concrete_doc: &Value) -> Vec<(String, String)> {
    let mut found = Vec::new();
    collect_targets(concrete_doc, "", &mut found);
    found
}

/// インスタンスの中で、to の Concrete に従うインスタンス id を指している欄の所（配列なら要素の所）。
/// concrete_doc はそのインスタンスが従う Concrete。インスタンスを消すとき、参照元の記載を探すのに使う。
#[must_use]
pub fn references_to(
    concrete_doc: &Value,
    instance_doc: &Value,
    to: &str,
    id: &str,
) -> Vec<String> {
    let mut out = Vec::new();
    find_refs(concrete_doc, instance_doc, "", to, id, &mut out);
    out
}

fn find_refs(schema: &Value, value: &Value, at: &str, to: &str, id: &str, out: &mut Vec<String>) {
    let Some(props) = schema.get("properties").and_then(Value::as_object) else {
        return;
    };
    let Some(obj) = value.as_object() else {
        return;
    };
    for (name, field) in props {
        let Some(child) = obj.get(name) else {
            continue;
        };
        let here = format!("{at}/{}", escape(name));
        let points = field
            .get("x-relationship")
            .or_else(|| field.get("items").and_then(|i| i.get("x-relationship")))
            .and_then(|r| r.get("to"))
            .and_then(Value::as_str);
        match points {
            Some(t) if t == to => match child {
                Value::String(s) if s == id => out.push(here),
                Value::Array(a) => out.extend(
                    a.iter()
                        .enumerate()
                        .filter(|(_, v)| v.as_str() == Some(id))
                        .map(|(i, _)| format!("{here}/{i}")),
                ),
                _ => {}
            },
            Some(_) => {}
            None => find_refs(field, child, &here, to, id, out),
        }
    }
}

fn collect_targets(v: &Value, at: &str, out: &mut Vec<(String, String)>) {
    match v {
        Value::Object(m) => {
            for (k, child) in m {
                let here = format!("{at}/{}", escape(k));
                if k == "x-relationship" {
                    if let Some(to) = child.get("to").and_then(Value::as_str) {
                        out.push((format!("{here}/to"), to.to_string()));
                    }
                } else if k != "x-extensions" {
                    collect_targets(child, &here, out);
                }
            }
        }
        Value::Array(a) => {
            for (i, child) in a.iter().enumerate() {
                collect_targets(child, &format!("{at}/{i}"), out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{references_to, relationship_targets};
    use serde_json::json;

    #[test]
    fn references_to_finds_scalar_and_array_items() {
        let concrete = json!({"properties": {
            "calls": {"type": "array", "items": {"type": "string", "x-relationship": {"to": "urn:semantic-agent-schema:t:uc:v1"}}},
            "parent": {"type": "string", "x-relationship": {"to": "urn:semantic-agent-schema:t:uc:v1"}},
            "owner": {"type": "string", "x-relationship": {"to": "urn:semantic-agent-schema:t:actor:v1"}}}});
        let doc = json!({"calls": ["U1", "U2", "U1"], "parent": "U1", "owner": "U1"});
        assert_eq!(
            references_to(&concrete, &doc, "urn:semantic-agent-schema:t:uc:v1", "U1"),
            vec!["/calls/0", "/calls/2", "/parent"]
        );
        assert!(
            references_to(&concrete, &doc, "urn:semantic-agent-schema:t:uc:v1", "U9").is_empty()
        );
    }

    #[test]
    fn relationship_targets_lists_every_to() {
        let concrete = json!({"properties": {"a": {"x-relationship": {"to": "urn:x"}}, "b": {"items": {"x-relationship": {"to": "urn:y"}}}}});
        assert_eq!(
            relationship_targets(&concrete),
            vec![
                (
                    "/properties/a/x-relationship/to".to_string(),
                    "urn:x".to_string()
                ),
                (
                    "/properties/b/items/x-relationship/to".to_string(),
                    "urn:y".to_string()
                )
            ]
        );
    }
}
