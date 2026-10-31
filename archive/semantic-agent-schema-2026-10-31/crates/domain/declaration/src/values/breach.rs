//! 宣言の誤り（rule ・ 場所 ・ 理由）

/// 宣言の誤り：Abstract の機械の rule のどれに外れたか、どこで、なぜ
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Breach {
    rule: String,
    pointer: String,
    reason: String,
}

impl Breach {
    /// rule の id ・ 場所（JSON Pointer）・ 理由から作る
    pub fn new(rule: &str, pointer: &str, reason: String) -> Self {
        Breach {
            rule: rule.to_string(),
            pointer: pointer.to_string(),
            reason,
        }
    }

    /// 外れた rule の id
    pub fn rule(&self) -> &str {
        &self.rule
    }

    /// 場所（JSON Pointer）
    pub fn pointer(&self) -> &str {
        &self.pointer
    }

    /// 理由
    pub fn reason(&self) -> &str {
        &self.reason
    }
}
