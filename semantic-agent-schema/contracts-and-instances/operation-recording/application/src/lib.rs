//! 業務領域「操作の記録」の application。
//!
//! ステップ1 では、呼び出し元のセッションを決め、検証の操作を追記するだけを持つ。
//! 行は文字列と数で持ち、conformance の domain に依存しない（記録は、何を検証したかの意味を解かない）。

pub mod ports;
pub mod use_cases;
