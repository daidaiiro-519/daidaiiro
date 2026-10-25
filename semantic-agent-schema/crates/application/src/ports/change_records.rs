//! 変更の記録を読む ・ 書く

use crate::ports::sessions::SessionRef;
use semantic_agent_schema_change::aggregates::change_record::ChangeRecord;
use semantic_agent_schema_change::values::change_id::ChangeId;
use semantic_agent_schema_declaration::values::target::Target;

/// 変更の記録を読む ・ 書く：作業した人の手元の DB に置く。確定しない（コミットしない）
pub trait ChangeRecords {
    /// 1件足す。書いた作業場所の名札を一緒に置き（手元の DB の中だけで使う）、セッションが渡されたら、手元のセッションの索引にも結ぶ
    ///
    /// # Errors
    /// 書けないか、同じ id があるとき
    fn add(
        &self,
        record: &ChangeRecord,
        workspace: Option<&str>,
        session: Option<&SessionRef>,
    ) -> Result<(), String>;

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

    /// その作業場所で書いて、まだまとめに入っていない記録を、古い順に返す
    ///
    /// # Errors
    /// 読めないとき
    fn pending(&self, workspace: Option<&str>) -> Result<Vec<ChangeRecord>, String>;

    /// 記録にまとめの id を付ける（1件の記録は1つのまとめにだけ入る。不変条件 I35）
    ///
    /// # Errors
    /// 書けないか、すでに別のまとめに入っている記録があるとき
    fn summarized(&self, ids: &[ChangeId], summary_id: &str) -> Result<(), String>;
}
