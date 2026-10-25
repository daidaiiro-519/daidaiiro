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
}
