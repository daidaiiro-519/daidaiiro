//! SF1 の手続き（by: ai の rule を検証用の子セッションに判定させる）を、ポートの代役で確かめる。
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
                            "result": self.verdict, "reason": self.reason, "grounds": ["/goal"]}]})
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
