//! 当たる rule を集める。rule はどれも AI（semantic-agent-schema を打ったセッション）が判定し、semantic-agent-schema validate --answer で渡す。
//!
//! rule は x-prompt.rules にある。欄の rule は欄に、x- の rule はその x- の値の形（Abstract の $defs）に書く。
//! 当たるのは、対象の文書にその欄や x- が実際にあるときだけ。Abstract の $defs の中の rule も、$ref でたどって当てる。

use crate::values::{By, RuleResult, Verdict};
use serde_json::Value;
use std::collections::BTreeSet;
use writing_domain::aggregates::{ConcreteSchema, escape};

/// 対象の1か所に当たる rule。
#[derive(Debug, Clone, PartialEq)]
pub struct AppliedRule {
    /// rule の id
    pub id: String,
    /// 対象の中の当たる所（JSON Pointer）
    pub pointer: String,
    /// rule の定義（x-prompt の rules の1件）。文が変われば前の判定は効かない
    pub definition: Value,
}

fn prompt_rules(prompt: Option<&Value>) -> Vec<(String, Value)> {
    prompt
        .and_then(|p| p.get("rules"))
        .and_then(Value::as_array)
        .map(|rs| {
            rs.iter()
                .filter_map(|r| {
                    let id = r.get("id")?.as_str()?.to_string();
                    Some((id, r.clone()))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Concrete に当たる Abstract の rule。Abstract を Concrete の形としてたどり、Concrete にある所ごとに当てる：
/// 根の rule、properties ・ additionalProperties ・ items ・ allOf の先（$ref で指す $defs を含む）の rule、
/// Concrete のどこかで使う x- の値の形（語彙がその x- に指す Abstract の $defs）の rule。
#[must_use]
pub fn rules_for_concrete(
    abstract_doc: &Value,
    vocabulary: Option<&Value>,
    concrete: &ConcreteSchema,
) -> Vec<AppliedRule> {
    let mut out = Vec::new();
    push_rules(abstract_doc, "", &mut out);
    shape(
        abstract_doc,
        abstract_doc,
        &[],
        concrete.document(),
        "",
        &mut out,
        &mut BTreeSet::new(),
    );
    keywords(abstract_doc, vocabulary, concrete.document(), "", &mut out);
    out
}

fn push_rules(schema: &Value, at: &str, out: &mut Vec<AppliedRule>) {
    for (id, definition) in prompt_rules(schema.get("x-prompt")) {
        out.push(AppliedRule {
            id,
            pointer: at.to_string(),
            definition,
        });
    }
}

/// 文書の中の "/$defs/名前"（JSON Pointer）と "名前"（$anchor）を引く。引けなければ None。
fn lookup<'a>(root: &'a Value, fragment: &str) -> Option<&'a Value> {
    if fragment.is_empty() {
        return None;
    }
    if fragment.starts_with('/') {
        return root.pointer(fragment);
    }
    root.get("$defs")?
        .as_object()?
        .values()
        .find(|d| d.get("$anchor").and_then(Value::as_str) == Some(fragment))
}

/// schema が $ref で指していれば、その先と、その先がある文書。指していなければ schema のまま。
/// 文書の中（#…）と、ほかの Concrete（<$id>#/$defs/…）を引く。
fn resolve<'a>(
    root: &'a Value,
    schema: &'a Value,
    others: &'a [(String, Value)],
) -> (&'a Value, &'a Value) {
    let Some(r) = schema.get("$ref").and_then(Value::as_str) else {
        return (root, schema);
    };
    let (base, fragment) = r.split_once('#').unwrap_or((r, ""));
    let doc = if base.is_empty() {
        Some(root)
    } else {
        others.iter().find(|(id, _)| id == base).map(|(_, d)| d)
    };
    let target = doc.and_then(|d| {
        if fragment.is_empty() {
            Some(d)
        } else {
            lookup(d, fragment)
        }
    });
    match (doc, target) {
        (Some(d), Some(t)) => (d, t),
        _ => (root, schema),
    }
}

/// schema を value の形としてたどり、子の所に当たる rule を集める（その所の rule は呼ぶ側が集める）。
/// 深さでは切らない。インスタンスの値は木なので、値をたどる限り終わる。値を進めずに $ref ・ allOf をたどる所は、
/// 同じ値の所に同じスキーマの所を1回だけ当てる（seen。自分を指す $ref でも終わる）。
fn shape(
    root: &Value,
    schema: &Value,
    others: &[(String, Value)],
    value: &Value,
    at: &str,
    out: &mut Vec<AppliedRule>,
    seen: &mut BTreeSet<(usize, String)>,
) {
    let (root, schema) = resolve(root, schema, others);
    if !seen.insert((std::ptr::from_ref(schema) as usize, at.to_string())) {
        return;
    }
    for sub in schema
        .get("allOf")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        shape(root, sub, others, value, at, out, seen);
    }
    if let Some(obj) = value.as_object() {
        let props = schema.get("properties").and_then(Value::as_object);
        for (name, child) in obj {
            if name.starts_with("x-") {
                continue;
            }
            let sub = props
                .and_then(|p| p.get(name))
                .or_else(|| schema.get("additionalProperties").filter(|a| a.is_object()));
            if let Some(sub) = sub {
                let here = format!("{at}/{}", escape(name));
                let (sub_root, sub_r) = resolve(root, sub, others);
                // $ref と横に並べた x-prompt と、$ref の先の x-prompt の両方
                push_rules(sub, &here, out);
                if !std::ptr::eq(sub, sub_r) {
                    push_rules(sub_r, &here, out);
                }
                shape(sub_root, sub_r, others, child, &here, out, seen);
            }
        }
    }
    if let (Some(items), Some(arr)) = (schema.get("items"), value.as_array()) {
        let (items_root, items_r) = resolve(root, items, others);
        for (i, child) in arr.iter().enumerate() {
            let here = format!("{at}/{i}");
            push_rules(items, &here, out);
            if !std::ptr::eq(items, items_r) {
                push_rules(items_r, &here, out);
            }
            shape(items_root, items_r, others, child, &here, out, seen);
        }
    }
}

/// x- の値の形：入れ子に書ける x- は語彙の $ref（abstract#名前）、根だけの x- は Abstract の根の properties の $ref が指す。
fn form_of<'a>(
    abstract_doc: &'a Value,
    vocabulary: Option<&Value>,
    key: &str,
) -> Option<&'a Value> {
    let at = format!("/properties/{}/$ref", escape(key));
    let r = vocabulary
        .and_then(|v| v.pointer(&at))
        .or_else(|| abstract_doc.pointer(&at))?
        .as_str()?;
    lookup(abstract_doc, r.split_once('#').map_or("", |(_, f)| f))
}

/// Concrete のどこかで使う x- ごとに、その x- の値の形（$defs）の rule と、値の中の rule を当てる。
fn keywords(
    abstract_doc: &Value,
    vocabulary: Option<&Value>,
    value: &Value,
    at: &str,
    out: &mut Vec<AppliedRule>,
) {
    match value {
        Value::Object(m) => {
            for (k, child) in m {
                let here = format!("{at}/{}", escape(k));
                if k.starts_with("x-") {
                    if let Some(form) = form_of(abstract_doc, vocabulary, k) {
                        push_rules(form, &here, out);
                        shape(
                            abstract_doc,
                            form,
                            &[],
                            child,
                            &here,
                            out,
                            &mut BTreeSet::new(),
                        );
                    }
                } else {
                    keywords(abstract_doc, vocabulary, child, &here, out);
                }
            }
        }
        Value::Array(a) => {
            for (i, child) in a.iter().enumerate() {
                keywords(abstract_doc, vocabulary, child, &format!("{at}/{i}"), out);
            }
        }
        _ => {}
    }
}

/// インスタンスに当たる Concrete の rule：根と、インスタンスにある欄。Concrete を値の形としてたどり、入れ子の properties ・
/// additionalProperties ・ items ・ allOf と、$ref の先（Concrete の $defs と、ほかの Concrete の $defs）の rule を当てる。
/// others は渡された置き場所から引いた文書（Concrete と $ref の先。$id と文書）。
#[must_use]
pub fn rules_for_instance(
    concrete_doc: &Value,
    instance_doc: &Value,
    others: &[(String, Value)],
) -> Vec<AppliedRule> {
    let mut out = Vec::new();
    push_rules(concrete_doc, "", &mut out);
    shape(
        concrete_doc,
        concrete_doc,
        others,
        instance_doc,
        "",
        &mut out,
        &mut BTreeSet::new(),
    );
    out
}

/// 意味の判定を渡されていないときの rule の結果：not-judged（まだ判定していない）。
#[must_use]
pub fn unjudged(rule: &AppliedRule) -> RuleResult {
    RuleResult::new(
        &rule.id,
        &rule.pointer,
        By::Ai,
        Verdict::NotJudged,
        "意味の判定を渡していない（semantic-agent-schema validate --semantic で手順を受け取り、判定を --answer で渡す）",
    )
}

#[cfg(test)]
mod tests {
    use super::{rules_for_concrete, rules_for_instance, unjudged};
    use crate::values::Verdict;
    use serde_json::json;
    use writing_domain::aggregates::ConcreteSchema;

    #[test]
    fn rules_follow_present_fields_only() {
        let concrete = json!({
            "x-prompt": {"rules": [{"id": "root"}]},
            "properties": {
                "goal": {"x-prompt": {"rules": [{"id": "goal-rule"}]}},
                "absent": {"x-prompt": {"rules": [{"id": "absent-rule"}]}}
            }
        });
        let rules = rules_for_instance(&concrete, &json!({"goal": "x"}), &[]);
        let ids: Vec<&str> = rules.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["root", "goal-rule"]);
        assert_eq!(rules[1].pointer, "/goal");
        assert_eq!(unjudged(&rules[1]).verdict(), Verdict::NotJudged);
    }

    #[test]
    fn instance_rules_follow_ref_into_defs_and_other_concretes() {
        let other = json!({"$id": "urn:example:t:person:v1", "$defs": {"person": {"properties": {
            "name": {"x-prompt": {"rules": [{"id": "name-rule"}]}}}}}});
        let concrete = json!({
            "$defs": {"step": {"properties": {"action": {"x-prompt": {"rules": [{"id": "action-rule"}]}}}}},
            "properties": {
                "steps": {"type": "array", "items": {"$ref": "#/$defs/step"}},
                "owner": {"$ref": "urn:example:t:person:v1#/$defs/person"}
            }
        });
        let others = vec![("urn:example:t:person:v1".to_string(), other)];
        let doc = json!({"steps": [{"action": "a"}, {"action": "b"}], "owner": {"name": "n"}});
        let mut got: Vec<(String, String)> = rules_for_instance(&concrete, &doc, &others)
            .into_iter()
            .map(|r| (r.id, r.pointer))
            .collect();
        got.sort();
        assert_eq!(
            got,
            [
                ("action-rule".to_string(), "/steps/0/action".to_string()),
                ("action-rule".to_string(), "/steps/1/action".to_string()),
                ("name-rule".to_string(), "/owner/name".to_string()),
            ]
        );
    }

    #[test]
    fn abstract_rules_inside_defs_reach_the_concrete() {
        // Abstract の $defs の中の rule（欄の title、rule の欄）も、$ref と x- の値の形をたどって当たる
        let abstract_doc = json!({
            "x-prompt": {"rules": [{"id": "root-rule"}]},
            "properties": {"properties": {"$ref": "#/$defs/fields"}},
            "$defs": {
                "fields": {"additionalProperties": {"$ref": "#/$defs/field"}},
                "field": {"properties": {"title": {"x-prompt": {"rules": [{"id": "title-rule"}]}}}},
                "prompt": {"$anchor": "prompt", "properties": {"rules": {"items": {"$ref": "#/$defs/rule"}}}},
                "rule": {"properties": {"why": {"$ref": "#/$defs/sentence", "x-prompt": {"rules": [{"id": "why-rule"}]}}}},
                "sentence": {"type": "string"}
            }
        });
        let vocabulary = json!({"properties": {"x-prompt": {"$ref": "urn:semantic-agent-schema:published-language:abstract:v1#prompt"}}});
        let concrete = ConcreteSchema::read(&json!({
            "$schema": "urn:semantic-agent-schema:published-language:abstract:v1", "$id": "urn:example:a:v1",
            "properties": {"goal": {"title": "目的", "x-prompt": {"rules": [{"id": "r", "why": "w"}]}}}
        }))
        .unwrap();
        let got: Vec<(String, String)> =
            rules_for_concrete(&abstract_doc, Some(&vocabulary), &concrete)
                .into_iter()
                .map(|r| (r.id, r.pointer))
                .collect();
        assert!(got.contains(&("root-rule".into(), String::new())));
        assert!(got.contains(&("title-rule".into(), "/properties/goal/title".into())));
        assert!(got.contains(&(
            "why-rule".into(),
            "/properties/goal/x-prompt/rules/0/why".into()
        )));
    }
}
