//! 検査と承認のユースケース（UC-4 ・ 5 ・ 8）。ディレクトリのインスタンスを読み、ドメインサービス 検査する と
//! 集約 承認記録 を使う。

use crate::application::instances::{Instances, UseCaseError};
use crate::application::paths;
use crate::domain::approval::ApprovalRecord;
use crate::domain::check::{check, Doc, Finding};
use crate::domain::schema::Schema;
use crate::domain::schema_rules::{check_schemas, SchemaFinding};
use crate::domain::values::{
    ApprovedInstance, Drift, InstancePath, JsonValue, Status, Unfilled, ValidationError,
};
use crate::ports::inbound::CheckUseCases;
use crate::ports::outbound::{Files, Query, Schemas, WriteIf};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

/// 承認記録のファイルの名前。検査するディレクトリの中に置き、インスタンスとしては読まない。
pub const APPROVAL_FILE: &str = "approval.json";

/// インスタンス1件の検証結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Validated {
    pub path: String,
    pub hash: String,
    pub errors: Vec<ValidationError>,
    pub unfilled: Vec<Unfilled>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Checked {
    pub instances: Vec<Validated>,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Approved {
    pub changed: bool,
    pub instances: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Deleted {
    pub remaining: Vec<Finding>,
}

/// 具体のスキーマの検査の結果（ボード schema-driven-build の論点6 D）。
#[derive(Debug, Clone, PartialEq)]
pub struct SchemasChecked {
    /// 検査したスキーマのファイルの名前
    pub schemas: Vec<String>,
    pub findings: Vec<SchemaFinding>,
}

pub struct Checks {
    files: Arc<dyn Files>,
    query: Arc<dyn Query>,
    instances: Instances,
}

pub(crate) struct Loaded {
    pub(crate) path: String,
    pub(crate) value: Value,
    pub(crate) text: String,
    pub(crate) schema: String,
}

impl Checks {
    pub fn new(files: Arc<dyn Files>, schemas: Arc<dyn Schemas>, query: Arc<dyn Query>) -> Self {
        Self {
            instances: Instances::new(files.clone(), schemas, query.clone()),
            files,
            query,
        }
    }

    pub(crate) fn load_dir(
        &self,
        dir: &str,
    ) -> Result<(Vec<Loaded>, BTreeMap<String, Schema>), UseCaseError> {
        let list = self
            .files
            .list(dir)
            .map_err(|error| UseCaseError::new("読めない", error.0))?;
        let mut loaded = Vec::new();
        let mut schemas = BTreeMap::new();
        for path in list {
            let name = path.rsplit('/').next().unwrap_or(&path);
            if !name.ends_with(".json") || name.ends_with(".schema.json") || name == APPROVAL_FILE {
                continue;
            }
            let (text, value) = self.instances.read_json(&path)?;
            let relative = value
                .get("$schema")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    UseCaseError::new("スキーマが分からない", format!("{path} に $schema が無い"))
                })?;
            let schema = paths::join(paths::parent(&path), relative);
            if !schemas.contains_key(&schema) {
                let loaded_schema = self.instances.load_schema(&schema)?;
                schemas.insert(schema.clone(), loaded_schema);
            }
            loaded.push(Loaded {
                path,
                value,
                text,
                schema,
            });
        }
        Ok((loaded, schemas))
    }

    fn approval(&self, dir: &str) -> Result<Option<(ApprovalRecord, String)>, UseCaseError> {
        let path = format!("{}/{APPROVAL_FILE}", dir.trim_end_matches('/'));
        if !self.files.exists(&path) {
            return Ok(None);
        }
        let text = self
            .files
            .read(&path)
            .map_err(|error| UseCaseError::new("読めない", error.0))?;
        let record = ApprovalRecord::from_json(&text).ok_or_else(|| {
            UseCaseError::new(
                "JSON として読めない",
                format!("{path} が承認記録の形でない"),
            )
        })?;
        Ok(Some((record, text)))
    }

    fn run(&self, dir: &str) -> Result<(Checked, Option<(ApprovalRecord, String)>), UseCaseError> {
        let (loaded, schemas) = self.load_dir(dir)?;
        let approval = self.approval(dir)?;
        let mut validated = Vec::new();
        for instance in &loaded {
            let validation = schemas[&instance.schema]
                .validate(&instance.value)
                .map_err(|error| UseCaseError::new("スキーマが壊れている", error.0))?;
            validated.push(Validated {
                path: instance.path.clone(),
                hash: JsonValue::new(&instance.text).hash().as_str().to_owned(),
                errors: validation.errors,
                unfilled: validation.unfilled,
            });
        }
        let docs: Vec<Doc> = loaded
            .iter()
            .map(|instance| Doc {
                path: instance.path.clone(),
                value: instance.value.clone(),
                hash: JsonValue::new(&instance.text).hash(),
                schema: &schemas[&instance.schema],
            })
            .collect();
        let map = approval.as_ref().map(|(record, _)| record.as_map());
        let findings = check(&docs, map.as_ref());
        Ok((
            Checked {
                instances: validated,
                findings,
            },
            approval,
        ))
    }
}

impl CheckUseCases for Checks {
    fn check(&self, dir: &str) -> Result<Checked, UseCaseError> {
        self.run(dir).map(|(checked, _)| checked)
    }

    fn check_schemas(&self, dir: &str) -> Result<SchemasChecked, UseCaseError> {
        let list = self
            .files
            .list(dir)
            .map_err(|error| UseCaseError::new("読めない", error.0))?;
        let mut schemas = Vec::new();
        for path in list {
            let name = path.rsplit('/').next().unwrap_or(&path).to_owned();
            if !name.ends_with(".schema.json") {
                continue;
            }
            let (_, value) = self.instances.read_json(&path)?;
            schemas.push((name, value));
        }
        let findings = check_schemas(&schemas, &|expression: &str| {
            self.query.parse(expression).map_err(|error| error.0)
        });
        Ok(SchemasChecked {
            schemas: schemas.into_iter().map(|(name, _)| name).collect(),
            findings,
        })
    }

    fn approve(&self, dir: &str) -> Result<Approved, UseCaseError> {
        let directory = InstancePath::new(dir)
            .map_err(|error| UseCaseError::new("値が不正である", error.to_string()))?;
        let (checked, previous) = self.run(dir)?;
        let errors: Vec<ValidationError> = checked
            .instances
            .iter()
            .flat_map(|instance| instance.errors.clone())
            .collect();
        let unfilled: Vec<Unfilled> = checked
            .instances
            .iter()
            .flat_map(|instance| instance.unfilled.clone())
            .collect();
        let drifts: Vec<Drift> = checked
            .findings
            .iter()
            .filter(|finding| finding.status == Status::Drift)
            .map(|finding| Drift::new(&finding.check, &finding.to))
            .collect();
        let mut list = Vec::new();
        for instance in &checked.instances {
            let hash = crate::domain::values::Hash::new(&instance.hash)
                .map_err(|error| UseCaseError::new("値が不正である", error.to_string()))?;
            list.push(
                ApprovedInstance::new(&instance.path, hash)
                    .map_err(|error| UseCaseError::new("値が不正である", error.to_string()))?,
            );
        }
        let pairs: Vec<(String, String)> = list
            .iter()
            .map(|approved| {
                (
                    approved.path().to_owned(),
                    approved.hash().as_str().to_owned(),
                )
            })
            .collect();
        if let Some((record, _)) = &previous {
            if record.same_as(&list)
                && errors.is_empty()
                && drifts.is_empty()
                && unfilled.is_empty()
            {
                return Ok(Approved {
                    changed: false,
                    instances: pairs,
                });
            }
        }
        let record = ApprovalRecord::record(directory, &errors, &drifts, &unfilled, list).map_err(
            |reject| {
                let detail = match reject {
                    crate::domain::approval::ApprovalReject::Drift => drifts
                        .iter()
                        .map(|drift| format!("{}：{}", drift.check(), drift.target()))
                        .collect::<Vec<_>>()
                        .join("、"),
                    _ => String::new(),
                };
                UseCaseError::new(reject.reason(), detail)
            },
        )?;
        let path = format!("{}/{APPROVAL_FILE}", dir.trim_end_matches('/'));
        let cond = match &previous {
            Some((_, text)) => WriteIf::Unchanged(JsonValue::new(text).hash()),
            None => WriteIf::Absent,
        };
        self.files
            .write(&path, &record.to_json(), cond)
            .map_err(Instances::write_error)?;
        Ok(Approved {
            changed: true,
            instances: pairs,
        })
    }

    fn delete(&self, path: &str) -> Result<Deleted, UseCaseError> {
        InstancePath::new(path)
            .map_err(|error| UseCaseError::new("値が不正である", error.to_string()))?;
        let (_, value) = self.instances.read_json(path)?;
        let id = value.get("id").and_then(Value::as_str).map(str::to_owned);
        self.files.remove(path).map_err(Instances::write_error)?;
        let (checked, _) = self.run(paths::parent(path))?;
        let remaining = match id {
            Some(id) => checked
                .findings
                .into_iter()
                .filter(|finding| {
                    finding.check == "指す先がある"
                        && finding.status == Status::Drift
                        && (finding.to == id || finding.to.starts_with(&format!("{id}.")))
                })
                .collect(),
            None => Vec::new(),
        };
        Ok(Deleted { remaining })
    }
}
