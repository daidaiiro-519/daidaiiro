//! 変更の記録を読む ・ 書く

use crate::ports::sessions::SessionRef;
use semantic_agent_schema_change::aggregates::change_record::ChangeRecord;
use semantic_agent_schema_change::values::change_id::ChangeId;
use semantic_agent_schema_declaration::values::target::Target;

/// 変更の記録を読む ・ 書く：作業した人の手元の DB に置く。確定しない（コミットしない）
pub trait ChangeRecords {
    /// 1件足す。セッションが渡されたら、手元のセッションの索引にも結ぶ
    ///
    /// # Errors
    /// 書けないか、同じ id があるとき
    fn add(&self, record: &ChangeRecord, session: Option<&SessionRef>) -> Result<(), String>;

    /// id で引く
    ///
    /// # Errors
    /// 読めないとき
    fn get(&self, id: &ChangeId) -> Result<Option<ChangeRecord>, String>;

    /// その対象を変えた記録を、古い順に返す
    ///
    /// # Errors
    /// 読めないとき
    fn touching(&self, target: &Target) -> Result<Vec<ChangeRecord>, String>;
}
