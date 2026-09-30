// SPDX-License-Identifier: MIT
//! 提供者だけが使う入口 ── 言語の組の雛形を直したときの検証（verify）と、references の道具の出力の
//! 突き合わせ（conform）。**利用者の道具の一覧と MCP には出さない** ── 使う場面が無い。cargo の example は
//! `cargo install` で組み立てられず、配布物にも入らない。規約は利用者の CLI と同じである。
//!
//!     cargo run -q --manifest-path tool/Cargo.toml -p sc_cli --example provider -- verify [--work <作業場所>] [--json]

use std::process::ExitCode;

fn main() -> ExitCode {
    // **ICU の無い環境では、dotnet のコマンドそのものが起動時に止まる** ── 文化に依存しない動きにする。
    // 生む C# の Skill は InvariantGlobalization を保持するので、検証の結果は変化しない
    std::env::set_var("DOTNET_SYSTEM_GLOBALIZATION_INVARIANT", "1");
    sc_cli::run(&sc_service::provider_tools())
}
