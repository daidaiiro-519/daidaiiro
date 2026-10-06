//! 描画の文脈（page.schema.json の context、ACDR 0133）のテスト（テストレベル component）。
//! this ・ schema ・ derived ・ links ・ referrers ・ instances の6つの形を確かめる。

use schema_driven_core::application::context::contexts;
use schema_driven_core::domain::check::Doc;
use schema_driven_core::domain::schema::Schema;
use schema_driven_core::domain::values::JsonValue;
use serde_json::{json, Value};

fn doc<'a>(path: &str, value: Value, schema: &'a Schema) -> Doc<'a> {
    let hash = JsonValue::new(&value.to_string()).hash();
    Doc {
        path: path.to_owned(),
        value,
        hash,
        schema,
    }
}

fn uc_schema() -> Value {
    json!({
        "properties": {
            "terms": {"x-ref": {"to": "glossary", "item": true}},
            "rules": {"x-ref": {"to": "rule"}},
            "classification": {"$ref": "#/$defs/classification"}
        },
        "$defs": {"classification": {
            "properties": {
                "a": {"title": "問いA"},
                "b": {"title": "問いB"},
                "category": {}
            },
            "x-derive": {
                "title": "導いた分類",
                "rules": [{"when": {"a": true, "b": false}, "then": "X"}, {"when": {"b": true}, "then": "Z"}],
                "otherwise": "Y",
                "declared": "category"
            }
        }}
    })
}

struct Fixture {
    uc: Schema,
    glo: Schema,
    rule: Schema,
}

fn fixture() -> Fixture {
    Fixture {
        uc: Schema::new("uc.schema.json", uc_schema(), vec![]),
        glo: Schema::new(
            "glossary.schema.json",
            json!({"properties": {"terms": {"items": {}}}}),
            vec![],
        ),
        rule: Schema::new("rule.schema.json", json!({}), vec![]),
    }
}

fn values() -> (Value, Value, Value) {
    (
        json!({"id": "UC-1", "kind": "uc", "terms": ["GLO-1.TERM-1"], "rules": ["R-1", "R-9"],
               "classification": {"a": true, "b": false, "category": "Y"}}),
        json!({"id": "GLO-1", "kind": "glossary", "terms": [{"id": "TERM-1", "word": "注文"}]}),
        json!({"id": "R-1", "kind": "rule"}),
    )
}

#[test]
fn this_schema_and_instances() {
    let f = fixture();
    let (uc, glo, rule) = values();
    let docs = vec![
        doc("uc.json", uc.clone(), &f.uc),
        doc("glo.json", glo.clone(), &f.glo),
        doc("r.json", rule.clone(), &f.rule),
    ];
    let c = contexts(&docs);
    assert_eq!(c.len(), 3);
    assert_eq!(c[0]["this"], uc);
    assert_eq!(c[0]["schema"], uc_schema());
    assert_eq!(c[0]["instances"], json!([uc, glo, rule]));
}

#[test]
fn derived_has_value_title_declared_and_questions_in_rule_order() {
    let f = fixture();
    let (uc, glo, rule) = values();
    let docs = vec![
        doc("uc.json", uc, &f.uc),
        doc("glo.json", glo, &f.glo),
        doc("r.json", rule, &f.rule),
    ];
    let c = contexts(&docs);
    assert_eq!(
        c[0]["derived"],
        json!({"classification": {
            "value": "X",
            "title": "導いた分類",
            "declared": "Y",
            "questions": [
                {"key": "a", "title": "問いA", "answer": true},
                {"key": "b", "title": "問いB", "answer": false}
            ]
        }})
    );
    assert_eq!(c[1]["derived"], json!({}));
}

#[test]
fn links_list_each_written_reference_with_resolved_target_or_null() {
    let f = fixture();
    let (uc, glo, rule) = values();
    let docs = vec![
        doc("uc.json", uc, &f.uc),
        doc("glo.json", glo, &f.glo),
        doc("r.json", rule, &f.rule),
    ];
    let c = contexts(&docs);
    assert_eq!(
        c[0]["links"],
        json!([
            {"at": "/terms", "value": "GLO-1.TERM-1", "target": "GLO-1.TERM-1"},
            {"at": "/rules", "value": "R-1", "target": "R-1"},
            {"at": "/rules", "value": "R-9", "target": null}
        ])
    );
    assert_eq!(c[2]["links"], json!([]));
}

#[test]
fn referrers_list_who_points_here_and_which_item() {
    let f = fixture();
    let (uc, glo, rule) = values();
    let docs = vec![
        doc("uc.json", uc, &f.uc),
        doc("glo.json", glo, &f.glo),
        doc("r.json", rule, &f.rule),
    ];
    let c = contexts(&docs);
    assert_eq!(
        c[1]["referrers"],
        json!([{"at": "/terms", "from": "UC-1", "item": "TERM-1"}])
    );
    assert_eq!(
        c[2]["referrers"],
        json!([{"at": "/rules", "from": "UC-1", "item": null}])
    );
    assert_eq!(c[0]["referrers"], json!([]));
}

#[test]
fn derived_key_is_place_from_root_for_nested_objects() {
    let s = Schema::new(
        "n.schema.json",
        json!({"properties": {"parts": {"items": {"properties": {"c": {
            "x-derive": {"rules": [{"when": {"a": true}, "then": 1}], "otherwise": 0}}}}}}}),
        vec![],
    );
    let docs = vec![doc(
        "n.json",
        json!({"id": "N-1", "kind": "n", "parts": [{"c": {"a": true}}]}),
        &s,
    )];
    let c = contexts(&docs);
    assert_eq!(
        c[0]["derived"],
        json!({"parts/0/c": {"value": 1, "title": null, "declared": null,
                             "questions": [{"key": "a", "title": "a", "answer": true}]}})
    );
}

#[test]
fn contexts_match_the_context_shape_in_page_schema() {
    let page: Value =
        serde_json::from_str(include_str!("../../../references/page.schema.json")).unwrap();
    let shape = Schema::new(
        "context.schema.json",
        json!({"$ref": "page.schema.json#/$defs/context"}),
        vec![("page.schema.json".into(), page)],
    );
    let f = fixture();
    let (uc, glo, rule) = values();
    let docs = vec![
        doc("uc.json", uc, &f.uc),
        doc("glo.json", glo, &f.glo),
        doc("r.json", rule, &f.rule),
    ];
    for c in contexts(&docs) {
        let v = shape.validate(&c).unwrap();
        assert!(v.errors.is_empty(), "{:?}", v.errors);
    }
}
