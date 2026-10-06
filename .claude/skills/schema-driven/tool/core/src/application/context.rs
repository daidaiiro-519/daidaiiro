//! 描画の文脈（page.schema.json の context、ACDR 0133）。ページテンプレートの JMESPath 式は、この値で評価する。

use crate::domain::check::{graph, Doc};
use serde_json::{json, Map, Value};

fn id_of(v: &Value) -> Value {
    v.get("id").cloned().unwrap_or(Value::Null)
}

/// インスタンスごとの描画の文脈を、docs と同じ順に返す。
pub fn contexts(docs: &[Doc]) -> Vec<Value> {
    let g = graph(docs);
    let instances: Vec<Value> = docs.iter().map(|d| d.value.clone()).collect();
    (0..docs.len())
        .map(|i| {
            let mut derived = Map::new();
            for d in g.derived.iter().filter(|d| d.doc == i) {
                derived.insert(
                    d.at.trim_start_matches('/').to_owned(),
                    json!({"value": d.value, "title": d.title, "declared": d.declared, "questions": d.questions}),
                );
            }
            let links: Vec<Value> = g
                .links
                .iter()
                .filter(|l| l.doc == i)
                .map(|l| json!({"at": l.at, "value": l.value, "target": l.label}))
                .collect();
            let referrers: Vec<Value> = g
                .links
                .iter()
                .filter_map(|l| match &l.target {
                    Some((t, item)) if *t == i => Some(json!({
                        "at": l.at,
                        "from": id_of(&docs[l.doc].value),
                        "item": item,
                    })),
                    _ => None,
                })
                .collect();
            json!({
                "this": docs[i].value,
                "schema": docs[i].schema.root(),
                "derived": derived,
                "links": links,
                "referrers": referrers,
                "instances": instances,
            })
        })
        .collect()
}
