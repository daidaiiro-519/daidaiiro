//! スキーマの中の所に当たる欄の定義を引く（意味で書く手順と、意味の判定の手順が、欄の title ・ description ・ x-prompt を入れるのに使う）。

use crate::services::SchemaIndex;
use serde_json::Value;

/// 対象の中の所に当たる欄の定義（title ・ description ・ x-prompt を引くため）。properties ・ additionalProperties ・ items を
/// たどり、$ref は文書の中（#…）とほかの文書（<$id>#…、others）を引く。$ref と横に並べた欄は、指す先の欄より先に読む。
#[must_use]
pub fn field_schema(schema: &Value, pointer: &str, others: &[(String, Value)]) -> Option<Value> {
    let mut root = schema;
    let mut cur = schema;
    for part in pointer.split('/').skip(1) {
        let part = part.replace("~1", "/").replace("~0", "~");
        let (r, c) = SchemaIndex::deref(root, cur, others);
        root = r;
        cur = if part.parse::<usize>().is_ok() {
            c.get("items")?
        } else {
            c.get("properties")
                .and_then(|p| p.get(&part))
                .or_else(|| c.get("additionalProperties").filter(|a| a.is_object()))?
        };
    }
    let (_, target) = SchemaIndex::deref(root, cur, others);
    let mut field = target.clone();
    if let (Some(f), Some(own)) = (field.as_object_mut(), cur.as_object()) {
        for (k, v) in own {
            if k != "$ref" {
                f.insert(k.clone(), v.clone());
            }
        }
    }
    Some(field)
}

#[cfg(test)]
mod tests {
    use super::field_schema;
    use serde_json::{Value, json};

    #[test]
    fn field_schema_follows_ref_and_additional_properties() {
        let other = json!({"$defs": {"person": {"properties": {"name": {"title": "名前", "description": "人の名前である。"}}}}});
        let concrete = json!({
            "$defs": {"step": {"title": "手順", "properties": {"action": {"title": "動作"}}}},
            "properties": {
                "steps": {"items": {"$ref": "#/$defs/step"}},
                "owner": {"$ref": "urn:example:t:person:v1#/$defs/person", "title": "持ち主"},
                "tags": {"additionalProperties": {"title": "札"}}
            }
        });
        let others = vec![("urn:example:t:person:v1".to_string(), other)];
        let title = |p: &str| {
            field_schema(&concrete, p, &others)
                .and_then(|f| f.get("title").cloned())
                .unwrap_or(Value::Null)
        };
        assert_eq!(title("/steps/0/action"), json!("動作"));
        assert_eq!(title("/steps/0"), json!("手順"));
        assert_eq!(
            title("/owner"),
            json!("持ち主"),
            "$ref と横に並べた欄を先に読む"
        );
        assert_eq!(title("/owner/name"), json!("名前"));
        assert_eq!(title("/tags/a"), json!("札"));
    }
}
