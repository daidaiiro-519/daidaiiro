//! UC1 ・ UC2 と取り込みの手続きを、ポートの代役で確かめる。代役は検証のモジュールの試験と共に使う（support.rs）。
//! ポートへの当て方は、書くモジュールのポートにここで書く。

#![allow(clippy::unwrap_used, clippy::panic)]

#[path = "../../../validation/application/tests/support.rs"]
mod support;

use serde_json::{Value, json};
use support::{HUMAN, Lenient, Log, UC, decided, files, tool};
use validation_domain::services::{instance_rules, reads_hash};
use validation_domain::values::{Judgment, RuleRow, ValidateRow};
use writing_application::ports::CommitReader;
use writing_application::use_cases::write::{
    AdoptAs, AdoptRequest, CreateRequest, UpdateRequest, adopt, create, interleaved, update,
};
use writing_domain::values::{ChangeRow, ContentHash, StoredChange, WriteRecord};

/// 代役を、書くモジュールのポートに当てる（ポートはモジュールごとに宣言する）。
impl writing_application::ports::DocumentStore for support::Files {
    fn read(&self, path: &str) -> Result<String, String> {
        Self::read(self, path)
    }
    fn write(&self, path: &str, text: &str) -> Result<(), String> {
        Self::write(self, path, text)
    }
    fn remove(&self, path: &str) -> Result<(), String> {
        Self::remove(self, path)
    }
}

impl writing_application::ports::SchemaValidator for support::Lenient {
    fn shipped(&self, id: &str) -> Option<Value> {
        Self::shipped(self, id)
    }
    fn check(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, String)>, String> {
        Self::check(self, schema, value, more)
    }
    fn annotations(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, Value)>, String> {
        Self::annotations(self, schema, value, more)
    }
}

impl writing_application::ports::ChangeLog for support::Log {
    fn append_change(&mut self, session: Option<&str>, row: &ChangeRow) -> Result<String, String> {
        Self::append_change(self, session, row)
    }
    fn find_request(&self, request_id: &str) -> Result<Option<StoredChange>, String> {
        Self::find_request(self, request_id)
    }
    fn writes(&self, target_path: &str) -> Result<Vec<WriteRecord>, String> {
        Self::writes(self, target_path)
    }
    fn last_judgments(&self, target_path: &str) -> Result<Vec<Judgment>, String> {
        Self::last_judgments(self, target_path)
    }
    fn instances_of(&self, schema: &str) -> Result<Vec<String>, String> {
        Self::instances_of(self, schema)
    }
}

fn create_request(value: Value, request_id: Option<&str>) -> CreateRequest {
    CreateRequest {
        path: "u1.json".into(),
        concrete: UC.into(),
        value,
        schemas: support::places(&["c.json"]),
        request_id: request_id.map(Into::into),
        reason: None,
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
        &tool(),
        &mut log,
    )
    .unwrap();
    let text = store.read("u1.json").unwrap();
    assert_eq!(c.after, ContentHash::of(text.as_bytes()));
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
        &tool(),
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
    let first = create(&req, &store, &Lenient, &HUMAN, &mut log).unwrap();
    let again = create(&req, &store, &Lenient, &HUMAN, &mut log).unwrap();
    assert!(again.replayed);
    assert_eq!((again.operation, again.diff), (first.operation, first.diff));
    assert_eq!(log.1.len(), 1);
    let other = UpdateRequest {
        path: "u1.json".into(),
        patch: json!([]),
        if_match: Some(first.after.as_str().into()),
        schemas: support::places(&["c.json"]),
        range: None,
        request_id: Some("r-1".into()),
        reason: None,
    };
    let r = update(&other, &store, &Lenient, &HUMAN, &mut log).unwrap_err();
    assert_eq!(r.code, "request_reused");
}

#[test]
fn update_checks_the_hash_before_writing() {
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
        schemas: support::places(&["c.json"]),
        range: None,
        request_id: None,
        reason: None,
    };
    let r = update(&req("sha256:x"), &store, &Lenient, &HUMAN, &mut log).unwrap_err();
    assert_eq!(r.code, "conflict");
    assert_eq!(store.read("u1.json").unwrap(), before);
    assert!(log.1.is_empty(), "断った操作は記録しない");
    let fp = ContentHash::of(before.as_bytes());
    let c = update(&req(fp.as_str()), &store, &Lenient, &HUMAN, &mut log).unwrap();
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
        schemas: support::places(&["c.json"]),
        range: None,
        request_id: None,
        reason: Some("目標を直す".into()),
    };
    // SAS が一度も書いていないので、比べるハッシュが無く、書く
    let first = update(&req("b"), &store, &Lenient, &HUMAN, &mut log).unwrap();
    // 最後に書いたハッシュといまの中身が同じなので、書く
    update(&req("c"), &store, &Lenient, &HUMAN, &mut log).unwrap();
    // SAS を通らずに直される
    store
        .write(
            "u1.json",
            &json!({"$schema": UC, "id": "U1", "goal": "x"}).to_string(),
        )
        .unwrap();
    let edited = store.read("u1.json").unwrap();
    let r = update(&req("d"), &store, &Lenient, &HUMAN, &mut log).unwrap_err();
    assert_eq!(r.code, "conflict");
    assert_eq!(store.read("u1.json").unwrap(), edited);
    let i = r.interleaved.unwrap();
    assert!(i.unrecorded);
    assert!(i.operations.is_empty());
    assert_eq!(i.last_recorded.map(|w| w.operation).as_deref(), Some("o-2"));
    // 古いハッシュを --if-match で渡すと、そのあとの操作が理由と一緒に返る
    let r = update(
        &UpdateRequest {
            if_match: Some(first.before.unwrap().as_str().into()),
            ..req("d")
        },
        &store,
        &Lenient,
        &HUMAN,
        &mut log,
    )
    .unwrap_err();
    let i = r.interleaved.unwrap();
    let ids: Vec<&str> = i.operations.iter().map(|w| w.operation.as_str()).collect();
    assert_eq!(ids, ["o-1", "o-2"]);
    assert_eq!(i.operations[0].row.reason.as_deref(), Some("目標を直す"));
    assert!(i.unrecorded);
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
    // 前の検証の判定：検証のモジュールが記録するのと同じ基準（validation-domain）で、いまの中身の rule ごとの読んだ所を記録する
    let text = store.read("u1.json").unwrap();
    let before: Value = serde_json::from_str(&text).unwrap();
    log.0.push(ValidateRow {
        target_path: "u1.json".into(),
        hash: ContentHash::of(text.as_bytes()).as_str().into(),
        schema: Some(UC.into()),
        results: instance_rules(&decided(), &before, &[(UC.to_string(), decided())])
            .into_iter()
            .map(|(r, at, definition)| RuleRow {
                rule: r.rule().into(),
                pointer: r.pointer().into(),
                by: r.by().as_str().into(),
                result: r.verdict().as_str().into(),
                reason: r.reason().into(),
                reads: reads_hash(&before, &at, definition.as_ref())
                    .as_str()
                    .into(),
            })
            .collect(),
    });
    let fp = ContentHash::of(text.as_bytes());
    let c = update(
        &UpdateRequest {
            path: "u1.json".into(),
            patch: json!([{"op": "replace", "path": "/status", "value": "approved"}, {"op": "replace", "path": "/goal", "value": "b"}]),
            if_match: Some(fp.as_str().into()),
            schemas: support::places(&["c.json"]),
            range: None,
            request_id: None,
            reason: None,
        },
        &store,
        &Lenient,
        &HUMAN,
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

/// コミット済みの版の代役：コミットの名前と中身の並び。
struct Commits(Vec<(String, String)>);
impl CommitReader for Commits {
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
        reason: Some("エディターで直した".into()),
        schemas: support::places(&["c.json"]),
        range: None,
        request_id: None,
        base: None,
    }
}

fn adopt_from(base: &str) -> AdoptRequest {
    AdoptRequest {
        base: Some(base.into()),
        ..adopt_request(AdoptAs::Update)
    }
}

#[test]
fn adopted_update_compares_with_the_base_commit() {
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
        &HUMAN,
        &mut log,
    )
    .unwrap();
    assert!(c.outside);
    assert!(c.before.is_none());
    // 外で goal を直す。HEAD は外でのコミット（中身が違う）、その前のコミットが記録の最後の事後で、それを --base に渡す
    let edited = json!({"$schema": UC, "id": "U1", "goal": "b", "status": "draft"}).to_string();
    store.write("u1.json", &edited).unwrap();
    let commits = Commits(vec![
        ("head".into(), edited.clone()),
        ("older".into(), committed),
    ]);
    let r = adopt(
        &adopt_from("head"),
        &store,
        &Lenient,
        &commits,
        &HUMAN,
        &mut log,
    )
    .unwrap_err();
    assert_eq!(
        r.code, "invalid_input",
        "記録に無い中身のコミットは使わない"
    );
    let c = adopt(
        &adopt_from("older"),
        &store,
        &Lenient,
        &commits,
        &HUMAN,
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

/// コミット済みの版を読めない代役。
struct Unreadable;
impl CommitReader for Unreadable {
    fn read(&self, _: &str, _: &str) -> Result<Option<Vec<u8>>, String> {
        Err("版を読めない".into())
    }
}

#[test]
fn unreadable_versions_are_refused_as_versions() {
    let first = json!({"$schema": UC, "id": "U1", "goal": "a", "status": "draft"});
    let store = files(&[("c.json", decided()), ("u1.json", first)]);
    let mut log = Log::default();
    adopt(
        &adopt_request(AdoptAs::Create),
        &store,
        &Lenient,
        &Commits(Vec::new()),
        &HUMAN,
        &mut log,
    )
    .unwrap();
    let edited = json!({"$schema": UC, "id": "U1", "goal": "b", "status": "draft"}).to_string();
    store.write("u1.json", &edited).unwrap();
    let r = adopt(
        &adopt_from("HEAD"),
        &store,
        &Lenient,
        &Unreadable,
        &HUMAN,
        &mut log,
    )
    .unwrap_err();
    assert_eq!(r.code, "versions", "CLI の出力の一覧にある名前で断る");
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
        &HUMAN,
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
        &tool(),
        &mut log,
    )
    .unwrap_err();
    assert_eq!(r.code, "human_field");
    assert!(r.message.contains("--base が無く"));
    assert!(r.hint.unwrap().contains("--human"));
    // 人なら、前の値を持たない差分で取り込める
    let c = adopt(
        &adopt_request(AdoptAs::Update),
        &store,
        &Lenient,
        &none,
        &HUMAN,
        &mut log,
    )
    .unwrap();
    assert!(c.diff.before_unavailable());
    assert!(c.diff.changes().iter().all(|f| f.before().is_none()));
}

fn written(n: usize, before: &str, after: &str) -> WriteRecord {
    WriteRecord {
        operation: format!("o-{n}"),
        session: None,
        at: "t".into(),
        same_worktree: true,
        row: ChangeRow {
            kind: "update".into(),
            source: "sas".into(),
            target_path: "a.json".into(),
            request_id: None,
            before: Some(before.into()),
            after: Some(after.into()),
            reason: None,
            schema: None,
            fields: Vec::new(),
        },
    }
}

#[test]
fn interleaved_follows_hashs_from_expected() {
    let writes = [written(1, "a", "b"), written(2, "b", "c")];

    // 前提の b のあとに、o-2（b → c）が入った
    let i = interleaved(&writes, "b", "c");
    let ids: Vec<&str> = i.operations.iter().map(|w| w.operation.as_str()).collect();
    assert_eq!(ids, ["o-2"]);
    assert!(!i.unrecorded);

    // 記録の最後（c）のあとに、記録に無い変更で d になった
    let i = interleaved(&writes, "c", "d");
    assert!(i.operations.is_empty());
    assert!(i.unrecorded);
    assert_eq!(i.last_recorded.map(|w| w.operation).as_deref(), Some("o-2"));

    // 前提のハッシュが記録に無いときは、記録した操作をすべて返す
    let i = interleaved(&writes, "z", "c");
    assert_eq!(i.operations.len(), 2);
    assert!(!i.unrecorded);
}
