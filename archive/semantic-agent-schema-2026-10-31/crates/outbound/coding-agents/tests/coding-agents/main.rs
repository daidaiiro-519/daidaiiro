//! 結合テスト：コーディングエージェントの代役（sh）を argv で起動し、プロンプトを受け取らせて答えを返させる。本物の Claude Code は呼ばない

use semantic_agent_schema_application::ports::delegates::{DelegateError, Delegates, Provenance};
use semantic_agent_schema_application::semantic_agent_schema_declaration::aggregates::concrete_schema::ConcreteSchema;
use semantic_agent_schema_application::semantic_agent_schema_declaration::aggregates::entities::delegate::Delegate;
use semantic_agent_schema_application::semantic_agent_schema_validation::values::call::{Call, CallRule, Input};
use semantic_agent_schema_coding_agents::{Agent, CodingAgents, claude_code};
use serde_json::{Value, json};
use std::time::Duration;

#[cfg(test)]
fn delegate(run: Value) -> Delegate {
    ConcreteSchema::from_document(json!({
        "$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v1",
        "$id": "urn:semantic-agent-schema:test:c:v1", "kind": "c", "type": "object",
        "x-delegates": {"reviewer": {"role": "決定の審査役", "run": run}}
    }))
    .unwrap()
    .delegates()
    .remove(0)
}

fn call() -> Call {
    Call::new(
        "reviewer",
        "r/1.json",
        vec![CallRule::new(
            "why-explains",
            "/why",
            &json!({"rule": "why が決定を説明する", "why": "w", "test": "t"}),
        )],
        vec![Input::new(
            "/why",
            json!("速いから"),
            json!("決めた理由"),
            Value::Null,
        )],
    )
}

fn agents(argv: &[&str]) -> CodingAgents {
    CodingAgents::new(
        Agent::Command(argv.iter().map(|s| s.to_string()).collect()),
        std::env::temp_dir(),
        Duration::from_secs(5),
    )
}

/// traces: U14:事後
#[test]
fn prompt_carries_role_rules_and_read_values_only() {
    // 代役はプロンプトに立場 ・ rule ・ 読む値が入っていることを確かめてから答える
    let a = agents(&["sh", "-c", "p=$(cat); case \"$p\" in *決定の審査役*why-explains*速いから*) echo '{\"results\":[]}';; *) exit 9;; esac"])
        .call(&delegate(json!({"kind": "judging"})), &call(), &json!({"type": "object"}))
        .unwrap();
    assert_eq!(a.output, json!({"results": []}));
    assert!(
        matches!(a.provenance, Provenance::Agent { ref prompt, .. } if prompt.contains("/why"))
    );
}

/// traces: U14:失敗時
#[test]
fn working_and_argv_delegates_are_unsupported_here() {
    let a = agents(&["true"]);
    assert!(matches!(
        a.call(&delegate(json!({"kind": "working"})), &call(), &json!({})),
        Err(DelegateError::Unsupported(_))
    ));
    assert!(matches!(
        a.call(
            &delegate(json!({"kind": "judging", "argv": ["x"]})),
            &call(),
            &json!({})
        ),
        Err(DelegateError::Unsupported(_))
    ));
}

/// traces: I39
#[test]
fn agents_without_verified_tool_removal_are_unsupported() {
    let a = CodingAgents::new(
        Agent::by_name("codex", None),
        std::env::temp_dir(),
        Duration::from_secs(5),
    );
    assert!(matches!(
        a.call(&delegate(json!({"kind": "judging"})), &call(), &json!({})),
        Err(DelegateError::Unsupported(_))
    ));
}

/// traces: I14
#[test]
fn claude_code_is_started_without_tools_and_session() {
    let a = claude_code::argv(&json!({"type": "object"}), Some("sonnet"));
    for flag in ["--bare", "-p", "--no-session-persistence"] {
        assert!(a.iter().any(|x| x == flag), "{flag}");
    }
    let at = |f: &str| a.iter().position(|x| x == f).unwrap();
    assert_eq!(a[at("--tools") + 1], "");
    assert_eq!(a[at("--disallowedTools") + 1], "*");
    assert_eq!(a[at("--model") + 1], "sonnet");
}

/// traces: I14
#[test]
fn used_model_is_read_from_model_usage() {
    assert_eq!(
        claude_code::used_model(&json!({"modelUsage": {"claude-sonnet-5-5": {}}})),
        Some("claude-sonnet-5-5".into())
    );
    assert_eq!(claude_code::used_model(&json!({})), None);
}
