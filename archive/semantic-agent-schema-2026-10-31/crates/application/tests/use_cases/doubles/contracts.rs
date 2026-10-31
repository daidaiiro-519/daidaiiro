//! 契約を読む ・ 書くの代役：メモリの上に置く

use semantic_agent_schema_application::ports::contracts::Contracts;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// 置き場所 → 文書をメモリに持つ。find_error を入れると $id で引くときにその誤りを返す
#[derive(Default)]
pub struct FakeContracts {
    pub files: RefCell<BTreeMap<String, Value>>,
    pub find_error: Option<String>,
}

impl Contracts for FakeContracts {
    fn read(&self, path: &str) -> Result<Option<Value>, String> {
        Ok(self.files.borrow().get(path).cloned())
    }

    fn find(&self, id: &str) -> Result<Option<Value>, String> {
        if let Some(e) = &self.find_error {
            return Err(e.clone());
        }
        Ok(self
            .files
            .borrow()
            .values()
            .find(|d| d.get("$id").and_then(Value::as_str) == Some(id))
            .cloned())
    }

    fn write(&self, path: &str, document: &Value) -> Result<(), String> {
        self.files
            .borrow_mut()
            .insert(path.to_string(), document.clone());
        Ok(())
    }
}
