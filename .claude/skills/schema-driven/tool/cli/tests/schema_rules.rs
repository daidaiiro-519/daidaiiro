//! 具体のスキーマの検査（references/meta-schema.json と、すべての深さの注釈。ボード schema-driven-build の論点6 D）のテスト。

use schema_driven_adapters::outbound::jmespath::Jmespath;
use schema_driven_core::domain::schema_rules::{check_schemas, SchemaFinding};
use schema_driven_core::ports::outbound::Query;
use serde_json::{json, Value};

fn run(schemas: Vec<(&str, Value)>) -> Vec<SchemaFinding> {
    let list: Vec<(String, Value)> = schemas
        .into_iter()
        .map(|(n, v)| (n.to_owned(), v))
        .collect();
    check_schemas(&list, &|e: &str| Jmespath.parse(e).map_err(|x| x.0))
}

fn rules(f: &[SchemaFinding]) -> Vec<&str> {
    f.iter().map(|x| x.rule.as_str()).collect()
}

fn field(extra: Value) -> Value {
    let mut base =
        json!({"type": "string", "description": "説明", "x-prompt": {"read": "r", "write": "w"}});
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
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
    let mut s = good();
    s["properties"]["terms"]
        .as_object_mut()
        .unwrap()
        .remove("x-prompt");
    s["properties"].as_object_mut().unwrap().remove("kind");
    let f = run(vec![("glossary.schema.json", s)]);
    assert_eq!(rules(&f), vec!["メタスキーマ", "メタスキーマ"], "{f:?}");
    assert!(f.iter().any(|x| x.at.contains("/properties/kind")));
    assert!(f
        .iter()
        .any(|x| x.at.contains("/properties/terms/x-prompt")));
}

#[test]
fn annotation_shapes_are_checked_at_every_depth() {
    let mut s = good();
    s["properties"]["terms"]["items"]["properties"]["word"]["x-ref"] =
        json!({"to": "glossary", "meaning": "語"});
    let f = run(vec![("glossary.schema.json", s)]);
    assert_eq!(rules(&f), vec!["注釈の形"], "{f:?}");
    assert_eq!(f[0].at, "/properties/terms/items/properties/word/x-ref");
}

#[test]
fn templates_must_be_jmespath_and_pipes_must_call_functions() {
    let mut s = good();
    let w = &mut s["properties"]["terms"]["items"]["properties"]["word"];
    w["x-view"] = json!({"text": "{word|name} と {word[} と {word | name(@)}"});
    let f = run(vec![("glossary.schema.json", s)]);
    assert_eq!(rules(&f), vec!["古い絞りの書き方", "文の型"], "{f:?}");
    assert!(f[0].message.contains("{word|name}"));
}

#[test]
fn undeclared_annotations_are_findings_and_declared_ones_pass() {
    let mut s = good();
    s["properties"]["terms"]["x-test-spec"] = json!({"checks": "x"});
    assert_eq!(
        rules(&run(vec![("glossary.schema.json", s.clone())])),
        vec!["申告していない注釈"]
    );
    s["x-annotations"] = json!({"x-test-spec": "test-spec.schema.json"});
    assert_eq!(run(vec![("glossary.schema.json", s)]), vec![]);
}

#[test]
fn referenced_kinds_must_have_id() {
    let mut glo = good();
    glo["properties"].as_object_mut().unwrap().remove("id");
    let uc = json!({"title": "ユースケース", "description": "やり取り", "x-generates": "decls/UC-<番号>.json",
                    "properties": {
                        "kind": field(json!({"const": "use_case"})),
                        "id": field(json!({})),
                        "terms": field(json!({"x-ref": {"to": "glossary", "item": true}}))}});
    let f = run(vec![
        ("glossary.schema.json", glo.clone()),
        ("use_case.schema.json", uc.clone()),
    ]);
    assert_eq!(rules(&f), vec!["参照される種類の id"], "{f:?}");
    assert_eq!(f[0].schema, "glossary.schema.json");
    // 指されていなければ、id は求めない
    assert_eq!(run(vec![("glossary.schema.json", glo)]), vec![]);
}

#[test]
fn check_schemas_tool_reads_a_directory_of_schemas() {
    use schema_driven_adapters::inbound::tools::Toolbox;
    use schema_driven_adapters::outbound::fs::FileSystem;
    use schema_driven_core::application::checks::Checks;
    use schema_driven_core::application::instances::Instances;
    let dir = std::env::temp_dir().join(format!("sd-check-schemas-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut bad = good();
    bad["properties"]["terms"]["x-foo"] = json!(1);
    std::fs::write(dir.join("glossary.schema.json"), bad.to_string()).unwrap();
    std::fs::write(dir.join("GLO-1.json"), "{}").unwrap();
    let (files, query) = (FileSystem, Jmespath);
    let (uc, cc) = (
        Instances::new(&files, &files, &query),
        Checks::new(&files, &files, &query),
    );
    let args = json!({"dir": dir.to_str().unwrap()})
        .as_object()
        .cloned()
        .unwrap();
    let (code, out) = Toolbox::base().dispatch("check-schemas", &args, &uc, &cc);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(code, 0);
    assert_eq!(out["schemas"], json!(["glossary.schema.json"]));
    assert_eq!(out["findings"][0]["rule"], "申告していない注釈");
    assert_eq!(out["findings"].as_array().unwrap().len(), 1);
}
