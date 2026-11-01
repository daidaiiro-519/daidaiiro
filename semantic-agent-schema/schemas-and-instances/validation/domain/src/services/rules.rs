//! 当たる rule を集め、機械で判定できないものを、by ごとに決まった値にする。
//!
//! rule は x-prompt.write（欄に書く）と、x-extensions の prompt.write（x- の名前に書く）にある。
//! 当たるのは、対象の文書にその欄や x- が実際にあるときだけ。

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

/// Concrete に当たる Abstract の rule：根、Concrete にある根の欄、Concrete が使う x-。
/// $defs の中の rule（入れ子の欄ごとに当たるもの）は、まだ集めない（探索の記録 step-01）。
#[must_use]
pub fn rules_for_concrete(abstract_doc: &Value, concrete: &ConcreteSchema) -> Vec<AppliedRule> {
    let mut out = Vec::new();
    for (id, by, definition) in write_rules(abstract_doc.get("x-prompt")) {
        out.push(AppliedRule {
            id,
            by,
            pointer: String::new(),
            definition: definition.clone(),
        });
    }
    if let Some(props) = abstract_doc.get("properties").and_then(Value::as_object) {
        for (name, field) in props {
            if concrete.document().get(name).is_some() {
                for (id, by, definition) in write_rules(field.get("x-prompt")) {
                    out.push(AppliedRule {
                        id,
                        by,
                        pointer: format!("/{}", escape(name)),
                        definition: definition.clone(),
                    });
                }
            }
        }
    }
    if let Some(exts) = abstract_doc.get("x-extensions").and_then(Value::as_object) {
        for (name, def) in exts {
            if concrete.uses(name) {
                for (id, by, definition) in write_rules(def.get("prompt")) {
                    out.push(AppliedRule {
                        id,
                        by,
                        pointer: format!("/{}", escape(name)),
                        definition: definition.clone(),
                    });
                }
            }
        }
    }
    out
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
    use super::{judge_others, rules_for_instance};
    use crate::values::{By, Verdict};
    use serde_json::json;

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
}
