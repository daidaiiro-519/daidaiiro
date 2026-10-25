//! 判定する者（machine ・ ai ・ human）

/// 判定する者：rule と検証結果を判定するのは機械 ・ AI ・ 人のどれか
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum By {
    /// 機械
    Machine,
    /// AI
    Ai,
    /// 人
    Human,
}

impl By {
    /// 文字列から読む。machine ・ ai ・ human のほかは None
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "machine" => Some(By::Machine),
            "ai" => Some(By::Ai),
            "human" => Some(By::Human),
            _ => None,
        }
    }

    /// 文字列にする
    pub fn as_str(self) -> &'static str {
        match self {
            By::Machine => "machine",
            By::Ai => "ai",
            By::Human => "human",
        }
    }
}
