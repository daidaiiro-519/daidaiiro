//! 追記の手続きを、置き場所の代役で確かめる。

#![allow(clippy::unwrap_used)]

use contracts_and_instances_operation_recording_application::ports::{
    Caller, IdentifyCaller, OperationLog, ValidateRow,
};
use contracts_and_instances_operation_recording_application::use_cases::history::record_validations;

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
