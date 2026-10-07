//! 入ってくる側のポート。CLI と MCP の受け口は、この trait だけを呼ぶ。

use crate::application::checks::{Approved as ApprovedOutcome, Checked, Deleted, SchemasChecked};
use crate::application::instances::{Created, Got, Prompted, Updated, UseCaseError};
use crate::application::renders::Rendered;

/// インスタンスの読み書きのユースケース（UC-1 ・ 2 ・ 3 ・ 7 と、集約の削除のコマンド）。
pub trait InstanceUseCases {
    /// UC-1 インスタンスを作成する
    fn create(&self, schema: &str, path: &str) -> Result<Created, UseCaseError>;
    /// UC-2 値を取得する
    fn get(&self, path: &str, query: &str) -> Result<Got, UseCaseError>;
    /// UC-3 インスタンスを更新する。`hash` は呼ぶ側が読んだ時点のハッシュ値（無ければ、このユースケースが読んだ時点）
    fn update(&self, path: &str, patch: &str, hash: Option<&str>) -> Result<Updated, UseCaseError>;
    /// UC-7 x-prompt を受け取る
    fn prompt(&self, schema: &str, property: &str) -> Result<Prompted, UseCaseError>;
}

/// 検査と承認のユースケース（UC-4 ・ 5 ・ 8）。
pub trait CheckUseCases {
    /// UC-5 ディレクトリのインスタンスを検査する
    fn check(&self, dir: &str) -> Result<Checked, UseCaseError>;
    /// ディレクトリの具体のスキーマ（*.schema.json）を、メタスキーマと注釈の仕様で検査する（ボード schema-driven-build の論点6 D）
    fn check_schemas(&self, dir: &str) -> Result<SchemasChecked, UseCaseError>;
    /// UC-8 承認を記録する
    fn approve(&self, dir: &str) -> Result<ApprovedOutcome, UseCaseError>;
    /// UC-4 インスタンスを削除する（消したインスタンスを、まだ指している参照を返す）
    fn delete(&self, path: &str) -> Result<Deleted, UseCaseError>;
}

/// 描画のユースケース（UC-6）。具体のデザインを持つものだけが実装する。
pub trait RenderUseCases {
    /// UC-6 ページを描画する。dir のインスタンスを、pages のページテンプレートで、out へ描画する
    fn render(&self, dir: &str, pages: &str, out: &str) -> Result<Rendered, UseCaseError>;
}
