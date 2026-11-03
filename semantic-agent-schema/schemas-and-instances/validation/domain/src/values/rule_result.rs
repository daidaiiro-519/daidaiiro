//! rule ごとの結果と、判定の5つの区分ごとの数。

use super::Verdict;

/// 誰が判定したか（結果の by）。machine は形の検査と組み込みの検査、ai は semantic-agent-schema を打ったセッションが渡した判定。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum By {
    /// 機械（SAS か JSON Schema のバリデーター）
    Machine,
    /// AI（semantic-agent-schema を打ったセッションが semantic-agent-schema validate --answer で渡した判定）
    Ai,
}

impl By {
    /// 公開された言葉での名前。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Machine => "machine",
            Self::Ai => "ai",
        }
    }
}

/// 1つの rule を1か所に当てた結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleResult {
    rule: String,
    pointer: String,
    by: By,
    verdict: Verdict,
    reason: String,
}

impl RuleResult {
    /// 結果を作る。
    #[must_use]
    pub fn new(rule: &str, pointer: &str, by: By, verdict: Verdict, reason: &str) -> Self {
        Self {
            rule: rule.to_string(),
            pointer: pointer.to_string(),
            by,
            verdict,
            reason: reason.to_string(),
        }
    }

    /// rule の id。
    #[must_use]
    pub fn rule(&self) -> &str {
        &self.rule
    }
    /// 当てた所（対象の中の JSON Pointer）。
    #[must_use]
    pub fn pointer(&self) -> &str {
        &self.pointer
    }
    /// 誰が判定する rule か。
    #[must_use]
    pub const fn by(&self) -> By {
        self.by
    }
    /// 結果。
    #[must_use]
    pub const fn verdict(&self) -> Verdict {
        self.verdict
    }
    /// 理由。
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// 判定の5つの区分ごとの数。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Counts([usize; 5]);

impl Counts {
    /// 結果の並びから数える。
    #[must_use]
    pub fn of<'a>(results: impl IntoIterator<Item = &'a RuleResult>) -> Self {
        let mut c = Self::default();
        for r in results {
            c.add(r.verdict());
        }
        c
    }

    /// 1件足す。
    pub fn add(&mut self, v: Verdict) {
        if let Some(i) = Verdict::ALL.iter().position(|x| *x == v) {
            self.0[i] += 1;
        }
    }

    /// ほかの数を足し合わせる。
    pub fn merge(&mut self, other: &Self) {
        for (a, b) in self.0.iter_mut().zip(other.0) {
            *a += b;
        }
    }

    /// その値の数。
    #[must_use]
    pub fn get(&self, v: Verdict) -> usize {
        Verdict::ALL
            .iter()
            .position(|x| *x == v)
            .map_or(0, |i| self.0[i])
    }

    /// violated か error が1件でもあるか。
    #[must_use]
    pub fn fails(&self) -> bool {
        self.get(Verdict::Violated) + self.get(Verdict::Error) > 0
    }
}

#[cfg(test)]
mod tests {
    use super::{By, Counts, RuleResult, Verdict};

    #[test]
    fn counts_each_value() {
        let rs = [
            RuleResult::new("a", "", By::Machine, Verdict::Verified, ""),
            RuleResult::new("b", "", By::Machine, Verdict::Violated, ""),
            RuleResult::new("c", "", By::Ai, Verdict::NotJudged, ""),
        ];
        let c = Counts::of(&rs);
        assert_eq!(c.get(Verdict::Verified), 1);
        assert_eq!(c.get(Verdict::Violated), 1);
        assert_eq!(c.get(Verdict::NotJudged), 1);
        assert!(c.fails());
    }
}
