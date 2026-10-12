//! 検証エラーの文。jsonschema の英語の文ではなく、キーワードと値を残した日本語の文で返す。
//! AI エージェントとデータの持ち主は、この文を読んで直す。

use schema_driven_core::domain::schema::Schema;
use serde_json::{json, Value};

/// スキーマでインスタンスを検証し、エラーの（場所, 理由）を返す。
fn reasons(schema: Value, instance: Value) -> Vec<(String, String)> {
    let validation = Schema::new("test.schema.json", schema, vec![])
        .validate(&instance)
        .unwrap();
    validation
        .errors
        .iter()
        .map(|error| (error.property().to_owned(), error.reason().to_owned()))
        .collect()
}

fn reason(schema: Value, instance: Value) -> String {
    let all = reasons(schema, instance);
    assert_eq!(all.len(), 1, "{all:?}");
    all[0].1.clone()
}

#[test]
fn type_names_the_expected_type_and_the_value() {
    assert_eq!(
        reason(json!({"type": "string"}), json!(1)),
        "型が string でない（値：1）"
    );
    assert_eq!(
        reason(json!({"type": ["string", "null"]}), json!(1)),
        "型が null ・ string のどれでもない（値：1）"
    );
}

#[test]
fn values_out_of_the_allowed_set() {
    assert_eq!(
        reason(json!({"enum": ["a", "b"]}), json!("c")),
        "enum の候補（\"a\" ・ \"b\"）のどれでもない（値：\"c\"）"
    );
    assert_eq!(
        reason(json!({"const": "document"}), json!("thing")),
        "const の \"document\" と違う（値：\"thing\"）"
    );
}

#[test]
fn lengths_and_numbers_keep_the_limit() {
    assert_eq!(
        reason(json!({"minLength": 1}), json!("")),
        "文字数が minLength の 1 より少ない"
    );
    assert_eq!(
        reason(json!({"maxLength": 2}), json!("abc")),
        "文字数が maxLength の 2 より多い"
    );
    assert_eq!(
        reason(json!({"minimum": 1}), json!(0)),
        "値が minimum の 1 より小さい（値：0）"
    );
    assert_eq!(
        reason(json!({"maximum": 1}), json!(2)),
        "値が maximum の 1 より大きい（値：2）"
    );
    assert_eq!(
        reason(json!({"minItems": 1}), json!([])),
        "要素の数が minItems の 1 より少ない"
    );
    assert_eq!(
        reason(json!({"uniqueItems": true}), json!([1, 1])),
        "同じ要素が2つ以上ある（uniqueItems）"
    );
}

#[test]
fn unknown_properties_are_listed_by_name() {
    assert_eq!(
        reason(
            json!({"type": "object", "properties": {"heading": {}}, "additionalProperties": false}),
            json!({"heading": "a", "body": [], "note": 1})
        ),
        "スキーマに無いプロパティがある：body ・ note（additionalProperties）"
    );
}

#[test]
fn patterns_and_formats_name_the_rule() {
    assert_eq!(
        reason(json!({"pattern": "^UC-[0-9]+$"}), json!("X")),
        "pattern の ^UC-[0-9]+$ に合わない（値：\"X\"）"
    );
}

#[test]
fn combinations_say_how_many_branches_matched() {
    let branches = json!({"oneOf": [{"type": "string"}, {"type": "integer"}]});
    assert_eq!(
        reason(branches.clone(), json!(true)),
        "oneOf のどのスキーマにも合わない"
    );
    assert_eq!(
        reason(
            json!({"oneOf": [{"type": "integer"}, {"minimum": 0}]}),
            json!(1)
        ),
        "oneOf のスキーマの2つ以上に合う"
    );
    assert_eq!(
        reason(json!({"anyOf": [{"type": "string"}]}), json!(1)),
        "anyOf のどのスキーマにも合わない"
    );
}

#[test]
fn the_place_stays_a_json_pointer() {
    let all = reasons(
        json!({"type": "object", "properties": {"sections": {"type": "array", "items": {"type": "string"}}}}),
        json!({"sections": ["a", 2]}),
    );
    assert_eq!(
        all,
        vec![(
            "/sections/1".to_owned(),
            "型が string でない（値：2）".to_owned()
        )]
    );
}
