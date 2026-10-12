//! 描画の文脈（page.schema.json の context）。ページテンプレートの JMESPath 式は、この値で評価する。

use crate::domain::check::{graph, Doc};
use serde_json::{json, Map, Value};

fn id_of(value: &Value) -> Value {
    value.get("id").cloned().unwrap_or(Value::Null)
}

/// インスタンスごとの描画の文脈を、docs と同じ順に返す。
pub fn contexts(docs: &[Doc]) -> Vec<Value> {
    let resolved = graph(docs);
    let instances: Vec<Value> = docs.iter().map(|doc| doc.value.clone()).collect();
    (0..docs.len())
        .map(|doc_index| {
            let mut derived = Map::new();
            for derived_value in resolved.derived.iter().filter(|derived_value| derived_value.doc == doc_index) {
                derived.insert(
                    derived_value.at.trim_start_matches('/').to_owned(),
                    json!({"value": derived_value.value, "title": derived_value.title, "declared": derived_value.declared, "questions": derived_value.questions}),
                );
            }
            let links: Vec<Value> = resolved
                .links
                .iter()
                .filter(|link| link.doc == doc_index)
                .map(|link| json!({"at": link.at, "value": link.value, "target": link.label}))
                .collect();
            let referrers: Vec<Value> = resolved
                .links
                .iter()
                .filter_map(|link| match &link.target {
                    Some((target_doc, item)) if *target_doc == doc_index => Some(json!({
                        "at": link.at,
                        "from": id_of(&docs[link.doc].value),
                        "item": item,
                    })),
                    _ => None,
                })
                .collect();
            json!({
                "this": docs[doc_index].value,
                "schema": docs[doc_index].schema.root(),
                "derived": derived,
                "links": links,
                "referrers": referrers,
                "instances": instances,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    //! 描画の文脈（page.schema.json の context）のテスト。
    //! this ・ schema ・ derived ・ links ・ referrers ・ instances の6つの形を確かめる。

    use super::contexts;
    use crate::domain::check::Doc;
    use crate::domain::schema::Schema;
    use crate::domain::values::JsonValue;
    use serde_json::{json, Value};

    fn doc<'schema>(path: &str, value: Value, schema: &'schema Schema) -> Doc<'schema> {
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

    /// 見本の3件（UC-1 ・ GLO-1 ・ R-1）の文脈。
    fn three() -> Vec<Value> {
        let schemas = fixture();
        let (uc, glo, rule) = values();
        let docs = vec![
            doc("uc.json", uc, &schemas.uc),
            doc("glo.json", glo, &schemas.glo),
            doc("r.json", rule, &schemas.rule),
        ];
        contexts(&docs)
    }

    /// 文脈のどれも、page.schema.json の context の形で検証エラーが無いか。検証エラーを並べて返す。
    fn errors_against_the_context_shape(list: &[Value]) -> Vec<String> {
        let page: Value =
            serde_json::from_str(crate::embedded::schema_text("schema-driven/page@v1")).unwrap();
        let shape = Schema::new(
            "context.schema.json",
            json!({"$ref": "page.schema.json#/$defs/context"}),
            vec![("page.schema.json".into(), page)],
        );
        list.iter()
            .flat_map(|context| shape.validate(context).unwrap().errors)
            .map(|error| format!("{} {}", error.property(), error.reason()))
            .collect()
    }

    #[test]
    fn this_schema_and_instances() {
        let (uc, glo, rule) = values();
        let page_contexts = three();
        assert_eq!(page_contexts.len(), 3);
        assert_eq!(page_contexts[0]["this"], uc);
        assert_eq!(page_contexts[0]["schema"], uc_schema());
        assert_eq!(page_contexts[0]["instances"], json!([uc, glo, rule]));
    }

    #[test]
    fn derived_has_value_title_declared_and_questions_in_rule_order() {
        let page_contexts = three();
        assert_eq!(
            page_contexts[0]["derived"],
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
        assert_eq!(page_contexts[1]["derived"], json!({}));
    }

    #[test]
    fn links_list_each_written_reference_with_resolved_target_or_null() {
        let page_contexts = three();
        assert_eq!(
            page_contexts[0]["links"],
            json!([
                {"at": "/terms", "value": "GLO-1.TERM-1", "target": "GLO-1.TERM-1"},
                {"at": "/rules", "value": "R-1", "target": "R-1"},
                {"at": "/rules", "value": "R-9", "target": null}
            ])
        );
        assert_eq!(page_contexts[2]["links"], json!([]));
    }

    #[test]
    fn referrers_list_who_points_here_and_which_item() {
        let page_contexts = three();
        assert_eq!(
            page_contexts[1]["referrers"],
            json!([{"at": "/terms", "from": "UC-1", "item": "TERM-1"}])
        );
        assert_eq!(
            page_contexts[2]["referrers"],
            json!([{"at": "/rules", "from": "UC-1", "item": null}])
        );
        assert_eq!(page_contexts[0]["referrers"], json!([]));
    }

    #[test]
    fn derived_key_is_place_from_root_for_nested_objects() {
        let nested_schema = Schema::new(
            "n.schema.json",
            json!({"properties": {"parts": {"items": {"properties": {"c": {
                "x-derive": {"rules": [{"when": {"a": true}, "then": 1}], "otherwise": 0}}}}}}}),
            vec![],
        );
        let docs = vec![doc(
            "n.json",
            json!({"id": "N-1", "kind": "n", "parts": [{"c": {"a": true}}]}),
            &nested_schema,
        )];
        let page_contexts = contexts(&docs);
        assert_eq!(
            page_contexts[0]["derived"],
            json!({"parts/0/c": {"value": 1, "title": null, "declared": null,
                                 "questions": [{"key": "a", "title": "a", "answer": true}]}})
        );
    }

    #[test]
    fn contexts_match_the_context_shape_in_page_schema() {
        let page_contexts = three();
        assert_eq!(page_contexts.len(), 3);
        assert_eq!(
            errors_against_the_context_shape(&page_contexts),
            Vec::<String>::new()
        );
    }
}
