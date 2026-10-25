//! 判定の記録を読む ・ 書くの代役：メモリに置く

use semantic_agent_schema_application::ports::judgments::Judgments;
use semantic_agent_schema_application::semantic_agent_schema_change::aggregates::judgment::IntentJudgment;
use semantic_agent_schema_application::semantic_agent_schema_change::values::change_id::ChangeId;
use std::cell::RefCell;

/// 意図の判定をメモリに置く（記録1件に最後の1件）
#[derive(Default)]
pub struct FakeJudgments {
    pub intents: RefCell<Vec<IntentJudgment>>,
}

impl Judgments for FakeJudgments {
    fn put_intent(&self, judgment: &IntentJudgment) -> Result<(), String> {
        let mut v = self.intents.borrow_mut();
        v.retain(|j| j.change_id() != judgment.change_id());
        v.push(judgment.clone());
        Ok(())
    }

    fn intent_of(&self, change_id: &ChangeId) -> Result<Option<IntentJudgment>, String> {
        Ok(self
            .intents
            .borrow()
            .iter()
            .find(|j| j.change_id() == change_id)
            .cloned())
    }
}
