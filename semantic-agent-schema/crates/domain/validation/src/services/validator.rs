//! Semantic Schema Validator（セマンティックスキーマバリデーター）

use semantic_agent_schema_change::values::verdict::{Finding, Verdict};
use semantic_agent_schema_declaration::aggregates::abstract_schema::AbstractSchema;
use semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema;
use semantic_agent_schema_declaration::aggregates::glossary::Glossary;
use semantic_agent_schema_resolver::services::resolver::resolve_term;
use serde_json::Value;

/// Semantic Schema Validator（セマンティックスキーマバリデーター）：宣言（Concrete か glossary）を、申告した Abstract の形と、形では見られない宣言の決まりに照らす。不変条件 I3 ・ I4 ・ I6 ・ I13
///
/// Concrete の欄が指す term は、渡された glossary から引く。返す検証結果は場所の順に並ぶ。空なら合っている
pub fn validate_declaration(
    document: &Value,
    of: &AbstractSchema,
    glossaries: &[Glossary],
) -> Vec<Finding> {
    let validator = match jsonschema::options().build(of.document()) {
        Ok(v) => v,
        Err(e) => {
            return vec![Finding::by_machine(
                "",
                Verdict::Error,
                &format!("Abstract を読めない：{e}"),
            )];
        }
    };
    let mut out: Vec<Finding> = validator
        .iter_errors(document)
        .map(|e| {
            Finding::by_machine(
                e.instance_path().as_str(),
                Verdict::Violated,
                &e.to_string(),
            )
        })
        .collect();
    if of.id().kind() == "glossary" {
        return sorted(out);
    }
    match ConcreteSchema::from_document(document.clone()) {
        Ok(concrete) => {
            out.extend(
                concrete
                    .breaches()
                    .iter()
                    .map(|b| Finding::by_machine(b.pointer(), Verdict::Violated, b.reason())),
            );
            for (at, term) in concrete.terms() {
                if let Err(e) = resolve_term(&term, glossaries) {
                    out.push(Finding::by_machine(&at, Verdict::Violated, &e));
                }
            }
        }
        Err(e) if out.is_empty() => out.push(Finding::by_machine("", Verdict::Violated, &e)),
        Err(_) => {}
    }
    sorted(out)
}

fn sorted(mut out: Vec<Finding>) -> Vec<Finding> {
    out.sort_by(|a, b| {
        a.pointer()
            .cmp(b.pointer())
            .then_with(|| a.reason().cmp(b.reason()))
    });
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::validate_declaration;
    use semantic_agent_schema_declaration::aggregates::abstract_schema::AbstractSchema;
    use semantic_agent_schema_declaration::aggregates::glossary::Glossary;
    use serde_json::{Value, json};

    fn abstract_v1() -> AbstractSchema {
        AbstractSchema::shipped().unwrap().remove(0)
    }

    fn concrete(extra: Value) -> Value {
        let mut doc = json!({
            "$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1",
            "$id": "urn:semantic-agent-schema:t:record:v1",
            "title": "t",
            "description": "d",
            "type": "object"
        });
        if let (Some(d), Some(e)) = (doc.as_object_mut(), extra.as_object()) {
            d.extend(e.clone());
        }
        doc
    }

    /// traces: I3
    #[test]
    fn concrete_with_required_fields_passes() {
        assert!(validate_declaration(&concrete(json!({})), &abstract_v1(), &[]).is_empty());
    }

    /// traces: I4
    #[test]
    fn rule_without_by_is_violated_at_the_rule() {
        let doc = concrete(
            json!({"properties": {"a": {"x-prompt": {"write": [{"id": "r", "rule": "r", "why": "w", "test": "t"}]}}}}),
        );
        assert!(
            validate_declaration(&doc, &abstract_v1(), &[])[0]
                .pointer()
                .starts_with("/properties/a/x-prompt/write/0")
        );
    }

    /// traces: I6
    #[test]
    fn extension_named_like_an_abstract_keyword_is_violated() {
        let doc =
            concrete(json!({"x-extensions": {"x-prompt": {"schema": true, "description": "d"}}}));
        assert_eq!(
            validate_declaration(&doc, &abstract_v1(), &[])[0]
                .verdict()
                .as_str(),
            "violated"
        );
    }

    /// traces: I13
    #[test]
    fn same_document_gives_the_same_findings() {
        let doc = concrete(json!({"$id": "bad", "x-color": 1}));
        assert_eq!(
            validate_declaration(&doc, &abstract_v1(), &[]),
            validate_declaration(&doc, &abstract_v1(), &[])
        );
    }

    /// traces: I4
    #[test]
    fn field_ref_to_undefined_term_is_violated() {
        let doc = concrete(
            json!({"properties": {"p": {"$ref": "urn:semantic-agent-schema:t:glossary:v1#Price"}}}),
        );
        let g = Glossary::from_document(
            json!({"$id": "urn:semantic-agent-schema:t:glossary:v1", "$defs": {}}),
        )
        .unwrap();
        assert_eq!(
            validate_declaration(&doc, &abstract_v1(), &[g])[0].pointer(),
            "/properties/p/$ref"
        );
    }

    /// traces: I4
    #[test]
    fn field_ref_to_defined_term_passes() {
        let doc = concrete(
            json!({"properties": {"p": {"$ref": "urn:semantic-agent-schema:t:glossary:v1#Price"}}}),
        );
        let g = Glossary::from_document(json!({"$id": "urn:semantic-agent-schema:t:glossary:v1", "$defs": {"p": {"$anchor": "Price"}}})).unwrap();
        assert!(validate_declaration(&doc, &abstract_v1(), &[g]).is_empty());
    }
}
