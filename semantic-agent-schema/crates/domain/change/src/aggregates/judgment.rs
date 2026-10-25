//! 判定の記録

use crate::values::change_id::ChangeId;
use crate::values::verdict::Verdict;
use semantic_agent_schema_declaration::values::by::By;

/// 意図の判定：変更の記録の意図と変更が合うか（不変条件 I33）と、人の承認（U16）。変更の記録1件に最後の1件
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentJudgment {
    change_id: ChangeId,
    by: By,
    result: Verdict,
    meaning: String,
    reason: String,
}

impl IntentJudgment {
    /// 作る。結果は verified ・ violated ・ unverified、判定した者は AI か人
    ///
    /// # Errors
    /// 結果がほかの値、判定した者が機械、意味か理由が空のとき
    pub fn new(
        change_id: ChangeId,
        by: By,
        result: Verdict,
        meaning: &str,
        reason: &str,
    ) -> Result<Self, String> {
        if !matches!(
            result,
            Verdict::Verified | Verdict::Violated | Verdict::Unverified
        ) {
            return Err(
                "意図の判定の結果は verified ・ violated ・ unverified のどれか".to_string(),
            );
        }
        if by == By::Machine {
            return Err("意図の判定をするのは AI か人".to_string());
        }
        if meaning.trim().is_empty() || reason.trim().is_empty() {
            return Err("意味（--meaning）と理由（--reason）を書く".to_string());
        }
        Ok(IntentJudgment {
            change_id,
            by,
            result,
            meaning: meaning.to_string(),
            reason: reason.to_string(),
        })
    }

    /// 判定した変更の記録
    pub fn change_id(&self) -> &ChangeId {
        &self.change_id
    }

    /// 判定した者
    pub fn by(&self) -> By {
        self.by
    }

    /// 結果
    pub fn result(&self) -> Verdict {
        self.result
    }

    /// 意味
    pub fn meaning(&self) -> &str {
        &self.meaning
    }

    /// 理由
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// 人が承認したか
    pub fn approved_by_human(&self) -> bool {
        self.by == By::Human && self.result == Verdict::Verified
    }
}

#[cfg(test)]
mod tests {
    use super::IntentJudgment;
    use crate::values::change_id::ChangeId;
    use crate::values::verdict::Verdict;
    use semantic_agent_schema_declaration::values::by::By;

    /// traces: U16:事後
    #[test]
    fn human_verified_is_an_approval() {
        let j = IntentJudgment::new(
            ChangeId::from_parts(1, 1),
            By::Human,
            Verdict::Verified,
            "意味",
            "理由",
        )
        .unwrap();
        assert!(j.approved_by_human());
    }

    /// traces: U16:失敗時
    #[test]
    fn machine_cannot_judge_an_intent() {
        assert!(
            IntentJudgment::new(
                ChangeId::from_parts(1, 1),
                By::Machine,
                Verdict::Verified,
                "意味",
                "理由"
            )
            .is_err()
        );
    }
}
