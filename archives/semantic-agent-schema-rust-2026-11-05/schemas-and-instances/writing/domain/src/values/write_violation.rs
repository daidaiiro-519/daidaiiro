//! 書く条件に当たって書けなかった理由。

/// 集約の書く条件に外れたこと。code は決まった名前で、出力に入る（断った操作は記録しない）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteViolation {
    code: &'static str,
    message: String,
}

impl WriteViolation {
    /// 外れた条件の名前（human_field）と理由（当たる所を含める）。
    #[must_use]
    pub fn new(code: &'static str, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }

    /// 外れた条件の名前。
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// 理由。
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}
