// SPDX-License-Identifier: MIT
//! 外の道具を1回だけ呼び、標準出力を受け取る。
//!
//! **ファイルごとに呼ばない** ── 呼び出し1回が数十ミリ秒である（実測 `python3` で
//! 約32ミリ秒）。**標準入力は閉じる** ── 道具が入力を待って止まらないようにする。

use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

/// 道具を1回呼んだ結果。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Ran {
    /// 呼べた ── 標準出力と終了コード。
    Output {
        /// 標準出力。
        stdout: String,
        /// 標準エラー。
        stderr: String,
        /// 終了コード。**無いのは信号で落ちた場合である。**
        code: Option<i32>,
    },
    /// 道具が無い。
    Absent,
}

/// 道具が在るかを見る。
#[must_use]
pub fn exists(program: &str) -> bool {
    matches!(
        run(program, &["--version"], Path::new(".")),
        Ok(Ran::Output { .. })
    )
}

/// 道具を1回呼ぶ。
///
/// # Errors
///
/// 起動が「見つからない」以外の理由で失敗したときに返す。
pub fn run(program: &str, args: &[&str], cwd: &Path) -> io::Result<Ran> {
    run_with(program, args, cwd, &[])
}

/// 環境を足して、道具を1回呼ぶ。
///
/// **判定の結果を変える値は渡さない。** 渡すのは、道具が起動できない環境を補う値
/// だけである（実測 ── 地域化の書庫が無いと `dotnet` は起動そのものが落ちる）。
///
/// # Errors
///
/// 起動が「見つからない」以外の理由で失敗したときに返す。
pub fn run_with(program: &str, args: &[&str], cwd: &Path, env: &[(&str, &str)]) -> io::Result<Ran> {
    let mut command = Command::new(program);
    command.args(args).current_dir(cwd).stdin(Stdio::null());
    for (key, value) in env {
        command.env(key, value);
    }
    match command.output() {
        Ok(out) => Ok(Ran::Output {
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            code: out.status.code(),
        }),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Ran::Absent),
        Err(e) => Err(e),
    }
}
