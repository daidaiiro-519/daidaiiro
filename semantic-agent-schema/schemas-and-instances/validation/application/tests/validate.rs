//! UC4 の手続きを、ポートの代役で確かめる。代役は support.rs に置く。

#![allow(clippy::unwrap_used, clippy::panic)]

mod support;

use serde_json::json;
use support::{HUMAN, Lenient, Log, UC, concrete, decided, files};
use validation_application::use_cases::validate::{Scope, validate, validate_stale};
use validation_domain::values::Verdict;
use writing_domain::values::TargetKind;

/// 代役を、検証のモジュールのポートに当てる（ポートはモジュールごとに宣言する）。
impl validation_application::ports::DocumentStore for support::Files {
    fn list(&self) -> Result<Vec<String>, String> {
        Self::list(self)
    }
    fn read(&self, path: &str) -> Result<String, String> {
        Self::read(self, path)
    }
}

impl validation_application::ports::SchemaValidator for support::Lenient {
    fn shipped(&self, id: &str) -> Option<serde_json::Value> {
        Self::shipped(self, id)
    }
    fn check(
        &self,
        schema: &str,
        value: &serde_json::Value,
        more: &[(String, serde_json::Value)],
    ) -> Result<Vec<(String, String)>, String> {
        Self::check(self, schema, value, more)
    }
}

impl validation_application::ports::ValidationLog for support::Log {
    fn append_validate(
        &mut self,
        session: Option<&str>,
        row: &validation_domain::values::ValidateRow,
    ) -> Result<String, String> {
        Self::append_validate(self, session, row)
    }
    fn writes(
        &self,
        target_path: &str,
    ) -> Result<Vec<writing_domain::values::WriteRecord>, String> {
        Self::writes(self, target_path)
    }
    fn last_judgments(
        &self,
        target_path: &str,
    ) -> Result<Vec<validation_domain::values::Judgment>, String> {
        Self::last_judgments(self, target_path)
    }
}

#[test]
fn concrete_gets_a_result_for_every_applied_rule() {
    let store = files(&[("c.json", concrete())]);
    let mut log = Log::default();
    let v = validate(
        &Scope::Paths(vec!["c.json".into()]),
        &store,
        &Lenient,
        &HUMAN,
        &mut log,
    )
    .unwrap();
    let t = &v.targets[0];
    assert_eq!(t.kind, TargetKind::Concrete);
    let rules: Vec<(&str, Verdict)> = t.results.iter().map(|r| (r.rule(), r.verdict())).collect();
    assert_eq!(
        rules,
        [
            ("json-schema", Verdict::Verified),
            ("instance-schema-name", Verdict::Verified),
            ("unique-id", Verdict::Verified),
            ("goal-is-ok", Verdict::NotJudged)
        ]
    );
    assert_eq!(log.0.len(), 1);
    assert_eq!(log.0[0].hash, t.hash.as_str());
}

#[test]
fn unlisted_version_is_refused_and_nothing_is_recorded() {
    let store = files(&[(
        "c.json",
        json!({"$schema": "urn:semantic-agent-schema:published-language:abstract:v2"}),
    )]);
    let mut log = Log::default();
    let Err(r) = validate(
        &Scope::Paths(vec!["c.json".into()]),
        &store,
        &Lenient,
        &HUMAN,
        &mut log,
    ) else {
        panic!("断るはず");
    };
    assert_eq!(r.code, "schema_not_listed");
    assert!(log.0.is_empty());
}

#[test]
fn all_skips_documents_of_other_schemas() {
    let store = files(&[
        ("c.json", concrete()),
        (
            "other.json",
            json!({"$schema": "https://json-schema.org/draft/2020-12/schema"}),
        ),
        ("u1.json", json!({"$schema": UC, "id": "U1"})),
    ]);
    let mut log = Log::default();
    let v = validate(&Scope::All, &store, &Lenient, &HUMAN, &mut log).unwrap();
    let paths: Vec<&str> = v.targets.iter().map(|t| t.path.as_str()).collect();
    assert_eq!(paths, ["c.json", "u1.json"]);
}

#[test]
fn stale_validation_rejudges_only_rules_whose_reads_changed() {
    let store = files(&[
        ("c.json", decided()),
        (
            "u1.json",
            json!({"$schema": UC, "id": "U1", "goal": "a", "status": "draft"}),
        ),
    ]);
    let mut log = Log::default();
    validate(&Scope::All, &store, &Lenient, &HUMAN, &mut log).unwrap();
    store
        .write(
            "u1.json",
            &json!({"$schema": UC, "id": "U1", "goal": "a", "status": "approved"}).to_string(),
        )
        .unwrap();
    let v = validate_stale(&Scope::All, &store, &Lenient, &HUMAN, &mut log).unwrap();
    assert_eq!(v.unchanged_targets, ["c.json"]);
    let t = &v.targets[0];
    let rules: Vec<&str> = t.results.iter().map(|r| r.rule()).collect();
    assert_eq!(rules, ["json-schema"], "goal を読む rule は読んだ所が同じ");
    assert_eq!(t.unchanged_rules, ["goal-is-outcome"]);
    let again = validate_stale(&Scope::All, &store, &Lenient, &HUMAN, &mut log).unwrap();
    assert!(again.targets.is_empty());
}
