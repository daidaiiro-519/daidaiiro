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
}
