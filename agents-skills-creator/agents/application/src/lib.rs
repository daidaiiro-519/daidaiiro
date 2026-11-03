//! 文脈 agents の application（業務領域のモジュールはまだ作らない）。
//!
//! UC2 Agent を作る（create ・ write-agent-md ・ validate と承認の材料 ・ distribute ・ withdraw）の手続きを use_cases に
//! トランザクションスクリプトで書き、手続きが使うポートだけを ports.rs に宣言する。文脈 skills には、公開された言葉
//! （skills コマンドの validate の結果）でだけ従い、skills の crate には依存しない。Semantic Agent Schema と skills の結果は書き換えずに返す。

pub mod ports;
pub mod use_cases;
