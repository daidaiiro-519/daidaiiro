//! 文脈 skills の application（業務領域のモジュールはまだ作らない。探索のステップ1）。
//!
//! UC1 作る（create ・ write-skill-md と承認の材料）と UC4 確かめる（validate）の手続きを use_cases にトランザクションスクリプトで書き、
//! 手続きが使うポート（SAS の検証と書かせること ・ Skill のフォルダとリンクを読む ・ SKILL.md を書く）だけを ports.rs に宣言する。
//! SAS は共用サービスで、その結果は書き換えずに返す。creator は AI を起動せず、skill.json も書かない（SAS が書く）。

pub mod ports;
pub mod use_cases;
