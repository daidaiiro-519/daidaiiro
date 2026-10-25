//! 変更のまとめを読む ・ 書く

use serde_json::Value;

/// 変更のまとめを読む ・ 書く：文脈のリポジトリにファイルとして置き、確定する（コミットする）
pub trait ChangeSummaries {
    /// まとめを置き、置いた場所を返す。同じ id があれば断る
    ///
    /// # Errors
    /// 書けないか、同じ id のまとめがあるとき
    fn put(&self, id: &str, summary: &Value) -> Result<String, String>;
}
