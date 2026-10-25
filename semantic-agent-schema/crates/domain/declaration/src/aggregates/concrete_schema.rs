//! Semantic Concrete Schema。欄は pub(in crate::aggregates::concrete_schema)
pub mod entities;
pub mod factories;

use crate::aggregates::abstract_schema::X_KEYWORDS;
use crate::aggregates::entities::delegate::Delegate;
use crate::values::breach::Breach;
use crate::values::by::By;
use crate::values::patch::{self, Refusal};
use crate::values::term::Term;
use crate::values::urn::Urn;
use entities::rule::Rule;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// Semantic Concrete Schema（セマンティックコンクリートスキーマ）：Abstract の版を1つ申告し、データの形と書き方の rule を持つ契約
#[derive(Debug, Clone, PartialEq)]
pub struct ConcreteSchema {
    pub(in crate::aggregates::concrete_schema) id: Urn,
    pub(in crate::aggregates::concrete_schema) abstract_id: Urn,
    pub(in crate::aggregates::concrete_schema) document: Value,
}

fn escape(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

/// 欄（入れ子を含む）を1つずつ渡す。たどる先は Abstract の field と同じ
fn walk_fields(field: &Value, at: &str, visit: &mut dyn FnMut(&str, &Map<String, Value>)) {
    let Some(map) = field.as_object() else {
        return;
    };
    visit(at, map);
    for key in ["properties", "$defs"] {
        if let Some(children) = map.get(key).and_then(Value::as_object) {
            for (name, child) in children {
                walk_fields(
                    child,
                    &format!("{at}/{}/{}", escape(key), escape(name)),
                    visit,
                );
            }
        }
    }
    for key in ["items", "additionalProperties"] {
        if let Some(child) = map.get(key) {
            walk_fields(child, &format!("{at}/{key}"), visit);
        }
    }
    if let Some(list) = map.get("prefixItems").and_then(Value::as_array) {
        for (n, child) in list.iter().enumerate() {
            walk_fields(child, &format!("{at}/prefixItems/{n}"), visit);
        }
    }
}

impl ConcreteSchema {
    /// 文書から読む。$id と $schema が URN であることだけを見る（形の検証は Validator が行う）。不変条件 I3
    ///
    /// # Errors
    /// $id か $schema が無いか、URN でないとき
    pub fn from_document(document: Value) -> Result<Self, String> {
        let id = document
            .get("$id")
            .and_then(Value::as_str)
            .ok_or("$id が無い")?;
        let abstract_id = document
            .get("$schema")
            .and_then(Value::as_str)
            .ok_or("$schema が無い")?;
        Ok(ConcreteSchema {
            id: Urn::parse(id)?,
            abstract_id: Urn::parse(abstract_id)?,
            document,
        })
    }

    /// 名前（$id）
    pub fn id(&self) -> &Urn {
        &self.id
    }

    /// 申告した Abstract の版（$schema）
    pub fn abstract_id(&self) -> &Urn {
        &self.abstract_id
    }

    /// 文書
    pub fn document(&self) -> &Value {
        &self.document
    }

    /// JSON Patch を適用した Concrete を返す。検査①：外れたら断り、自分は変えない。不変条件 I3
    ///
    /// # Errors
    /// 断ったとき（理由は [`Refusal`]）
    pub fn patched(&self, ops: &Value) -> Result<ConcreteSchema, Refusal> {
        let document = patch::apply(&self.document, ops)?;
        Ok(ConcreteSchema {
            id: self.id.clone(),
            abstract_id: self.abstract_id.clone(),
            document,
        })
    }

    /// 任せる先（x-delegates）
    pub fn delegates(&self) -> Vec<Delegate> {
        self.document
            .get("x-delegates")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .map(|(name, d)| Delegate {
                name: name.clone(),
                kind: d
                    .get("kind")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            })
            .collect()
    }

    /// rule の一覧（根と欄の x-prompt.write）
    pub fn rules(&self) -> Vec<Rule> {
        let mut out = Vec::new();
        walk_fields(&self.document, "", &mut |at, map| {
            let written = map
                .get("x-prompt")
                .and_then(|p| p.get("write"))
                .and_then(Value::as_array);
            for (n, r) in written.into_iter().flatten().enumerate() {
                let text = |k: &str| r.get(k).and_then(Value::as_str).map(str::to_string);
                out.push(Rule {
                    id: text("id").unwrap_or_default(),
                    pointer: format!("{at}/x-prompt/write/{n}"),
                    by: text("by").as_deref().and_then(By::parse),
                    agent: text("agent"),
                    reads: r
                        .get("reads")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect(),
                });
            }
        });
        out
    }

    /// 欄が $ref で指す glossary の term（場所と term）
    pub fn terms(&self) -> Vec<(String, Term)> {
        let mut out = Vec::new();
        walk_fields(&self.document, "", &mut |at, map| {
            if let Some(term) = map
                .get("$ref")
                .and_then(Value::as_str)
                .and_then(Term::from_ref)
            {
                out.push((format!("{at}/$ref"), term));
            }
        });
        out
    }

    /// 形では見られない宣言の誤り：rule の id の重なり ・ 宣言されていない任せる先 ・ 定義の無い x-。不変条件 I4 ・ I6
    pub fn breaches(&self) -> Vec<Breach> {
        let mut out = Vec::new();
        if self.id.kind() == "glossary" {
            out.push(Breach::new("/$id", "種類 glossary は glossary の Abstract のファイルに取ってある（Concrete では使わない）".to_string()));
        }
        let mut first: BTreeMap<String, String> = BTreeMap::new();
        let delegates: BTreeSet<String> = self
            .delegates()
            .iter()
            .map(|d| d.name().to_string())
            .collect();
        for rule in self.rules() {
            if let Some(earlier) = first.get(rule.id()) {
                out.push(Breach::new(
                    rule.pointer(),
                    format!("rule の id「{}」が {earlier} と重なる", rule.id()),
                ));
            } else {
                first.insert(rule.id().to_string(), rule.pointer().to_string());
            }
            if let (Some(By::Ai), Some(agent)) = (rule.by(), rule.agent())
                && !delegates.contains(agent)
            {
                out.push(Breach::new(
                    rule.pointer(),
                    format!("任せる先「{agent}」が x-delegates に無い"),
                ));
            }
        }
        let declared: BTreeSet<String> = self
            .document
            .get("x-extensions")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .map(|(k, _)| k.clone())
            .collect();
        for key in &declared {
            if X_KEYWORDS.contains(&key.as_str()) {
                out.push(Breach::new(
                    &format!("/x-extensions/{}", escape(key)),
                    format!("{key} は Abstract の x- と同じ名前（Concrete の拡張では定義しない）"),
                ));
            }
        }
        walk_fields(&self.document, "", &mut |at, map| {
            for key in map.keys().filter(|k| k.starts_with("x-")) {
                if !X_KEYWORDS.contains(&key.as_str()) && !declared.contains(key) {
                    out.push(Breach::new(
                        &format!("{at}/{}", escape(key)),
                        format!("{key} は Abstract にも x-extensions にも無い"),
                    ));
                }
            }
        });
        out
    }
}

#[cfg(test)]
mod tests {
    use super::ConcreteSchema;
    use serde_json::json;

    fn concrete(properties: serde_json::Value) -> ConcreteSchema {
        ConcreteSchema::from_document(json!({
            "$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1",
            "$id": "urn:semantic-agent-schema:t:record:v1",
            "x-delegates": {"judge": {"role": "読む", "run": {"kind": "judging"}}},
            "x-extensions": {"x-owner": {"schema": true, "description": "責任者", "prompt": {"read": {"means": "責任者"}}}},
            "properties": properties
        }))
        .unwrap()
    }

    fn rule(id: &str, by: &str, agent: &str) -> serde_json::Value {
        json!({"id": id, "rule": "r", "why": "w", "test": "t", "by": by, "agent": agent})
    }

    /// traces: I3
    #[test]
    fn schema_that_is_not_a_urn_is_refused() {
        assert!(ConcreteSchema::from_document(json!({"$schema": "https://example.com/s", "$id": "urn:semantic-agent-schema:t:r:v1"})).is_err());
    }

    /// traces: I4
    #[test]
    fn rules_are_found_in_nested_fields() {
        let c = concrete(
            json!({"a": {"type": "object", "properties": {"b": {"x-prompt": {"write": [rule("b-rule", "machine", "")]}}}}}),
        );
        assert_eq!(
            c.rules()[0].pointer(),
            "/properties/a/properties/b/x-prompt/write/0"
        );
    }

    /// traces: I4
    #[test]
    fn duplicate_rule_id_is_a_breach() {
        let c = concrete(
            json!({"a": {"x-prompt": {"write": [rule("same", "human", "")]}}, "b": {"x-prompt": {"write": [rule("same", "human", "")]}}}),
        );
        assert_eq!(c.breaches().len(), 1);
    }

    /// traces: I4
    #[test]
    fn ai_rule_with_undeclared_agent_is_a_breach() {
        let c = concrete(json!({"a": {"x-prompt": {"write": [rule("r", "ai", "nobody")]}}}));
        assert_eq!(c.breaches()[0].pointer(), "/properties/a/x-prompt/write/0");
    }

    /// traces: I4
    #[test]
    fn ai_rule_with_declared_agent_has_no_breach() {
        let c = concrete(json!({"a": {"x-prompt": {"write": [rule("r", "ai", "judge")]}}}));
        assert!(c.breaches().is_empty());
    }

    /// traces: I6
    #[test]
    fn x_keyword_without_definition_is_a_breach() {
        let c = concrete(json!({"a": {"x-color": "red"}}));
        assert_eq!(c.breaches()[0].pointer(), "/properties/a/x-color");
    }

    /// traces: I6
    #[test]
    fn x_keyword_declared_in_extensions_has_no_breach() {
        let c = concrete(json!({"a": {"x-owner": "me"}}));
        assert!(c.breaches().is_empty());
    }

    /// traces: I4
    #[test]
    fn field_ref_to_glossary_is_a_term() {
        let c =
            concrete(json!({"price": {"$ref": "urn:semantic-agent-schema:t:glossary:v1#Price"}}));
        assert_eq!(c.terms()[0].0, "/properties/price/$ref");
    }

    /// traces: I4
    #[test]
    fn concrete_of_kind_glossary_is_a_breach() {
        let c = ConcreteSchema::from_document(json!({"$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1", "$id": "urn:semantic-agent-schema:t:glossary:v1"})).unwrap();
        assert_eq!(c.breaches()[0].pointer(), "/$id");
    }

    /// traces: I6
    #[test]
    fn extension_named_like_an_abstract_keyword_is_a_breach() {
        let mut c = concrete(json!({}));
        c = c.patched(&json!([{"op": "add", "path": "/x-extensions/x-prompt", "value": {"schema": true, "description": "d", "prompt": {"read": {"means": "m"}}}}])).unwrap();
        assert!(
            c.breaches()
                .iter()
                .any(|b| b.pointer() == "/x-extensions/x-prompt")
        );
    }

    /// traces: I3
    #[test]
    fn refused_patch_leaves_the_concrete_unchanged() {
        let c = concrete(json!({}));
        let before = c.clone();
        assert!(
            c.patched(&json!([{"op": "replace", "path": "/$schema", "value": "x"}]))
                .is_err()
        );
        assert_eq!(c, before);
    }
}
