//! 取得のアダプタ（ポート Query の実装）。jmespath の crate はここにだけ現れる（ACDR 0122）。

use jmespath::{ErrorReason, JmespathError, Rcvar, Runtime, Variable};
use schema_driven_core::ports::outbound::{Functions, Query, QueryError};
use serde_json::Value;
use std::sync::Arc;

pub struct Jmespath;

fn to_json(variable: &Rcvar) -> Value {
    serde_json::to_value(&**variable).unwrap_or(Value::Null)
}

impl Query for Jmespath {
    fn search(&self, expression: &str, json: &Value) -> Result<Value, QueryError> {
        let expr = jmespath::compile(expression).map_err(|error| QueryError(error.to_string()))?;
        let found = expr
            .search(json)
            .map_err(|error| QueryError(error.to_string()))?;
        Ok(to_json(&found))
    }

    fn parse(&self, expression: &str) -> Result<(), QueryError> {
        jmespath::compile(expression)
            .map(|_| ())
            .map_err(|error| QueryError(error.to_string()))
    }

    fn evaluate(
        &self,
        expression: &str,
        json: &Value,
        functions: Arc<dyn Functions>,
    ) -> Result<Value, QueryError> {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        for name in functions.names() {
            let host = functions.clone();
            let key = name.clone();
            runtime.register_function(
                &key,
                Box::new(move |args: &[Rcvar], ctx: &mut jmespath::Context<'_>| {
                    let values: Vec<Value> = args.iter().map(to_json).collect();
                    let out = host.call(&name, &values).map_err(|error| {
                        JmespathError::from_ctx(ctx, ErrorReason::Parse(format!("{name}: {error}")))
                    })?;
                    Variable::from_serializable(out).map(Rcvar::new)
                }),
            );
        }
        let expr = runtime
            .compile(expression)
            .map_err(|error| QueryError(error.to_string()))?;
        let found = expr
            .search(json)
            .map_err(|error| QueryError(error.to_string()))?;
        Ok(to_json(&found))
    }
}
