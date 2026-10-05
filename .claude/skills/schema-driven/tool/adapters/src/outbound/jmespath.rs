//! 取得のアダプタ（ポート Query の実装）。jmespath の crate はここにだけ現れる（ACDR 0122）。

use schema_driven_core::ports::outbound::{Query, QueryError};
use serde_json::Value;

pub struct Jmespath;

impl Query for Jmespath {
    fn search(&self, expression: &str, json: &Value) -> Result<Value, QueryError> {
        let expr = jmespath::compile(expression).map_err(|e| QueryError(e.to_string()))?;
        let found = expr.search(json).map_err(|e| QueryError(e.to_string()))?;
        serde_json::to_value(&*found).map_err(|e| QueryError(e.to_string()))
    }
}
