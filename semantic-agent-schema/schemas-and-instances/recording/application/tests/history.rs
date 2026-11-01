//! 追記の手続きを、置き場所の代役で確かめる。

#![allow(clippy::unwrap_used)]

use recording_application::ports::{
    Caller, ChangeRow, IdentifyCaller, Judgment, OperationLog, StoredChange, ValidateRow,
    WriteRecord,
};
use recording_application::use_cases::history::{
    interleaved, last_written, record_change, record_validations, untracked_since,
};

struct Fixed(Caller);
impl IdentifyCaller for Fixed {
    fn identify(&self) -> Caller {
        self.0.clone()
    }
}

#[derive(Default)]
struct Memory {
    parents: Vec<String>,
    ops: Vec<(Option<String>, ValidateRow)>,
    changes: Vec<(Option<String>, ChangeRow)>,
}
impl OperationLog for Memory {
    fn parent_session(&mut self, caller: &Caller) -> Result<Option<String>, String> {
        let Caller::Parent { id, .. } = caller else {
            return Ok(None);
        };
        if !self.parents.contains(id) {
            self.parents.push(id.clone());
        }
        Ok(Some(id.clone()))
    }
    fn append_validate(
        &mut self,
        parent: Option<&str>,
        row: &ValidateRow,
    ) -> Result<String, String> {
        self.ops.push((parent.map(Into::into), row.clone()));
        Ok(format!("o-{}", self.ops.len() + self.changes.len()))
    }
    fn append_change(&mut self, parent: Option<&str>, row: &ChangeRow) -> Result<String, String> {
        self.changes.push((parent.map(Into::into), row.clone()));
        Ok(format!("o-{}", self.ops.len() + self.changes.len()))
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
            .filter(|(_, (_, r))| r.target_path == target_path)
            .map(|(i, (parent, r))| WriteRecord {
                operation: format!("o-{}", i + 1),
                parent: parent.clone(),
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

fn parent(id: &str) -> Fixed {
    Fixed(Caller::Parent {
        id: id.into(),
        tool: "claude-code".into(),
    })
}

#[test]
fn parent_is_recorded_once_and_a_human_has_none() {
    let mut log = Memory::default();
    let a = record_validations(&parent("p1"), &mut log, &[row("h")]).unwrap();
    let b = record_validations(&Fixed(Caller::Human), &mut log, &[row("h")]).unwrap();
    let c = record_validations(&parent("p1"), &mut log, &[row("h"), row("h")]).unwrap();
    assert_eq!(a.parent.as_deref(), Some("p1"));
    assert_eq!(b.parent, None);
    assert_eq!(c.parent, a.parent);
    assert_eq!(log.parents, ["p1"]);
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
        cause: None,
        fields: Vec::new(),
    }
}

#[test]
fn interleaved_follows_fingerprints_from_expected() {
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

    // 前提の指紋が記録に無いときは、記録した操作をすべて返す
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
