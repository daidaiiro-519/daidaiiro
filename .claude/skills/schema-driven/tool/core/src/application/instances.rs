//! インスタンスの読み書きのユースケース（UC-1 ・ 2 ・ 3 ・ 7 と、集約の削除のコマンド）。
//! トランザクションスクリプトとして、出ていく側のポートを直接使う。

use crate::application::paths;
use crate::domain::instance::{Instance, Reject};
use crate::domain::schema::Schema;
use crate::domain::values::{
    Hash, InstancePath, JsonPatch, JsonValue, SchemaPath, ValidationError,
};
use crate::ports::inbound::InstanceUseCases;
use crate::ports::outbound::{Files, Query, Schemas, WriteError, WriteIf};
use serde_json::Value;
use std::sync::Arc;

/// ユースケースが失敗した理由。`reason` は用語集の語（拒否の理由 ・ 失敗の種類）で書く。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseCaseError {
    pub reason: String,
    pub detail: String,
    pub errors: Vec<ValidationError>,
}

impl UseCaseError {
    pub(crate) fn new(reason: &str, detail: impl Into<String>) -> Self {
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
pub struct Instances {
    files: Arc<dyn Files>,
    schemas: Arc<dyn Schemas>,
    query: Arc<dyn Query>,
}

impl Instances {
    pub fn new(files: Arc<dyn Files>, schemas: Arc<dyn Schemas>, query: Arc<dyn Query>) -> Self {
        Self {
            files,
            schemas,
            query,
        }
    }

    pub(crate) fn read_json(&self, path: &str) -> Result<(String, Value), UseCaseError> {
        let text = self
            .files
            .read(path)
            .map_err(|error| UseCaseError::new("読めない", error.0))?;
        let value = serde_json::from_str(&text)
            .map_err(|error| UseCaseError::new("JSON として読めない", error.to_string()))?;
        Ok((text, value))
    }

    pub(crate) fn load_schema(&self, path: &str) -> Result<Schema, UseCaseError> {
        let (_, root) = self.read_json(path)?;
        let name = path.rsplit('/').next().unwrap_or(path);
        let siblings = self
            .schemas
            .referenced(path)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(file_name, text)| {
                serde_json::from_str(&text)
                    .ok()
                    .map(|schema| (file_name, schema))
            })
            .collect();
        Ok(Schema::new(name, root, siblings))
    }

    fn with_prompts(
        schema: &Schema,
        unfilled: &[crate::domain::values::Unfilled],
    ) -> Vec<UnfilledWithPrompt> {
        unfilled
            .iter()
            .map(|unfilled| UnfilledWithPrompt {
                property: unfilled.property().to_owned(),
                prompt: schema.prompt(unfilled.property()),
            })
            .collect()
    }

    pub(crate) fn write_error(error: WriteError) -> UseCaseError {
        match error {
            WriteError::Conflict => UseCaseError::new("ほかの更新と競合した", ""),
            WriteError::Unwritable(detail) => UseCaseError::new("書けない", detail),
        }
    }
}

fn reject(reject: Reject) -> UseCaseError {
    match reject {
        Reject::AlreadyExists => UseCaseError::new("パスにインスタンスが既にある", ""),
        Reject::CannotApply(detail) => UseCaseError::new("JSON Patch を適用できない", detail),
        Reject::Conflict => UseCaseError::new("ほかの更新と競合した", ""),
        Reject::Invalid(errors) => UseCaseError {
            errors,
            ..UseCaseError::new("検証を通過しない", "")
        },
        Reject::BrokenSchema(detail) => UseCaseError::new("スキーマが壊れている", detail),
    }
}

fn invalid(error: crate::domain::values::InvalidValue) -> UseCaseError {
    UseCaseError::new("値が不正である", error.to_string())
}

impl InstanceUseCases for Instances {
    fn create(&self, schema: &str, path: &str) -> Result<Created, UseCaseError> {
        let schema_path = SchemaPath::new(schema).map_err(invalid)?;
        let instance_path = InstancePath::new(path).map_err(invalid)?;
        let loaded = self.load_schema(schema)?;
        // インスタンスの $schema には、インスタンスのファイルからの相対パスを書く
        let stored = SchemaPath::new(&paths::relative(paths::parent(path), schema_path.as_str()))
            .map_err(invalid)?;
        let instance =
            Instance::create(instance_path, stored, self.files.exists(path)).map_err(reject)?;
        self.files
            .write(path, instance.value().as_str(), WriteIf::Absent)
            .map_err(|error| match error {
                WriteError::Conflict => reject(Reject::AlreadyExists),
                other => Self::write_error(other),
            })?;
        let parsed: Value = serde_json::from_str(instance.value().as_str()).unwrap_or(Value::Null);
        let validation = loaded
            .validate(&parsed)
            .map_err(|error| UseCaseError::new("スキーマが壊れている", error.0))?;
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
            .map_err(|error| UseCaseError::new("JMESPath 式が読めない", error.0))?;
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
            })?;
        let schema = paths::join(paths::parent(path), schema);
        let loaded = self.load_schema(&schema)?;
        let instance = Instance::load(
            instance_path,
            SchemaPath::new(&schema).map_err(invalid)?,
            JsonValue::new(&text),
        );
        let read = match hash {
            Some(hash) => Hash::new(hash).map_err(invalid)?,
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
