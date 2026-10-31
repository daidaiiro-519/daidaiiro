//! UC4 の手続きを、ポートの代役で確かめる。代役はこの試験の中に置く
//! （test-support に置くと、test-support が application に依存し、application の試験が test-support に依存して循環する）。

#![allow(clippy::unwrap_used, clippy::panic)]

use contracts_and_instances_conformance_application::ports::{DocumentStore, SchemaChecker};
use contracts_and_instances_conformance_application::use_cases::documents::{
    Scope, TargetKind, validate,
};
use contracts_and_instances_conformance_domain::values::Verdict;
use contracts_and_instances_operation_recording_application::ports::{
    Caller, IdentifyCaller, OperationLog, ValidateRow,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const ABSTRACT: &str = "urn:semantic-agent-schema:base:semantic-abstract-schema:v1";
const UC: &str = "urn:semantic-agent-schema:t:uc:v1";

struct Files(BTreeMap<String, String>);
impl DocumentStore for Files {
    fn list(&self) -> Result<Vec<String>, String> {
        Ok(self.0.keys().cloned().collect())
    }
    fn read(&self, path: &str) -> Result<String, String> {
        self.0
            .get(path)
            .cloned()
            .ok_or_else(|| format!("{path} が無い"))
    }
}

/// 形の検査はいつも通す代役。Abstract は根に by: human と by: machine の rule を1つずつ持つ。
struct Lenient;
impl SchemaChecker for Lenient {
    fn shipped(&self, id: &str) -> Option<Value> {
        (id == ABSTRACT).then(|| {
            json!({"x-prompt": {"write": [{"id": "closed-list", "by": "machine"}, {"id": "someone-decides", "by": "human"}]},
                   "x-extensions": {"x-prompt": {}, "x-relationship": {}}})
        })
    }
    fn check(
        &self,
        _: &str,
        _: &Value,
        _: &[(String, Value)],
    ) -> Result<Vec<(String, String)>, String> {
        Ok(Vec::new())
    }
}

struct Human;
impl IdentifyCaller for Human {
    fn identify(&self) -> Caller {
        Caller::Human
    }
}

#[derive(Default)]
struct Log(Vec<ValidateRow>);
impl OperationLog for Log {
    fn caller_session(&mut self, _: &Caller) -> Result<String, String> {
        Ok("s-1".into())
    }
    fn append_validate(&mut self, _: &str, row: &ValidateRow) -> Result<String, String> {
        self.0.push(row.clone());
        Ok(format!("o-{}", self.0.len()))
    }
}

fn files(entries: &[(&str, Value)]) -> Files {
    Files(
        entries
            .iter()
            .map(|(p, v)| ((*p).to_string(), v.to_string()))
            .collect(),
    )
}

fn concrete() -> Value {
    json!({"$schema": ABSTRACT, "$id": UC, "properties": {"calls": {"items": {"x-relationship": {"to": UC}}}}})
}

#[test]
fn concrete_gets_a_result_for_every_applied_rule() {
    let store = files(&[("c.json", concrete())]);
    let mut log = Log::default();
    let v = validate(
        &Scope::Paths(vec!["c.json".into()]),
        &store,
        &Lenient,
        &Human,
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
            ("closed-list", Verdict::Verified),
            ("someone-decides", Verdict::Unverified)
        ]
    );
    assert_eq!(log.0.len(), 1);
    assert_eq!(log.0[0].before, log.0[0].after);
}

#[test]
fn unlisted_version_is_refused_and_nothing_is_recorded() {
    let store = files(&[(
        "c.json",
        json!({"$schema": "urn:semantic-agent-schema:base:semantic-abstract-schema:v2"}),
    )]);
    let mut log = Log::default();
    let Err(r) = validate(
        &Scope::Paths(vec!["c.json".into()]),
        &store,
        &Lenient,
        &Human,
        &mut log,
    ) else {
        panic!("断るはず");
    };
    assert_eq!(r.code, "schema_not_listed");
    assert!(log.0.is_empty());
}

#[test]
fn instance_relationship_is_checked_against_existing_instances() {
    let store = files(&[
        ("c.json", concrete()),
        ("u1.json", json!({"$schema": UC, "id": "U1", "calls": []})),
        (
            "u2.json",
            json!({"$schema": UC, "id": "U2", "calls": ["U1", "U3"]}),
        ),
    ]);
    let mut log = Log::default();
    let v = validate(
        &Scope::Paths(vec!["u2.json".into()]),
        &store,
        &Lenient,
        &Human,
        &mut log,
    )
    .unwrap();
    let rel: Vec<Verdict> = v.targets[0]
        .results
        .iter()
        .filter(|r| r.rule() == "relationship-target")
        .map(|r| r.verdict())
        .collect();
    assert_eq!(rel, [Verdict::Verified, Verdict::Violated]);
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
    let v = validate(&Scope::All, &store, &Lenient, &Human, &mut log).unwrap();
    let paths: Vec<&str> = v.targets.iter().map(|t| t.path.as_str()).collect();
    assert_eq!(paths, ["c.json", "u1.json"]);
}
