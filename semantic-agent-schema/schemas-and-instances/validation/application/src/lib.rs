//! 業務領域「書いて検証する」の検証のモジュールの application。
//!
//! Concrete とインスタンスの検証（UC4）と、入力が変わった rule だけの検証と
//! Semantic Agent Schema を通らない変更の知らせ（ステップ3）を持つ。検証のユースケースが使うポート（文書を読む ・ 形を照らす ・ 検証を記録する）は ports.rs に宣言する。
//! 誰が打ったか（Caller）は入口が決めて値で渡す。

pub mod ports;
pub mod use_cases;
