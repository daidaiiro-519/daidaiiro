//! 文脈 skills の application（業務領域のモジュールはまだ作らない。探索のステップ1）。
//!
//! UC4 確かめる（Skill だけ）の手続きを use_cases にトランザクションスクリプトで書き、手続きが使うポート（SAS の検証 ・
//! Skill のフォルダとリンクを読む）だけを ports.rs に宣言する。SAS は共用サービスで、その結果は書き換えずに返す。

pub mod ports;
pub mod use_cases;
