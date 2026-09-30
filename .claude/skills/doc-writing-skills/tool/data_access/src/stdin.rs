// SPDX-License-Identifier: MIT
//! 標準入力の読み込み。**この Skill に固有の入出力である** ── 雛形の `files` ・ `process` は持たない。
//!
//! `reply` ・ `review` が、Claude Code の Stop フックの入力を標準入力から受け取るために使う。
//! **判定を置かない** ── 読んだものをどう扱うかは、業務ロジック層が決める。

use std::io::{self, Read};

/// 標準入力を終わりまで読み、文字列として返す。
///
/// # Errors
///
/// 読めないか、UTF-8 でないときに返す。
pub fn read_to_string() -> io::Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}
