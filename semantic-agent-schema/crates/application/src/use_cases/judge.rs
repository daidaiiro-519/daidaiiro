//! judge（人の判定だけ）

use crate::use_cases::{Outcome, Ports};
use semantic_agent_schema_change::aggregates::judgment::IntentJudgment;
use semantic_agent_schema_change::values::change_id::ChangeId;
use semantic_agent_schema_change::values::verdict::Verdict;
use semantic_agent_schema_declaration::values::by::By;
use serde_json::json;

/// 人が、変更の記録の意図と変更が合うかを判定する（承認は verified）。前の判定は置き換わる。不変条件 I33 ・ 使う場面 U16
pub fn judge_intent(
    ports: &Ports<'_>,
    change_id: &str,
    result: &str,
    meaning: &str,
    reason: &str,
) -> Outcome {
    let id = match ChangeId::parse(change_id) {
        Ok(id) => id,
        Err(e) => return Outcome::refused("", &e),
    };
    match ports.records.get(&id) {
        Ok(Some(_)) => {}
        Ok(None) => return Outcome::refused("", &format!("変更の記録 {change_id} が無い")),
        Err(e) => return Outcome::refused("", &e),
    }
    let verdict = match result {
        "verified" => Verdict::Verified,
        "violated" => Verdict::Violated,
        "unverified" => Verdict::Unverified,
        other => {
            return Outcome::refused(
                "",
                &format!("結果「{other}」は verified ・ violated ・ unverified のどれか"),
            );
        }
    };
    let judgment = match IntentJudgment::new(id, By::Human, verdict, meaning, reason) {
        Ok(j) => j,
        Err(e) => return Outcome::refused("", &e),
    };
    if let Err(e) = ports.judgments.put_intent(&judgment) {
        return Outcome::refused("", &e);
    }
    Outcome::checked(
        &[],
        json!({"change_id": change_id, "result": result, "by": "human"}),
    )
}
