//! 指紋：ファイルの中身の sha256。中身が同じなら同じ指紋になる。

use sha2::{Digest, Sha256};
use std::fmt::Write as _;

/// ファイルの中身の指紋（`sha256:` に続く16進）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fingerprint(String);

impl Fingerprint {
    /// 中身のバイト列から指紋を作る。
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
