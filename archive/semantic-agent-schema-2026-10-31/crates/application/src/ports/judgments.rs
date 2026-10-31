//! 判定の記録を読む ・ 書く

use semantic_agent_schema_change::aggregates::judgment::IntentJudgment;
use semantic_agent_schema_change::values::change_id::ChangeId;

/// 判定の記録を読む ・ 書く。意図の判定は作業した人の手元の DB に、変更の記録1件につき最後の1件を置く
/// （rule の判定のファイルは、AI と外のツールの判定を入れるステップ4 で足す）
pub trait Judgments {
    /// 意図の判定を置く（前の判定は置き換わる）
    ///
    /// # Errors
    /// 書けないとき
    fn put_intent(&self, judgment: &IntentJudgment) -> Result<(), String>;

    /// 変更の記録の意図の判定
    ///
    /// # Errors
    /// 読めないとき
    fn intent_of(&self, change_id: &ChangeId) -> Result<Option<IntentJudgment>, String>;
}

/// rule の判定の記録を読む ・ 書く：文脈のリポジトリのファイル（judgment.schema.json の形。対象 × rule で1件、上書きで最後の1件）
pub trait RuleJudgments {
    /// 対象 × rule の判定の記録を置く。key はファイルの名前にもなる識別（<インスタンスの id>/<rule の id>）
    ///
    /// # Errors
    /// 書けないとき
    fn put_rule(&self, key: &str, record: &serde_json::Value) -> Result<(), String>;

    /// 対象 × rule の判定の記録。無ければ None
    ///
    /// # Errors
    /// 読めないとき
    fn rule_of(&self, key: &str) -> Result<Option<serde_json::Value>, String>;
}
