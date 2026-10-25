//! インスタンスを読む ・ 書くの代役：契約の代役と同じメモリの置き場所を使う（本物も同じファイルの置き場所を使う）

use crate::doubles::contracts::FakeContracts;
use semantic_agent_schema_application::ports::contracts::Contracts;
use semantic_agent_schema_application::ports::instances::Instances;
use serde_json::Value;

impl Instances for FakeContracts {
    fn read(&self, path: &str) -> Result<Option<Value>, String> {
        Contracts::read(self, path)
    }

    fn write(&self, path: &str, document: &Value) -> Result<(), String> {
        Contracts::write(self, path, document)
    }

    fn all(&self) -> Result<Vec<String>, String> {
        Ok(self
            .files
            .borrow()
            .iter()
            .filter(|(_, d)| {
                d.get("$schema")
                    .and_then(Value::as_str)
                    .is_some_and(|s| s.starts_with("urn:semantic-agent-schema:"))
            })
            .map(|(p, _)| p.clone())
            .collect())
    }
}
