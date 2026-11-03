//! 出口：SAS（semantic-agent-schema コマンド）。文脈 skills と文脈 agents のポートを、それぞれのモジュールで実装する。
//! どちらも semantic-agent-schema を PATH から探して外の実行ファイルとして呼び、出力を書き換えずに返す。

pub mod agents;
pub mod skills;
