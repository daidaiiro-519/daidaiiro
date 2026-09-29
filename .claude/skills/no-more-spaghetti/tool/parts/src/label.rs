// SPDX-License-Identifier: MIT
//! 画面へ出す語。**機械が分岐する値は ASCII である** ── 1つの語が識別子と表示を
//! 兼ねると、表示を直した瞬間に分岐が壊れる。

/// 判定の3値。**「実行しない」を合格に寄せない。**
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Verdict {
    /// 終了コードが0である。
    Pass,
    /// 終了コードが0以外である。
    Fail,
    /// 実行していない。
    Skip,
}

impl Verdict {
    /// 機械が分岐する値。
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Skip => "skip",
        }
    }

    /// 画面へ出す語。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Pass => "合格",
            Self::Fail => "不合格",
            Self::Skip => "実行しない",
        }
    }
}

/// 原典の立場。**外を指す規則だけが持つ** ── 内を指す規則には原典が無い。
#[must_use]
pub fn authority_label(key: &str) -> Option<&'static str> {
    match key {
        "spec" => Some("仕様"),
        "recommendation" => Some("推奨"),
        "third-party" => Some("第三者"),
        _ => None,
    }
}

/// 検査した種類。
#[must_use]
pub fn kind_label(key: &str) -> &'static str {
    match key {
        "concepts" => "概念",
        "schema" => "スキーマ",
        "generated" => "生成物",
        _ => "規則",
    }
}
