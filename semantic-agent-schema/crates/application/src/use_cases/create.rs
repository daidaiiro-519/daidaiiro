//! create

use crate::ports::contracts::Contracts;
use crate::use_cases::Outcome;
use crate::use_cases::validate::write_checked;
use semantic_agent_schema_declaration::aggregates::abstract_schema::AbstractSchema;
use semantic_agent_schema_declaration::aggregates::concrete_schema::factories::concrete_template::concrete_template;
use semantic_agent_schema_declaration::aggregates::glossary::Glossary;
use semantic_agent_schema_declaration::values::urn::Urn;
use serde_json::{Value, json};

/// 置き場所に作る。種類が glossary なら glossary を、ほかは Abstract の本体から Concrete を、雛形に渡した値（create-request）を重ねて作る。
/// 既にあれば断る。出力の unfilled に、まだ書いていない項目を案内つきで返す
pub fn create(
    contracts: &dyn Contracts,
    path: &str,
    owner: &str,
    kind: &str,
    value: &Value,
) -> Outcome {
    match contracts.read(path) {
        Ok(None) => {}
        Ok(Some(_)) => return Outcome::refused("", &format!("{path} は既にある（作り直さない）")),
        Err(e) => return Outcome::refused("", &e),
    }
    let id = match Urn::new(owner, kind, 1) {
        Ok(id) => id,
        Err(e) => return Outcome::refused("/$id", &e),
    };
    let made = if kind == "glossary" {
        Glossary::template(&id, value)
    } else {
        AbstractSchema::shipped()
            .map(|mut l| l.remove(0))
            .and_then(|of| concrete_template(&id, &of, value))
            .map(|c| c.document().clone())
    };
    let document = match made {
        Ok(d) => d,
        Err(e) => return Outcome::refused("", &e),
    };
    write_checked(
        contracts,
        path,
        None,
        &document,
        json!({"path": path, "id": id.to_string()}),
    )
}
