// SPDX-License-Identifier: MIT
//! skills-creator のプレゼンテーション層（CLI）。シェルから呼ぶ唯一の経路である。
//!
//!     skills-creator <動詞> [対象…] [--json]
//!
//! **道具ごとに実行ファイルを作らない** ── 実行ファイルが増えると、呼ぶ側が形を推測することになる。
//! 依存の向きは `Cargo.toml` が宣言する ── この crate はサービス層だけを参照する。

use std::process::ExitCode;

fn main() -> ExitCode {
    sc_cli::run(&sc_service::tools())
}
