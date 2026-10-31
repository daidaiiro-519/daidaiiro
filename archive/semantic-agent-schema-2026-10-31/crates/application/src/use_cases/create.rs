//! create

use crate::use_cases::validate::write_checked;
use crate::use_cases::{Outcome, Ports};
use semantic_agent_schema_declaration::aggregates::abstract_schema::AbstractSchema;
use semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema;
use semantic_agent_schema_declaration::aggregates::concrete_schema::factories::concrete_template::concrete_template;
use semantic_agent_schema_declaration::aggregates::glossary::Glossary;
use semantic_agent_schema_declaration::values::urn::Urn;
use semantic_agent_schema_instance::aggregates::instance::factories::instance_factory::instance_template;
use serde_json::{Value, json};

/// 置き場所に作る。種類が glossary なら glossary を、ほかは Abstract の本体から Concrete を、雛形に渡した値（create-request）を重ねて作る。
/// 既にあれば断る。出力の unfilled に、まだ書いていない項目を案内つきで返す
pub fn create(ports: &Ports<'_>, path: &str, owner: &str, kind: &str, value: &Value) -> Outcome {
    match ports.contracts.read(path) {
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
        ports,
        path,
        None,
        &document,
        json!({"path": path, "id": id.to_string()}),
        &|doc| ports.contracts.write(path, doc),
    )
}

/// 置き場所に、Concrete（$schema の URN）からインスタンスを作る。id は呼ぶ側が決めて渡す。既にあれば断る。
/// 出力の unfilled に、Concrete の欄のうちまだ書いていないものを案内つきで返す
pub fn create_instance(
    ports: &Ports<'_>,
    path: &str,
    schema: &str,
    id: &str,
    value: &Value,
) -> Outcome {
    match ports.instances.read(path) {
        Ok(None) => {}
        Ok(Some(_)) => return Outcome::refused("", &format!("{path} は既にある（作り直さない）")),
        Err(e) => return Outcome::refused("", &e),
    }
    let concrete = match ports
        .contracts
        .find(schema)
        .and_then(|f| f.map(ConcreteSchema::from_document).transpose())
    {
        Ok(Some(c)) => c,
        Ok(None) => return Outcome::refused("/$schema", &format!("Concrete「{schema}」が無い")),
        Err(e) => return Outcome::refused("/$schema", &e),
    };
    let instance = match instance_template(&concrete, id, value) {
        Ok(i) => i,
        Err(e) => return Outcome::refused("", &e),
    };
    write_checked(
        ports,
        path,
        None,
        instance.document(),
        json!({"path": path, "id": id}),
        &|doc| ports.instances.write(path, doc),
    )
}
