//! ポート JsonSchemaChecker を、jsonschema（SAS と同じ版）で実装する。Skill の参照する資料が、Skill のフォルダに置いた
//! ふつうの JSON Schema に従うかを確かめる（SAS の Concrete に従う資料は sas validate で確かめるので、ここは通らない）。

use serde_json::Value;
use skills_application::ports::JsonSchemaChecker;

/// JSON Schema の検査器。
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonSchema;

impl JsonSchemaChecker for JsonSchema {
    fn check(&self, schema: &Value, value: &Value) -> Result<Vec<(String, String)>, String> {
        let v = jsonschema::validator_for(schema)
            .map_err(|e| format!("schema を JSON Schema として組めない：{e}"))?;
        Ok(v.iter_errors(value)
            .map(|e| (e.instance_path().to_string(), e.to_string()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::JsonSchema;
    use serde_json::json;
    use skills_application::ports::JsonSchemaChecker;

    #[test]
    fn a_value_that_breaks_the_schema_is_reported_with_its_place() {
        let schema = json!({"type": "object", "properties": {"items": {"type": "array", "items": {"required": ["id"]}}}});
        let ok = JsonSchema
            .check(&schema, &json!({"items": [{"id": "a"}]}))
            .unwrap_or_default();
        assert!(ok.is_empty());
        let broken = JsonSchema
            .check(&schema, &json!({"items": [{"id": "a"}, {}]}))
            .unwrap_or_default();
        assert_eq!(broken.len(), 1);
        assert_eq!(broken[0].0, "/items/1");
    }
}
