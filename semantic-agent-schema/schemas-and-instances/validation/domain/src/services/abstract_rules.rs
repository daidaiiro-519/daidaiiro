//! Concrete に当たる Abstract の機械の rule のうち、Concrete の文書だけで決まるもの（instance-schema-name ・ unique-id ・
//! human-entry-only）を判定する。JSON Schema の構文で書けることは Abstract の JSON Schema が見るので、ここが見るのは
//! JSON Schema で書けないことだけ（欄どうしの値の一致 ・ 配列の中の id の重なり ・ 書く前に断る振る舞い）。
//! 集約 Concrete は writing-domain にあり、ここはそれを読んで rule に照らす。

use crate::values::{By, RuleResult, Verdict};
use serde_json::Value;
use std::collections::BTreeMap;
use writing_domain::aggregates::ConcreteSchema;

/// Concrete の文書だけで決まる機械の rule を判定する。知らない id は None。
#[must_use]
pub fn judge_concrete(c: &ConcreteSchema, rule: &str, pointer: &str) -> Option<Vec<RuleResult>> {
    match rule {
        "instance-schema-name" => Some(vec![instance_schema_name(c, pointer)]),
        "unique-id" => Some(unique_id(c, pointer)),
        // 人が決める欄を、親セッションのある呼び出しが変えようとすると、SAS は書く前に断る（human_field）
        "human-entry-only" => Some(vec![RuleResult::new(
            rule,
            pointer,
            By::Machine,
            Verdict::Verified,
            "親セッションのある呼び出しがこの欄を変えようとすると、SAS は書く前に断る（human_field）",
        )]),
        _ => None,
    }
}

/// properties.$schema の const が $id と同じか。
fn instance_schema_name(c: &ConcreteSchema, pointer: &str) -> RuleResult {
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
    let at = if pointer.is_empty() {
        "/properties/$schema/const"
    } else {
        pointer
    };
    RuleResult::new("instance-schema-name", at, By::Machine, verdict, &reason)
}

/// 当たる所（rule の id）の id が、Concrete の中のほかの rule と重ならないか。所が空なら Concrete 全体を見る。
fn unique_id(c: &ConcreteSchema, pointer: &str) -> Vec<RuleResult> {
    let mut seen: BTreeMap<String, Vec<String>> = BTreeMap::new();
    collect_ids(c.document(), "", &mut seen);
    let mine = c.document().pointer(pointer).and_then(Value::as_str);
    let dup: Vec<(&String, &Vec<String>)> = seen
        .iter()
        .filter(|(id, at)| at.len() > 1 && mine.is_none_or(|m| m == id.as_str()))
        .collect();
    if dup.is_empty() {
        return vec![RuleResult::new(
            "unique-id",
            pointer,
            By::Machine,
            Verdict::Verified,
            "rule の id はほかと重ならない",
        )];
    }
    dup.into_iter()
        .map(|(id, at)| {
            RuleResult::new(
                "unique-id",
                if pointer.is_empty() { &at[1] } else { pointer },
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
                .and_then(|p| p.get("write"))
                .and_then(Value::as_array)
            {
                for (i, r) in rules.iter().enumerate() {
                    if let Some(id) = r.get("id").and_then(Value::as_str) {
                        out.entry(id.to_string())
                            .or_default()
                            .push(format!("{at}/x-prompt/write/{i}/id"));
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
    use super::judge_concrete;
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
        assert_eq!(
            judge_concrete(&ok, "instance-schema-name", "").unwrap()[0].verdict(),
            Verdict::Verified
        );
        assert_eq!(
            judge_concrete(&ng, "instance-schema-name", "").unwrap()[0].verdict(),
            Verdict::Violated
        );
    }

    #[test]
    fn rule_ids_must_not_repeat() {
        let c = concrete(
            json!({"$schema": "urn:semantic-agent-schema:published-language:abstract:v1",
            "$id": "urn:example:a:v1", "properties": {
                "a": {"x-prompt": {"write": [{"id": "r1"}]}},
                "b": {"x-prompt": {"write": [{"id": "r1"}, {"id": "r2"}]}}}}),
        );
        let r = judge_concrete(&c, "unique-id", "").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].verdict(), Verdict::Violated);
        assert_eq!(r[0].pointer(), "/properties/b/x-prompt/write/0/id");
        let here = judge_concrete(&c, "unique-id", "/properties/a/x-prompt/write/0/id").unwrap();
        assert_eq!(here[0].verdict(), Verdict::Violated);
        let other = judge_concrete(&c, "unique-id", "/properties/b/x-prompt/write/1/id").unwrap();
        assert_eq!(other[0].verdict(), Verdict::Verified);
    }

    #[test]
    fn unknown_rule_is_none() {
        let c = concrete(
            json!({"$schema": "urn:semantic-agent-schema:published-language:abstract:v1", "$id": "urn:example:a:v1"}),
        );
        assert!(judge_concrete(&c, "closed-list", "").is_none());
    }
}
