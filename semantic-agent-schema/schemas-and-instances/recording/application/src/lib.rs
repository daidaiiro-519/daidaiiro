//! 業務領域「記録」のモジュールの application。
//!
//! 記録を読む（UC7）と、入口（CLI）だけが呼ぶ「誰が打ったかを決めて sessions に登録する」手続き（semantic-agent-schema init を含む）を持つ。
//! 検証と変更の追記は、追記する書く ・ 検証 ・ 描画のモジュールが自分のポートで行う。ほかのモジュールの application はここを使わない。
//! 誰が打ったか（Caller）と記録の行の形は、使うモジュールの domain（writing-domain ・ validation-domain）に置く。

pub mod ports;
pub mod use_cases;
