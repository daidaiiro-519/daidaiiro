//! 業務領域「スキーマへの適合」の application。
//!
//! Concrete とインスタンスの検証（UC4 の機械の rule・ステップ1）と、作成 ・ 更新（UC1 ・ UC2 の機械・ステップ2）、
//! SAS を通らない変更の取り込みと、入力が変わった rule だけの検証（ステップ3）を持つ。

pub mod ports;
pub mod use_cases;
