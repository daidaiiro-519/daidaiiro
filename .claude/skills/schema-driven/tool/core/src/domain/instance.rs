//! 集約 インスタンス（AGG-1）。パス ・ スキーマ ・ JSON の値 ・ ハッシュ値を持つ。
//! INV-1：ハッシュ値は、JSON の値の sha256 である（作成と更新のあとに成り立つ）。

use crate::domain::schema::{Schema, SchemaError, Validation};
use crate::domain::values::{
    Hash, InstancePath, JsonPatch, JsonValue, SchemaPath, ValidationError,
};

/// コマンドを拒んだ理由（宣言の拒否の理由）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reject {
    /// TERM-37 パスにインスタンスが既にある
    AlreadyExists,
    /// TERM-38 JSON Patch を適用できない
    CannotApply(String),
    /// TERM-52 ほかの更新と競合した
    Conflict,
    /// TERM-39 検証を通過しない
    Invalid(Vec<ValidationError>),
    /// スキーマから検証器を作れない
    BrokenSchema(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    path: InstancePath,
    schema: SchemaPath,
    value: JsonValue,
    hash: Hash,
}

impl Instance {
    /// 読み込んだ内容からインスタンスを組む。ハッシュ値は内容から求める（INV-1）。
    pub fn load(path: InstancePath, schema: SchemaPath, value: JsonValue) -> Self {
        let hash = value.hash();
        Self {
            path,
            schema,
            value,
            hash,
        }
    }

    /// CMD-1 作成する。BR-1：パスにインスタンスが既にあれば拒む。
    /// 必須のプロパティは置かない（未記入として残す。ACDR 0118）。スキーマのパスは `$schema` に持つ。
    pub fn create(path: InstancePath, schema: SchemaPath, exists: bool) -> Result<Self, Reject> {
        if exists {
            return Err(Reject::AlreadyExists);
        }
        let value = JsonValue::new(&serde_json::json!({"$schema": schema.as_str()}).to_string());
        Ok(Self::load(path, schema, value))
    }

    /// CMD-2 更新する。BR-1：読んだ時点のハッシュ値が今と違えば拒む。
    /// 適用したあとに、未記入以外の検証エラーがあれば拒む（ACDR 0118）。
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
            .map_err(|e| Reject::CannotApply(e.0))?;
        let parsed: serde_json::Value =
            serde_json::from_str(value.as_str()).map_err(|e| Reject::CannotApply(e.to_string()))?;
        let validation = schema
            .validate(&parsed)
            .map_err(|SchemaError(e)| Reject::BrokenSchema(e))?;
        if !validation.errors.is_empty() {
            return Err(Reject::Invalid(validation.errors));
        }
        Ok((
            Self::load(self.path.clone(), self.schema.clone(), value),
            validation,
        ))
    }

    pub fn path(&self) -> &InstancePath {
        &self.path
    }
    pub fn schema(&self) -> &SchemaPath {
        &self.schema
    }
    pub fn value(&self) -> &JsonValue {
        &self.value
    }
    pub fn hash(&self) -> &Hash {
        &self.hash
    }
}
