//! 参照の先：Concrete の x-relationship の to が引ける Concrete か（target-exists）と、
//! インスタンスの x-relationship の欄の値が、指す先の Concrete に従うインスタンスの id として在るか（relationship-target）。
//! 削除のために、対象を指している所（参照元）を探す。

use crate::aggregates::escape;
use crate::values::{By, RuleResult, Verdict};
use serde_json::Value;
use std::collections::BTreeSet;

/// Concrete の中の x-relationship の to を、引ける Concrete の $id（閉じた一覧の版を含む）に照らす。
#[must_use]
pub fn check_targets(concrete_doc: &Value, known: &BTreeSet<String>) -> Vec<RuleResult> {
    let mut found = Vec::new();
    collect_targets(concrete_doc, "", &mut found);
    found
        .into_iter()
        .map(|(at, to)| {
            if known.contains(&to) {
                RuleResult::new(
                    "target-exists",
                    &at,
                    By::Machine,
                    Verdict::Verified,
                    &format!("{to} を引ける"),
                )
            } else {
                RuleResult::new(
                    "target-exists",
                    &at,
                    By::Machine,
                    Verdict::Violated,
                    &format!("{to} を引けない"),
                )
            }
        })
        .collect()
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

/// インスタンスの x-relationship の欄を、在るインスタンス（Concrete の $id と id の組）に照らす。
#[must_use]
pub fn check_relationships(
    concrete_doc: &Value,
    instance_doc: &Value,
    known: &BTreeSet<(String, String)>,
) -> Vec<RuleResult> {
    let mut out = Vec::new();
    walk(concrete_doc, instance_doc, "", known, &mut out);
    out
}

fn walk(
    schema: &Value,
    value: &Value,
    at: &str,
    known: &BTreeSet<(String, String)>,
    out: &mut Vec<RuleResult>,
) {
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
        let to = field
            .get("x-relationship")
            .or_else(|| field.get("items").and_then(|i| i.get("x-relationship")))
            .and_then(|r| r.get("to"))
            .and_then(Value::as_str);
        if let Some(to) = to {
            let targets: Vec<(String, String)> = match child {
                Value::String(s) => vec![(here.clone(), s.clone())],
                Value::Array(a) => a
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| v.as_str().map(|s| (format!("{here}/{i}"), s.to_string())))
                    .collect(),
                _ => Vec::new(),
            };
            for (p, id) in targets {
                let found = known.contains(&(to.to_string(), id.clone()));
                out.push(RuleResult::new(
                    "relationship-target",
                    &p,
                    By::Machine,
                    if found {
                        Verdict::Verified
                    } else {
                        Verdict::Violated
                    },
                    &if found {
                        format!("{id} は {to} のインスタンスとして在る")
                    } else {
                        format!("{id} という {to} のインスタンスが無い")
                    },
                ));
            }
        } else {
            walk(field, child, &here, known, out);
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::{check_relationships, check_targets, references_to, relationship_targets};
    use crate::values::Verdict;
    use serde_json::json;
    use std::collections::BTreeSet;

    #[test]
    fn missing_target_is_violated() {
        let concrete = json!({"properties": {"calls": {"type": "array", "items": {"type": "string", "x-relationship": {"to": "urn:semantic-agent-schema:t:uc:v1"}}}}});
        let known: BTreeSet<(String, String)> = [(
            "urn:semantic-agent-schema:t:uc:v1".to_string(),
            "U1".to_string(),
        )]
        .into();
        let r = check_relationships(&concrete, &json!({"calls": ["U1", "U9"]}), &known);
        assert_eq!(r[0].verdict(), Verdict::Verified);
        assert_eq!(r[1].verdict(), Verdict::Violated);
        assert_eq!(r[1].pointer(), "/calls/1");
    }

    #[test]
    fn unknown_target_concrete_is_violated() {
        let concrete = json!({"properties": {"a": {"x-relationship": {"to": "urn:semantic-agent-schema:t:x:v1"}}}});
        let r = check_targets(&concrete, &BTreeSet::new());
        assert_eq!(r[0].verdict(), Verdict::Violated);
        assert_eq!(r[0].pointer(), "/properties/a/x-relationship/to");
    }

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
