//! 検証の結果の判定の5つの区分。

/// rule ごとの結果。verified に数えるのは verified だけ。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Verdict {
    /// 守られている
    Verified,
    /// 破られている
    Violated,
    /// 子セッションが判定できなかった（決めきれない）
    Unverified,
    /// 判定しようとして失敗した
    Error,
    /// 判定する手続きやつながる先が無い
    Unsupported,
}

impl Verdict {
    /// 判定の5つの区分の並び（数えるときの順）。
    pub const ALL: [Self; 5] = [
        Self::Verified,
        Self::Violated,
        Self::Unverified,
        Self::Error,
        Self::Unsupported,
    ];

    /// 公開された言葉での名前。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Violated => "violated",
            Self::Unverified => "unverified",
            Self::Error => "error",
            Self::Unsupported => "unsupported",
        }
    }

    /// 対象を「通らない」とする値か（violated と error）。
    #[must_use]
    pub const fn fails(self) -> bool {
        matches!(self, Self::Violated | Self::Error)
    }
}
