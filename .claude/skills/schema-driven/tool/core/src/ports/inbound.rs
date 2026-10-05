//! 入ってくる側のポート。CLI と MCP の受け口は、この trait だけを呼ぶ。

use crate::application::instances::{Created, Got, Prompted, Updated, UseCaseError};

/// インスタンスの読み書きのユースケース（UC-1 ・ 2 ・ 3 ・ 7 と、集約の削除のコマンド）。
pub trait InstanceUseCases {
    /// UC-1 インスタンスを作成する
    fn create(&self, schema: &str, path: &str) -> Result<Created, UseCaseError>;
    /// UC-2 値を取得する
    fn get(&self, path: &str, query: &str) -> Result<Got, UseCaseError>;
    /// UC-3 インスタンスを更新する。`hash` は呼ぶ側が読んだ時点のハッシュ値（無ければ、このユースケースが読んだ時点）
    fn update(&self, path: &str, patch: &str, hash: Option<&str>) -> Result<Updated, UseCaseError>;
    /// 集約 インスタンスの CMD-3 削除する（UC-4 の残りの手順は2回目）
    fn delete(&self, path: &str) -> Result<(), UseCaseError>;
    /// UC-7 x-prompt を受け取る
    fn prompt(&self, schema: &str, property: &str) -> Result<Prompted, UseCaseError>;
}
