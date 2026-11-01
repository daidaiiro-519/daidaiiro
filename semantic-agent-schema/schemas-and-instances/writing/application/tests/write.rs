//! UC1 ・ UC2 と取り込みの手続きを、ポートの代役で確かめる。代役は検証のモジュールの試験と共に使う（support.rs）。

#![allow(clippy::unwrap_used, clippy::panic)]

#[path = "../../../validation/application/tests/support.rs"]
mod support;

use serde_json::{Value, json};
use support::{Human, Lenient, Log, Tool, UC, decided, files};
use validation_application::ports::DocumentStore;
use validation_application::use_cases::validate::{Scope, validate};
use writing_application::ports::CommitReader;
use writing_application::use_cases::write::{
    AdoptAs, AdoptRequest, CreateRequest, UpdateRequest, adopt, create, update,
};
use writing_domain::values::Fingerprint;

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
fn create_refused_by_a_write_rule_writes_and_records_nothing() {
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
    assert_eq!(r.code, "human_field");
    assert!(store.read("u1.json").is_err());
    assert!(log.1.is_empty(), "断った操作は記録しない");
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
    assert!(log.1.is_empty(), "断った操作は記録しない");
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

/// コミット済みの版の代役：コミットの名前と中身の並び（新しい順）。
struct Commits(Vec<(String, String)>);
impl CommitReader for Commits {
    fn commits(&self, _: &str, max: usize) -> Result<Vec<String>, String> {
        Ok(self.0.iter().take(max).map(|(c, _)| c.clone()).collect())
    }
    fn read(&self, rev: &str, _: &str) -> Result<Option<Vec<u8>>, String> {
        Ok(self
            .0
            .iter()
            .find(|(c, _)| c == rev)
            .map(|(_, t)| t.clone().into_bytes()))
    }
}

fn adopt_request(as_kind: AdoptAs) -> AdoptRequest {
    AdoptRequest {
        path: "u1.json".into(),
        as_kind,
        concrete: None,
        intent: Some("エディターで直した".into()),
        request_id: None,
    }
}

#[test]
fn adopted_update_compares_with_the_commit_of_the_last_recorded_state() {
    let first = json!({"$schema": UC, "id": "U1", "goal": "a", "status": "draft"});
    let store = files(&[("c.json", decided()), ("u1.json", first)]);
    let committed = store.read("u1.json").unwrap();
    let mut log = Log::default();
    let none = Commits(Vec::new());
    let c = adopt(
        &adopt_request(AdoptAs::Create),
        &store,
        &Lenient,
        &none,
        &Human,
        &mut log,
    )
    .unwrap();
    assert!(c.outside);
    assert!(c.before.is_none());
    // 外で goal を直す。HEAD は外でのコミット（中身が違う）、その前のコミットが記録の最後の事後
    let edited = json!({"$schema": UC, "id": "U1", "goal": "b", "status": "draft"}).to_string();
    store.write("u1.json", &edited).unwrap();
    let commits = Commits(vec![
        ("head".into(), edited.clone()),
        ("older".into(), committed),
    ]);
    let c = adopt(
        &adopt_request(AdoptAs::Update),
        &store,
        &Lenient,
        &commits,
        &Human,
        &mut log,
    )
    .unwrap();
    assert_eq!(c.before_commit.as_deref(), Some("older"));
    let changes: Vec<(&str, Option<&Value>)> = c
        .diff
        .changes()
        .iter()
        .map(|f| (f.pointer(), f.before()))
        .collect();
    assert_eq!(changes, [("/goal", Some(&json!("a")))]);
    assert_eq!(log.1[1].1.source, "outside");
    assert_eq!(
        store.read("u1.json").unwrap(),
        edited,
        "取り込みはファイルを書かない"
    );
}

#[test]
fn tool_cannot_adopt_without_knowing_whether_a_human_field_changed() {
    let store = files(&[
        ("c.json", decided()),
        (
            "u1.json",
            json!({"$schema": UC, "id": "U1", "goal": "a", "status": "draft"}),
        ),
    ]);
    let mut log = Log::default();
    let none = Commits(Vec::new());
    adopt(
        &adopt_request(AdoptAs::Create),
        &store,
        &Lenient,
        &none,
        &Human,
        &mut log,
    )
    .unwrap();
    store
        .write(
            "u1.json",
            &json!({"$schema": UC, "id": "U1", "goal": "b", "status": "approved"}).to_string(),
        )
        .unwrap();
    let r = adopt(
        &adopt_request(AdoptAs::Update),
        &store,
        &Lenient,
        &none,
        &Tool,
        &mut log,
    )
    .unwrap_err();
    assert_eq!(r.code, "human_field");
    assert!(r.message.contains("前の中身のコミットが無く"));
    assert!(r.hint.unwrap().contains("--session を付けずに"));
    // 人なら、前の値を持たない差分で取り込める
    let c = adopt(
        &adopt_request(AdoptAs::Update),
        &store,
        &Lenient,
        &none,
        &Human,
        &mut log,
    )
    .unwrap();
    assert!(c.diff.before_unavailable());
    assert!(c.diff.changes().iter().all(|f| f.before().is_none()));
}
