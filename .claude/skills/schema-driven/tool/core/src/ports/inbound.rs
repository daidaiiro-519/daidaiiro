//! 入ってくる側のポート。CLI と MCP の受け口は、この trait だけを呼ぶ。

use crate::application::checks::{Approved as ApprovedOutcome, Checked, Deleted, SchemasChecked};
use crate::application::instances::{Created, Got, Prompted, Updated, UseCaseError};
use crate::application::renders::Rendered;

/// インスタンスの読み書きのユースケースと、削除のコマンド。
pub trait InstanceUseCases {
    /// インスタンスを作成する
    fn create(&self, schema: &str, path: &str) -> Result<Created, UseCaseError>;
    /// 値を取得する
    fn get(&self, path: &str, query: &str) -> Result<Got, UseCaseError>;
    /// インスタンスを更新する。`hash` は呼ぶ側が読んだ時点のハッシュ値（無ければ、このユースケースが読んだ時点）
    fn update(&self, path: &str, patch: &str, hash: Option<&str>) -> Result<Updated, UseCaseError>;
    /// x-prompt を受け取る
    fn prompt(&self, schema: &str, property: &str) -> Result<Prompted, UseCaseError>;
}

/// 検査と承認のユースケース。
pub trait CheckUseCases {
    /// ディレクトリのインスタンスを検査する
    fn check(&self, dir: &str) -> Result<Checked, UseCaseError>;
    /// ディレクトリの具体のスキーマ（*.schema.json）を、メタスキーマと注釈の仕様で検査する
    fn check_schemas(&self, dir: &str) -> Result<SchemasChecked, UseCaseError>;
    /// 承認を記録する
    fn approve(&self, dir: &str) -> Result<ApprovedOutcome, UseCaseError>;
    /// インスタンスを削除する（消したインスタンスを、まだ指している参照を返す）
    fn delete(&self, path: &str) -> Result<Deleted, UseCaseError>;
}

/// 描画のユースケース。
pub trait RenderUseCases {
    /// ページを描画する。dir のインスタンスを、pages のページテンプレートで、out へ描画する
    fn render(&self, dir: &str, pages: &str, out: &str) -> Result<Rendered, UseCaseError>;
}
