//! 文書が Concrete かインスタンスか。$schema の申告から見分ける（書く ・ 検証 ・ 描画 ・ 検索のどれも使う、集約の見分け方）。

use crate::aggregates::{ABSTRACT_VERSIONS, DIALECT};

/// 対象の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    /// Abstract に従う schema
    Concrete,
    /// Concrete に従う JSON
    Instance,
}

impl TargetKind {
    /// 出力と記録に書く名前。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Concrete => "concrete",
            Self::Instance => "instance",
        }
    }

    /// $schema の申告から対象の種類を見分ける。Abstract の版ならば Concrete、ほかの名前ならインスタンス。
    /// インスタンスが従う Concrete は、渡された置き場所から $id で引く（SchemaIndex）。
    ///
    /// # Errors
    /// 版の一覧に無い ・ $schema が無いとき（取りに行かずに断る理由）。
    pub fn classify(declared: &str) -> Result<Self, Unclassified> {
        if ABSTRACT_VERSIONS.contains(&declared) {
            Ok(Self::Concrete)
        } else if declared.starts_with("urn:semantic-agent-schema:published-language:abstract:") {
            Err(Unclassified::new(
                "schema_not_listed",
                format!(
                    "{declared} は SAS が入れて出す Abstract の版（{}）に無い。取りに行かない",
                    ABSTRACT_VERSIONS.join(", ")
                ),
            ))
        } else if declared.is_empty() {
            Err(Unclassified::new("no_schema", "$schema が無い".to_string()))
        } else {
            Ok(Self::Instance)
        }
    }

    /// 形（JSON Schema）を照らすスキーマの $id。Concrete は申告した Abstract と方言の両方、インスタンスは従う Concrete。
    #[must_use]
    pub fn form_schemas(self, declared: &str) -> Vec<&str> {
        match self {
            Self::Concrete => vec![declared, DIALECT],
            Self::Instance => vec![declared],
        }
    }
}

/// 対象の種類を見分けられなかった理由。code は決まった名前で、出力に入る。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unclassified {
    code: &'static str,
    message: String,
}

impl Unclassified {
    const fn new(code: &'static str, message: String) -> Self {
        Self { code, message }
    }

    /// 決まった名前（schema_not_listed ・ no_schema）。
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
