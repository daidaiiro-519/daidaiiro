//! 集約 インスタンス。パス ・ スキーマ ・ JSON の値 ・ ハッシュ値を持つ。
//! 不変条件：ハッシュ値は、JSON の値の sha256 である（作成と更新のあとに成り立つ）。

use crate::domain::schema::{Schema, SchemaError, Validation};
use crate::domain::values::{
    Hash, InstancePath, JsonPatch, JsonValue, SchemaPath, ValidationError,
};
use std::fmt;

/// コマンドを拒んだ理由（宣言の拒否の理由）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reject {
    /// パスにインスタンスが既にある
    AlreadyExists,
    /// JSON Patch を適用できない
    CannotApply(String),
    /// ほかの更新と競合した
    Conflict,
    /// 検証を通過しない
    Invalid(Vec<ValidationError>),
    /// スキーマから検証器を作れない
    BrokenSchema(String),
}

impl fmt::Display for Reject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reject::AlreadyExists => formatter.write_str("パスにインスタンスが既にある"),
            Reject::CannotApply(detail) => write!(formatter, "JSON Patch を適用できない：{detail}"),
            Reject::Conflict => formatter.write_str("ほかの更新と競合した"),
            Reject::Invalid(_) => formatter.write_str("検証を通過しない"),
            Reject::BrokenSchema(detail) => write!(formatter, "スキーマが壊れている：{detail}"),
        }
    }
}

impl std::error::Error for Reject {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    path: InstancePath,
    schema: SchemaPath,
    value: JsonValue,
    hash: Hash,
}

impl Instance {
    /// 読み込んだ内容からインスタンスを作る。ハッシュ値は内容から求める。
    pub fn load(path: InstancePath, schema: SchemaPath, value: JsonValue) -> Self {
        let hash = value.hash();
        Self {
            path,
            schema,
            value,
            hash,
        }
    }

    /// 作成する。パスにインスタンスが既にあれば拒む。
    /// 必須のプロパティは置かない（未記入として残す）。スキーマのパスは `$schema` に持つ。
    pub fn create(path: InstancePath, schema: SchemaPath, exists: bool) -> Result<Self, Reject> {
        if exists {
            return Err(Reject::AlreadyExists);
        }
        let value = JsonValue::new(&serde_json::json!({"$schema": schema.as_str()}).to_string());
        Ok(Self::load(path, schema, value))
    }

    /// 更新する。読んだ時点のハッシュ値が今と違えば拒む。
    /// 適用したあとに、未記入以外の検証エラーがあれば拒む。
    pub fn update(
        &self,
        patch: &JsonPatch,
        read: &Hash,
        schema: &Schema,
    ) -> Result<(Self, Validation), Reject> {
        if &self.hash != read {
            return Err(Reject::Conflict);
        }
        let value = patch
            .apply(&self.value)
            .map_err(|error| Reject::CannotApply(error.0))?;
        let parsed: serde_json::Value = serde_json::from_str(value.as_str())
            .map_err(|error| Reject::CannotApply(error.to_string()))?;
        let validation = schema
            .validate(&parsed)
            .map_err(|SchemaError(error)| Reject::BrokenSchema(error))?;
        if !validation.errors.is_empty() {
            return Err(Reject::Invalid(validation.errors));
        }
        Ok((
            Self::load(self.path.clone(), self.schema.clone(), value),
            validation,
        ))
    }

    pub fn value(&self) -> &JsonValue {
        &self.value
    }
    pub fn hash(&self) -> &Hash {
        &self.hash
    }
}
