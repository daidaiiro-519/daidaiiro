//! 出ていく側のポート。core はファイルにも JMESPath の実装にも直接触れず、この trait を通す。

use crate::domain::values::Hash;

/// ファイルを読めなかった理由（失敗の種類「読めない」）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadError(pub String);

/// ファイルを書けなかった理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteError {
    /// 書く直前の内容のハッシュ値が、期待した値と違う（拒否の理由「ほかの更新と競合した」）
    Conflict,
    /// 失敗の種類「書けない」
    Unwritable(String),
}

/// 書く条件。作成は「まだ無いこと」、更新は「読んだ時点のハッシュ値のままであること」を求める。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteIf {
    Absent,
    Unchanged(Hash),
}

/// ファイルシステム（支援アクター）。
pub trait Files {
    fn exists(&self, path: &str) -> bool;
    fn read(&self, path: &str) -> Result<String, ReadError>;
    /// ディレクトリの直下のファイルのパスを、名前の順に返す。
    fn list(&self, dir: &str) -> Result<Vec<String>, ReadError>;
    fn write(&self, path: &str, content: &str, cond: WriteIf) -> Result<(), WriteError>;
    fn remove(&self, path: &str) -> Result<(), WriteError>;
}

/// スキーマの供給元。スキーマが `$ref` で指す先のスキーマを、ファイル名と内容の組で返す。
/// 基盤のアダプタは同じディレクトリから読む。外のスキーマを使いたい Skill は、自分のアダプタで取得して渡す
/// （基盤はネットワークに出ない。ACDR 0122）。
pub trait Schemas {
    fn referenced(&self, schema_path: &str) -> Result<Vec<(String, String)>, ReadError>;
}

/// JMESPath 式が読めない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryError(pub String);

/// 取得（JMESPath 式で値を取り出す）。差し替えられるように trait にする（ACDR 0122）。
pub trait Query {
    /// 式を JSON の値に当て、結果を JSON の文字列で返す。何も指さなければ `null` を返す。
    fn search(
        &self,
        expression: &str,
        json: &serde_json::Value,
    ) -> Result<serde_json::Value, QueryError>;
}
