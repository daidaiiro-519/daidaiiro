//! 宣言の誤り（場所と理由）

/// 宣言の誤り：形では見られない、rule ・ x- ・ 名前の宣言の誤りの場所と理由
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Breach {
    pointer: String,
    reason: String,
}

impl Breach {
    /// 場所（JSON Pointer）と理由から作る
    pub fn new(pointer: &str, reason: String) -> Self {
        Breach {
            pointer: pointer.to_string(),
            reason,
        }
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
