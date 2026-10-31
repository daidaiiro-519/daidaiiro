//! 結合テスト：sh を argv で起動し、標準入力の呼び出しを受け取って JSON を返させる

use semantic_agent_schema_application::ports::delegates::{DelegateError, Delegates, Provenance};
use semantic_agent_schema_application::semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema;
use semantic_agent_schema_application::semantic_agent_schema_declaration::aggregates::entities::delegate::Delegate;
use semantic_agent_schema_application::semantic_agent_schema_validation::values::call::Call;
use semantic_agent_schema_tools::run::ArgvDelegates;
use serde_json::{Value, json};
use std::time::Duration;

#[cfg(test)]
fn delegate(run: Value) -> Delegate {
    ConcreteSchema::from_document(json!({
        "$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1",
        "$id": "urn:semantic-agent-schema:test:c:v1", "kind": "c", "type": "object",
        "x-delegates": {"lint": {"role": "検査器", "run": run}}
    }))
    .unwrap()
    .delegates()
    .remove(0)
}

fn call() -> Call {
    Call::new("lint", "r/1.json", vec![], vec![])
}

fn tools(ms: u64) -> ArgvDelegates {
    ArgvDelegates::new(std::env::temp_dir(), Duration::from_millis(ms))
}

/// traces: U14:事後
#[test]
fn tool_reads_the_call_and_returns_json() {
    let d = delegate(
        json!({"kind": "judging", "argv": ["sh", "-c", "grep -q '\"delegate\":\"lint\"' && echo '{\"results\":[]}'"]}),
    );
    let a = tools(5000).call(&d, &call(), &json!({})).unwrap();
    assert_eq!(a.output, json!({"results": []}));
    assert!(matches!(a.provenance, Provenance::Process { ref argv } if argv[0] == "sh"));
}

/// traces: U14:失敗時
#[test]
fn tool_that_does_not_end_is_a_failure() {
    let d = delegate(json!({"kind": "judging", "argv": ["sleep", "5"]}));
    assert!(
        matches!(tools(200).call(&d, &call(), &json!({})), Err(DelegateError::Failed(m)) if m.contains("時間切れ"))
    );
}

/// traces: U14:失敗時
#[test]
fn tool_that_does_not_return_json_is_a_failure() {
    let d = delegate(json!({"kind": "judging", "argv": ["sh", "-c", "cat >/dev/null; echo nope"]}));
    assert!(matches!(
        tools(5000).call(&d, &call(), &json!({})),
        Err(DelegateError::Failed(_))
    ));
}

/// traces: U14:失敗時
#[test]
fn delegate_without_argv_is_unsupported() {
    let d = delegate(json!({"kind": "judging"}));
    assert!(matches!(
        tools(5000).call(&d, &call(), &json!({})),
        Err(DelegateError::Unsupported(_))
    ));
}

/// traces: U14:失敗時
#[test]
fn secrets_are_not_passed_to_tools() {
    // SAFETY 相当の注意：この試験だけが環境変数を足す。ほかの試験はこの名前を読まない
    unsafe { std::env::set_var("ANTHROPIC_API_KEY", "secret") };
    let d = delegate(
        json!({"kind": "judging", "argv": ["sh", "-c", "cat >/dev/null; if [ -n \"$ANTHROPIC_API_KEY\" ]; then echo '{\"leak\":true}'; else echo '{\"leak\":false}'; fi"]}),
    );
    let a = tools(5000).call(&d, &call(), &json!({})).unwrap();
    assert_eq!(a.output, json!({"leak": false}));
}

/// traces: U14:失敗時
#[test]
fn stderr_tail_is_attached_to_a_failure() {
    let d = delegate(
        json!({"kind": "judging", "argv": ["sh", "-c", "cat >/dev/null; echo broken >&2; exit 3"]}),
    );
    assert!(
        matches!(tools(5000).call(&d, &call(), &json!({})), Err(DelegateError::Failed(m)) if m.contains("broken"))
    );
}
