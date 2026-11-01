//! 当たる rule を集め、機械で判定できないものを、by ごとに決まった値にする。
//!
//! rule は x-prompt.write にある。欄の rule は欄に、x- の rule はその x- の値の形（Abstract の $defs）に書く。
//! 当たるのは、対象の文書にその欄や x- が実際にあるときだけ。Abstract の $defs の中の rule も、$ref でたどって当てる。

use crate::values::{By, RuleResult, Verdict};
use serde_json::Value;
use writing_domain::aggregates::{ConcreteSchema, escape};

/// 対象の1か所に当たる rule。
#[derive(Debug, Clone, PartialEq)]
pub struct AppliedRule {
    /// rule の id
    pub id: String,
    /// 誰が判定するか
    pub by: By,
    /// 対象の中の当たる所（JSON Pointer）
    pub pointer: String,
    /// rule の定義（x-prompt の write の1行）。文が変われば前の判定は効かない
    pub definition: Value,
}

fn write_rules(prompt: Option<&Value>) -> Vec<(String, By, Value)> {
    prompt
        .and_then(|p| p.get("write"))
        .and_then(Value::as_array)
        .map(|rs| {
            rs.iter()
                .filter_map(|r| {
                    let id = r.get("id")?.as_str()?.to_string();
                    let by = By::parse(r.get("by")?.as_str()?)?;
                    Some((id, by, r.clone()))
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
        concrete.document(),
        "",
        &mut out,
        0,
    );
    keywords(abstract_doc, vocabulary, concrete.document(), "", &mut out);
    out
}

fn push_rules(schema: &Value, at: &str, out: &mut Vec<AppliedRule>) {
    for (id, by, definition) in write_rules(schema.get("x-prompt")) {
        out.push(AppliedRule {
            id,
            by,
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

/// schema が文書の中を $ref で指していれば、その先。指していなければ schema のまま。
fn resolve<'a>(root: &'a Value, schema: &'a Value) -> &'a Value {
    schema
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|r| r.strip_prefix('#'))
        .and_then(|f| lookup(root, f))
        .unwrap_or(schema)
}

/// schema を value の形としてたどり、子の所に当たる rule を集める（その所の rule は呼ぶ側が集める）。
fn shape(
    root: &Value,
    schema: &Value,
    value: &Value,
    at: &str,
    out: &mut Vec<AppliedRule>,
    depth: usize,
) {
    if depth > 16 {
        return;
    }
    let schema = resolve(root, schema);
    for sub in schema
        .get("allOf")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        shape(root, sub, value, at, out, depth + 1);
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
                let sub_r = resolve(root, sub);
                // $ref と横に並べた x-prompt と、$ref の先の x-prompt の両方
                push_rules(sub, &here, out);
                if !std::ptr::eq(sub, sub_r) {
                    push_rules(sub_r, &here, out);
                }
                shape(root, sub_r, child, &here, out, depth + 1);
            }
        }
    }
    if let (Some(items), Some(arr)) = (schema.get("items"), value.as_array()) {
        let items_r = resolve(root, items);
        for (i, child) in arr.iter().enumerate() {
            let here = format!("{at}/{i}");
            push_rules(items, &here, out);
            if !std::ptr::eq(items, items_r) {
                push_rules(items_r, &here, out);
            }
            shape(root, items_r, child, &here, out, depth + 1);
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
                        shape(abstract_doc, form, child, &here, out, 0);
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

/// インスタンスに当たる Concrete の rule：根と、インスタンスにある欄（入れ子の properties と items も）。
#[must_use]
pub fn rules_for_instance(concrete_doc: &Value, instance_doc: &Value) -> Vec<AppliedRule> {
    let mut out = Vec::new();
    for (id, by, definition) in write_rules(concrete_doc.get("x-prompt")) {
        out.push(AppliedRule {
            id,
            by,
            pointer: String::new(),
            definition: definition.clone(),
        });
    }
    walk(concrete_doc, instance_doc, "", &mut out);
    out
}

fn walk(schema: &Value, value: &Value, at: &str, out: &mut Vec<AppliedRule>) {
    if let (Some(props), Some(obj)) = (
        schema.get("properties").and_then(Value::as_object),
        value.as_object(),
    ) {
        for (name, field) in props {
            if let Some(child) = obj.get(name) {
                let here = format!("{at}/{}", escape(name));
                for (id, by, definition) in write_rules(field.get("x-prompt")) {
                    out.push(AppliedRule {
                        id,
                        by,
                        pointer: here.clone(),
                        definition: definition.clone(),
                    });
                }
                walk(field, child, &here, out);
            }
        }
    }
    if let (Some(items), Some(arr)) = (schema.get("items"), value.as_array()) {
        for (i, child) in arr.iter().enumerate() {
            walk(items, child, &format!("{at}/{i}"), out);
        }
    }
}

/// 機械の判定が無い rule を、by ごとに決まった値にする。
/// 人は unverified（人が決める。人の判定は使う側のセッションで行う）、AI は unsupported（エージェントはまだつながっていない）、
/// 判定の手続きが無い機械の rule は unsupported。
#[must_use]
pub fn judge_others(rule: &AppliedRule) -> RuleResult {
    let (verdict, reason) = match rule.by {
        By::Human => (Verdict::Unverified, "人が決める"),
        By::Ai => (Verdict::Unsupported, "エージェントがまだつながっていない"),
        By::Machine => (
            Verdict::Unsupported,
            "この rule を判定する手続きが、まだ無い",
        ),
    };
    RuleResult::new(&rule.id, &rule.pointer, rule.by, verdict, reason)
}

#[cfg(test)]
mod tests {
    use super::{judge_others, rules_for_concrete, rules_for_instance};
    use crate::values::{By, Verdict};
    use serde_json::json;
    use writing_domain::aggregates::ConcreteSchema;

    #[test]
    fn rules_follow_present_fields_only() {
        let concrete = json!({
            "x-prompt": {"write": [{"id": "root", "by": "ai"}]},
            "properties": {
                "goal": {"x-prompt": {"write": [{"id": "goal-rule", "by": "human"}]}},
                "absent": {"x-prompt": {"write": [{"id": "absent-rule", "by": "machine"}]}}
            }
        });
        let rules = rules_for_instance(&concrete, &json!({"goal": "x"}));
        let ids: Vec<&str> = rules.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["root", "goal-rule"]);
        assert_eq!(rules[1].pointer, "/goal");
        assert_eq!(rules[1].by, By::Human);
        assert_eq!(judge_others(&rules[1]).verdict(), Verdict::Unverified);
        assert_eq!(judge_others(&rules[0]).verdict(), Verdict::Unsupported);
    }

    #[test]
    fn abstract_rules_inside_defs_reach_the_concrete() {
        // Abstract の $defs の中の rule（欄の title、rule の欄）も、$ref と x- の値の形をたどって当たる
        let abstract_doc = json!({
            "x-prompt": {"write": [{"id": "root-rule", "by": "ai"}]},
            "properties": {"properties": {"$ref": "#/$defs/fields"}},
            "$defs": {
                "fields": {"additionalProperties": {"$ref": "#/$defs/field"}},
                "field": {"properties": {"title": {"x-prompt": {"write": [{"id": "title-rule", "by": "ai"}]}}}},
                "prompt": {"$anchor": "prompt", "properties": {"write": {"items": {"$ref": "#/$defs/rule"}}}},
                "rule": {"properties": {"why": {"$ref": "#/$defs/sentence", "x-prompt": {"write": [{"id": "why-rule", "by": "ai"}]}}}},
                "sentence": {"type": "string"}
            }
        });
        let vocabulary = json!({"properties": {"x-prompt": {"$ref": "urn:semantic-agent-schema:published-language:abstract:v1#prompt"}}});
        let concrete = ConcreteSchema::read(&json!({
            "$schema": "urn:semantic-agent-schema:published-language:abstract:v1", "$id": "urn:example:a:v1",
            "properties": {"goal": {"title": "目的", "x-prompt": {"write": [{"id": "r", "why": "w", "by": "ai"}]}}}
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
            "/properties/goal/x-prompt/write/0/why".into()
        )));
    }
}
