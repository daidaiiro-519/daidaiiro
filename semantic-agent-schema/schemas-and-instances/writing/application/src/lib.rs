//! 業務領域「書いて検証する」の書くモジュールの application。
//!
//! 作成 ・ 更新（UC1 ・ UC2 の機械 ・ ステップ2）、SAS を通らない変更の取り込み（ステップ3）、削除（UC3 ・ ステップ4）を持つ。
//! 書く前後の確かめは、検証のモジュール（validation-application）を使う。

pub mod ports;
pub mod use_cases;
