//! UC4 の意味の部分（判定の手順を返し、打ったセッションが渡した判定を記録する）を、ポートの代役で確かめる。

#![allow(clippy::unwrap_used, clippy::panic)]

mod support;

use serde_json::{Value, json};
use support::{HUMAN, Lenient, Log, UC, files, meaning};
use validation_application::use_cases::validate::{
    Scope, Selection, semantic_procedures, validate_with_answer,
};
use validation_domain::values::Verdict;
use writing_domain::values::{Caller, ContentHash, Via};

/// 代役を、検証のモジュールのポートに当てる（ポートはモジュールごとに宣言する）。
impl validation_application::ports::DocumentStore for support::Files {
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

/// 既定の選び方（前の判定から変わった rule だけ。前の判定が無ければすべて）。
const CHANGED: Selection = Selection {
    all_rules: false,
    stale: false,
};

fn store() -> support::Files {
    files(&[
        ("c.json", meaning()),
        ("u1.json", json!({"$schema": UC, "goal": "注文が届く"})),
    ])
}

fn procedures(store: &support::Files, log: &Log, caller: &Caller) -> Vec<Value> {
    semantic_procedures(
        &Scope {
            paths: vec!["u1.json".into()],
            schemas: vec!["c.json".into()],
        },
        CHANGED,
        store,
        &Lenient,
        caller,
        log,
    )
    .unwrap()
}

fn hash(store: &support::Files) -> String {
    ContentHash::of(store.read("u1.json").unwrap().as_bytes())
        .as_str()
        .to_string()
}

fn answer(hash: &str, rule: &str, verdict: &str, reason: &str) -> String {
    json!({"hash": hash, "results": [{"step": 1, "rule": rule, "pointer": "/goal",
                                      "result": verdict, "reason": reason}]})
    .to_string()
}

fn give(store: &support::Files, log: &mut Log, answer: &str) -> Result<Verdict, &'static str> {
    let scope = Scope {
        paths: vec!["u1.json".into()],
        schemas: vec!["c.json".into()],
    };
    validate_with_answer(&scope, CHANGED, answer, store, &Lenient, &HUMAN, log)
        .map(|v| {
            v.targets[0]
                .results
                .iter()
                .find(|r| r.rule() == "goal-is-outcome")
                .unwrap()
                .verdict()
        })
        .map_err(|r| r.code)
}

#[test]
fn procedure_is_returned_and_nothing_is_recorded() {
    let store = store();
    let log = Log::default();
    let p = procedures(&store, &log, &HUMAN);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0]["steps"][0]["rules"][0], json!("goal-is-outcome"));
    assert_eq!(p[0]["steps"][1]["action"], json!("review-document"));
    assert_eq!(p[0]["rules"][0]["id"], json!("goal-is-outcome"));
    assert_eq!(p[0]["steps"][0]["current"], json!("注文が届く"));
    assert_eq!(p[0]["target"]["hash"], json!(hash(&store)));
    assert_eq!(
        p[0]["command"],
        json!("semantic-agent-schema validate u1.json --answer <判定の JSON のファイル | ->")
    );
    assert!(log.0.is_empty(), "手順を返すだけで記録しない");
}

#[test]
fn a_named_session_names_itself_again_in_the_command() {
    let named = Caller::Session {
        id: "ci 1".into(),
        tool: "unknown".into(),
        folder: "/w".into(),
        via: Via::Flag,
    };
    let p = procedures(&store(), &Log::default(), &named);
    assert!(
        p[0]["command"]
            .as_str()
            .unwrap()
            .ends_with("--session 'ci 1'")
    );
}

#[test]
fn the_answer_is_recorded_with_the_machine_results() {
    let store = store();
    let mut log = Log::default();
    let h = hash(&store);
    assert_eq!(
        give(
            &store,
            &mut log,
            &answer(&h, "goal-is-outcome", "violated", "手段だ")
        ),
        Ok(Verdict::Violated)
    );
    assert_eq!(log.0.len(), 1);
    assert!(
        log.0[0]
            .results
            .iter()
            .any(|r| r.by == "ai" && r.result == "violated" && r.reason == "手段だ")
    );
    assert!(log.0[0].results.iter().any(|r| r.by == "machine"));
}

#[test]
fn an_answer_for_other_content_is_refused_and_not_recorded() {
    let store = store();
    let mut log = Log::default();
    let old = hash(&store);
    store
        .write(
            "u1.json",
            &json!({"$schema": UC, "goal": "画面を作る"}).to_string(),
        )
        .unwrap();
    assert_eq!(
        give(
            &store,
            &mut log,
            &answer(&old, "goal-is-outcome", "verified", "結果だ")
        ),
        Err("conflict")
    );
    assert!(log.0.is_empty());
}

#[test]
fn an_answer_that_does_not_match_the_steps_is_refused() {
    let store = store();
    let mut log = Log::default();
    let h = hash(&store);
    assert_eq!(
        give(
            &store,
            &mut log,
            &answer(&h, "other-rule", "verified", "結果だ")
        ),
        Err("invalid_answer")
    );
    assert_eq!(
        give(&store, &mut log, "判定しました。verified です。"),
        Err("invalid_answer")
    );
    assert!(log.0.is_empty());
}

#[test]
fn procedure_follows_the_current_content_and_not_the_previous_judgment() {
    // 手順に入れるのは、いまの中身だけ。前の判定の理由は入れない
    let store = store();
    let mut log = Log::default();
    let h = hash(&store);
    give(
        &store,
        &mut log,
        &answer(&h, "goal-is-outcome", "violated", "前の判定の理由"),
    )
    .unwrap();
    store
        .write(
            "u1.json",
            &json!({"$schema": UC, "goal": "画面を作る"}).to_string(),
        )
        .unwrap();
    let text = procedures(&store, &log, &HUMAN)[0].to_string();
    assert!(text.contains("画面を作る"));
    assert!(!text.contains("注文が届く"));
    assert!(!text.contains("前の判定の理由"));
}

#[test]
fn reading_of_the_field_goes_into_the_procedure() {
    // x-prompt の meaning ・ when は、値を読む手がかりとして手順に同じ名前で入る
    let mut c = meaning();
    c["properties"]["goal"]["x-prompt"]["meaning"] = json!("利用者の言葉で書いた結果として読む");
    c["properties"]["goal"]["x-prompt"]["when"] = json!("検証するときに読む");
    let store = files(&[
        ("c.json", c),
        ("u1.json", json!({"$schema": UC, "goal": "注文が届く"})),
    ]);
    let p = procedures(&store, &Log::default(), &HUMAN);
    assert_eq!(
        p[0]["steps"][0]["meaning"],
        json!("利用者の言葉で書いた結果として読む")
    );
    assert_eq!(p[0]["steps"][0]["when"], json!("検証するときに読む"));
}

#[test]
fn only_json_is_read_as_an_answer() {
    use validation_application::use_cases::meaning::answer_json;
    assert_eq!(answer_json(" {\"a\": 1}\n").unwrap(), json!({"a": 1}));
    assert!(
        answer_json("```json\n{\"a\": 1}\n```").is_err(),
        "囲みを外して読まない"
    );
}

#[test]
fn command_line_quotes_only_what_needs_it() {
    use validation_application::use_cases::meaning::command_line;
    assert_eq!(
        command_line(&[
            "semantic-agent-schema",
            "update",
            "a b.json",
            "--reason",
            "it's",
            "<JSON Patch>"
        ]),
        "semantic-agent-schema update 'a b.json' --reason 'it'\\''s' <JSON Patch>"
    );
}
