//! 名前の型（URN）

use std::fmt;

/// 名前の型（URN）：`urn:semantic-agent-schema:<持ち主>:<種類>:v<版>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Urn {
    owner: String,
    kind: String,
    version: u32,
}

const PREFIX: &str = "urn:semantic-agent-schema:";

fn is_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && s.chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

impl Urn {
    /// 持ち主 ・ 種類 ・ 版から作る
    ///
    /// # Errors
    /// 持ち主か種類が英小文字 ・ 数字 ・ ハイフンでない、または版が0のとき
    pub fn new(owner: &str, kind: &str, version: u32) -> Result<Self, String> {
        if !is_name(owner) {
            return Err(format!(
                "持ち主「{owner}」は英小文字 ・ 数字 ・ ハイフンで書く"
            ));
        }
        if !is_name(kind) {
            return Err(format!(
                "種類「{kind}」は英小文字 ・ 数字 ・ ハイフンで書く"
            ));
        }
        if version == 0 {
            return Err("版は1から数える".to_string());
        }
        Ok(Urn {
            owner: owner.to_string(),
            kind: kind.to_string(),
            version,
        })
    }

    /// 文字列から読む
    ///
    /// # Errors
    /// URN の形でないとき
    pub fn parse(s: &str) -> Result<Self, String> {
        let rest = s
            .strip_prefix(PREFIX)
            .ok_or_else(|| format!("「{s}」は {PREFIX} で始まらない"))?;
        let parts: Vec<&str> = rest.split(':').collect();
        let [owner, kind, v] = parts.as_slice() else {
            return Err(format!("「{s}」は <持ち主>:<種類>:v<版> の形でない"));
        };
        let version = v
            .strip_prefix('v')
            .and_then(|n| n.parse::<u32>().ok())
            .ok_or_else(|| format!("「{s}」の版が v<数> でない"))?;
        Urn::new(owner, kind, version)
    }

    /// 持ち主
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// 種類
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// 版
    pub fn version(&self) -> u32 {
        self.version
    }
}

impl fmt::Display for Urn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{PREFIX}{}:{}:v{}", self.owner, self.kind, self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::Urn;

    /// traces: I3
    #[test]
    fn parse_then_display_round_trip() {
        let u = Urn::parse("urn:semantic-agent-schema:acme:record:v2").unwrap();
        assert_eq!((u.owner(), u.kind(), u.version()), ("acme", "record", 2));
        assert_eq!(u.to_string(), "urn:semantic-agent-schema:acme:record:v2");
    }

    /// traces: I3
    #[test]
    fn other_namespace_is_refused() {
        assert!(Urn::parse("urn:other:acme:record:v1").is_err());
    }

    /// traces: I3
    #[test]
    fn missing_version_is_refused() {
        assert!(Urn::parse("urn:semantic-agent-schema:acme:record").is_err());
    }

    /// traces: I3
    #[test]
    fn version_zero_is_refused() {
        assert!(Urn::parse("urn:semantic-agent-schema:acme:record:v0").is_err());
    }

    /// traces: I3
    #[test]
    fn upper_case_owner_is_refused() {
        assert!(Urn::new("Acme", "record", 1).is_err());
    }
}
