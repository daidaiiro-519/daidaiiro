//! validate（検査②。依頼の一覧を返し、答えをまとめる）

use crate::ports::contracts::Contracts;
use crate::use_cases::Outcome;
use semantic_agent_schema_change::values::verdict::{Finding, Verdict};
use semantic_agent_schema_declaration::aggregates::abstract_schema::AbstractSchema;
use semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema;
use semantic_agent_schema_declaration::aggregates::glossary::Glossary;
use semantic_agent_schema_resolver::services::resolver::resolve_abstract;
use semantic_agent_schema_validation::services::validator::{introduced, validate_declaration};
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
    Ok(validate_declaration(document, of, &closed, &glossaries))
}

/// Concrete に、Abstract の本体の根に書けるもののうち、まだ書いていないものを案内つきで返す。Concrete でなければ空
pub(crate) fn unfilled(document: &Value) -> Vec<Value> {
    let Ok(closed) = AbstractSchema::shipped() else {
        return Vec::new();
    };
    match resolve_abstract(document, &closed) {
        Ok(of) if of.is_body() => of.unfilled(document),
        _ => Vec::new(),
    }
}

/// 書き込む前に、適用した後の姿を検証して書く。この書き込みが新しく持ち込んだ機械の violated があれば書かずに返す
/// （断るのは止めることではなく、書き手が findings を見て直し、出し直すため）。前からある violated と、AI ・ 人の rule は書き込みを止めない
pub(crate) fn write_checked(
    contracts: &dyn Contracts,
    path: &str,
    before: Option<&Value>,
    next: &Value,
    data: Value,
) -> Outcome {
    let after = match findings_of(contracts, next) {
        Ok(f) => f,
        Err(e) => return Outcome::refused("/$schema", &e),
    };
    let earlier = before
        .and_then(|b| findings_of(contracts, b).ok())
        .unwrap_or_default();
    let new = introduced(&earlier, &after);
    let mut data = data;
    data["unfilled"] = json!(unfilled(next));
    if !new.is_empty() {
        data["saved"] = json!(false);
        return Outcome::checked(&new, data);
    }
    if let Err(e) = contracts.write(path, next) {
        return Outcome::refused("", &e);
    }
    data["saved"] = json!(true);
    Outcome::checked(&after, data)
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
