//! 検証結果（5つの値 ・ 根拠 ・ 判定した者）

use semantic_agent_schema_declaration::values::by::By;

/// 検証の値（5つ）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// 合っている
    Verified,
    /// 外れている
    Violated,
    /// まだ判定していない
    Unverified,
    /// 判定できる者がいない
    Unsupported,
    /// 判定の途中で失敗した
    Error,
}

impl Verdict {
    /// 文字列にする
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Verified => "verified",
            Verdict::Violated => "violated",
            Verdict::Unverified => "unverified",
            Verdict::Unsupported => "unsupported",
            Verdict::Error => "error",
        }
    }
}

/// 検証結果：どこで ・ どの値か ・ 根拠 ・ 判定した者。不変条件 I12
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pointer: String,
    verdict: Verdict,
    reason: String,
    by: By,
}

impl Finding {
    /// 機械が判定した検証結果を作る
    pub fn by_machine(pointer: &str, verdict: Verdict, reason: &str) -> Self {
        Finding {
            pointer: pointer.to_string(),
            verdict,
            reason: reason.to_string(),
            by: By::Machine,
        }
    }

    /// 場所（JSON Pointer）
    pub fn pointer(&self) -> &str {
        &self.pointer
    }

    /// 値
    pub fn verdict(&self) -> Verdict {
        self.verdict
    }

    /// 根拠
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// 判定した者
    pub fn by(&self) -> By {
        self.by
    }
}

#[cfg(test)]
mod tests {
    use super::{Finding, Verdict};

    /// traces: U1:失敗時
    #[test]
    fn machine_finding_keeps_value_reason_and_judge() {
        let f = Finding::by_machine("/why", Verdict::Violated, "必須の欄が空");
        assert_eq!(
            (f.verdict().as_str(), f.reason(), f.by().as_str()),
            ("violated", "必須の欄が空", "machine")
        );
    }
}
