//! インスタンスを読む ・ 書く

use serde_json::Value;

/// インスタンスを読む ・ 書く：インスタンスの文書を、置き場所（作業の根からのパス）で読み書きする
pub trait Instances {
    /// 読む。無ければ None
    ///
    /// # Errors
    /// 読めないか、JSON でないとき
    fn read(&self, path: &str) -> Result<Option<Value>, String>;

    /// 書く。途中で止まっても前の中身を壊さない
    ///
    /// # Errors
    /// 書けないとき
    fn write(&self, path: &str, document: &Value) -> Result<(), String>;
}
