//! abstract スキーマのテスト。具体のスキーマは `x-abstract` で使う版を示し、v1 では x-prompt を新しい形で書く。

use schema_driven_adapters::outbound::jmespath::Jmespath;
use schema_driven_core::domain::schema::Schema;
use schema_driven_core::domain::schema_rules::{check_schemas, SchemaFinding};
use schema_driven_core::ports::outbound::Query;
use serde_json::{json, Value};

const V1: &str = "schema-driven/abstract@v1";
const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

fn run(schema: Value) -> Vec<SchemaFinding> {
    check_schemas(
        &[("glossary.schema.json".to_owned(), schema)],
        &|expression: &str| Jmespath.parse(expression).map_err(|error| error.0),
    )
}

fn rules(findings: &[SchemaFinding]) -> Vec<&str> {
    findings
        .iter()
        .map(|finding| finding.rule.as_str())
        .collect()
}

/// v1 の x-prompt。
fn prompt() -> Value {
    json!({
        "read": {"means": "用語の一覧である。", "shown": "見出しの下に表で出る。"},
        "write": [
            {"id": "one-word", "rule": "1行に1語を書く。", "why": "語ごとに参照できるようにする。", "test": "1行に語が1つだけかを見る。"}
        ]
    })
}

fn field(extra: Value) -> Value {
    let mut base = json!({"type": "string", "description": "説明", "x-prompt": prompt()});
    for (key, value) in extra.as_object().unwrap() {
        base[key] = value.clone();
    }
    base
}

/// v1 に従っている具体のスキーマ。
fn good() -> Value {
    json!({"$schema": DIALECT, "x-abstract": V1, "$id": "lexicon/glossary@v1", "title": "用語集", "description": "語の一覧", "x-prompt": prompt(),
           "x-generates": "declarations/GLO-<番号>.json",
           "properties": {
               "kind": field(json!({"const": "glossary"})),
               "word": field(json!({}))},
           "required": ["kind", "word"]})
}

#[test]
fn a_v1_schema_validates_instances() {
    let schema = Schema::new("glossary.schema.json", good(), vec![]);
    let validation = schema
        .validate(&json!({"kind": "glossary"}))
        .expect("v1 の $schema で検証器を作れる");
    assert_eq!(validation.errors, vec![]);
    assert_eq!(validation.unfilled.len(), 1);
}

#[test]
fn a_v1_schema_in_the_new_form_has_no_findings() {
    assert_eq!(run(good()), vec![]);
}

#[test]
fn a_v1_schema_rejects_string_prompts() {
    let mut schema = good();
    schema["properties"]["word"]["x-prompt"] = json!({"read": "r", "write": "w"});
    assert!(rules(&run(schema)).contains(&"注釈の形"));
}

#[test]
fn a_v1_read_needs_means() {
    let mut schema = good();
    schema["properties"]["word"]["x-prompt"]["read"] = json!({"shown": "表で出る。"});
    assert!(rules(&run(schema)).contains(&"注釈の形"));
}

#[test]
fn rule_ids_do_not_repeat_within_one_prompt() {
    let mut schema = good();
    let rule = schema["properties"]["word"]["x-prompt"]["write"][0].clone();
    schema["properties"]["word"]["x-prompt"]["write"]
        .as_array_mut()
        .unwrap()
        .push(rule);
    assert!(rules(&run(schema)).contains(&"決まりの id の重なり"));
}

#[test]
fn a_sentence_is_at_most_120_characters() {
    let mut schema = good();
    schema["properties"]["word"]["x-prompt"]["write"][0]["why"] = json!("あ".repeat(121));
    assert!(rules(&run(schema)).contains(&"注釈の形"));
}

#[test]
fn bad_and_good_come_as_a_pair() {
    let mut schema = good();
    schema["properties"]["word"]["x-prompt"]["write"][0]["bad"] = json!("2語を1行に書く。");
    assert!(rules(&run(schema)).contains(&"注釈の形"));
}

#[test]
fn a_schema_without_a_version_is_refused() {
    let unversioned = json!({"title": "用語集", "description": "語の一覧",
        "properties": {"word": {"type": "string", "description": "語", "x-prompt": {"read": "r", "write": "w"}}}});
    assert_eq!(rules(&run(unversioned)), vec!["版が無い"]);
}

#[test]
fn an_unknown_version_is_reported() {
    let mut schema = good();
    schema["x-abstract"] = json!("schema-driven/abstract@v9");
    assert!(rules(&run(schema)).contains(&"知らない版"));
}

#[test]
fn names_under_defs_are_not_annotations() {
    let mut schema = good();
    schema["$defs"] = json!({"x-ref": {"type": "object", "description": "参照の形",
        "x-prompt": prompt(),
        "properties": {"to": field(json!({}))}}});
    let findings = run(schema);
    assert!(
        !rules(&findings).contains(&"注釈の形")
            && !rules(&findings).contains(&"申告していない注釈"),
        "{findings:?}"
    );
}
