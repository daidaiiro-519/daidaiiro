//! 変更の記録の id（ULID）

use std::fmt;

/// 変更の記録の id：打ち合わせなしで一意な値（ULID。Crockford の base32 で26字、先頭は 0〜7）
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChangeId(String);

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

impl ChangeId {
    /// 文字列から読む
    ///
    /// # Errors
    /// ULID の形でないとき
    pub fn parse(s: &str) -> Result<Self, String> {
        let ok = s.len() == 26
            && s.bytes().all(|b| ALPHABET.contains(&b))
            && s.as_bytes()
                .first()
                .is_some_and(|b| (b'0'..=b'7').contains(b));
        if ok {
            Ok(ChangeId(s.to_string()))
        } else {
            Err(format!("「{s}」は変更の記録の id（ULID）の形でない"))
        }
    }

    /// 時刻（ミリ秒）と乱数から作る。時刻と乱数は呼ぶ側（inbound）が渡す
    pub fn from_parts(millis: u64, random: u128) -> Self {
        let value: u128 =
            ((u128::from(millis) & 0xFFFF_FFFF_FFFF) << 80) | (random & ((1u128 << 80) - 1));
        let mut out = [0u8; 26];
        for (i, slot) in out.iter_mut().enumerate() {
            let shift = 125 - 5 * i;
            let idx = usize::try_from((value >> shift) & 0x1F).unwrap_or(0);
            *slot = ALPHABET[idx];
        }
        ChangeId(String::from_utf8_lossy(&out).to_string())
    }

    /// 文字列
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ChangeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::ChangeId;

    /// traces: U5:事後
    #[test]
    fn made_id_parses_back() {
        let id = ChangeId::from_parts(1_792_000_000_000, 12345);
        assert_eq!(ChangeId::parse(id.as_str()).unwrap(), id);
    }

    /// traces: U5:失敗時
    #[test]
    fn lowercase_or_short_is_refused() {
        assert!(ChangeId::parse("01jb7").is_err());
    }
}
