//! 検査と承認のユースケース（UC-4 ・ 5 ・ 8）。ディレクトリのインスタンスを読み、ドメインサービス 検査する と
//! 集約 承認記録 を使う。

use crate::application::instances::{Instances, UseCaseError};
use crate::application::paths;
use crate::domain::approval::ApprovalRecord;
use crate::domain::check::{check, Doc, Finding};
use crate::domain::schema::Schema;
use crate::domain::values::{
    ApprovedInstance, Drift, InstancePath, JsonValue, Status, Unfilled, ValidationError,
};
use crate::ports::inbound::CheckUseCases;
use crate::ports::outbound::{Files, Query, Schemas, WriteIf};
use serde_json::Value;
use std::collections::BTreeMap;

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

pub struct Checks<'a> {
    files: &'a dyn Files,
    instances: Instances<'a>,
}

struct Loaded {
    path: String,
    value: Value,
    text: String,
    schema: String,
}

impl<'a> Checks<'a> {
    pub fn new(files: &'a dyn Files, schemas: &'a dyn Schemas, query: &'a dyn Query) -> Self {
        Self {
            files,
            instances: Instances::new(files, schemas, query),
        }
    }

    fn load_dir(&self, dir: &str) -> Result<(Vec<Loaded>, BTreeMap<String, Schema>), UseCaseError> {
        let list = self
            .files
            .list(dir)
            .map_err(|e| UseCaseError::new("読めない", e.0))?;
        let mut loaded = Vec::new();
        let mut schemas = BTreeMap::new();
        for path in list {
            let name = path.rsplit('/').next().unwrap_or(&path);
            if !name.ends_with(".json") || name.ends_with(".schema.json") || name == APPROVAL_FILE {
                continue;
            }
            let (text, value) = self.instances.read_json(&path)?;
            let rel = value
                .get("$schema")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    UseCaseError::new("スキーマが分からない", format!("{path} に $schema が無い"))
                })?;
            let schema = paths::join(paths::parent(&path), rel);
            if !schemas.contains_key(&schema) {
                let s = self.instances.load_schema(&schema)?;
                schemas.insert(schema.clone(), s);
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
            .map_err(|e| UseCaseError::new("読めない", e.0))?;
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
        for l in &loaded {
            let v = schemas[&l.schema]
                .validate(&l.value)
                .map_err(|e| UseCaseError::new("スキーマが壊れている", e.0))?;
            validated.push(Validated {
                path: l.path.clone(),
                hash: JsonValue::new(&l.text).hash().as_str().to_owned(),
                errors: v.errors,
                unfilled: v.unfilled,
            });
        }
        let docs: Vec<Doc> = loaded
            .iter()
            .map(|l| Doc {
                path: l.path.clone(),
                value: l.value.clone(),
                hash: JsonValue::new(&l.text).hash(),
                schema: &schemas[&l.schema],
            })
            .collect();
        let map = approval.as_ref().map(|(r, _)| r.as_map());
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

impl CheckUseCases for Checks<'_> {
    fn check(&self, dir: &str) -> Result<Checked, UseCaseError> {
        self.run(dir).map(|(c, _)| c)
    }

    fn approve(&self, dir: &str) -> Result<Approved, UseCaseError> {
        let directory = InstancePath::new(dir)
            .map_err(|e| UseCaseError::new("値が不正である", e.to_string()))?;
        let (checked, previous) = self.run(dir)?;
        let errors: Vec<ValidationError> = checked
            .instances
            .iter()
            .flat_map(|i| i.errors.clone())
            .collect();
        let unfilled: Vec<Unfilled> = checked
            .instances
            .iter()
            .flat_map(|i| i.unfilled.clone())
            .collect();
        let drifts: Vec<Drift> = checked
            .findings
            .iter()
            .filter(|f| f.status == Status::Drift)
            .map(|f| Drift::new(&f.check, &f.to))
            .collect();
        let mut list = Vec::new();
        for i in &checked.instances {
            let hash = crate::domain::values::Hash::new(&i.hash)
                .map_err(|e| UseCaseError::new("値が不正である", e.to_string()))?;
            list.push(
                ApprovedInstance::new(&i.path, hash)
                    .map_err(|e| UseCaseError::new("値が不正である", e.to_string()))?,
            );
        }
        let pairs: Vec<(String, String)> = list
            .iter()
            .map(|i| (i.path().to_owned(), i.hash().as_str().to_owned()))
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
        let record =
            ApprovalRecord::record(directory, &errors, &drifts, &unfilled, list).map_err(|r| {
                let detail = match r {
                    crate::domain::approval::ApprovalReject::Drift => drifts
                        .iter()
                        .map(|d| format!("{}：{}", d.check(), d.target()))
                        .collect::<Vec<_>>()
                        .join("、"),
                    _ => String::new(),
                };
                UseCaseError::new(r.reason(), detail)
            })?;
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
        InstancePath::new(path).map_err(|e| UseCaseError::new("値が不正である", e.to_string()))?;
        let (_, value) = self.instances.read_json(path)?;
        let id = value.get("id").and_then(Value::as_str).map(str::to_owned);
        self.files.remove(path).map_err(Instances::write_error)?;
        let (checked, _) = self.run(paths::parent(path))?;
        let remaining = match id {
            Some(id) => checked
                .findings
                .into_iter()
                .filter(|f| {
                    f.check == "指す先がある"
                        && f.status == Status::Drift
                        && (f.to == id || f.to.starts_with(&format!("{id}.")))
                })
                .collect(),
            None => Vec::new(),
        };
        Ok(Deleted { remaining })
    }
}
