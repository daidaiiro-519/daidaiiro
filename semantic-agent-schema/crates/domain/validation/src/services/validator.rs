//! Semantic Schema Validator（セマンティックスキーマバリデーター）

use semantic_agent_schema_change::values::verdict::{Finding, Verdict};
use semantic_agent_schema_declaration::aggregates::abstract_schema::AbstractSchema;
use semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema;
use semantic_agent_schema_declaration::aggregates::glossary::Glossary;
use semantic_agent_schema_declaration::values::by::By;
use semantic_agent_schema_resolver::services::resolver::resolve_term;
use serde_json::Value;

/// 基盤の vocabulary（Abstract の本体が $vocabulary で必須にしている）。基盤がこれを実装する
const VOCABULARY: &str = "urn:semantic-agent-schema:base:vocab:v1";

/// 宣言を照らす JSON Schema の検証器を組む。Abstract の本体は自分自身を $schema に持つので、
/// draft 2020-12 を明示してメタスキーマの連鎖をたどらせず、入れて出す版をすべてレジストリに置く
fn compile(
    of: &AbstractSchema,
    shipped: &[AbstractSchema],
) -> Result<jsonschema::Validator, String> {
    let mut builder = jsonschema::Registry::new();
    for a in shipped {
        builder = builder
            .add(a.id().to_string(), a.document().clone())
            .map_err(|e| e.to_string())?;
    }
    let registry = builder.prepare().map_err(|e| e.to_string())?;
    jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .with_vocabulary(VOCABULARY)
        .with_registry(&registry)
        .build(of.document())
        .map_err(|e| e.to_string())
}

/// Semantic Schema Validator（セマンティックスキーマバリデーター）：宣言（Concrete か glossary）を、申告した Abstract の形と、形では見られない宣言の決まりに照らす。不変条件 I3 ・ I4 ・ I6 ・ I13
///
/// Concrete の欄が指す term は、渡された glossary から引く。返す検証結果は場所の順に並ぶ。空なら合っている
pub fn validate_declaration(
    document: &Value,
    of: &AbstractSchema,
    shipped: &[AbstractSchema],
    glossaries: &[Glossary],
) -> Vec<Finding> {
    let validator = match compile(of, shipped) {
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
    // Abstract のファイルのインスタンスと、本体そのもの（9つの x- を定義する側。不変条件 I1）には、Concrete の決まりを当てない
    if !of.is_body()
        || document.get("$id").and_then(Value::as_str) == Some(of.id().to_string().as_str())
    {
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

/// この書き込みが新しく持ち込んだ、機械の violated：書き込んだ後にだけある violated（場所と理由で比べる）。
/// 前からある violated と、AI ・ 人の判定は含めない
pub fn introduced(before: &[Finding], after: &[Finding]) -> Vec<Finding> {
    after
        .iter()
        .filter(|f| f.verdict() == Verdict::Violated && f.by() == By::Machine)
        .filter(|f| {
            !before
                .iter()
                .any(|b| b.pointer() == f.pointer() && b.reason() == f.reason())
        })
        .cloned()
        .collect()
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
    use super::{introduced, validate_declaration};
    use semantic_agent_schema_change::values::verdict::{Finding, Verdict};
    use semantic_agent_schema_declaration::aggregates::abstract_schema::AbstractSchema;
    use semantic_agent_schema_declaration::aggregates::glossary::Glossary;
    use serde_json::{Value, json};

    fn check(doc: &Value, glossaries: &[Glossary]) -> Vec<Finding> {
        let shipped = AbstractSchema::shipped().unwrap();
        validate_declaration(doc, &shipped[0], &shipped, glossaries)
    }

    fn concrete(extra: Value) -> Value {
        let mut doc = json!({
            "$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1",
            "$id": "urn:semantic-agent-schema:t:record:v1",
            "title": "t",
            "description": "d"
        });
        if let (Some(d), Some(e)) = (doc.as_object_mut(), extra.as_object()) {
            d.extend(e.clone());
        }
        doc
    }

    fn field(extra: Value) -> Value {
        let mut f = json!({"description": "欄", "x-prompt": {"read": {"means": "意味"}}});
        if let (Some(d), Some(e)) = (f.as_object_mut(), extra.as_object()) {
            d.extend(e.clone());
        }
        f
    }

    /// traces: I3
    #[test]
    fn concrete_with_required_fields_passes() {
        assert!(check(&concrete(json!({})), &[]).is_empty());
    }

    /// traces: I3
    #[test]
    fn the_shipped_abstract_passes_against_itself() {
        let shipped = AbstractSchema::shipped().unwrap();
        let f = validate_declaration(shipped[0].document(), &shipped[0], &shipped, &[]);
        assert!(f.is_empty(), "{f:?}");
    }

    /// traces: I4
    #[test]
    fn field_without_description_and_prompt_is_violated() {
        let doc = concrete(json!({"properties": {"a": {"type": "string"}}}));
        assert_eq!(check(&doc, &[]).len(), 2);
    }

    /// traces: I4
    #[test]
    fn rule_without_by_is_violated_at_the_rule() {
        let rule = json!({"id": "r", "rule": "決まり", "why": "理由", "test": "確かめ方"});
        let doc = concrete(
            json!({"properties": {"a": field(json!({"x-prompt": {"read": {"means": "m"}, "write": [rule]}}))}}),
        );
        assert!(
            check(&doc, &[])[0]
                .pointer()
                .starts_with("/properties/a/x-prompt/write/0")
        );
    }

    /// traces: I6
    #[test]
    fn extension_named_like_an_abstract_keyword_is_violated() {
        let doc = concrete(
            json!({"x-extensions": {"x-prompt": {"schema": true, "description": "d", "prompt": {"read": {"means": "m"}}}}}),
        );
        assert_eq!(check(&doc, &[])[0].verdict().as_str(), "violated");
    }

    /// traces: I13
    #[test]
    fn same_document_gives_the_same_findings() {
        let doc = concrete(json!({"$id": "bad", "x-color": 1}));
        assert_eq!(check(&doc, &[]), check(&doc, &[]));
    }

    /// traces: I4
    #[test]
    fn field_ref_to_undefined_term_is_violated() {
        let doc = concrete(
            json!({"properties": {"p": field(json!({"$ref": "urn:semantic-agent-schema:t:glossary:v1#Price"}))}}),
        );
        let g = Glossary::from_document(
            json!({"$id": "urn:semantic-agent-schema:t:glossary:v1", "$defs": {}}),
        )
        .unwrap();
        assert_eq!(check(&doc, &[g])[0].pointer(), "/properties/p/$ref");
    }

    /// traces: I4
    #[test]
    fn field_ref_to_defined_term_passes() {
        let doc = concrete(
            json!({"properties": {"p": field(json!({"$ref": "urn:semantic-agent-schema:t:glossary:v1#Price"}))}}),
        );
        let g = Glossary::from_document(json!({"$id": "urn:semantic-agent-schema:t:glossary:v1", "$defs": {"p": {"$anchor": "Price"}}})).unwrap();
        assert!(check(&doc, &[g]).is_empty());
    }

    /// traces: U1:失敗時
    #[test]
    fn violation_only_after_the_write_is_introduced() {
        let v = Finding::by_machine("/a", Verdict::Violated, "必須");
        assert_eq!(introduced(&[], std::slice::from_ref(&v)), vec![v]);
    }

    /// traces: U1:事後
    #[test]
    fn violation_present_before_the_write_is_not_introduced() {
        let v = Finding::by_machine("/a", Verdict::Violated, "必須");
        let same = std::slice::from_ref(&v);
        assert!(introduced(same, same).is_empty());
    }
}
