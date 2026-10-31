//! UC4 ・ UC1 ・ UC2 の手続きを、ポートの代役で確かめる。代役はこの試験の中に置く
//! （test-support に置くと、test-support が application に依存し、application の試験が test-support に依存して循環する）。

#![allow(clippy::unwrap_used, clippy::panic)]

use schemas_and_instances_conformance_application::ports::{DocumentStore, SchemaChecker};
use schemas_and_instances_conformance_application::use_cases::documents::{
    CreateRequest, Scope, TargetKind, UpdateRequest, create, update, validate,
};
use schemas_and_instances_conformance_domain::values::Fingerprint;
use schemas_and_instances_conformance_domain::values::Verdict;
use schemas_and_instances_operation_recording_application::ports::{
    Caller, ChangeRow, IdentifyCaller, Judgment, OperationLog, RuleRow, StoredChange, ValidateRow,
    WriteRecord,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;

const ABSTRACT: &str = "urn:semantic-agent-schema:base:semantic-abstract-schema:v1";
const UC: &str = "urn:semantic-agent-schema:t:uc:v1";

struct Files(RefCell<BTreeMap<String, String>>);
impl DocumentStore for Files {
    fn list(&self) -> Result<Vec<String>, String> {
        Ok(self.0.borrow().keys().cloned().collect())
    }
    fn read(&self, path: &str) -> Result<String, String> {
        self.0
            .borrow()
            .get(path)
            .cloned()
            .ok_or_else(|| format!("{path} が無い"))
    }
    fn write(&self, path: &str, text: &str) -> Result<(), String> {
        self.0.borrow_mut().insert(path.into(), text.into());
        Ok(())
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

struct Tool;
impl IdentifyCaller for Tool {
    fn identify(&self) -> Caller {
        Caller::Tool {
            tool_ref: "agent".into(),
        }
    }
}

/// 検証と変更を、足した順に1つの並びで持つ代役。
#[derive(Default)]
struct Log(Vec<ValidateRow>, Vec<(String, ChangeRow)>);
impl Log {
    fn next(&self) -> String {
        format!("o-{}", self.0.len() + self.1.len() + 1)
    }
}
impl OperationLog for Log {
    fn caller_session(&mut self, _: &Caller) -> Result<String, String> {
        Ok("s-1".into())
    }
    fn append_validate(&mut self, _: &str, row: &ValidateRow) -> Result<String, String> {
        let id = self.next();
        self.0.push(row.clone());
        Ok(id)
    }
    fn append_change(&mut self, _: &str, row: &ChangeRow) -> Result<String, String> {
        let id = self.next();
        self.1.push((id.clone(), row.clone()));
        Ok(id)
    }
    fn find_request(&self, request_id: &str) -> Result<Option<StoredChange>, String> {
        Ok(self
            .1
            .iter()
            .find(|(_, r)| r.request_id.as_deref() == Some(request_id))
            .map(|(id, r)| StoredChange {
                operation: id.clone(),
                session: "s-1".into(),
                row: r.clone(),
            }))
    }
    fn writes(&self, target_path: &str) -> Result<Vec<WriteRecord>, String> {
        Ok(self
            .1
            .iter()
            .filter(|(_, r)| r.target_path == target_path && r.refusal.is_none())
            .map(|(id, r)| WriteRecord {
                operation: id.clone(),
                session: "s-1".into(),
                at: "t".into(),
                row: r.clone(),
            })
            .collect())
    }
    fn last_judgments(&self, target_path: &str) -> Result<Vec<Judgment>, String> {
        Ok(self
            .0
            .iter()
            .enumerate()
            .rfind(|(_, r)| r.target_path == target_path)
            .map(|(i, r)| {
                r.results
                    .iter()
                    .map(|rule: &RuleRow| Judgment {
                        operation: format!("o-{}", i + 1),
                        rule: rule.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default())
    }
}

fn files(entries: &[(&str, Value)]) -> Files {
    Files(RefCell::new(
        entries
            .iter()
            .map(|(p, v)| ((*p).to_string(), v.to_string()))
            .collect(),
    ))
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

fn decided() -> Value {
    json!({"$schema": ABSTRACT, "$id": UC, "properties": {
        "status": {"x-decided-by": "human", "default": "draft"},
        "goal": {"x-prompt": {"write": [{"id": "goal-is-outcome", "by": "human"}]}}
    }})
}

fn create_request(value: Value, request_id: Option<&str>) -> CreateRequest {
    CreateRequest {
        path: "u1.json".into(),
        concrete: UC.into(),
        value,
        request_id: request_id.map(Into::into),
        intent: None,
    }
}

#[test]
fn create_writes_records_and_returns_the_diff() {
    let store = files(&[("c.json", decided())]);
    let mut log = Log::default();
    let c = create(
        &create_request(json!({"id": "U1", "status": "draft"}), None),
        &store,
        &Lenient,
        &Tool,
        &mut log,
    )
    .unwrap();
    let text = store.read("u1.json").unwrap();
    assert_eq!(c.after, Fingerprint::of(text.as_bytes()));
    let pointers: Vec<&str> = c.diff.changes().iter().map(|f| f.pointer()).collect();
    assert_eq!(pointers, ["/$schema", "/id", "/status"]);
    assert_eq!(log.1[0].1.after.as_deref(), Some(c.after.as_str()));
    assert_eq!(log.1[0].1.fields.len(), 3);
}

#[test]
fn create_refused_by_a_write_rule_is_recorded_without_writing() {
    let store = files(&[("c.json", decided())]);
    let mut log = Log::default();
    let r = create(
        &create_request(json!({"id": "U1", "status": "approved"}), None),
        &store,
        &Lenient,
        &Tool,
        &mut log,
    )
    .unwrap_err();
    assert_eq!(
        (r.code, r.operation.as_deref()),
        ("human_field", Some("o-1"))
    );
    assert!(store.read("u1.json").is_err());
    let row = &log.1[0].1;
    assert_eq!((row.before.as_deref(), row.after.as_deref()), (None, None));
    assert!(row.fields.is_empty());
}

#[test]
fn same_request_is_one_operation() {
    let store = files(&[("c.json", decided())]);
    let mut log = Log::default();
    let req = create_request(json!({"id": "U1"}), Some("r-1"));
    let first = create(&req, &store, &Lenient, &Human, &mut log).unwrap();
    let again = create(&req, &store, &Lenient, &Human, &mut log).unwrap();
    assert!(again.replayed);
    assert_eq!((again.operation, again.diff), (first.operation, first.diff));
    assert_eq!(log.1.len(), 1);
    let other = UpdateRequest {
        path: "u1.json".into(),
        patch: json!([]),
        if_match: Some(first.after.as_str().into()),
        request_id: Some("r-1".into()),
        intent: None,
    };
    let r = update(&other, &store, &Lenient, &Human, &mut log).unwrap_err();
    assert_eq!(r.code, "request_reused");
}

#[test]
fn update_checks_the_fingerprint_before_writing() {
    let store = files(&[
        ("c.json", decided()),
        ("u1.json", json!({"$schema": UC, "id": "U1", "goal": "a"})),
    ]);
    let before = store.read("u1.json").unwrap();
    let mut log = Log::default();
    let req = |if_match: &str| UpdateRequest {
        path: "u1.json".into(),
        patch: json!([{"op": "replace", "path": "/goal", "value": "b"}]),
        if_match: Some(if_match.into()),
        request_id: None,
        intent: None,
    };
    let r = update(&req("sha256:x"), &store, &Lenient, &Human, &mut log).unwrap_err();
    assert_eq!(r.code, "conflict");
    assert_eq!(store.read("u1.json").unwrap(), before);
    assert_eq!(log.1[0].1.before, log.1[0].1.after);
    let fp = Fingerprint::of(before.as_bytes());
    let c = update(&req(fp.as_str()), &store, &Lenient, &Human, &mut log).unwrap();
    assert_eq!(c.before, Some(fp));
    assert_eq!(c.diff.changes().len(), 1);
}

#[test]
fn update_without_if_match_compares_with_the_last_write() {
    let store = files(&[
        ("c.json", decided()),
        ("u1.json", json!({"$schema": UC, "id": "U1", "goal": "a"})),
    ]);
    let mut log = Log::default();
    let req = |goal: &str| UpdateRequest {
        path: "u1.json".into(),
        patch: json!([{"op": "replace", "path": "/goal", "value": goal}]),
        if_match: None,
        request_id: None,
        intent: Some("目標を直す".into()),
    };
    // SAS が一度も書いていないので、比べる指紋が無く、書く
    let first = update(&req("b"), &store, &Lenient, &Human, &mut log).unwrap();
    // 最後に書いた指紋といまの中身が同じなので、書く
    update(&req("c"), &store, &Lenient, &Human, &mut log).unwrap();
    // SAS を通らずに直される
    store
        .write(
            "u1.json",
            &json!({"$schema": UC, "id": "U1", "goal": "x"}).to_string(),
        )
        .unwrap();
    let edited = store.read("u1.json").unwrap();
    let r = update(&req("d"), &store, &Lenient, &Human, &mut log).unwrap_err();
    assert_eq!(r.code, "conflict");
    assert_eq!(store.read("u1.json").unwrap(), edited);
    let i = r.interleaved.unwrap();
    assert!(i.untracked);
    assert!(i.operations.is_empty());
    assert_eq!(i.last_recorded.map(|w| w.operation).as_deref(), Some("o-2"));
    // 古い指紋を --if-match で渡すと、そのあとの操作が意図と一緒に返る
    let r = update(
        &UpdateRequest {
            if_match: Some(first.before.unwrap().as_str().into()),
            ..req("d")
        },
        &store,
        &Lenient,
        &Human,
        &mut log,
    )
    .unwrap_err();
    let i = r.interleaved.unwrap();
    let ids: Vec<&str> = i.operations.iter().map(|w| w.operation.as_str()).collect();
    assert_eq!(ids, ["o-1", "o-2"]);
    assert_eq!(i.operations[0].row.intent.as_deref(), Some("目標を直す"));
    assert!(i.untracked);
}

#[test]
fn update_reports_the_previous_judgments_it_made_stale() {
    let store = files(&[
        ("c.json", decided()),
        (
            "u1.json",
            json!({"$schema": UC, "id": "U1", "goal": "a", "status": "draft"}),
        ),
    ]);
    let mut log = Log::default();
    validate(
        &Scope::Paths(vec!["u1.json".into()]),
        &store,
        &Lenient,
        &Human,
        &mut log,
    )
    .unwrap();
    let fp = Fingerprint::of(store.read("u1.json").unwrap().as_bytes());
    let c = update(
        &UpdateRequest {
            path: "u1.json".into(),
            patch: json!([{"op": "replace", "path": "/status", "value": "approved"}, {"op": "replace", "path": "/goal", "value": "b"}]),
            if_match: Some(fp.as_str().into()),
            request_id: None,
            intent: None,
        },
        &store,
        &Lenient,
        &Human,
        &mut log,
    )
    .unwrap();
    let stale: Vec<(&str, &str, &str)> = c
        .stale
        .iter()
        .map(|s| (s.rule.as_str(), s.pointer.as_str(), s.operation.as_str()))
        .collect();
    assert_eq!(stale, [("goal-is-outcome", "/goal", "o-1")]);
}
