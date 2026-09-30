// SPDX-License-Identifier: MIT
//! 検出1件と、判定の宣言。
//!
//! **拠って立つものを添える。** どの概念から来た判定かが読めないと、直すべきか
//! 外すべきかを判定できない。

/// 検出1件。
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Finding {
    /// どの判定か。
    pub check: String,
    /// 拠って立つもの。概念の番号、または媒体の決め。
    pub basis: String,
    /// 原文の行。**0 は、文書の全体に対する検出である。**
    pub line: usize,
    /// 読み手が開けるだけの抜粋。
    pub excerpt: String,
}

impl Finding {
    /// 検出を組む。
    #[must_use]
    pub fn new(check: &str, basis: &str, line: usize, excerpt: String) -> Self {
        Self {
            check: check.to_owned(),
            basis: basis.to_owned(),
            line,
            excerpt,
        }
    }
}
