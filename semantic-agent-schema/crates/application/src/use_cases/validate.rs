//! validate（検査②。依頼の一覧を返し、答えをまとめる）

use crate::use_cases::{Outcome, Ports};
use semantic_agent_schema_change::values::instance_state::InstanceState;
use semantic_agent_schema_change::values::verdict::{Finding, Verdict};
use semantic_agent_schema_declaration::aggregates::abstract_schema::AbstractSchema;
use semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema;
use semantic_agent_schema_declaration::aggregates::glossary::Glossary;
use semantic_agent_schema_declaration::values::etag::ETag;
use semantic_agent_schema_declaration::values::target::Target;
use semantic_agent_schema_instance::aggregates::instance::Instance;
use semantic_agent_schema_resolver::services::resolver::resolve_abstract;
use semantic_agent_schema_validation::services::validator::{
    introduced, problems, validate_declaration, validate_instance,
};
use serde_json::{Value, json};

/// 宣言を申告した Abstract に照らした検証結果。$schema を引けなければ Err（断りの理由）。
/// 欄が指す glossary を読めなかったときは、その欄の error の検証結果にする
pub(crate) fn findings_of(
    ports: &Ports<'_>,
    document: &Value,
    path: Option<&str>,
) -> Result<Vec<Finding>, String> {
    let closed = AbstractSchema::shipped()?;
    let of = resolve_abstract(document, &closed)?;
    let mut glossaries: Vec<Glossary> = Vec::new();
    let mut unreadable: Vec<Finding> = Vec::new();
    let mut known: Vec<String> = Vec::new();
    if let Ok(concrete) = ConcreteSchema::from_document(document.clone()) {
        for (_, to) in concrete.relationship_targets() {
            if matches!(ports.contracts.find(&to), Ok(Some(_))) {
                known.push(to);
            }
        }
        for (at, term) in concrete.terms() {
            if glossaries.iter().any(|g| g.id() == term.glossary()) {
                continue;
            }
            match ports
                .contracts
                .find(&term.glossary().to_string())
                .and_then(|found| found.map(Glossary::from_document).transpose())
            {
                Ok(Some(g)) => glossaries.push(g),
                Ok(None) => {}
                Err(e) => {
                    unreadable.push(Finding::by_machine("term-by-ref", &at, Verdict::Error, &e))
                }
            }
        }
    }
    if !unreadable.is_empty() {
        return Ok(unreadable);
    }
    Ok(validate_declaration(
        document,
        of,
        &closed,
        &glossaries,
        path,
        &known,
    ))
}

/// 文書が宣言（Concrete か、Abstract のファイルのインスタンス）か。$schema が入れて出す版を指していれば宣言、ほかはインスタンス
pub(crate) fn is_declaration(document: &Value) -> bool {
    let declared = document
        .get("$schema")
        .and_then(Value::as_str)
        .unwrap_or_default();
    AbstractSchema::shipped()
        .map(|l| l.iter().any(|a| a.id().to_string() == declared))
        .unwrap_or(false)
}

/// インスタンスが従う Concrete を引く。引けなければ None（不変条件 I8：インスタンスは下書きとして返る）
pub(crate) fn concrete_of(
    ports: &Ports<'_>,
    document: &Value,
) -> Result<Option<ConcreteSchema>, String> {
    let Some(schema) = document.get("$schema").and_then(Value::as_str) else {
        return Ok(None);
    };
    ports
        .contracts
        .find(schema)?
        .map(ConcreteSchema::from_document)
        .transpose()
}

/// インスタンスを従う Concrete に照らした検証結果
fn instance_findings(
    ports: &Ports<'_>,
    document: &Value,
    path: Option<&str>,
) -> Result<Vec<Finding>, String> {
    let Some(concrete) = concrete_of(ports, document)? else {
        return Ok(vec![Finding::by_machine(
            "concrete-resolves",
            "/$schema",
            Verdict::Violated,
            "$schema の Concrete を引けない（下書きとして返る）",
        )]);
    };
    let mut glossaries: Vec<Glossary> = Vec::new();
    for (_, term) in concrete.terms() {
        if glossaries.iter().any(|g| g.id() == term.glossary()) {
            continue;
        }
        if let Some(found) = ports.contracts.find(&term.glossary().to_string())? {
            glossaries.push(Glossary::from_document(found)?);
        }
    }
    let approved = approved_fields(ports, path, document)?;
    Ok(validate_instance(
        document,
        &concrete,
        &AbstractSchema::shipped()?,
        &glossaries,
        path,
        &approved,
    ))
}

/// その対象の項目のうち、変えた変更の記録を人が承認したもの（JSON Pointer）
fn approved_fields(
    ports: &Ports<'_>,
    path: Option<&str>,
    document: &Value,
) -> Result<Vec<String>, String> {
    let target = Target::of_document(path.unwrap_or_default(), Some(document));
    let mut out = Vec::new();
    for record in ports.records.touching(&target)? {
        if ports
            .judgments
            .intent_of(record.id())?
            .is_some_and(|j| j.approved_by_human())
        {
            out.extend(
                record
                    .items()
                    .iter()
                    .filter(|i| i.target() == &target)
                    .filter_map(|i| i.field().map(str::to_string)),
            );
        }
    }
    Ok(out)
}

/// 宣言かインスタンスかを見て、照らした検証結果を返す
pub(crate) fn results_of(
    ports: &Ports<'_>,
    document: &Value,
    path: Option<&str>,
) -> Result<Vec<Finding>, String> {
    if is_declaration(document) {
        findings_of(ports, document, path)
    } else {
        instance_findings(ports, document, path)
    }
}

/// まだ書いていない項目を案内つきで返す。Concrete は Abstract の本体の根、インスタンスは従う Concrete の根の欄
pub(crate) fn guidance(ports: &Ports<'_>, document: &Value) -> Vec<Value> {
    if is_declaration(document) {
        return unfilled(document);
    }
    match (
        concrete_of(ports, document),
        Instance::from_document(document.clone()),
    ) {
        (Ok(Some(c)), Ok(i)) => i.unfilled(c.document()),
        _ => Vec::new(),
    }
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
    ports: &Ports<'_>,
    path: &str,
    before: Option<&Value>,
    next: &Value,
    data: Value,
    write: &dyn Fn(&Value) -> Result<(), String>,
) -> Outcome {
    let after = match results_of(ports, next, Some(path)) {
        Ok(f) => f,
        Err(e) => return Outcome::refused("/$schema", &e),
    };
    let earlier = before
        .and_then(|b| results_of(ports, b, Some(path)).ok())
        .unwrap_or_default();
    let new = introduced(&earlier, &after);
    let mut data = data;
    data["unfilled"] = json!(guidance(ports, next));
    if !new.is_empty() {
        data["saved"] = json!(false);
        let mut refused = Outcome::validated(&after, data);
        refused.findings = Outcome::checked(&new, Value::Null).findings;
        return refused;
    }
    if let Err(e) = write(next) {
        return Outcome::refused("", &e);
    }
    data["saved"] = json!(true);
    data["etag"] = json!(ETag::of(next).as_str());
    Outcome::validated(&after, data)
}

/// Concrete かインスタンスを検証する。"." なら作業の根の下のすべて（ほかの変更を取り込んだあとの姿。U8 ・ U18）
pub fn validate(ports: &Ports<'_>, path: &str) -> Outcome {
    if path == "." {
        return validate_all(ports);
    }
    let document = match ports.instances.read(path) {
        Ok(Some(d)) => d,
        Ok(None) => return Outcome::refused("", &format!("{path} が無い")),
        Err(e) => return Outcome::refused("", &e),
    };
    match results_of(ports, &document, Some(path)) {
        Ok(findings) => Outcome::validated(&findings, json!({"path": path})),
        Err(e) => Outcome::refused("/$schema", &e),
    }
}

/// 作業の根の下のすべての宣言とインスタンスを検証する。findings の場所は「置き場所#JSON Pointer」
fn validate_all(ports: &Ports<'_>) -> Outcome {
    let paths = match ports.instances.all() {
        Ok(p) => p,
        Err(e) => return Outcome::refused("", &e),
    };
    let mut problems_all: Vec<Finding> = Vec::new();
    let mut files: Vec<Value> = Vec::new();
    for path in &paths {
        let results = match ports.instances.read(path) {
            Ok(Some(d)) => results_of(ports, &d, Some(path)),
            Ok(None) => Err(format!("{path} が無い")),
            Err(e) => Err(e),
        }
        .unwrap_or_else(|e| vec![Finding::by_machine("readable", "", Verdict::Error, &e)]);
        let state = InstanceState::from_results(&results);
        let found = problems(&results);
        files.push(json!({"path": path, "state": {"name": state.as_str(), "label": state.label()}, "problems": found.len()}));
        problems_all.extend(found.iter().map(|f| {
            Finding::new(
                f.rule(),
                &format!("{path}#{}", f.pointer()),
                f.verdict(),
                f.reason(),
                f.by(),
            )
        }));
    }
    Outcome::checked(&problems_all, json!({"path": ".", "files": files}))
}
