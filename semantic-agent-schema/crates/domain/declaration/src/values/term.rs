//! term：glossary の term を、版付きの id で指す

use crate::values::urn::Urn;

/// term：glossary の term を、版付きの id（`urn:semantic-agent-schema:<持ち主>:glossary:v<版>#<名前>`）で指す
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    glossary: Urn,
    name: String,
}

impl Term {
    /// $ref の値から読む。glossary を指していなければ None
    pub fn from_ref(reference: &str) -> Option<Self> {
        let (id, name) = reference.split_once('#')?;
        let glossary = Urn::parse(id).ok().filter(|u| u.kind() == "glossary")?;
        let ok = name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        ok.then(|| Term {
            glossary,
            name: name.to_string(),
        })
    }

    /// 指す glossary の版
    pub fn glossary(&self) -> &Urn {
        &self.glossary
    }

    /// term の名前（$anchor）
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::Term;

    /// traces: I4
    #[test]
    fn glossary_ref_is_read() {
        let t = Term::from_ref("urn:semantic-agent-schema:acme:glossary:v2#Price").unwrap();
        assert_eq!((t.glossary().version(), t.name()), (2, "Price"));
    }

    /// traces: I4
    #[test]
    fn ref_to_other_than_glossary_is_not_a_term() {
        assert!(Term::from_ref("urn:semantic-agent-schema:acme:record:v1#Price").is_none());
    }

    /// traces: I4
    #[test]
    fn local_ref_is_not_a_term() {
        assert!(Term::from_ref("#/$defs/price").is_none());
    }
}
