//! インスタンスの状態の4つ

use crate::values::verdict::{Finding, Verdict};

/// 状態：rule の結果の5つの値から、上の行ほど先に当てはめて導く（決定表 C100）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceState {
    /// 下書き：violated が1つでもある（直すのは書く側）
    Draft,
    /// 確かめ直しを待つ：violated が無く、error が1つでもある
    AwaitingRecheck,
    /// 判断待ち：violated と error が無く、unverified か unsupported が1つでもある
    AwaitingJudgment,
    /// 検証済み：すべてが verified
    Verified,
}

impl InstanceState {
    /// 結果から導く
    pub fn from_results(results: &[Finding]) -> Self {
        let has = |v: Verdict| results.iter().any(|f| f.verdict() == v);
        if has(Verdict::Violated) {
            InstanceState::Draft
        } else if has(Verdict::Error) {
            InstanceState::AwaitingRecheck
        } else if has(Verdict::Unverified) || has(Verdict::Unsupported) {
            InstanceState::AwaitingJudgment
        } else {
            InstanceState::Verified
        }
    }

    /// 機械が読む名前
    pub fn as_str(self) -> &'static str {
        match self {
            InstanceState::Draft => "draft",
            InstanceState::AwaitingRecheck => "awaiting-recheck",
            InstanceState::AwaitingJudgment => "awaiting-judgment",
            InstanceState::Verified => "verified",
        }
    }

    /// 人が読む名前
    pub fn label(self) -> &'static str {
        match self {
            InstanceState::Draft => "下書き",
            InstanceState::AwaitingRecheck => "確かめ直しを待つ",
            InstanceState::AwaitingJudgment => "判断待ち",
            InstanceState::Verified => "検証済み",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InstanceState;
    use crate::values::verdict::{Finding, Verdict};

    fn r(v: Verdict) -> Finding {
        Finding::by_machine("r", "", v, "")
    }

    /// traces: U1:事後
    #[test]
    fn violated_comes_first_even_with_error() {
        assert_eq!(
            InstanceState::from_results(&[r(Verdict::Error), r(Verdict::Violated)]),
            InstanceState::Draft
        );
    }

    /// traces: U1:事後
    #[test]
    fn error_without_violation_awaits_recheck() {
        assert_eq!(
            InstanceState::from_results(&[r(Verdict::Unverified), r(Verdict::Error)]),
            InstanceState::AwaitingRecheck
        );
    }

    /// traces: U1:事後
    #[test]
    fn unsupported_awaits_judgment() {
        assert_eq!(
            InstanceState::from_results(&[r(Verdict::Verified), r(Verdict::Unsupported)]),
            InstanceState::AwaitingJudgment
        );
    }

    /// traces: U1:事後
    #[test]
    fn all_verified_is_verified() {
        assert_eq!(
            InstanceState::from_results(&[r(Verdict::Verified)]),
            InstanceState::Verified
        );
    }
}
