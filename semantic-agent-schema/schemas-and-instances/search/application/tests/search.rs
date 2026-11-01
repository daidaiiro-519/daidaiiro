//! UC6 検索する（範囲 ・ 構造で絞る ・ 意味で探す手順 ・ 答えの出どころの確かめ）を、ポートの代役で確かめる。
//! 代役は検証のモジュールの試験と共に使う（support.rs）。ポートへの当て方は、検索のモジュールのポートにここで書く。

#![allow(clippy::unwrap_used, clippy::panic)]

#[path = "../../../validation/application/tests/support.rs"]
mod support;

use search_application::use_cases::search::{
    RANGE_LIMIT, Range, search_procedure, search_where, search_with_answer,
};
use serde_json::{Value, json};
use support::{HUMAN, Lenient, UC, files};
use writing_domain::values::ContentHash;

/// 代役を、検索のモジュールのポートに当てる（ポートはモジュールごとに宣言する）。
impl search_application::ports::DocumentStore for support::Files {
    fn list(&self) -> Result<Vec<String>, String> {
        Self::list(self)
    }
    fn read(&self, path: &str) -> Result<String, String> {
        Self::read(self, path)
    }
}

impl search_application::ports::SchemaValidator for support::Lenient {
    fn check(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, String)>, String> {
        Self::check(self, schema, value, more)
    }
}

fn concrete() -> Value {
    json!({
        "$schema": support::ABSTRACT,
        "$id": UC,
        "title": "ユースケース",
        "description": "利用者が目的を果たす手順",
        "properties": {"$schema": {"const": UC}, "level": {"type": "string"}, "actors": {"type": "array"}}
    })
}

fn store() -> support::Files {
    files(&[
        ("c.json", concrete()),
        (
            "u0.json",
            json!({"$schema": UC, "level": "summary", "actors": ["利用者"]}),
        ),
        (
            "u1.json",
            json!({"$schema": UC, "level": "user", "actors": ["利用者", "運用担当"]}),
        ),
        ("other.json", json!({"name": "型の無い文書"})),
    ])
}

fn hash(store: &support::Files, path: &str) -> String {
    ContentHash::of(store.read(path).unwrap().as_bytes())
        .as_str()
        .to_string()
}

fn concrete_range() -> Range {
    Range::Concrete(UC.to_string())
}

#[test]
fn the_range_is_returned_and_only_its_documents_are_searched() {
    let s = store();
    let found = search_where(
        &concrete_range(),
        &[("/level".into(), json!("summary"))],
        &s,
    )
    .unwrap();
    let paths: Vec<&str> = found.range.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(paths, ["u0.json", "u1.json"]);
    assert_eq!(found.found.len(), 1);
    assert_eq!(found.found[0].path, "u0.json");
    let all = search_where(&Range::All, &[("/level".into(), json!("user"))], &s).unwrap();
    assert_eq!(
        all.range.len(),
        3,
        "Concrete とインスタンスだけが範囲に入る"
    );
    assert_eq!(all.found[0].path, "u1.json");
}

#[test]
fn every_condition_must_match() {
    let s = store();
    let both = [
        ("/level".to_string(), json!("user")),
        ("/actors/1".to_string(), json!("運用担当")),
    ];
    let found = search_where(&concrete_range(), &both, &s).unwrap();
    assert_eq!(found.found.len(), 2);
    let none = [
        ("/level".to_string(), json!("summary")),
        ("/actors/1".to_string(), json!("運用担当")),
    ];
    assert!(
        search_where(&concrete_range(), &none, &s)
            .unwrap()
            .found
            .is_empty()
    );
}

#[test]
fn an_untyped_path_or_unknown_concrete_is_refused() {
    let s = store();
    let r = search_where(&Range::Paths(vec!["other.json".into()]), &[], &s).unwrap_err();
    assert_eq!(r.code, "no_schema");
    let r = search_where(&Range::Concrete("urn:example:none:v1".into()), &[], &s).unwrap_err();
    assert_eq!(r.code, "unknown_schema");
}

#[test]
fn the_procedure_has_one_step_per_document_and_the_answer_command() {
    let s = store();
    let p = search_procedure(&concrete_range(), "運用担当が出てくるのは？", &s, &HUMAN).unwrap();
    assert_eq!(p["operation_kind"], json!("search"));
    assert_eq!(p["steps"].as_array().unwrap().len(), 2);
    assert_eq!(p["steps"][1]["path"], json!("u1.json"));
    assert_eq!(p["steps"][1]["hash"], json!(hash(&s, "u1.json")));
    assert_eq!(p["steps"][1]["current"]["level"], json!("user"));
    assert_eq!(p["answer"]["format"], json!("search"));
    assert_eq!(
        p["command"],
        json!("sas search --concrete urn:example:t:uc:v1 --answer <答えの JSON のファイル | ->")
    );
}

#[test]
fn a_range_over_the_limit_is_refused_without_cutting() {
    let big = "あ".repeat(RANGE_LIMIT / 3 + 1);
    let s = files(&[
        ("c.json", concrete()),
        ("u0.json", json!({"$schema": UC, "level": big})),
    ]);
    let r = search_procedure(&concrete_range(), "問い", &s, &HUMAN).unwrap_err();
    assert_eq!(r.code, "range_too_large");
}

fn answer(sources: &Value) -> String {
    json!({"answer": "u1 に出てくる", "sources": sources}).to_string()
}

#[test]
fn sources_in_the_range_are_returned_with_their_current_values() {
    let s = store();
    let h = hash(&s, "u1.json");
    let found = search_with_answer(
        &concrete_range(),
        &answer(&json!([{"path": "u1.json", "pointer": "/actors/1", "hash": h}])),
        &s,
        &Lenient,
    )
    .unwrap();
    assert_eq!(found.answer.as_deref(), Some("u1 に出てくる"));
    assert_eq!(found.found[0].value, json!("運用担当"));
}

#[test]
fn sources_outside_changed_or_missing_are_refused() {
    let s = store();
    let h = hash(&s, "u1.json");
    let code = |sources: Value| {
        search_with_answer(&concrete_range(), &answer(&sources), &s, &Lenient)
            .map(|_| ())
            .map_err(|r| r.code)
    };
    assert_eq!(
        code(json!([{"path": "c.json", "pointer": "/title", "hash": hash(&s, "c.json")}])),
        Err("invalid_answer")
    );
    assert_eq!(
        code(json!([{"path": "u1.json", "pointer": "/actors/1", "hash": hash(&s, "u0.json")}])),
        Err("conflict")
    );
    assert_eq!(
        code(json!([{"path": "u1.json", "pointer": "/actors/9", "hash": h}])),
        Err("invalid_answer")
    );
}
