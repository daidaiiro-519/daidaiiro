//! バリデーターが、方言の $dynamicAnchor: meta と独自の $vocabulary を扱えることの実地の証拠（ステップ1 のはじめの照合）。
//! 方言を通すと、入れ子のサブスキーマの x- の形にも語彙のメタスキーマが掛かる。

#![allow(clippy::unwrap_used)]

use outbound_jsonschema::Validator;
use serde_json::json;
use validation_application::ports::SchemaValidator;

const DIALECT: &str = "urn:semantic-agent-schema:published-language:dialect:v1";
const ABSTRACT: &str = "urn:semantic-agent-schema:published-language:abstract:v1";

#[test]
fn abstract_follows_dialect() {
    let c = Validator::new().unwrap();
    let body = c.shipped(ABSTRACT).unwrap();
    assert_eq!(
        c.check(DIALECT, &body, &[]).unwrap(),
        Vec::<(String, String)>::new()
    );
}

#[test]
fn nested_x_shape_is_checked_through_dialect() {
    let c = Validator::new().unwrap();
    let bad = json!({
        "$schema": ABSTRACT,
        "properties": {"a": {"properties": {"b": {"x-decided-by": 1, "x-color": "red"}}}}
    });
    let errors = c.check(DIALECT, &bad, &[]).unwrap();
    assert!(
        errors
            .iter()
            .any(|(p, _)| p == "/properties/a/properties/b/x-decided-by"),
        "{errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|(p, m)| p == "/properties/a/properties/b" && m.contains("x-color")),
        "x- の閉じた一覧に無いキーは断る：{errors:?}"
    );
}

#[test]
fn concrete_can_be_used_as_schema() {
    let c = Validator::new().unwrap();
    let concrete = json!({
        "$schema": ABSTRACT, "$id": "urn:example:t:thing:v1",
        "title": "もの", "description": "試験のものである。",
        "type": "object", "properties": {"n": {"title": "数", "description": "数である。", "x-prompt": {}, "type": "integer"}}
    });
    let more = vec![("urn:example:t:thing:v1".to_string(), concrete)];
    let errors = c
        .check("urn:example:t:thing:v1", &json!({"n": "x"}), &more)
        .unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].0, "/n");
}
