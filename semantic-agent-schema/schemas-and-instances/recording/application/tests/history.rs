//! 追記の手続きを、置き場所の代役で確かめる。

#![allow(clippy::unwrap_used)]

use recording_application::ports::{
    Caller, ChangeRow, IdentifyCaller, Judgment, OperationLog, SessionRow, StoredChange,
    ValidateRow, Via, WriteRecord,
};
use recording_application::use_cases::history::{
    interleaved, last_written, record_change, record_validations, register_session, untracked_since,
};

struct Fixed(Caller);
impl IdentifyCaller for Fixed {
    fn identify(&self) -> Caller {
        self.0.clone()
    }
}

#[derive(Default)]
struct Memory {
    sessions: Vec<SessionRow>,
    ops: Vec<(Option<String>, ValidateRow)>,
    changes: Vec<(Option<String>, ChangeRow)>,
}
impl OperationLog for Memory {
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
    fn append_validate(
        &mut self,
        session: Option<&str>,
        row: &ValidateRow,
    ) -> Result<String, String> {
        self.ops.push((session.map(Into::into), row.clone()));
        Ok(format!("o-{}", self.ops.len() + self.changes.len()))
    }
    fn append_change(&mut self, session: Option<&str>, row: &ChangeRow) -> Result<String, String> {
        self.changes.push((session.map(Into::into), row.clone()));
        Ok(format!("o-{}", self.ops.len() + self.changes.len()))
    }
    fn find_request(&self, _: &str) -> Result<Option<StoredChange>, String> {
        Ok(None)
    }
    fn writes(&self, target_path: &str) -> Result<Vec<WriteRecord>, String> {
        Ok(self
            .changes
            .iter()
            .enumerate()
            .filter(|(_, (_, r))| r.target_path == target_path)
            .map(|(i, (session, r))| WriteRecord {
                operation: format!("o-{}", i + 1),
                session: session.clone(),
                at: "t".into(),
                row: r.clone(),
            })
            .collect())
    }
    fn last_judgments(&self, _: &str) -> Result<Vec<Judgment>, String> {
        Ok(Vec::new())
    }
}

fn row(hash: &str) -> ValidateRow {
    ValidateRow {
        target_path: "a.json".into(),
        hash: hash.into(),
        results: Vec::new(),
    }
}

fn session(id: &str) -> Fixed {
    Fixed(Caller::Session {
        id: id.into(),
        tool: "claude-code".into(),
        folder: "/w".into(),
        via: Via::Env,
    })
}

#[test]
fn session_is_recorded_once_and_a_human_has_none() {
    let mut log = Memory::default();
    let a = record_validations(&session("p1"), &mut log, &[row("h")]).unwrap();
    let b = record_validations(&Fixed(Caller::Human), &mut log, &[row("h")]).unwrap();
    let c = record_validations(&session("p1"), &mut log, &[row("h"), row("h")]).unwrap();
    assert_eq!(a.session.as_deref(), Some("p1"));
    assert_eq!(b.session, None);
    assert_eq!(c.session, a.session);
    let ids: Vec<&str> = log.sessions.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, ["p1"]);
    assert_eq!(c.operations, ["o-3", "o-4"]);
}

fn change(before: Option<&str>, after: Option<&str>) -> ChangeRow {
    ChangeRow {
        kind: "update".into(),
        source: "sas".into(),
        target_path: "a.json".into(),
        request_id: None,
        before: before.map(Into::into),
        after: after.map(Into::into),
        intent: None,
        fields: Vec::new(),
    }
}

#[test]
fn interleaved_follows_hashs_from_expected() {
    let mut log = Memory::default();
    let human = Fixed(Caller::Human);
    assert_eq!(last_written(&log, "a.json").unwrap(), None);
    record_change(&human, &mut log, &change(Some("a"), Some("b"))).unwrap();
    record_change(&human, &mut log, &change(Some("b"), Some("c"))).unwrap();
    assert_eq!(last_written(&log, "a.json").unwrap().as_deref(), Some("c"));

    // 前提の b のあとに、o-2（b → c）が入った
    let i = interleaved(&log, "a.json", "b", "c").unwrap();
    let ids: Vec<&str> = i.operations.iter().map(|w| w.operation.as_str()).collect();
    assert_eq!(ids, ["o-2"]);
    assert!(!i.untracked);

    // 記録の最後（c）のあとに、記録に無い変更で d になった
    let i = interleaved(&log, "a.json", "c", "d").unwrap();
    assert!(i.operations.is_empty());
    assert!(i.untracked);
    assert_eq!(i.last_recorded.map(|w| w.operation).as_deref(), Some("o-2"));

    // 前提のハッシュが記録に無いときは、記録した操作をすべて返す
    let i = interleaved(&log, "a.json", "z", "c").unwrap();
    assert_eq!(i.operations.len(), 2);
    assert!(!i.untracked);
}

#[test]
fn any_recorded_after_is_tracked() {
    let mut log = Memory::default();
    let human = Fixed(Caller::Human);
    assert_eq!(untracked_since(&log, "a.json", "x").unwrap(), None);
    record_change(&human, &mut log, &change(Some("a"), Some("b"))).unwrap();
    record_change(&human, &mut log, &change(Some("b"), Some("c"))).unwrap();
    // ほかのワークツリーが c まで書いても、このワークツリーの b は SAS を通った中身
    assert_eq!(untracked_since(&log, "a.json", "b").unwrap(), None);
    assert_eq!(untracked_since(&log, "a.json", "c").unwrap(), None);
    // 記録のどの事後でもない中身は、最後の事後と比べた外の変更
    assert_eq!(
        untracked_since(&log, "a.json", "d").unwrap().as_deref(),
        Some("c")
    );
}

#[test]
fn init_registers_the_tool_session_or_a_new_one() {
    let mut log = Memory::default();
    // ツールの ID があれば、それを init として登録する
    let r = register_session(&session("c5c5"), "/w", &mut log).unwrap();
    assert_eq!((r.id.as_str(), r.via.as_str()), ("c5c5", "init"));
    // ID が無ければ、置き場所が決めた ID を登録する
    let h = register_session(&Fixed(Caller::Human), "/w", &mut log).unwrap();
    assert_eq!((h.id.as_str(), h.tool.as_str()), ("sas-2", "unknown"));
    assert_eq!(log.initialized("/w").unwrap(), Some(h));
}
