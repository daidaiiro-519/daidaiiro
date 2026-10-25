//! Semantic Schema Validator（セマンティックスキーマバリデーター）

use semantic_agent_schema_change::values::verdict::{Finding, SHAPE, Verdict};
use semantic_agent_schema_declaration::aggregates::abstract_schema::AbstractSchema;
use semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema;
use semantic_agent_schema_declaration::aggregates::glossary::Glossary;
use semantic_agent_schema_declaration::values::by::By;
use semantic_agent_schema_instance::aggregates::instance::Instance;
use semantic_agent_schema_resolver::services::resolver::resolve_term;
use serde_json::Value;
use std::collections::BTreeMap;

/// 基盤の vocabulary（Abstract の本体が $vocabulary で必須にしている）。基盤がこれを実装する
const VOCABULARY: &str = "urn:semantic-agent-schema:base:vocab:v1";

/// 宣言を照らす JSON Schema の検証器を組む。Abstract の本体は自分自身を $schema に持つので、
/// draft 2020-12 を明示してメタスキーマの連鎖をたどらせず、入れて出す版をすべてレジストリに置く
fn compile(
    of: &AbstractSchema,
    shipped: &[AbstractSchema],
) -> Result<jsonschema::Validator, String> {
    compile_with(of.document(), shipped, &[])
}

/// schema を検証器に組む。入れて出す版と、渡した文書（Concrete ・ glossary）をレジストリに置く
fn compile_with(
    schema: &Value,
    shipped: &[AbstractSchema],
    more: &[(String, Value)],
) -> Result<jsonschema::Validator, String> {
    let mut builder = jsonschema::Registry::new();
    for a in shipped {
        builder = builder
            .add(a.id().to_string(), a.document().clone())
            .map_err(|e| e.to_string())?;
    }
    for (id, doc) in more {
        builder = builder
            .add(id.clone(), doc.clone())
            .map_err(|e| e.to_string())?;
    }
    let registry = builder.prepare().map_err(|e| e.to_string())?;
    jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .with_vocabulary(VOCABULARY)
        .with_registry(&registry)
        .build(schema)
        .map_err(|e| e.to_string())
}

/// 機械の rule のうち、基盤がまだ判定できないもの（判定に要る記録を作るステップ）
const NOT_YET: [(&str, &str); 2] = [
    (
        "human-entry-only",
        "変更の記録と人の判定（ステップ3a）がまだ無いので判定できない",
    ),
    (
        "never-reuse",
        "公開の記録（ステップ7b）がまだ無いので判定できない",
    ),
];

/// Abstract の rule が、この Concrete に当たるか。rule を書いた場所で決まる
fn applies(site: &str, concrete: &ConcreteSchema) -> bool {
    let doc = concrete.document();
    let non_empty = |k: &str| {
        doc.get(k)
            .and_then(Value::as_object)
            .is_some_and(|m| !m.is_empty())
    };
    if let Some(rest) = site.strip_prefix("/x-extensions/") {
        let keyword = rest.split('/').next().unwrap_or_default();
        return concrete.uses(keyword);
    }
    if let Some(rest) = site.strip_prefix("/properties/") {
        let name = rest
            .split('/')
            .next()
            .unwrap_or_default()
            .replace("~1", "/")
            .replace("~0", "~");
        return doc.get(name.as_str()).is_some();
    }
    if let Some(rest) = site.strip_prefix("/$defs/") {
        return match rest.split('/').next().unwrap_or_default() {
            "field" => non_empty("properties"),
            "derived" => concrete.uses("x-derived-from"),
            "prompt" | "reading" => concrete.uses("x-prompt"),
            "rule" => !concrete.rules().is_empty(),
            "delegate" => non_empty("x-delegates"),
            "extensions" => non_empty("x-extensions"),
            _ => true,
        };
    }
    true
}

/// Semantic Schema Validator（セマンティックスキーマバリデーター）：宣言（Concrete か、Abstract のファイルのインスタンス）を、
/// 申告した Abstract の形と、Abstract の rule の1件ずつに照らす。不変条件 I3 ・ I4 ・ I6 ・ I12 ・ I13
///
/// 形の外れは rule json-schema の violated。Concrete には、当たる Abstract の rule ごとに結果を1つ以上返す：
/// 機械の rule は verified か violated（まだ判定できないものは unsupported）、AI の rule は unverified（判定の依頼を待つ）。
/// path はファイルの置き場所（持ち主の名前と照らす）、known は引ける Concrete の URN（x-relationship の先）
pub fn validate_declaration(
    document: &Value,
    of: &AbstractSchema,
    shipped: &[AbstractSchema],
    glossaries: &[Glossary],
    path: Option<&str>,
    known: &[String],
) -> Vec<Finding> {
    let validator = match compile(of, shipped) {
        Ok(v) => v,
        Err(e) => {
            return vec![Finding::by_machine(
                SHAPE,
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
                SHAPE,
                e.instance_path().as_str(),
                Verdict::Violated,
                &e.to_string(),
            )
        })
        .collect();
    // Abstract のファイルのインスタンスと、本体そのもの（9つの x- を定義する側。不変条件 I1）には、Concrete の rule を当てない
    if !of.is_body()
        || document.get("$id").and_then(Value::as_str) == Some(of.id().to_string().as_str())
    {
        return sorted(out);
    }
    let concrete = match ConcreteSchema::from_document(document.clone()) {
        Ok(c) => c,
        Err(e) => {
            out.push(Finding::by_machine(
                "owner-kind-version",
                "/$id",
                Verdict::Violated,
                &e,
            ));
            return sorted(out);
        }
    };
    // 違反を rule ごとに集める（基盤が中で見るものと、外の材料で見るもの）
    let mut broken: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for b in concrete.breaches(&of.delegate_names()) {
        broken
            .entry(b.rule().to_string())
            .or_default()
            .push((b.pointer().to_string(), b.reason().to_string()));
    }
    for (at, term) in concrete.terms() {
        if let Err(e) = resolve_term(&term, glossaries) {
            broken
                .entry("term-by-ref".to_string())
                .or_default()
                .push((at, e));
        }
    }
    let shipped_ids: Vec<String> = shipped.iter().map(|a| a.id().to_string()).collect();
    for (at, to) in concrete.relationship_targets() {
        if !shipped_ids.contains(&to) && !known.contains(&to) {
            broken
                .entry("target-exists".to_string())
                .or_default()
                .push((at, format!("{to} を引けない")));
        }
    }
    if let Some(p) = path {
        let owner = concrete.id().owner();
        if owner != "base" && !p.split('/').any(|seg| seg == owner) {
            broken
                .entry("owner-kind-version".to_string())
                .or_default()
                .push((
                    "/$id".to_string(),
                    format!("持ち主「{owner}」が、置き場所 {p} のフォルダの名前に無い"),
                ));
        }
    }
    let agent_broken =
        broken.contains_key("agent-exists") || broken.contains_key("by-matches-kind");
    for (site, id, by) in of.rules() {
        if !applies(&site, &concrete) {
            continue;
        }
        match by {
            By::Ai => out.push(Finding::new(
                &id,
                &site,
                Verdict::Unverified,
                "AI の判定を依頼する（任せる先 judge）",
                By::Ai,
            )),
            By::Human => out.push(Finding::new(
                &id,
                &site,
                Verdict::Unverified,
                "人の判定を待つ",
                By::Human,
            )),
            By::Machine => {
                if let Some((_, why)) = NOT_YET.iter().find(|(r, _)| *r == id) {
                    out.push(Finding::by_machine(&id, &site, Verdict::Unsupported, why));
                } else if let Some(list) = broken.get(&id) {
                    out.extend(
                        list.iter()
                            .map(|(at, why)| Finding::by_machine(&id, at, Verdict::Violated, why)),
                    );
                } else if id == "rule-agent" && agent_broken {
                    out.push(Finding::by_machine(
                        &id,
                        &site,
                        Verdict::Violated,
                        "agent-exists か by-matches-kind に外れている",
                    ));
                } else {
                    out.push(Finding::by_machine(
                        &id,
                        &site,
                        Verdict::Verified,
                        "合っている",
                    ));
                }
            }
        }
    }
    sorted(out)
}

/// Concrete の rule を書いた場所（/properties/a/properties/b/x-prompt/write/0）から、インスタンスの欄の場所（/a/b）を出す
fn field_of(rule_pointer: &str) -> String {
    let head = rule_pointer.split("/x-prompt/").next().unwrap_or_default();
    head.split('/')
        .filter(|seg| !seg.is_empty() && *seg != "properties")
        .map(|seg| format!("/{seg}"))
        .collect()
}

/// インスタンスを、従う Concrete に照らす。不変条件 I8 ・ I12
///
/// 形の外れは json-schema の violated、必須の欄が無いか空なら根拠「必須の欄が空」の violated（rule required）、
/// 置き場所が x-instance-path と違えば level1-template の violated。Concrete の rule は1件ずつ結果を返す：
/// AI と人の rule は unverified、機械の rule は、任せる先を起動する仕組み（ステップ4）ができるまで unsupported。
/// path はインスタンスの置き場所、glossaries は Concrete の欄が指す glossary
pub fn validate_instance(
    document: &Value,
    concrete: &ConcreteSchema,
    shipped: &[AbstractSchema],
    glossaries: &[Glossary],
    path: Option<&str>,
) -> Vec<Finding> {
    let more: Vec<(String, Value)> =
        std::iter::once((concrete.id().to_string(), concrete.document().clone()))
            .chain(
                glossaries
                    .iter()
                    .map(|g| (g.id().to_string(), g.document().clone())),
            )
            .collect();
    let validator = match compile_with(concrete.document(), shipped, &more) {
        Ok(v) => v,
        Err(e) => {
            return vec![Finding::by_machine(
                SHAPE,
                "",
                Verdict::Error,
                &format!("Concrete を検証器に組めない：{e}"),
            )];
        }
    };
    let required: Vec<String> = concrete
        .document()
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|r| r.as_str().map(str::to_string))
        .collect();
    // 根の required の欄は、下で「必須の欄が空」として出すので、形の結果からは外す（if/then などほかの required は残す）
    let covered = |e: &jsonschema::ValidationError<'_>| {
        e.instance_path().as_str().is_empty()
            && matches!(e.kind(), jsonschema::error::ValidationErrorKind::Required { property }
                if property.as_str().is_some_and(|p| required.iter().any(|r| r == p)))
    };
    let mut out: Vec<Finding> = validator
        .iter_errors(document)
        .filter(|e| !covered(e))
        .map(|e| {
            Finding::by_machine(
                SHAPE,
                e.instance_path().as_str(),
                Verdict::Violated,
                &e.to_string(),
            )
        })
        .collect();
    let instance = match Instance::from_document(document.clone()) {
        Ok(i) => i,
        Err(e) => {
            out.push(Finding::by_machine("required", "", Verdict::Violated, &e));
            return sorted(out);
        }
    };
    for name in instance.empty_required(&required) {
        out.push(Finding::by_machine(
            "required",
            &format!("/{name}"),
            Verdict::Violated,
            &format!("必須の欄が空：{name}"),
        ));
    }
    if let (Some(template), Some(p)) = (
        concrete
            .document()
            .get("x-instance-path")
            .and_then(Value::as_str),
        path,
    ) {
        match instance.expected_path(template) {
            Some(expected) if expected == p => out.push(Finding::by_machine(
                "level1-template",
                "",
                Verdict::Verified,
                "置き場所が x-instance-path に合う",
            )),
            Some(expected) => out.push(Finding::by_machine(
                "level1-template",
                "",
                Verdict::Violated,
                &format!("置き場所 {p} が x-instance-path から決まる {expected} と違う"),
            )),
            None => out.push(Finding::by_machine(
                "level1-template",
                "",
                Verdict::Violated,
                &format!("x-instance-path {template} の名前を、欄の値で埋められない"),
            )),
        }
    }
    let delegates = concrete.delegates();
    for rule in concrete.rules() {
        let at = field_of(rule.pointer());
        let (verdict, by, why) = match rule.by() {
            Some(By::Ai) => (
                Verdict::Unverified,
                By::Ai,
                format!(
                    "AI の判定を依頼する（任せる先 {}）",
                    rule.agent().unwrap_or("judge")
                ),
            ),
            Some(By::Human) => (Verdict::Unverified, By::Human, "人の判定を待つ".to_string()),
            _ => {
                let cli = rule
                    .agent()
                    .is_some_and(|a| delegates.iter().any(|d| d.name() == a && d.has_argv()));
                let why = if cli {
                    "外のツールに任せる仕組み（ステップ4）がまだ無い"
                } else {
                    "判定する者がいない（値で決まるなら JSON Schema で書き、ほかは argv のある任せる先を指す）"
                };
                (Verdict::Unsupported, By::Machine, why.to_string())
            }
        };
        out.push(Finding::new(rule.id(), &at, verdict, &why, by));
    }
    for (keyword, rule, why) in [
        (
            "x-relationship",
            "target-exists",
            "参照先のインスタンスを id で引く仕組み（ステップ6 の search）がまだ無い",
        ),
        ("x-derived-from", "x-derived-from", "導出値の評価がまだ無い"),
        (
            "x-decided-by",
            "human-entry-only",
            "変更の記録と人の判定（ステップ3a）がまだ無いので判定できない",
        ),
    ] {
        if concrete.uses(keyword) {
            out.push(Finding::by_machine(rule, "", Verdict::Unsupported, why));
        }
    }
    sorted(out)
}

/// 違反（violated と error）だけを返す
pub fn problems(results: &[Finding]) -> Vec<Finding> {
    results
        .iter()
        .filter(|f| matches!(f.verdict(), Verdict::Violated | Verdict::Error))
        .cloned()
        .collect()
}

/// この書き込みが新しく持ち込んだ、機械の violated：書き込んだ後にだけある violated（場所と理由で比べる）。
/// 前からある violated と、AI ・ 人の判定は含めない
pub fn introduced(before: &[Finding], after: &[Finding]) -> Vec<Finding> {
    after
        .iter()
        .filter(|f| f.verdict() == Verdict::Violated && f.by() == By::Machine)
        .filter(|f| {
            !before.iter().any(|b| {
                b.rule() == f.rule() && b.pointer() == f.pointer() && b.reason() == f.reason()
            })
        })
        .cloned()
        .collect()
}

fn sorted(mut out: Vec<Finding>) -> Vec<Finding> {
    out.sort_by(|a, b| {
        a.pointer()
            .cmp(b.pointer())
            .then_with(|| a.rule().cmp(b.rule()))
            .then_with(|| a.reason().cmp(b.reason()))
    });
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::{introduced, problems, validate_declaration};
    use semantic_agent_schema_change::values::verdict::{Finding, Verdict};
    use semantic_agent_schema_declaration::aggregates::abstract_schema::AbstractSchema;
    use semantic_agent_schema_declaration::aggregates::glossary::Glossary;
    use serde_json::{Value, json};

    fn results(doc: &Value, glossaries: &[Glossary]) -> Vec<Finding> {
        let shipped = AbstractSchema::shipped().unwrap();
        validate_declaration(doc, &shipped[0], &shipped, glossaries, None, &[])
    }

    fn check(doc: &Value, glossaries: &[Glossary]) -> Vec<Finding> {
        problems(&results(doc, glossaries))
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

    fn verdict_of(all: &[Finding], rule: &str) -> Vec<&'static str> {
        all.iter()
            .filter(|f| f.rule() == rule)
            .map(|f| f.verdict().as_str())
            .collect()
    }

    /// traces: I3
    #[test]
    fn concrete_with_required_fields_has_no_problem() {
        assert!(check(&concrete(json!({})), &[]).is_empty());
    }

    /// traces: U1:事後
    #[test]
    fn every_applicable_abstract_rule_has_a_result() {
        let all = results(&concrete(json!({})), &[]);
        assert_eq!(
            (
                verdict_of(&all, "rule-agent"),
                verdict_of(&all, "noun"),
                verdict_of(&all, "never-reuse")
            ),
            (vec!["verified"], vec!["unverified"], vec!["unsupported"])
        );
    }

    /// traces: U1:事後
    #[test]
    fn rules_for_unused_keywords_are_not_listed() {
        assert!(verdict_of(&results(&concrete(json!({})), &[]), "level1-template").is_empty());
    }

    /// traces: I3
    #[test]
    fn the_shipped_abstract_passes_against_itself() {
        let shipped = AbstractSchema::shipped().unwrap();
        let f = validate_declaration(shipped[0].document(), &shipped[0], &shipped, &[], None, &[]);
        assert!(f.is_empty(), "{f:?}");
    }

    /// traces: I4
    #[test]
    fn field_without_description_and_prompt_is_violated() {
        let doc = concrete(json!({"properties": {"a": {"type": "string"}}}));
        assert_eq!(
            check(&doc, &[])
                .iter()
                .filter(|f| f.rule() == "json-schema")
                .count(),
            2
        );
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
        assert_eq!(
            verdict_of(&check(&doc, &[]), "no-abstract-names"),
            vec!["violated"]
        );
    }

    /// traces: I13
    #[test]
    fn same_document_gives_the_same_results() {
        let doc = concrete(json!({"$id": "bad", "x-color": 1}));
        assert_eq!(results(&doc, &[]), results(&doc, &[]));
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

    /// traces: I4
    #[test]
    fn relationship_to_unknown_concrete_is_violated() {
        let doc = concrete(
            json!({"properties": {"p": field(json!({"type": "string", "x-relationship": {"to": "urn:semantic-agent-schema:t:other:v1"}}))}}),
        );
        assert_eq!(
            verdict_of(&check(&doc, &[]), "target-exists"),
            vec!["violated"]
        );
    }

    /// traces: I4
    #[test]
    fn owner_not_in_the_path_is_violated() {
        let shipped = AbstractSchema::shipped().unwrap();
        let all = validate_declaration(
            &concrete(json!({})),
            &shipped[0],
            &shipped,
            &[],
            Some("other/record.schema.json"),
            &[],
        );
        assert_eq!(
            verdict_of(&problems(&all), "owner-kind-version"),
            vec!["violated"]
        );
    }

    /// traces: U1:失敗時
    #[test]
    fn violation_only_after_the_write_is_introduced() {
        let v = Finding::by_machine("r", "/a", Verdict::Violated, "必須");
        assert_eq!(introduced(&[], std::slice::from_ref(&v)), vec![v]);
    }

    /// traces: U1:事後
    #[test]
    fn violation_present_before_the_write_is_not_introduced() {
        let v = Finding::by_machine("r", "/a", Verdict::Violated, "必須");
        let same = std::slice::from_ref(&v);
        assert!(introduced(same, same).is_empty());
    }

    fn record() -> semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema {
        semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema::from_document(concrete(json!({
            "$id": "urn:semantic-agent-schema:t:record:v1",
            "x-instance-path": "t/{id}.json",
            "x-delegates": {"lint": {"role": "検査", "run": {"kind": "judging", "argv": ["lint"]}}},
            "type": "object",
            "required": ["$schema", "kind", "id", "why"],
            "properties": {
                "$schema": {"const": "urn:semantic-agent-schema:t:record:v1"},
                "kind": field(json!({"const": "record"})),
                "id": field(json!({"type": "string"})),
                "why": field(json!({"type": "string", "x-prompt": {"read": {"means": "理由"}, "write": [
                    {"id": "why-ai", "rule": "r", "why": "w", "test": "t", "by": "ai", "agent": "judge"},
                    {"id": "why-cli", "rule": "r", "why": "w", "test": "t", "by": "machine", "agent": "lint"}]}}))
            }
        }))).unwrap()
    }

    fn inst(extra: Value) -> Value {
        let mut d = json!({"$schema": "urn:semantic-agent-schema:t:record:v1", "kind": "record", "id": "r-1"});
        if let (Some(m), Some(e)) = (d.as_object_mut(), extra.as_object()) {
            m.extend(e.clone());
        }
        d
    }

    fn of_instance(doc: &Value, path: Option<&str>) -> Vec<Finding> {
        super::validate_instance(
            doc,
            &record(),
            &AbstractSchema::shipped().unwrap(),
            &[],
            path,
        )
    }

    /// traces: I12
    #[test]
    fn missing_required_field_is_violated_with_the_reason() {
        let f = problems(&of_instance(&inst(json!({})), None));
        assert_eq!(
            (f[0].rule(), f[0].reason()),
            ("required", "必須の欄が空：why")
        );
    }

    /// traces: I12
    #[test]
    fn empty_required_field_is_violated() {
        assert_eq!(
            problems(&of_instance(&inst(json!({"why": ""})), None))[0].pointer(),
            "/why"
        );
    }

    /// traces: U5:事後
    #[test]
    fn concrete_rules_get_one_result_each() {
        let all = of_instance(&inst(json!({"why": "理由"})), None);
        assert_eq!(
            (verdict_of(&all, "why-ai"), verdict_of(&all, "why-cli")),
            (vec!["unverified"], vec!["unsupported"])
        );
    }

    /// traces: U5:失敗時
    #[test]
    fn instance_at_another_path_breaks_level1_template() {
        assert_eq!(
            verdict_of(
                &of_instance(&inst(json!({"why": "理由"})), Some("t/other.json")),
                "level1-template"
            ),
            vec!["violated"]
        );
    }

    /// traces: U5:事後
    #[test]
    fn instance_at_its_path_passes_level1_template() {
        assert_eq!(
            verdict_of(
                &of_instance(&inst(json!({"why": "理由"})), Some("t/r-1.json")),
                "level1-template"
            ),
            vec!["verified"]
        );
    }

    /// traces: U6:失敗時
    #[test]
    fn conditional_required_is_kept_while_root_required_is_reported_once() {
        let mut doc = record().document().clone();
        doc["allOf"] = json!([{"if": {"properties": {"why": {"const": "x"}}}, "then": {"required": ["docs"]}}]);
        let c = semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema::from_document(doc).unwrap();
        let f = problems(&super::validate_instance(
            &inst(json!({"why": "x"})),
            &c,
            &AbstractSchema::shipped().unwrap(),
            &[],
            None,
        ));
        assert!(f.iter().any(|x| x.reason().contains("docs")));
    }
}
