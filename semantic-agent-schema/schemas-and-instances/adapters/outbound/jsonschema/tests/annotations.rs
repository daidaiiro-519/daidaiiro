//! 注釈：形が通った所ごとに、当たったサブスキーマの注釈（x- を含む）がインスタンスの所付きで返る。

#![allow(clippy::unwrap_used)]

use outbound_jsonschema::Validator;
use serde_json::json;

#[test]
fn nested_x_decided_by_comes_with_the_instance_location() {
    let c = Validator::new().unwrap();
    let concrete = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "urn:example:t:nested:v1",
        "$defs": {"status": {"type": "string", "x-decided-by": "human", "default": "draft"}},
        "type": "object",
        "properties": {
            "steps": {"type": "array", "items": {"type": "object", "properties": {
                "approved": {"$ref": "#/$defs/status"}}}}
        }
    });
    let more = vec![("urn:example:t:nested:v1".to_string(), concrete)];
    let got = c
        .annotations(
            "urn:example:t:nested:v1",
            &json!({"steps": [{"approved": "draft"}, {"approved": "ok"}]}),
            &more,
        )
        .unwrap();
    let human: Vec<(&str, &serde_json::Value)> = got
        .iter()
        .filter(|(_, a)| a.get("x-decided-by") == Some(&json!("human")))
        .map(|(at, a)| (at.as_str(), &a["default"]))
        .collect();
    assert_eq!(
        human,
        [
            ("/steps/0/approved", &json!("draft")),
            ("/steps/1/approved", &json!("draft"))
        ]
    );
}
