//! 取得のアダプタ（ポート Query の実装）。jmespath の crate はここにだけ現れる。

use jmespath::{ErrorReason, JmespathError, Rcvar, Runtime, Variable};
use schema_driven_core::ports::Functions;
use schema_driven_core::ports::{Query, QueryError};
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone, Copy)]
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

#[cfg(test)]
mod tests {
    //! 本物の JMESPath に、core の基盤の関数（ViewEngine が渡す Functions）を登録して評価する。
    //! x-view の文の型の組み立て（cases ・ parts ・ maps など）は、core の view.rs の単体テストが代役で確かめる。

    use super::Jmespath;
    use schema_driven_core::application::view::{Instance, ViewEngine};
    use schema_driven_core::domain::schema::Schema;
    use schema_driven_core::ports::Query;
    use serde_json::{json, Value};

    fn engine(schema: Value, instances: Vec<Value>) -> ViewEngine<Jmespath> {
        let list = instances
            .into_iter()
            .map(|value| Instance {
                value,
                schema: "t.schema.json".into(),
            })
            .collect();
        ViewEngine::new(
            Jmespath,
            vec![(
                "t.schema.json".into(),
                Schema::new("t.schema.json", schema, vec![]),
            )],
            list,
        )
    }

    fn render(schema: Value, instances: Vec<Value>, value: Value) -> Result<String, String> {
        engine(schema, instances)
            .render("t.schema.json", "/$defs/x", &value)
            .map_err(|error| error.0)
    }

    #[test]
    fn search_returns_the_value_or_null() {
        let json = json!({"items": ["x"]});
        assert_eq!(Jmespath.search("items[0]", &json).unwrap(), json!("x"));
        assert_eq!(Jmespath.search("note", &json).unwrap(), Value::Null);
        assert!(Jmespath.search("items[0", &json).is_err());
    }

    #[test]
    fn parse_only_reads_the_expression() {
        assert_eq!(Jmespath.parse("this.tags[?@ == 'a']"), Ok(()));
        assert!(Jmespath.parse("a[").is_err());
    }

    #[test]
    fn base_functions_join_with_builtin_functions_in_a_pipe() {
        assert_eq!(
            render(
                json!({"$defs": {"x": {"x-view": {"text": "{names | quote(@) | join('', @)}が無い"}}}}),
                vec![],
                json!({"names": ["甲", "乙"]})
            ),
            Ok("「甲」「乙」が無い".to_owned())
        );
    }

    #[test]
    fn name_finds_the_label_of_an_instance_or_an_item() {
        assert_eq!(
            render(
                json!({
                    "x-view": {"label": "header.name"},
                    "properties": {"terms": {"items": {"x-view": {"label": "word"}}}},
                    "$defs": {"x": {"x-view": {"text": "{refs | name(@) | join('、', @)}"}}}
                }),
                vec![
                    json!({"id": "UC-1", "kind": "uc", "header": {"name": "注文する"}}),
                    json!({"id": "GLO-1", "kind": "glossary", "terms": [{"id": "TERM-1", "word": "注文"}]}),
                ],
                json!({"refs": ["UC-1", "GLO-1.TERM-1", "TERM-1", "UC-9"]})
            ),
            Ok("注文する、注文、注文、UC-9".to_owned())
        );
    }

    #[test]
    fn a_function_outside_the_base_is_an_error() {
        assert!(render(
            json!({"$defs": {"x": {"x-view": {"text": "{clause(s)}"}}}}),
            vec![],
            json!({"s": "値は空"})
        )
        .is_err());
    }

    #[test]
    fn an_unreadable_expression_is_an_error() {
        assert!(render(
            json!({"$defs": {"x": {"x-view": {"text": "{a[}"}}}}),
            vec![],
            json!({"a": 1})
        )
        .is_err());
    }
}
