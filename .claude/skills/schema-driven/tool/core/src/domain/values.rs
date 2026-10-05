//! 値オブジェクト。宣言 VO-n の成分 ・ 不変条件 ・ 操作をそのまま持つ。作れない値は `new` が拒む。

use sha2::{Digest, Sha256};
use std::fmt;

/// 値オブジェクトを作れなかった理由。どの宣言のどの不変条件に反したかを持つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidValue {
    pub invariant: &'static str,
    pub message: String,
}

impl fmt::Display for InvalidValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.invariant, self.message)
    }
}

impl std::error::Error for InvalidValue {}

fn non_empty(value: &str, invariant: &'static str, what: &str) -> Result<String, InvalidValue> {
    if value.chars().count() >= 1 {
        Ok(value.to_owned())
    } else {
        Err(InvalidValue {
            invariant,
            message: format!("{what}が空である"),
        })
    }
}

/// VO-1 パス。インスタンスのファイルの場所。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InstancePath(String);

impl InstancePath {
    pub fn new(value: &str) -> Result<Self, InvalidValue> {
        non_empty(value, "VO-1.INV-1", "ファイルのパス").map(Self)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// VO-2 スキーマ。インスタンスの形と注釈を書いた JSON Schema のファイルのパス。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaPath(String);

impl SchemaPath {
    pub fn new(value: &str) -> Result<Self, InvalidValue> {
        non_empty(value, "VO-2.INV-1", "スキーマのファイルのパス").map(Self)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// VO-5 検証エラー。スキーマを満たさないプロパティと、その理由。未記入は含めない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    property: String,
    reason: String,
}

impl ValidationError {
    /// プロパティは JSON Pointer で、空ならインスタンスの根を指す。
    pub fn new(property: &str, reason: &str) -> Result<Self, InvalidValue> {
        let reason = non_empty(reason, "VO-5.INV-1", "理由")?;
        Ok(Self {
            property: property.to_owned(),
            reason,
        })
    }
    pub fn property(&self) -> &str {
        &self.property
    }
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// VO-6 ハッシュ値。ファイルの内容の sha256 を16進の文字列で表す。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hash(String);

impl Hash {
    pub fn new(value: &str) -> Result<Self, InvalidValue> {
        if value.chars().count() == 64 {
            Ok(Self(value.to_owned()))
        } else {
            Err(InvalidValue {
                invariant: "VO-6.INV-1",
                message: format!(
                    "16進の文字列の文字数が64でない（{}）",
                    value.chars().count()
                ),
            })
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// VO-7 JSON Patch（RFC 6902）。インスタンスへ適用する操作の並び。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonPatch(String);

/// JSON Patch を適用できなかった理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchError(pub String);

impl JsonPatch {
    pub fn new(value: &str) -> Result<Self, InvalidValue> {
        non_empty(value, "VO-7.INV-1", "操作の並び").map(Self)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// OP-1 適用する。JSON Patch を JSON の値に当てて、新しい値を求める。
    pub fn apply(&self, target: &JsonValue) -> Result<JsonValue, PatchError> {
        let patch: json_patch::Patch = serde_json::from_str(&self.0)
            .map_err(|e| PatchError(format!("JSON Patch として読めない: {e}")))?;
        let mut doc: serde_json::Value = serde_json::from_str(target.as_str())
            .map_err(|e| PatchError(format!("JSON の値として読めない: {e}")))?;
        json_patch::patch(&mut doc, &patch)
            .map_err(|e| PatchError(format!("適用できない: {e}")))?;
        Ok(JsonValue::new(&doc.to_string()))
    }
}

/// VO-10 JSON の値。インスタンスのファイルに書いてある JSON。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonValue(String);

impl JsonValue {
    pub fn new(value: &str) -> Self {
        Self(value.to_owned())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// OP-1 ハッシュ値を求める。JSON の値の sha256。
    pub fn hash(&self) -> Hash {
        let digest = Sha256::digest(self.0.as_bytes());
        let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        Hash(hex)
    }
}

/// VO-12 未記入。スキーマの必須のプロパティが、インスタンスに無いこと（JSON Schema の required の違反）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unfilled(String);

impl Unfilled {
    pub fn new(property: &str) -> Result<Self, InvalidValue> {
        non_empty(property, "VO-12.INV-1", "プロパティ").map(Self)
    }
    pub fn property(&self) -> &str {
        &self.0
    }
}
