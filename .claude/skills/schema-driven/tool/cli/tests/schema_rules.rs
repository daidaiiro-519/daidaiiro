//! 具体のスキーマの検査（references/meta-schema.json と、すべての深さの注釈。ボード schema-driven-build の論点6 D）のテスト。

use schema_driven_adapters::outbound::jmespath::Jmespath;
use schema_driven_core::domain::schema_rules::{check_schemas, SchemaFinding};
use schema_driven_core::ports::outbound::Query;
use serde_json::{json, Value};

fn run(schemas: Vec<(&str, Value)>) -> Vec<SchemaFinding> {
    let list: Vec<(String, Value)> = schemas
        .into_iter()
        .map(|(file_name, schema)| (file_name.to_owned(), schema))
        .collect();
    check_schemas(&list, &|expression: &str| {
        Jmespath.parse(expression).map_err(|error| error.0)
    })
}

fn rules(findings: &[SchemaFinding]) -> Vec<&str> {
    findings
        .iter()
        .map(|finding| finding.rule.as_str())
        .collect()
}

fn field(extra: Value) -> Value {
    let mut base =
        json!({"type": "string", "description": "説明", "x-prompt": {"read": "r", "write": "w"}});
    for (key, value) in extra.as_object().unwrap() {
        base[key] = value.clone();
    }
    base
}

/// 決まりに従っている具体のスキーマ。
fn good() -> Value {
    json!({"title": "用語集", "description": "語の一覧", "x-generates": "decls/GLO-<番号>.json",
           "x-view": {"label": "header.name"},
           "properties": {
               "kind": field(json!({"const": "glossary"})),
               "id": field(json!({"pattern": "^GLO-[0-9]+$", "x-view": {"hidden": true}})),
               "terms": field(json!({"type": "array", "items": {"type": "object", "properties": {
                   "word": {"type": "string", "x-view": {"text": "{word | quote(@)}"}}}}}))}})
}

#[test]
fn conforming_schema_has_no_findings() {
    assert_eq!(run(vec![("glossary.schema.json", good())]), vec![]);
}

#[test]
fn meta_schema_requires_prompt_per_field_and_kind_for_generating_schemas() {
    let mut schema = good();
    schema["properties"]["terms"]
        .as_object_mut()
        .unwrap()
        .remove("x-prompt");
    schema["properties"].as_object_mut().unwrap().remove("kind");
    let findings = run(vec![("glossary.schema.json", schema)]);
    assert_eq!(
        rules(&findings),
        vec!["メタスキーマ", "メタスキーマ"],
        "{findings:?}"
    );
    assert!(findings
        .iter()
        .any(|finding| finding.at.contains("/properties/kind")));
    assert!(findings
        .iter()
        .any(|finding| finding.at.contains("/properties/terms/x-prompt")));
}

#[test]
fn annotation_shapes_are_checked_at_every_depth() {
    let mut schema = good();
    schema["properties"]["terms"]["items"]["properties"]["word"]["x-ref"] =
        json!({"to": "glossary", "meaning": "語"});
    let findings = run(vec![("glossary.schema.json", schema)]);
    assert_eq!(rules(&findings), vec!["注釈の形"], "{findings:?}");
    assert_eq!(
        findings[0].at,
        "/properties/terms/items/properties/word/x-ref"
    );
}

#[test]
fn templates_must_be_jmespath_and_pipes_must_call_functions() {
    let mut schema = good();
    let word = &mut schema["properties"]["terms"]["items"]["properties"]["word"];
    word["x-view"] = json!({"text": "{word|name} と {word[} と {word | name(@)}"});
    let findings = run(vec![("glossary.schema.json", schema)]);
    assert_eq!(
        rules(&findings),
        vec!["古い絞りの書き方", "文の型"],
        "{findings:?}"
    );
    assert!(findings[0].message.contains("{word|name}"));
}

#[test]
fn undeclared_annotations_are_findings_and_declared_ones_pass() {
    let mut schema = good();
    schema["properties"]["terms"]["x-test-spec"] = json!({"checks": "x"});
    assert_eq!(
        rules(&run(vec![("glossary.schema.json", schema.clone())])),
        vec!["申告していない注釈"]
    );
    schema["x-annotations"] = json!({"x-test-spec": "test-spec.schema.json"});
    assert_eq!(run(vec![("glossary.schema.json", schema)]), vec![]);
}

#[test]
fn referenced_kinds_must_have_id() {
    let mut glossary = good();
    glossary["properties"].as_object_mut().unwrap().remove("id");
    let use_case = json!({"title": "ユースケース", "description": "やり取り", "x-generates": "decls/UC-<番号>.json",
                    "properties": {
                        "kind": field(json!({"const": "use_case"})),
                        "id": field(json!({})),
                        "terms": field(json!({"x-ref": {"to": "glossary", "item": true}}))}});
    let findings = run(vec![
        ("glossary.schema.json", glossary.clone()),
        ("use_case.schema.json", use_case.clone()),
    ]);
    assert_eq!(
        rules(&findings),
        vec!["参照される種類の id"],
        "{findings:?}"
    );
    assert_eq!(findings[0].schema, "glossary.schema.json");
    // 指されていなければ、id は求めない
    assert_eq!(run(vec![("glossary.schema.json", glossary)]), vec![]);
}

#[test]
fn check_schemas_tool_reads_a_directory_of_schemas() {
    use schema_driven_adapters::inbound::tools::Toolbox;
    use schema_driven_adapters::outbound::fs::FileSystem;
    use schema_driven_core::application::checks::Checks;
    use schema_driven_core::application::instances::Instances;
    use std::sync::Arc;
    let dir = std::env::temp_dir().join(format!("sd-check-schemas-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut bad = good();
    bad["properties"]["terms"]["x-foo"] = json!(1);
    std::fs::write(dir.join("glossary.schema.json"), bad.to_string()).unwrap();
    std::fs::write(dir.join("GLO-1.json"), "{}").unwrap();
    let (files, query) = (Arc::new(FileSystem), Arc::new(Jmespath));
    let (instances, checks) = (
        Instances::new(files.clone(), files.clone(), query.clone()),
        Checks::new(files.clone(), files, query),
    );
    let args = json!({"dir": dir.to_str().unwrap()})
        .as_object()
        .cloned()
        .unwrap();
    let (code, out) = Toolbox::base().dispatch("check-schemas", &args, &instances, &checks);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(code, 0);
    assert_eq!(out["schemas"], json!(["glossary.schema.json"]));
    assert_eq!(out["findings"][0]["rule"], "申告していない注釈");
    assert_eq!(out["findings"].as_array().unwrap().len(), 1);
}
