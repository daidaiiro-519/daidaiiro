//! 検証の結果の判定の5つの区分。

/// rule ごとの結果。verified に数えるのは verified だけ。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Verdict {
    /// 守られている
    Verified,
    /// 破られている
    Violated,
    /// 判定したセッションが判定できないと答えた（決めきれない）
    Unverified,
    /// 判定しようとして失敗した
    Error,
    /// まだ判定していない（意味の rule の判定が渡されていない ・ 判定する手続きが無い）
    NotJudged,
}

impl Verdict {
    /// 判定の5つの区分の並び（数えるときの順）。
    pub const ALL: [Self; 5] = [
        Self::Verified,
        Self::Violated,
        Self::Unverified,
        Self::Error,
        Self::NotJudged,
    ];

    /// 公開された言葉での名前。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Violated => "violated",
            Self::Unverified => "unverified",
            Self::Error => "error",
            Self::NotJudged => "not-judged",
        }
    }

    /// 公開された言葉での名前から読む。知らない名前は None。
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == name)
    }

    /// 判定できた値か（verified ・ violated ・ unverified）。not-judged と error は判定できていない。
    #[must_use]
    pub const fn judged(self) -> bool {
        matches!(self, Self::Verified | Self::Violated | Self::Unverified)
    }

    /// 対象を「通らない」とする値か（violated と error）。
    #[must_use]
    pub const fn fails(self) -> bool {
        matches!(self, Self::Violated | Self::Error)
    }
}
