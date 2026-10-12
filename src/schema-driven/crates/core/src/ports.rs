//! 出ていく側のポート。application が定義し、adapters が実装する。
//! core はファイルにも JMESPath の実装にも部品のカタログにも直接触れず、この trait を通す。

use crate::domain::values::Hash;
use serde_json::Value;
use std::fmt;
use std::sync::Arc;

/// ファイルを読めなかった理由（失敗の種類「読めない」）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadError(pub String);

impl fmt::Display for ReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "読めない：{}", self.0)
    }
}

impl std::error::Error for ReadError {}

/// ファイルを書けなかった理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteError {
    /// 書く直前の内容のハッシュ値が、期待した値と違う（拒否の理由「ほかの更新と競合した」）
    Conflict,
    /// 失敗の種類「書けない」
    Unwritable(String),
}

impl fmt::Display for WriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteError::Conflict => formatter.write_str("ほかの更新と競合した"),
            WriteError::Unwritable(detail) => write!(formatter, "書けない：{detail}"),
        }
    }
}

impl std::error::Error for WriteError {}

/// 書く条件。作成は「まだ無いこと」、更新は「読んだ時点のハッシュ値のままであること」を求める。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteIf {
    Absent,
    Unchanged(Hash),
}

/// ジェネリクスで使う。ファイルシステム（支援アクター）。1つの実行ファイルの中で実装が1つに決まる。
pub trait Files: Clone + Send + Sync + 'static {
    fn exists(&self, path: &str) -> bool;
    fn read(&self, path: &str) -> Result<String, ReadError>;
    /// ディレクトリの直下のファイルのパスを、名前の順に返す。
    fn list(&self, dir: &str) -> Result<Vec<String>, ReadError>;
    /// ディレクトリの下のすべてのファイルのパス（下のディレクトリも含む）を、パスの順に返す。ディレクトリが無ければ空。
    fn list_tree(&self, dir: &str) -> Result<Vec<String>, ReadError>;
    fn write(&self, path: &str, content: &str, cond: WriteIf) -> Result<(), WriteError>;
    fn remove(&self, path: &str) -> Result<(), WriteError>;
}

/// ジェネリクスで使う。スキーマの供給元。スキーマが `$ref` で指す先のスキーマを、ファイル名と内容の組で返す。
/// 基盤のアダプタは同じディレクトリから読む（基盤はネットワークに出ない）。
pub trait Schemas: Clone + Send + Sync + 'static {
    fn referenced(&self, schema_path: &str) -> Result<Vec<(String, String)>, ReadError>;
}

/// JMESPath 式が読めない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryError(pub String);

impl fmt::Display for QueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "JMESPath 式を読めない：{}", self.0)
    }
}

impl std::error::Error for QueryError {}

/// ジェネリクスで使う。取得（JMESPath 式で値を取り出す）。1つの実行ファイルの中で実装が1つに決まる。
pub trait Query: Clone + Send + Sync + 'static {
    /// 式を JSON の値に当て、結果を JSON の値で返す。何も指さなければ `null` を返す。
    fn search(
        &self,
        expression: &str,
        json: &serde_json::Value,
    ) -> Result<serde_json::Value, QueryError>;
    /// 式が JMESPath として読めるかだけを見る（評価はしない）。
    fn parse(&self, expression: &str) -> Result<(), QueryError>;
    /// functions の関数（基盤の関数）を追加して、式を JSON の値で評価する。
    fn evaluate(
        &self,
        expression: &str,
        json: &serde_json::Value,
        functions: Arc<dyn Functions>,
    ) -> Result<serde_json::Value, QueryError>;
}

/// 部品のカタログの中身。concrete（Skill）が持つファイルの文字で、基盤は部品の名前も見た目も持たない。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Catalog {
    /// 部品の一覧のスキーマ（schemas/catalog.schema.json の枠に従う JSON）
    pub schema: String,
    /// 部品の HTML（<template id="…"> の並び。プレースホルダーは {{名前}}）
    pub parts: String,
    /// 部品の見た目（CSS）。無ければ空
    pub style: String,
    /// 部品の操作（JS）。無ければ空
    pub script: String,
    /// デザインシステム（schemas/design-system.schema.json に従う JSON）
    pub design_system: String,
}

/// dyn で使う。部品のカタログの供給元。外から実装を足していくので、型は実行時に決まる。
/// カタログの中身は concrete が持ち、基盤は描画のたびにここから受け取る。基盤の中にカタログの中身は無い。
pub trait Design: Send + Sync {
    /// カタログを読む。無いか読めなければ、その理由を返す。
    fn catalog(&self) -> Result<Catalog, ReadError>;
}

/// dyn で使う。core が実装し、port のメソッドの引数としてアダプタへ渡すコールバックで、port ではない。JMESPath に足す関数の名前と中身。基盤の6つは core が持つ。
/// Query のアダプタは、names() の名前を登録して呼び出しを渡すだけにする。
pub trait Functions: Send + Sync {
    fn names(&self) -> Vec<String>;
    fn call(&self, name: &str, args: &[Value]) -> Result<Value, String>;
}

#[cfg(test)]
mod tests {
    use super::{QueryError, ReadError, WriteError};
    use std::error::Error;

    // 基盤のエラー型は、Display と std::error::Error を実装する（Rust API Guidelines の C-GOOD-ERR）。
    // Display の文は短く、句点で終えない。具体は、基盤のエラーを `?` で自分のエラーへ渡せる。
    #[test]
    fn port_errors_describe_themselves_without_punctuation() {
        assert_eq!(
            Box::<dyn Error>::from(ReadError("a.json が無い".into())).to_string(),
            "読めない：a.json が無い"
        );
        assert_eq!(
            Box::<dyn Error>::from(WriteError::Conflict).to_string(),
            "ほかの更新と競合した"
        );
        assert_eq!(
            Box::<dyn Error>::from(WriteError::Unwritable("権限が無い".into())).to_string(),
            "書けない：権限が無い"
        );
        assert_eq!(
            Box::<dyn Error>::from(QueryError("[ が閉じていない".into())).to_string(),
            "JMESPath 式を読めない：[ が閉じていない"
        );
        let read = Box::<dyn Error>::from(ReadError("x".into())).to_string();
        assert!(!read.ends_with(['。', '.']), "{read}");
        let query = Box::<dyn Error>::from(QueryError("x".into())).to_string();
        assert!(!query.ends_with(['。', '.']), "{query}");
    }
}
