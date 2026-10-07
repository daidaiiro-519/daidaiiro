//! 描画の文脈（page.schema.json の context、ACDR 0133）。ページテンプレートの JMESPath 式は、この値で評価する。

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
