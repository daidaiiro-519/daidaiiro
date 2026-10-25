//! create

use crate::ports::contracts::Contracts;
use crate::use_cases::Outcome;
use crate::use_cases::validate::after_saving;
use semantic_agent_schema_declaration::aggregates::abstract_schema::AbstractSchema;
use semantic_agent_schema_declaration::aggregates::concrete_schema::factories::concrete_template::concrete_template;
use semantic_agent_schema_declaration::aggregates::glossary::Glossary;
use semantic_agent_schema_declaration::values::urn::Urn;
use serde_json::json;

/// 置き場所に、Abstract の v1 から Concrete の雛形を作る。種類が glossary なら term の無い glossary を作る。既にあれば断る
pub fn create(contracts: &dyn Contracts, path: &str, owner: &str, kind: &str) -> Outcome {
    match contracts.read(path) {
        Ok(None) => {}
        Ok(Some(_)) => return Outcome::refused("", &format!("{path} は既にある（作り直さない）")),
        Err(e) => return Outcome::refused("", &e),
    }
    let id = match Urn::new(owner, kind, 1) {
        Ok(id) => id,
        Err(e) => return Outcome::refused("/$id", &e),
    };
    let document = if kind == "glossary" {
        match Glossary::empty(&id) {
            Ok(g) => g.document().clone(),
            Err(e) => return Outcome::refused("/$id", &e),
        }
    } else {
        match AbstractSchema::shipped().map(|mut l| l.remove(0)) {
            Ok(of) => concrete_template(&id, &of).document().clone(),
            Err(e) => return Outcome::refused("/$schema", &e),
        }
    };
    if let Err(e) = contracts.write(path, &document) {
        return Outcome::refused("", &e);
    }
    after_saving(
        contracts,
        &document,
        json!({"path": path, "id": id.to_string()}),
    )
}
