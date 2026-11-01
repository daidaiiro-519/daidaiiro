//! SF1 の手続き（rule を検証用の子セッションに判定させる）を、ポートの代役で確かめる。
//! 子セッションを起動する代役は、渡されたプロンプトを残し、決めた答えを返す。

#![allow(clippy::unwrap_used, clippy::panic)]

mod support;

use recording_application::ports::ChildRow;
use serde_json::{Value, json};
use std::cell::RefCell;
use support::{Human, Lenient, Log, UC, files, meaning};
use validation_application::ports::{DocumentStore, ReadMeaning, Reply};
use validation_application::use_cases::meaning::MeaningCall;
use validation_application::use_cases::validate::{Scope, validate_with_meaning};
use validation_domain::values::Verdict;

/// 決めた答えを返し、渡されたプロンプトを (子セッションの ID か None, プロンプト) で残す代役。
struct Scripted {
    sent: RefCell<Vec<(Option<String>, String)>>,
    verdict: &'static str,
    reason: &'static str,
}
impl Scripted {
    fn new(verdict: &'static str, reason: &'static str) -> Self {
        Self {
            sent: RefCell::new(Vec::new()),
            verdict,
            reason,
        }
    }
    fn answer(&self) -> String {
        json!({"results": [{"step": 1, "rule": "goal-is-outcome", "pointer": "/goal",
                            "result": self.verdict, "reason": self.reason}]})
        .to_string()
    }
}
impl ReadMeaning for Scripted {
    fn tool(&self) -> String {
        "scripted".into()
    }
    fn model(&self) -> Option<String> {
        None
    }
    fn launch(&self) -> Value {
        json!({"argv": ["scripted"]})
    }
    fn cache_ttl_s(&self) -> Option<u64> {
        None
    }
    fn start(&self, prompt: &str, _: &Value) -> Reply {
        self.sent.borrow_mut().push((None, prompt.into()));
        Reply::Answered {
            tool_ref: "checker-1".into(),
            answer: self.answer(),
        }
    }
    fn resume(&self, tool_ref: &str, _: &Value, prompt: &str, _: &Value) -> Reply {
        self.sent
            .borrow_mut()
            .push((Some(tool_ref.into()), prompt.into()));
        Reply::Answered {
            tool_ref: tool_ref.into(),
            answer: self.answer(),
        }
    }
}

fn store() -> support::Files {
    files(&[
        ("c.json", meaning()),
        ("u1.json", json!({"$schema": UC, "goal": "注文が届く"})),
    ])
}

fn run(
    store: &support::Files,
    log: &mut Log,
    reader: &Scripted,
    continue_child: Option<&str>,
) -> Result<Verdict, &'static str> {
    let call = MeaningCall {
        reader,
        continue_child: continue_child.map(str::to_string),
    };
    validate_with_meaning(
        &Scope::Paths(vec!["u1.json".into()]),
        false,
        store,
        &Lenient,
        &Human,
        log,
        &call,
    )
    .map(|(v, _)| {
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
fn writer_is_refused_as_checker_and_nothing_is_sent() {
    let store = store();
    let mut log = Log::default();
    log.append_child_row("w-1", "writer");
    let reader = Scripted::new("verified", "結果を書いている");
    assert_eq!(
        run(&store, &mut log, &reader, Some("w-1")),
        Err("writer_as_checker")
    );
    assert!(
        reader.sent.borrow().is_empty(),
        "書き込み用の子セッションへは送らない"
    );
    assert!(log.0.is_empty(), "断った検証は記録しない");
}

#[test]
fn continued_child_gets_the_same_prompt_as_a_new_child() {
    // 続けた子セッションに渡すのは、いまの中身だけ。前の判定の理由を渡さないので、同じ中身なら新しい子セッションと同じプロンプトになる
    let store = store();
    let mut log = Log::default();
    let reader = Scripted::new("violated", "前の判定の理由");
    assert_eq!(run(&store, &mut log, &reader, None), Ok(Verdict::Violated));
    assert_eq!(log.0[0].child_session.as_deref(), Some("checker-1"));
    assert_eq!(
        run(&store, &mut log, &reader, Some("checker-1")),
        Ok(Verdict::Violated)
    );
    let sent = reader.sent.borrow();
    assert_eq!(sent[1].0.as_deref(), Some("checker-1"));
    assert_eq!(sent[0].1, sent[1].1);
    assert!(!sent[1].1.contains("前の判定の理由"));
}

#[test]
fn prompt_follows_the_current_content() {
    let store = store();
    let mut log = Log::default();
    let reader = Scripted::new("verified", "結果を書いている");
    run(&store, &mut log, &reader, None).unwrap();
    store
        .write(
            "u1.json",
            &json!({"$schema": UC, "goal": "画面を作る"}).to_string(),
        )
        .unwrap();
    run(&store, &mut log, &reader, Some("checker-1")).unwrap();
    let sent = reader.sent.borrow();
    assert!(sent[1].1.contains("画面を作る"));
    assert!(!sent[1].1.contains("注文が届く"));
}

trait SeedChild {
    fn append_child_row(&mut self, id: &str, role: &str);
}
impl SeedChild for Log {
    fn append_child_row(&mut self, id: &str, role: &str) {
        use recording_application::ports::OperationLog;
        self.append_child(
            None,
            &ChildRow {
                id: id.into(),
                role: role.into(),
                tool: "scripted".into(),
                model: None,
                launch: "{}".into(),
            },
        )
        .unwrap();
    }
}

#[test]
fn reading_of_the_field_goes_to_the_child_session() {
    // x-prompt の read（interpret ・ when）は、子セッションが値を読む手がかりとして手順に入る
    let mut c = meaning();
    c["properties"]["goal"]["x-prompt"]["read"] =
        json!({"interpret": "利用者の言葉で書いた結果として読む", "when": "検証するときに読む"});
    let store = files(&[
        ("c.json", c),
        ("u1.json", json!({"$schema": UC, "goal": "注文が届く"})),
    ]);
    let mut log = Log::default();
    let reader = Scripted::new("verified", "結果を書いている");
    run(&store, &mut log, &reader, None).unwrap();
    let sent = reader.sent.borrow();
    assert!(sent[0].1.contains("利用者の言葉で書いた結果として読む"));
    assert!(sent[0].1.contains("検証するときに読む"));
}

#[test]
fn field_schema_follows_ref_and_additional_properties() {
    use validation_application::use_cases::meaning::field_schema;
    let other = json!({"$defs": {"person": {"properties": {"name": {"title": "名前", "description": "人の名前である。"}}}}});
    let concrete = json!({
        "$defs": {"step": {"title": "手順", "properties": {"action": {"title": "動作"}}}},
        "properties": {
            "steps": {"items": {"$ref": "#/$defs/step"}},
            "owner": {"$ref": "urn:example:t:person:v1#/$defs/person", "title": "持ち主"},
            "tags": {"additionalProperties": {"title": "札"}}
        }
    });
    let others = vec![("urn:example:t:person:v1".to_string(), other)];
    let title = |p: &str| {
        field_schema(&concrete, p, &others)
            .and_then(|f| f.get("title").cloned())
            .unwrap_or(Value::Null)
    };
    assert_eq!(title("/steps/0/action"), json!("動作"));
    assert_eq!(title("/steps/0"), json!("手順"));
    assert_eq!(
        title("/owner"),
        json!("持ち主"),
        "$ref と横に並べた欄を先に読む"
    );
    assert_eq!(title("/owner/name"), json!("名前"));
    assert_eq!(title("/tags/a"), json!("札"));
}
