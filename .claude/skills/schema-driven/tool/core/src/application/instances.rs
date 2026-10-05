//! インスタンスの読み書きのユースケース（UC-1 ・ 2 ・ 3 ・ 7 と、集約の削除のコマンド）。
//! トランザクションスクリプトとして、出ていく側のポートを直接使う。

use crate::domain::instance::{Instance, Reject};
use crate::domain::schema::Schema;
use crate::domain::values::{
    Hash, InstancePath, JsonPatch, JsonValue, SchemaPath, ValidationError,
};
use crate::ports::inbound::InstanceUseCases;
use crate::ports::outbound::{Files, Query, Schemas, WriteError, WriteIf};
use serde_json::Value;

/// ユースケースが失敗した理由。`reason` は用語集の語（拒否の理由 ・ 失敗の種類）で書く。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseCaseError {
    pub reason: String,
    pub detail: String,
    pub errors: Vec<ValidationError>,
}

impl UseCaseError {
    fn new(reason: &str, detail: impl Into<String>) -> Self {
        Self {
            reason: reason.to_owned(),
            detail: detail.into(),
            errors: Vec::new(),
        }
    }
}

/// 未記入のプロパティと、その x-prompt。
#[derive(Debug, Clone, PartialEq)]
pub struct UnfilledWithPrompt {
    pub property: String,
    pub prompt: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Created {
    pub path: String,
    pub hash: String,
    pub unfilled: Vec<UnfilledWithPrompt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Got {
    pub value: Value,
    pub found: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Updated {
    pub hash: String,
    pub changed: bool,
    pub errors: Vec<ValidationError>,
    pub unfilled: Vec<UnfilledWithPrompt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Prompted {
    pub prompt: Value,
}

/// インスタンスの読み書きのユースケースの実装。ファイル ・ スキーマの供給元 ・ 取得は、ポートで受け取る。
pub struct Instances<'a> {
    files: &'a dyn Files,
    schemas: &'a dyn Schemas,
    query: &'a dyn Query,
}

impl<'a> Instances<'a> {
    pub fn new(files: &'a dyn Files, schemas: &'a dyn Schemas, query: &'a dyn Query) -> Self {
        Self {
            files,
            schemas,
            query,
        }
    }

    fn read_json(&self, path: &str) -> Result<(String, Value), UseCaseError> {
        let text = self
            .files
            .read(path)
            .map_err(|e| UseCaseError::new("読めない", e.0))?;
        let value = serde_json::from_str(&text)
            .map_err(|e| UseCaseError::new("JSON として読めない", e.to_string()))?;
        Ok((text, value))
    }

    fn load_schema(&self, path: &str) -> Result<Schema, UseCaseError> {
        let (_, root) = self.read_json(path)?;
        let name = path.rsplit('/').next().unwrap_or(path);
        let siblings = self
            .schemas
            .referenced(path)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(n, text)| serde_json::from_str(&text).ok().map(|v| (n, v)))
            .collect();
        Ok(Schema::new(name, root, siblings))
    }

    fn with_prompts(
        schema: &Schema,
        unfilled: &[crate::domain::values::Unfilled],
    ) -> Vec<UnfilledWithPrompt> {
        unfilled
            .iter()
            .map(|u| UnfilledWithPrompt {
                property: u.property().to_owned(),
                prompt: schema.prompt(u.property()),
            })
            .collect()
    }

    fn write_error(e: WriteError) -> UseCaseError {
        match e {
            WriteError::Conflict => UseCaseError::new("ほかの更新と競合した", ""),
            WriteError::Unwritable(d) => UseCaseError::new("書けない", d),
        }
    }
}

fn reject(r: Reject) -> UseCaseError {
    match r {
        Reject::AlreadyExists => UseCaseError::new("パスにインスタンスが既にある", ""),
        Reject::CannotApply(d) => UseCaseError::new("JSON Patch を適用できない", d),
        Reject::Conflict => UseCaseError::new("ほかの更新と競合した", ""),
        Reject::Invalid(errors) => UseCaseError {
            errors,
            ..UseCaseError::new("検証を通過しない", "")
        },
        Reject::BrokenSchema(d) => UseCaseError::new("スキーマが壊れている", d),
    }
}

fn invalid(e: crate::domain::values::InvalidValue) -> UseCaseError {
    UseCaseError::new("値が不正である", e.to_string())
}

impl InstanceUseCases for Instances<'_> {
    fn create(&self, schema: &str, path: &str) -> Result<Created, UseCaseError> {
        let schema_path = SchemaPath::new(schema).map_err(invalid)?;
        let instance_path = InstancePath::new(path).map_err(invalid)?;
        let loaded = self.load_schema(schema)?;
        let instance = Instance::create(instance_path, schema_path, self.files.exists(path))
            .map_err(reject)?;
        self.files
            .write(path, instance.value().as_str(), WriteIf::Absent)
            .map_err(|e| match e {
                WriteError::Conflict => reject(Reject::AlreadyExists),
                other => Self::write_error(other),
            })?;
        let parsed: Value = serde_json::from_str(instance.value().as_str()).unwrap_or(Value::Null);
        let validation = loaded
            .validate(&parsed)
            .map_err(|e| UseCaseError::new("スキーマが壊れている", e.0))?;
        Ok(Created {
            path: path.to_owned(),
            hash: instance.hash().as_str().to_owned(),
            unfilled: Self::with_prompts(&loaded, &validation.unfilled),
        })
    }

    fn get(&self, path: &str, query: &str) -> Result<Got, UseCaseError> {
        if query.is_empty() {
            return Err(UseCaseError::new(
                "JMESPath 式が読めない",
                "JMESPath 式が空である",
            ));
        }
        let (_, value) = self.read_json(path)?;
        let found = self
            .query
            .search(query, &value)
            .map_err(|e| UseCaseError::new("JMESPath 式が読めない", e.0))?;
        Ok(Got {
            found: !found.is_null(),
            value: found,
        })
    }

    fn update(&self, path: &str, patch: &str, hash: Option<&str>) -> Result<Updated, UseCaseError> {
        let patch =
            JsonPatch::new(patch).map_err(|_| UseCaseError::new("JSON Patch が空である", ""))?;
        let instance_path = InstancePath::new(path).map_err(invalid)?;
        let (text, value) = self.read_json(path)?;
        let schema = value
            .get("$schema")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                UseCaseError::new("スキーマが分からない", "インスタンスに $schema が無い")
            })?
            .to_owned();
        let loaded = self.load_schema(&schema)?;
        let instance = Instance::load(
            instance_path,
            SchemaPath::new(&schema).map_err(invalid)?,
            JsonValue::new(&text),
        );
        let read = match hash {
            Some(h) => Hash::new(h).map_err(invalid)?,
            None => instance.hash().clone(),
        };
        let (next, validation) = instance.update(&patch, &read, &loaded).map_err(reject)?;
        let changed = next.hash() != instance.hash();
        if changed {
            self.files
                .write(
                    path,
                    next.value().as_str(),
                    WriteIf::Unchanged(instance.hash().clone()),
                )
                .map_err(Self::write_error)?;
        }
        Ok(Updated {
            hash: next.hash().as_str().to_owned(),
            changed,
            errors: validation.errors.clone(),
            unfilled: Self::with_prompts(&loaded, &validation.unfilled),
        })
    }

    fn delete(&self, path: &str) -> Result<(), UseCaseError> {
        InstancePath::new(path).map_err(invalid)?;
        if !self.files.exists(path) {
            return Err(UseCaseError::new("読めない", "インスタンスが無い"));
        }
        self.files.remove(path).map_err(Self::write_error)
    }

    fn prompt(&self, schema: &str, property: &str) -> Result<Prompted, UseCaseError> {
        let loaded = self.load_schema(schema)?;
        loaded
            .prompt(property)
            .map(|prompt| Prompted { prompt })
            .ok_or_else(|| {
                UseCaseError::new(
                    "x-prompt を持つプロパティがスキーマに無い",
                    property.to_owned(),
                )
            })
    }
}
