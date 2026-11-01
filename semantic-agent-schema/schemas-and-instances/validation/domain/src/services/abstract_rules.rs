//! Concrete に当たる Abstract の機械の rule のうち、Concrete の文書だけで決まる3つ（closed-list ・ no-abstract-names ・
//! abstract-keywords-closed）を判定する。集約 Concrete は writing-domain にあり、ここはそれを読んで rule に照らす。

use crate::values::{By, RuleResult, Verdict};
use std::collections::BTreeSet;
use writing_domain::aggregates::{ConcreteSchema, escape};

/// Concrete の文書だけで決まる機械の rule を判定する。知らない id は None。
#[must_use]
pub fn judge_concrete(
    c: &ConcreteSchema,
    rule: &str,
    pointer: &str,
    abstract_names: &BTreeSet<String>,
) -> Option<Vec<RuleResult>> {
    match rule {
        "closed-list" => Some(vec![RuleResult::new(
            rule,
            pointer,
            By::Machine,
            Verdict::Verified,
            "申告した版は閉じた一覧にある",
        )]),
        "no-abstract-names" => Some(no_abstract_names(c, abstract_names)),
        "abstract-keywords-closed" => Some(keywords_closed(c, abstract_names)),
        _ => None,
    }
}

fn no_abstract_names(c: &ConcreteSchema, abstract_names: &BTreeSet<String>) -> Vec<RuleResult> {
    let clash: Vec<RuleResult> = c
        .own_extensions()
        .intersection(abstract_names)
        .map(|name| {
            RuleResult::new(
                "no-abstract-names",
                &format!("/x-extensions/{}", escape(name)),
                By::Machine,
                Verdict::Violated,
                &format!("{name} は Abstract の本体がすでに持つ名前"),
            )
        })
        .collect();
    if clash.is_empty() {
        vec![RuleResult::new(
            "no-abstract-names",
            "/x-extensions",
            By::Machine,
            Verdict::Verified,
            "本体と同じ名前を足していない",
        )]
    } else {
        clash
    }
}

fn keywords_closed(c: &ConcreteSchema, abstract_names: &BTreeSet<String>) -> Vec<RuleResult> {
    let own = c.own_extensions();
    let unknown: Vec<RuleResult> = c
        .used_keywords()
        .into_iter()
        .filter(|(k, _)| !abstract_names.contains(k) && !own.contains(k))
        .map(|(k, at)| {
            RuleResult::new(
                "abstract-keywords-closed",
                &at,
                By::Machine,
                Verdict::Violated,
                &format!("{k} は本体にも、このスキーマの x-extensions にも無い"),
            )
        })
        .collect();
    if unknown.is_empty() {
        vec![RuleResult::new(
            "abstract-keywords-closed",
            "",
            By::Machine,
            Verdict::Verified,
            "使っている x- はどれも定義がある",
        )]
    } else {
        unknown
    }
}

#[cfg(test)]
mod tests {
    use super::judge_concrete;
    use crate::values::Verdict;
    use serde_json::json;
    use std::collections::BTreeSet;
    use writing_domain::aggregates::ConcreteSchema;

    fn names() -> BTreeSet<String> {
        ["x-extensions", "x-prompt", "x-relationship"]
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn clash_with_abstract_name_is_violated() {
        let c = ConcreteSchema::read(&json!({
            "$schema": "urn:semantic-agent-schema:published-language:abstract:v1",
            "x-extensions": {"x-prompt": {}}
        }))
        .unwrap();
        let r = judge_concrete(&c, "no-abstract-names", "", &names()).unwrap();
        assert_eq!(r[0].verdict(), Verdict::Violated);
        assert_eq!(r[0].pointer(), "/x-extensions/x-prompt");
    }

    #[test]
    fn unknown_keyword_is_violated_and_own_is_allowed() {
        let c = ConcreteSchema::read(&json!({
            "$schema": "urn:semantic-agent-schema:published-language:abstract:v1",
            "x-extensions": {"x-mine": {}},
            "properties": {"a": {"x-mine": 1, "x-other": 2}}
        }))
        .unwrap();
        let r = judge_concrete(&c, "abstract-keywords-closed", "", &names()).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].pointer(), "/properties/a/x-other");
    }
}
