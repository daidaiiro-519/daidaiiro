//! インスタンスの読み書きのユースケースと、削除のコマンド。
//! トランザクションスクリプトとして、出ていく側のポートを直接使う。

use crate::domain::contract;
use crate::domain::instance::{Instance, Reject};
use crate::domain::schema::Schema;
use crate::domain::schema_name;
use crate::domain::values::{
    Hash, InstancePath, JsonPatch, JsonValue, SchemaPath, ValidationError,
};
use crate::ports::inbound::InstanceUseCases;
use crate::ports::outbound::{Files, Query, Schemas, WriteError, WriteIf};
use serde_json::Value;
use std::fmt;
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

/// 理由を先に置き、詳しい説明があれば「：」のあとに続ける。
impl fmt::Display for UseCaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.detail.is_empty() {
            formatter.write_str(&self.reason)
        } else {
            write!(formatter, "{}：{}", self.reason, self.detail)
        }
    }
}

impl std::error::Error for UseCaseError {}

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
    /// --skill で渡した Skill のフォルダ。スキーマの名前は、この Skill の assets から引く
    skill: Option<String>,
}

impl Instances {
    pub fn new(files: Arc<dyn Files>, schemas: Arc<dyn Schemas>, query: Arc<dyn Query>) -> Self {
        Self {
            files,
            schemas,
            query,
            skill: None,
        }
    }

    /// スキーマの名前を引く Skill のフォルダを渡す。
    pub fn with_skill(mut self, dir: &str) -> Self {
        self.skill = Some(dir.trim_end_matches(['/', '\\']).to_owned());
        self
    }

    /// インスタンスの $schema の名前から、スキーマのファイルのパスを決める。
    /// --skill の Skill の assets から $id で引く。名前でない値は断る。
    pub(crate) fn schema_path(&self, instance: &str, value: &str) -> Result<String, UseCaseError> {
        let Some(owner) = schema_name::owner(value) else {
            return Err(UseCaseError::new(
                "スキーマの名前でない",
                format!(
                    "{instance} の $schema「{value}」は「<Skill の名前>/<種類>@<版>」の形でない"
                ),
            ));
        };
        // エンジンが持つスキーマは、--skill を渡さなくても引ける
        if contract::builtin_file(value).is_some() {
            return Ok(value.to_owned());
        }
        let refuse = |detail: String| UseCaseError::new("スキーマの名前を引けない", detail);
        let Some(dir) = &self.skill else {
            return Err(refuse(format!(
                "{value} を引くには、--skill に Skill {owner} のフォルダを渡す"
            )));
        };
        let skill = dir.rsplit(['/', '\\']).next().unwrap_or(dir);
        if owner != skill {
            return Err(refuse(format!(
                "{value} は Skill {owner} のスキーマで、Skill {skill} のものではない。Skill {owner} を --skill に渡して検査する"
            )));
        }
        let assets = format!("{dir}/assets");
        let mut found = Vec::new();
        for path in self
            .files
            .list_tree(&assets)
            .map_err(|error| UseCaseError::new("読めない", error.0))?
        {
            if !path.ends_with(".schema.json") {
                continue;
            }
            let (_, schema) = self.read_json(&path)?;
            if schema.get("$id").and_then(Value::as_str) == Some(value) {
                found.push(path);
            }
        }
        match found.len() {
            1 => Ok(found.remove(0)),
            0 => Err(refuse(format!(
                "{value} を $id に持つスキーマが、Skill {skill} の assets に無い"
            ))),
            _ => {
                let names: Vec<&str> = found
                    .iter()
                    .map(|path| path.rsplit('/').next().unwrap_or(path))
                    .collect();
                Err(UseCaseError::new(
                    "同じ名前のスキーマが2つ以上ある",
                    format!("{value}：{}", names.join(" ・ ")),
                ))
            }
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
        if let (Some(file), Some(root)) = (
            contract::builtin_file(path),
            contract::abstract_schema(path),
        ) {
            return Ok(Schema::new(file, root, contract::builtin_files()));
        }
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
        // インスタンスにはスキーマの名前（$id）を書く。名前を持たないスキーマからは作らない
        let stored = match loaded.root().get("$id").and_then(Value::as_str) {
            Some(name) if schema_name::is_name(name) => SchemaPath::new(name).map_err(invalid)?,
            _ => {
                return Err(UseCaseError::new(
                    "スキーマの名前が無い",
                    format!(
                        "{} の $id に「<Skill の名前>/<種類>@<版>」の名前を書く",
                        schema_path.as_str()
                    ),
                ))
            }
        };
        // abstract から作るのは concrete スキーマで、$schema に dialect を、x-abstract に版を書く
        let instance = if stored.as_str() == contract::V1 {
            if self.files.exists(path) {
                return Err(reject(Reject::AlreadyExists));
            }
            let start =
                serde_json::json!({"$schema": contract::DIALECT, "x-abstract": contract::V1});
            Instance::load(instance_path, stored, JsonValue::new(&start.to_string()))
        } else {
            Instance::create(instance_path, stored, self.files.exists(path)).map_err(reject)?
        };
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
        let schema = contract::schema_of(&value).ok_or_else(|| {
            UseCaseError::new("スキーマが分からない", "インスタンスに $schema が無い")
        })?;
        let schema = self.schema_path(path, schema)?;
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
