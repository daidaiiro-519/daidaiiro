//! 業務領域「契約への適合」のドメインモデル。
//!
//! UC4 の機械の検証（ステップ1）と、UC1 ・ UC2 の機械の作成 ・ 更新（ステップ2）を書く中で引き上げたものだけを持つ。
//! 何にも依存しない（外のライブラリは JSON の値と指紋の計算だけ）。

pub mod aggregates;
pub mod services;
pub mod values;
