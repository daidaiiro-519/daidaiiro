//! ETag（文書の中身から計算する短い値。楽観ロックに使う）

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt;

/// ETag：文書の中身（JSON を同じ順で書き出したもの）の SHA-256 の先頭20字。保存せず、読むたび ・ 書くたびに計算する。
/// 中身が1文字でも変われば別の値になる（HTTP の ETag と If-Match と同じ考え方の楽観ロック）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ETag(String);

impl ETag {
    /// 文書から計算する
    pub fn of(document: &Value) -> Self {
        let digest = Sha256::digest(document.to_string().as_bytes());
        let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        ETag(hex.chars().take(20).collect())
    }

    /// 文字列
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 渡された ETag と同じか
    pub fn matches(&self, given: &str) -> bool {
        self.0 == given
    }
}

impl fmt::Display for ETag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::ETag;
    use serde_json::json;

    /// traces: U6:事後
    #[test]
    fn same_content_gives_the_same_etag() {
        assert_eq!(
            ETag::of(&json!({"a": 1, "b": "x"})),
            ETag::of(&json!({"b": "x", "a": 1}))
        );
    }

    /// traces: U6:失敗時
    #[test]
    fn changed_content_gives_another_etag() {
        assert_ne!(
            ETag::of(&json!({"why": "a"})),
            ETag::of(&json!({"why": "b"}))
        );
    }
}
