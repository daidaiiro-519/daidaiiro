//! 集約インスタンス（ステップ1 では読む部分だけ）：Concrete に従って書いた JSON。

use serde_json::Value;

/// インスタンスの文書。
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    document: Value,
}

impl Instance {
    /// 文書から作る。$schema が文字列でなければ None。
    #[must_use]
    pub fn read(document: &Value) -> Option<Self> {
        document.get("$schema")?.as_str()?;
        Some(Self {
            document: document.clone(),
        })
    }

    /// 従う Concrete の $id。
    #[must_use]
    pub fn concrete(&self) -> &str {
        self.document
            .get("$schema")
            .and_then(Value::as_str)
            .unwrap_or_default()
    }

    /// インスタンスの id（参照の先として指される名前）。
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.document.get("id").and_then(Value::as_str)
    }

    /// 文書。
    #[must_use]
    pub const fn document(&self) -> &Value {
        &self.document
    }
}
