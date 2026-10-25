//! 変更の記録を読む ・ 書くの代役：メモリに並べる

use semantic_agent_schema_application::ports::change_records::ChangeRecords;
use semantic_agent_schema_application::ports::sessions::SessionRef;
use semantic_agent_schema_application::semantic_agent_schema_change::aggregates::change_record::ChangeRecord;
use semantic_agent_schema_application::semantic_agent_schema_change::values::change_id::ChangeId;
use semantic_agent_schema_application::semantic_agent_schema_declaration::values::target::Target;
use std::cell::RefCell;

/// 変更の記録をメモリに並べる
#[derive(Default)]
pub struct FakeChangeRecords {
    pub records: RefCell<Vec<ChangeRecord>>,
    pub summarized: RefCell<Vec<(ChangeId, String)>>,
}

impl ChangeRecords for FakeChangeRecords {
    fn add(&self, record: &ChangeRecord, _session: Option<&SessionRef>) -> Result<(), String> {
        self.records.borrow_mut().push(record.clone());
        Ok(())
    }

    fn get(&self, id: &ChangeId) -> Result<Option<ChangeRecord>, String> {
        Ok(self.records.borrow().iter().find(|r| r.id() == id).cloned())
    }

    fn touching(&self, target: &Target) -> Result<Vec<ChangeRecord>, String> {
        Ok(self
            .records
            .borrow()
            .iter()
            .filter(|r| r.items().iter().any(|i| i.target() == target))
            .cloned()
            .collect())
    }

    fn pending(&self) -> Result<Vec<ChangeRecord>, String> {
        let done = self.summarized.borrow();
        Ok(self
            .records
            .borrow()
            .iter()
            .filter(|r| !done.iter().any(|(id, _)| id == r.id()))
            .cloned()
            .collect())
    }

    fn summarized(&self, ids: &[ChangeId], summary_id: &str) -> Result<(), String> {
        let mut done = self.summarized.borrow_mut();
        if ids.iter().any(|id| done.iter().any(|(d, _)| d == id)) {
            return Err("すでに別のまとめに入っている".into());
        }
        done.extend(ids.iter().map(|id| (id.clone(), summary_id.to_string())));
        Ok(())
    }
}
