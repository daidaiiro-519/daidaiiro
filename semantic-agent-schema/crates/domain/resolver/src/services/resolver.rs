//! Semantic Schema Resolver（セマンティックスキーマリゾルバー）

use semantic_agent_schema_declaration::aggregates::abstract_schema::AbstractSchema;
use semantic_agent_schema_declaration::aggregates::glossary::Glossary;
use semantic_agent_schema_declaration::values::term::Term;
use serde_json::Value;

/// Semantic Schema Resolver（セマンティックスキーマリゾルバー）：文書の $schema を閉じた一覧から引く。一覧に無ければ取りに行かずに断る。不変条件 I2
///
/// # Errors
/// $schema が無いか、閉じた一覧に無いとき
pub fn resolve_abstract<'a>(
    document: &Value,
    closed: &'a [AbstractSchema],
) -> Result<&'a AbstractSchema, String> {
    let declared = document
        .get("$schema")
        .and_then(Value::as_str)
        .ok_or("$schema が無い")?;
    closed
        .iter()
        .find(|a| a.id().to_string() == declared)
        .ok_or_else(|| format!("$schema「{declared}」は閉じた一覧に無い（取りに行かない）"))
}

/// term を、渡された glossary の版から引く。版付きの id で一致を決める。不変条件 I4
///
/// # Errors
/// その版の glossary が無いか、glossary にその名前の term が無いとき
pub fn resolve_term<'a>(term: &Term, glossaries: &'a [Glossary]) -> Result<&'a Glossary, String> {
    let glossary = glossaries
        .iter()
        .find(|g| g.id() == term.glossary())
        .ok_or_else(|| format!("glossary {} が無い", term.glossary()))?;
    if glossary.defines(term.name()) {
        Ok(glossary)
    } else {
        Err(format!(
            "glossary {} に term「{}」が無い",
            term.glossary(),
            term.name()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_abstract, resolve_term};
    use semantic_agent_schema_declaration::aggregates::abstract_schema::AbstractSchema;
    use semantic_agent_schema_declaration::aggregates::glossary::Glossary;
    use semantic_agent_schema_declaration::values::term::Term;
    use serde_json::json;

    fn glossary_v1() -> Vec<Glossary> {
        vec![Glossary::from_document(json!({"$id": "urn:semantic-agent-schema:a:glossary:v1", "$defs": {"p": {"$anchor": "Price"}}})).unwrap()]
    }

    /// traces: I4
    #[test]
    fn defined_term_is_resolved() {
        let t = Term::from_ref("urn:semantic-agent-schema:a:glossary:v1#Price").unwrap();
        assert!(resolve_term(&t, &glossary_v1()).is_ok());
    }

    /// traces: I4
    #[test]
    fn undefined_term_is_not_resolved() {
        let t = Term::from_ref("urn:semantic-agent-schema:a:glossary:v1#Cost").unwrap();
        assert!(resolve_term(&t, &glossary_v1()).is_err());
    }

    /// traces: I4
    #[test]
    fn term_of_another_version_is_another_word() {
        let t = Term::from_ref("urn:semantic-agent-schema:a:glossary:v2#Price").unwrap();
        assert!(resolve_term(&t, &glossary_v1()).is_err());
    }

    /// traces: I2
    #[test]
    fn listed_schema_is_resolved() {
        let closed = AbstractSchema::shipped().unwrap();
        let doc = json!({"$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1"});
        assert!(resolve_abstract(&doc, &closed).is_ok());
    }

    /// traces: I2
    #[test]
    fn unlisted_schema_is_refused_without_fetching() {
        let closed = AbstractSchema::shipped().unwrap();
        let doc = json!({"$schema": "https://example.com/schema.json"});
        assert!(resolve_abstract(&doc, &closed).is_err());
    }
}
