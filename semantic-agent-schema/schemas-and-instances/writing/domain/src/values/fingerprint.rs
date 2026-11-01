//! ハッシュ：ファイルの中身の sha256。中身が同じなら同じハッシュになる。
//! 判定が読んだ所のハッシュ（of_read）も同じ形で持つ。読んだ所と値が同じなら同じハッシュになる。

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

/// ファイルの中身のハッシュ（`sha256:` に続く16進）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fingerprint(String);

impl Fingerprint {
    /// 中身のバイト列からハッシュを作る。
    #[must_use]
    pub fn of(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        let mut hex = String::with_capacity(7 + digest.len() * 2);
        hex.push_str("sha256:");
        for b in digest {
            let _ = write!(hex, "{b:02x}");
        }
        Self(hex)
    }

    /// 判定が読んだ所のハッシュ。読んだ所（JSON Pointer。"" は文書全体）と、そこの値から作る。
    /// 値が無いことと null の値は別のハッシュになり、同じ値を別の所へ移しても別のハッシュになる。
    #[must_use]
    pub fn of_read(pointer: &str, value: Option<&Value>) -> Self {
        let value = value.map_or_else(|| "\u{1}absent".to_string(), Value::to_string);
        Self::of(format!("{pointer}\u{0}{value}").as_bytes())
    }

    /// 記録や引数の文字列から読む。`sha256:` に続く小文字の16進64字でなければ None。
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let hex = text.strip_prefix("sha256:")?;
        (hex.len() == 64
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
        .then(|| Self(text.to_string()))
    }

    /// 文字列として読む。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Fingerprint;
    use serde_json::json;

    #[test]
    fn read_fingerprint_depends_on_place_and_value() {
        let a = json!("x");
        assert_eq!(
            Fingerprint::of_read("/goal", Some(&a)),
            Fingerprint::of_read("/goal", Some(&json!("x")))
        );
        assert_ne!(
            Fingerprint::of_read("/goal", Some(&a)),
            Fingerprint::of_read("/name", Some(&a))
        );
        assert_ne!(
            Fingerprint::of_read("/goal", None),
            Fingerprint::of_read("/goal", Some(&json!(null)))
        );
    }

    #[test]
    fn same_bytes_same_fingerprint() {
        assert_eq!(Fingerprint::of(b"{}"), Fingerprint::of(b"{}"));
        assert_ne!(Fingerprint::of(b"{}"), Fingerprint::of(b"{ }"));
        assert!(Fingerprint::of(b"").as_str().starts_with("sha256:"));
        let fp = Fingerprint::of(b"{}");
        assert_eq!(Fingerprint::parse(fp.as_str()), Some(fp));
        assert_eq!(Fingerprint::parse("sha256:00"), None);
        assert_eq!(Fingerprint::parse("md5:00"), None);
    }
}
