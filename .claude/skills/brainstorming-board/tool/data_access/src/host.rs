// SPDX-License-Identifier: MIT
//! 動いているプロセス自身についての値。**この Skill に固有の入出力である。**
//!
//! **判定を置かない** ── 値をどう使うかは、業務ロジック層が決める。

/// このプロセスの番号。
#[must_use]
pub fn pid() -> u32 {
    std::process::id()
}
