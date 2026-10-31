//! 追記の手続きを、置き場所の代役で確かめる。

#![allow(clippy::unwrap_used)]

use schemas_and_instances_operation_recording_application::ports::{
    Caller, ChangeRow, IdentifyCaller, Judgment, OperationLog, StoredChange, ValidateRow,
    WriteRecord,
};
use schemas_and_instances_operation_recording_application::use_cases::history::{
    interleaved, last_written, record_change, record_validations,
};

struct Fixed(Caller);
impl IdentifyCaller for Fixed {
    fn identify(&self) -> Caller {
        self.0.clone()
    }
}

#[derive(Default)]
struct Memory {
    sessions: Vec<Caller>,
    ops: Vec<(String, ValidateRow)>,
    changes: Vec<(String, ChangeRow)>,
}
impl OperationLog for Memory {
    fn caller_session(&mut self, caller: &Caller) -> Result<String, String> {
        let at = match self.sessions.iter().position(|c| c == caller) {
            Some(i) => i,
            None => {
                self.sessions.push(caller.clone());
                self.sessions.len() - 1
            }
        };
        Ok(format!("s-{}", at + 1))
    }
    fn append_validate(&mut self, session: &str, row: &ValidateRow) -> Result<String, String> {
        self.ops.push((session.to_string(), row.clone()));
        Ok(format!("o-{}", self.ops.len()))
    }
    fn append_change(&mut self, session: &str, row: &ChangeRow) -> Result<String, String> {
        self.changes.push((session.to_string(), row.clone()));
        Ok(format!("o-{}", self.changes.len()))
    }
    fn find_request(&self, _: &str) -> Result<Option<StoredChange>, String> {
        Ok(None)
    }
    fn caused_by(&self, _: &str) -> Result<Vec<StoredChange>, String> {
        Ok(Vec::new())
    }
    fn writes(&self, target_path: &str) -> Result<Vec<WriteRecord>, String> {
        Ok(self
            .changes
            .iter()
            .enumerate()
            .filter(|(_, (_, r))| r.target_path == target_path && r.refusal.is_none())
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

fn row(before: &str, after: &str) -> ValidateRow {
    ValidateRow {
        target_kind: "concrete".into(),
        target_path: "a.json".into(),
        before: before.into(),
        after: after.into(),
        results: Vec::new(),
    }
}

#[test]
fn same_caller_reuses_session() {
    let mut log = Memory::default();
    let tool = Fixed(Caller::Tool {
        tool_ref: "t".into(),
    });
    let a = record_validations(&tool, &mut log, &[row("h", "h")]).unwrap();
    let b = record_validations(&Fixed(Caller::Human), &mut log, &[row("h", "h")]).unwrap();
    let c = record_validations(&tool, &mut log, &[row("h", "h"), row("h", "h")]).unwrap();
    assert_eq!(a.session, c.session);
    assert_ne!(a.session, b.session);
    assert_eq!(c.operations, ["o-3", "o-4"]);
}

#[test]
fn changed_fingerprint_is_refused() {
    let mut log = Memory::default();
    assert!(record_validations(&Fixed(Caller::Human), &mut log, &[row("a", "b")]).is_err());
    assert!(log.ops.is_empty());
}

fn change(before: Option<&str>, after: Option<&str>, refused: bool) -> ChangeRow {
    ChangeRow {
        kind: "update".into(),
        source: "sas".into(),
        target_kind: "instance".into(),
        target_path: "a.json".into(),
        concrete: "urn:x:v1".into(),
        request_id: None,
        before: before.map(Into::into),
        after: after.map(Into::into),
        refusal: refused.then(|| ("conflict".into(), "指紋が違う".into())),
        intent: None,
        cause: None,
        fields: Vec::new(),
    }
}

#[test]
fn refused_change_keeps_target() {
    let mut log = Memory::default();
    let human = Fixed(Caller::Human);
    assert!(record_change(&human, &mut log, &change(Some("a"), Some("b"), true)).is_err());
    assert!(log.changes.is_empty());
    let r = record_change(&human, &mut log, &change(Some("a"), Some("a"), true)).unwrap();
    assert_eq!(r.operations, ["o-1"]);
    let w = record_change(&human, &mut log, &change(Some("a"), Some("b"), false)).unwrap();
    assert_eq!(w.session, r.session);
}

#[test]
fn interleaved_follows_fingerprints_from_expected() {
    let mut log = Memory::default();
    let human = Fixed(Caller::Human);
    assert_eq!(last_written(&log, "a.json").unwrap(), None);
    record_change(&human, &mut log, &change(Some("a"), Some("b"), false)).unwrap();
    record_change(&human, &mut log, &change(Some("b"), Some("b"), true)).unwrap();
    record_change(&human, &mut log, &change(Some("b"), Some("c"), false)).unwrap();
    assert_eq!(last_written(&log, "a.json").unwrap().as_deref(), Some("c"));

    // 前提の b のあとに、o-3（b → c）が入った
    let i = interleaved(&log, "a.json", "b", "c").unwrap();
    let ids: Vec<&str> = i.operations.iter().map(|w| w.operation.as_str()).collect();
    assert_eq!(ids, ["o-3"]);
    assert!(!i.untracked);

    // 記録の最後（c）のあとに、記録に無い変更で d になった
    let i = interleaved(&log, "a.json", "c", "d").unwrap();
    assert!(i.operations.is_empty());
    assert!(i.untracked);
    assert_eq!(i.last_recorded.map(|w| w.operation).as_deref(), Some("o-3"));

    // 前提の指紋が記録に無いときは、記録した操作をすべて返す
    let i = interleaved(&log, "a.json", "z", "c").unwrap();
    assert_eq!(i.operations.len(), 2);
    assert!(!i.untracked);
}
