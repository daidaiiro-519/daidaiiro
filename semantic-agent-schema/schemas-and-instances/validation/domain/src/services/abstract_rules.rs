//! Concrete の組み込みの検査（instance-schema-name ・ unique-id）。JSON Schema で書けない Concrete の決まり2つ
//! （欄どうしの値の一致 ・ 文書のどこかの rule の id の重なり）を、Concrete の検証のたびに確かめる。
//! 集約 Concrete は writing-domain にあり、ここはそれを読んで rule に照らす。

use crate::values::{By, RuleResult, Verdict};
use serde_json::Value;
use std::collections::BTreeMap;
use writing_domain::aggregates::ConcreteSchema;

/// Concrete の組み込みの検査2つの結果。
#[must_use]
pub fn builtin_checks(c: &ConcreteSchema) -> Vec<RuleResult> {
    let mut out = vec![instance_schema_name(c)];
    out.extend(unique_id(c));
    out
}

/// properties.$schema の const が $id と同じか。
fn instance_schema_name(c: &ConcreteSchema) -> RuleResult {
    let doc = c.document();
    let id = doc.get("$id").and_then(Value::as_str);
    let konst = doc
        .pointer("/properties/$schema/const")
        .and_then(Value::as_str);
    let (verdict, reason) = match (id, konst) {
        (Some(i), Some(k)) if i == k => (
            Verdict::Verified,
            "properties.$schema の const が $id と同じ".to_string(),
        ),
        (Some(i), Some(k)) => (
            Verdict::Violated,
            format!("properties.$schema の const（{k}）が $id（{i}）と違う"),
        ),
        _ => (
            Verdict::Violated,
            "$id か properties.$schema の const が無い".to_string(),
        ),
    };
    RuleResult::new(
        "instance-schema-name",
        "/properties/$schema/const",
        By::Machine,
        verdict,
        &reason,
    )
}

/// rule の id が、Concrete のどこ（$defs の中も）でもほかの rule と重ならないか。
fn unique_id(c: &ConcreteSchema) -> Vec<RuleResult> {
    let mut seen: BTreeMap<String, Vec<String>> = BTreeMap::new();
    collect_ids(c.document(), "", &mut seen);
    let dup: Vec<(&String, &Vec<String>)> = seen.iter().filter(|(_, at)| at.len() > 1).collect();
    if dup.is_empty() {
        return vec![RuleResult::new(
            "unique-id",
            "",
            By::Machine,
            Verdict::Verified,
            "rule の id はほかと重ならない",
        )];
    }
    dup.into_iter()
        .map(|(id, at)| {
            RuleResult::new(
                "unique-id",
                &at[1],
                By::Machine,
                Verdict::Violated,
                &format!(
                    "rule の id {id} が {} か所にある（{}）",
                    at.len(),
                    at.join(" ・ ")
                ),
            )
        })
        .collect()
}

fn collect_ids(v: &Value, at: &str, out: &mut BTreeMap<String, Vec<String>>) {
    match v {
        Value::Object(m) => {
            if let Some(rules) = m
                .get("x-prompt")
                .and_then(|p| p.get("rules"))
                .and_then(Value::as_array)
            {
                for (i, r) in rules.iter().enumerate() {
                    if let Some(id) = r.get("id").and_then(Value::as_str) {
                        out.entry(id.to_string())
                            .or_default()
                            .push(format!("{at}/x-prompt/rules/{i}/id"));
                    }
                }
            }
            for (k, child) in m {
                if k != "x-prompt" {
                    collect_ids(
                        child,
                        &format!("{at}/{}", k.replace('~', "~0").replace('/', "~1")),
                        out,
                    );
                }
            }
        }
        Value::Array(a) => {
            for (i, child) in a.iter().enumerate() {
                collect_ids(child, &format!("{at}/{i}"), out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::builtin_checks;
    use crate::values::Verdict;
    use serde_json::json;
    use writing_domain::aggregates::ConcreteSchema;

    fn concrete(v: serde_json::Value) -> ConcreteSchema {
        ConcreteSchema::read(&v).unwrap()
    }

    #[test]
    fn schema_const_must_equal_id() {
        let ok = concrete(
            json!({"$schema": "urn:semantic-agent-schema:published-language:abstract:v1",
            "$id": "urn:example:a:v1", "properties": {"$schema": {"const": "urn:example:a:v1"}}}),
        );
        let ng = concrete(
            json!({"$schema": "urn:semantic-agent-schema:published-language:abstract:v1",
            "$id": "urn:example:a:v1", "properties": {"$schema": {"const": "urn:example:b:v1"}}}),
        );
        assert_eq!(builtin_checks(&ok)[0].verdict(), Verdict::Verified);
        assert_eq!(builtin_checks(&ng)[0].verdict(), Verdict::Violated);
    }

    #[test]
    fn rule_ids_must_not_repeat() {
        let c = concrete(
            json!({"$schema": "urn:semantic-agent-schema:published-language:abstract:v1",
            "$id": "urn:example:a:v1", "properties": {
                "a": {"x-prompt": {"rules": [{"id": "r1"}]}},
                "b": {"x-prompt": {"rules": [{"id": "r1"}, {"id": "r2"}]}}}}),
        );
        let r: Vec<_> = builtin_checks(&c).into_iter().skip(1).collect();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].verdict(), Verdict::Violated);
        assert_eq!(r[0].pointer(), "/properties/b/x-prompt/rules/0/id");
    }
}
