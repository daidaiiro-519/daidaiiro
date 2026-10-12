//! 公開した版のスキーマの指紋。公開の記録（published.json）と、いまのスキーマを比べる。
//! 指紋は、キーを名前の順に並べ、空白を除いた JSON の SHA-256 である（書き方の違いで変わらない）。

use crate::domain::schema_name;
use crate::domain::schema_rules::SchemaFinding;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// 公開の記録のファイル名。スキーマと同じフォルダに置く。
pub const RECORD_FILE: &str = "published.json";

/// 値を、キーを名前の順に並べた JSON の文字にする。
fn canonical(value: &Value) -> String {
    match value {
        Value::Object(object) => {
            let sorted: BTreeMap<&String, &Value> = object.iter().collect();
            let fields: Vec<String> = sorted
                .into_iter()
                .map(|(key, child)| format!("{}:{}", Value::String(key.clone()), canonical(child)))
                .collect();
            format!("{{{}}}", fields.join(","))
        }
        Value::Array(elements) => {
            let items: Vec<String> = elements.iter().map(canonical).collect();
            format!("[{}]", items.join(","))
        }
        other => other.to_string(),
    }
}

/// スキーマの指紋（16進の小文字）。
pub fn fingerprint(schema: &Value) -> String {
    Sha256::digest(canonical(schema).as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// まだ公開の記録に無い名前と、その指紋。
#[derive(Debug, Clone, PartialEq)]
pub struct Unpublished {
    pub name: String,
    pub sha256: String,
}

/// 名前の版を1つ上げた名前の版の部分（acdr/record@v1 なら @v2）。
fn next_version(name: &str) -> String {
    name.rsplit_once("@v")
        .and_then(|(_, number)| number.parse::<u64>().ok())
        .map(|number| format!("@v{}", number + 1))
        .unwrap_or_else(|| "新しい版".into())
}

/// スキーマ（ファイル名と中身）を、公開の記録と比べる。記録が無ければ、すべての名前がまだ公開していないものである。
pub fn check(
    schemas: &[(String, Value)],
    record: Option<&Value>,
) -> (Vec<SchemaFinding>, Vec<Unpublished>) {
    let recorded: BTreeMap<String, String> = record
        .and_then(|record| record.get("schemas"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            Some((
                item.get("name")?.as_str()?.to_owned(),
                item.get("sha256")?.as_str()?.to_owned(),
            ))
        })
        .collect();
    let mut seen = BTreeMap::new();
    let (mut findings, mut unpublished) = (Vec::new(), Vec::new());
    for (file, schema) in schemas {
        let Some(name) = schema
            .get("$id")
            .and_then(Value::as_str)
            .filter(|name| schema_name::is_name(name))
        else {
            continue;
        };
        let sha256 = fingerprint(schema);
        seen.insert(name.to_owned(), ());
        match recorded.get(name) {
            Some(expected) if *expected == sha256 => {}
            Some(_) => findings.push(SchemaFinding {
                schema: file.clone(),
                at: "/$id".into(),
                rule: "公開した版が書き換えられている".into(),
                message: format!(
                    "{name} の中身が、公開したときと違う。中身を変えるなら、元に戻して {} の新しい名前で足す",
                    next_version(name)
                ),
            }),
            None => unpublished.push(Unpublished {
                name: name.to_owned(),
                sha256,
            }),
        }
    }
    for name in recorded.keys().filter(|name| !seen.contains_key(*name)) {
        findings.push(SchemaFinding {
            schema: RECORD_FILE.into(),
            at: "/schemas".into(),
            rule: "公開した版が無い".into(),
            message: format!("{name} は公開した版なのに、このフォルダに無い。公開した版は消さない"),
        });
    }
    (findings, unpublished)
}
