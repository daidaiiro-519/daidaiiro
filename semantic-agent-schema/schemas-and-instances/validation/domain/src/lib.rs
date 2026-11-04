//! 業務領域「書いて検証する」の検証のモジュールのドメインモデル。
//!
//! 判定の値（判定の4つの区分 ・ rule の結果 ・ 誰が判定するか）と、書くモジュールの集約（Concrete ・ インスタンス）を読んで
//! rule に照らす業務サービスを持つ。集約は持たない（集約は writing-domain）。

pub mod services;
pub mod values;
