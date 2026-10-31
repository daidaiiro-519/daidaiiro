//! 契約を読む ・ 書く

use serde_json::Value;

/// 契約を読む ・ 書く：Concrete の文書を、置き場所（作業の根からのパス）で読み書きする
pub trait Contracts {
    /// 読む。無ければ None
    ///
    /// # Errors
    /// 読めないか、JSON でないとき
    fn read(&self, path: &str) -> Result<Option<Value>, String>;

    /// $id で引く。無ければ None
    ///
    /// # Errors
    /// 読めないか、同じ $id の文書が2つ以上あるとき
    fn find(&self, id: &str) -> Result<Option<Value>, String>;

    /// 書く。途中で止まっても前の中身を壊さない
    ///
    /// # Errors
    /// 書けないとき
    fn write(&self, path: &str, document: &Value) -> Result<(), String>;
}
