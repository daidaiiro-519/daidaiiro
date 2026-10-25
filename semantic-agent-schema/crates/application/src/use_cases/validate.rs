//! validate（検査②。依頼の一覧を返し、答えをまとめる）

use crate::ports::contracts::Contracts;
use crate::use_cases::Outcome;
use semantic_agent_schema_change::values::verdict::{Finding, Verdict};
use semantic_agent_schema_declaration::aggregates::abstract_schema::AbstractSchema;
use semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema;
use semantic_agent_schema_declaration::aggregates::glossary::Glossary;
use semantic_agent_schema_resolver::services::resolver::resolve_abstract;
use semantic_agent_schema_validation::services::validator::validate_declaration;
use serde_json::{Value, json};

/// 宣言を申告した Abstract に照らした検証結果。$schema を引けなければ Err（断りの理由）。
/// 欄が指す glossary を読めなかったときは、その欄の error の検証結果にする
pub(crate) fn findings_of(
    contracts: &dyn Contracts,
    document: &Value,
) -> Result<Vec<Finding>, String> {
    let closed = AbstractSchema::shipped()?;
    let of = resolve_abstract(document, &closed)?;
    let mut glossaries: Vec<Glossary> = Vec::new();
    let mut unreadable: Vec<Finding> = Vec::new();
    if let Ok(concrete) = ConcreteSchema::from_document(document.clone()) {
        for (at, term) in concrete.terms() {
            if glossaries.iter().any(|g| g.id() == term.glossary()) {
                continue;
            }
            match contracts
                .find(&term.glossary().to_string())
                .and_then(|found| found.map(Glossary::from_document).transpose())
            {
                Ok(Some(g)) => glossaries.push(g),
                Ok(None) => {}
                Err(e) => unreadable.push(Finding::by_machine(&at, Verdict::Error, &e)),
            }
        }
    }
    if !unreadable.is_empty() {
        return Ok(unreadable);
    }
    Ok(validate_declaration(document, of, &glossaries))
}

/// 保存したあとの検証結果。保存したので断りにはせず、引けなかった理由は error にする
pub(crate) fn after_saving(contracts: &dyn Contracts, document: &Value, data: Value) -> Outcome {
    match findings_of(contracts, document) {
        Ok(findings) => Outcome::checked(&findings, data),
        Err(e) => Outcome::checked(&[Finding::by_machine("/$schema", Verdict::Error, &e)], data),
    }
}

/// Concrete を検証する
pub fn validate(contracts: &dyn Contracts, path: &str) -> Outcome {
    let document = match contracts.read(path) {
        Ok(Some(d)) => d,
        Ok(None) => return Outcome::refused("", &format!("{path} が無い")),
        Err(e) => return Outcome::refused("", &e),
    };
    match findings_of(contracts, &document) {
        Ok(findings) => Outcome::checked(&findings, json!({"path": path})),
        Err(e) => Outcome::refused("/$schema", &e),
    }
}
