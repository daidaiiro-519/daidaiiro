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
    fn identify(&self, initialized: Option<SessionRow>) -> Result<Option<Caller>, NotIdentified> {
        Ok(match (&self.0, initialized) {
            (Some(c), _) => Some(c.clone()),
            (None, Some(row)) => Some(Caller::Session {
                id: row.id,
                tool: row.tool,
                folder: row.folder,
                via: Via::Init,
            }),
            (None, None) => None,
        })
    }
}

/// 2つのツールの環境変数に値があって決められない呼び出し元。
struct Nested;
impl IdentifyCaller for Nested {
    fn identify(&self, _: Option<SessionRow>) -> Result<Option<Caller>, NotIdentified> {
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
    fn initialized(&self, folder: &str) -> Result<Option<SessionRow>, String> {
        Ok(self
            .sessions
            .iter()
            .rev()
            .find(|s| s.folder == folder && s.via == "init")
            .cloned())
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
        identify(&Fixed(Some(session("p1"))), "/w", &log).unwrap(),
        session("p1")
    );
    assert_eq!(
        identify(&Fixed(Some(Caller::Human)), "/w", &log).unwrap(),
        Caller::Human
    );
    assert_eq!(
        identify(&Fixed(None), "/w", &log),
        Err(Unidentified::Missing),
        "何も無ければ人とみなさない"
    );
    assert!(log.sessions.is_empty());
    assert_eq!(
        identify(&Nested, "/w", &log),
        Err(Unidentified::Ambiguous(vec![
            "claude-code".into(),
            "codex".into()
        ]))
    );
}

#[test]
fn init_registers_the_tool_session_or_a_new_one() {
    let mut log = Memory::default();
    // ツールの ID があれば、それを init として登録する
    let r = register_session(&Fixed(Some(session("c5c5"))), "/w", &mut log).unwrap();
    assert_eq!((r.id.as_str(), r.via.as_str()), ("c5c5", "init"));
    // ID が無ければ、置き場所が決めた ID を登録する（前の登録は使わない）
    let h = register_session(&Fixed(None), "/w", &mut log).unwrap();
    assert_eq!((h.id.as_str(), h.tool.as_str()), ("sas-2", "unknown"));
    assert_eq!(log.initialized("/w").unwrap(), Some(h.clone()));
    // 登録したあとは、名乗らない呼び出しがこのフォルダの登録になる
    assert_eq!(
        identify(&Fixed(None), "/w", &log).unwrap(),
        Caller::Session {
            id: h.id,
            tool: h.tool,
            folder: "/w".into(),
            via: Via::Init,
        }
    );
}
