//! 誰が打ったかを決めて登録する手続きを、置き場所の代役で確かめる。

#![allow(clippy::unwrap_used)]

use recording_application::ports::{IdentifyCaller, NotIdentified, SessionLog, SessionRow};
use recording_application::use_cases::history::{
    Unidentified, identify, register, register_session,
};
use writing_domain::values::{Caller, Via};

/// 引数と環境変数で決まる呼び出し元（None は、どちらにも無い）。semantic-agent-schema init の登録があればそれを使う。
struct Fixed(Option<Caller>);
impl IdentifyCaller for Fixed {
    fn identify(&self) -> Result<Option<Caller>, NotIdentified> {
        Ok(self.0.clone())
    }
}

/// 2つのツールの環境変数に値があって決められない呼び出し元。
struct Nested;
impl IdentifyCaller for Nested {
    fn identify(&self) -> Result<Option<Caller>, NotIdentified> {
        Err(NotIdentified::Ambiguous(vec![
            "claude-code".into(),
            "codex".into(),
        ]))
    }
}

#[derive(Default)]
struct Memory {
    sessions: Vec<SessionRow>,
}
impl SessionLog for Memory {
    fn session(&mut self, caller: &Caller) -> Result<Option<String>, String> {
        let Caller::Session {
            id,
            tool,
            folder,
            via,
        } = caller
        else {
            return Ok(None);
        };
        if !self.sessions.iter().any(|s| &s.id == id) {
            self.sessions.push(SessionRow {
                id: id.clone(),
                tool: tool.clone(),
                folder: folder.clone(),
                via: via.as_str().to_string(),
            });
        }
        Ok(Some(id.clone()))
    }
    fn session_row(&self, id: &str) -> Result<Option<SessionRow>, String> {
        Ok(self.sessions.iter().find(|s| s.id == id).cloned())
    }
    fn new_session_id(&self) -> String {
        format!("sas-{}", self.sessions.len() + 1)
    }
}

fn session(id: &str) -> Caller {
    Caller::Session {
        id: id.into(),
        tool: "claude-code".into(),
        folder: "/w".into(),
        via: Via::Env,
    }
}

#[test]
fn session_is_registered_once_and_a_human_adds_nothing() {
    let mut log = Memory::default();
    register(&session("p1"), &mut log).unwrap();
    register(&Caller::Human, &mut log).unwrap();
    register(&session("p1"), &mut log).unwrap();
    let ids: Vec<&str> = log.sessions.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, ["p1"]);
}

#[test]
fn identifying_does_not_register() {
    let log = Memory::default();
    assert_eq!(
        identify(&Fixed(Some(session("p1")))).unwrap(),
        session("p1")
    );
    assert_eq!(
        identify(&Fixed(Some(Caller::Human))).unwrap(),
        Caller::Human
    );
    assert_eq!(
        identify(&Fixed(None)),
        Err(Unidentified::Missing),
        "何も無ければ人とみなさない"
    );
    assert!(log.sessions.is_empty());
    assert_eq!(
        identify(&Nested),
        Err(Unidentified::Ambiguous(vec![
            "claude-code".into(),
            "codex".into()
        ]))
    );
}

#[test]
fn init_registers_a_new_session_that_is_not_a_default() {
    let mut log = Memory::default();
    let h = register_session("/w", &mut log).unwrap();
    assert_eq!(
        (h.tool.as_str(), h.via.as_str()),
        ("unknown", "init"),
        "新しい ID を登録する"
    );
    assert_eq!(log.session_row(&h.id).unwrap(), Some(h.clone()));
    // 登録しても、--session も環境変数も無い呼び出しの既定にはならない
    assert_eq!(identify(&Fixed(None)), Err(Unidentified::Missing));
}
