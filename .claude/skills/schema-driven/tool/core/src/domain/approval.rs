//! 集約 承認記録（AGG-2）。ディレクトリと、承認したインスタンス（パスとハッシュ値の組）を持つ。
//! INV-1：承認したインスタンスは1件以上。

use crate::domain::check::Approved;
use crate::domain::values::{
    ApprovedInstance, Drift, Hash, InstancePath, Unfilled, ValidationError,
};
use serde_json::{json, Value};

/// 承認を拒んだ理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalReject {
    /// BR-1：TERM-53 検証を通過しないインスタンスがある
    InvalidInstances,
    /// BR-2：TERM-55 参照と導出値のずれがある
    Drift,
    /// BR-3：TERM-65 未記入のプロパティがある
    Unfilled,
    /// INV-1：承認するインスタンスが無い
    Empty,
}

impl ApprovalReject {
    pub fn reason(&self) -> &'static str {
        match self {
            ApprovalReject::InvalidInstances => "検証を通過しないインスタンスがある",
            ApprovalReject::Drift => "参照と導出値のずれがある",
            ApprovalReject::Unfilled => "未記入のプロパティがある",
            ApprovalReject::Empty => "承認するインスタンスが無い",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalRecord {
    directory: InstancePath,
    instances: Vec<ApprovedInstance>,
}

impl ApprovalRecord {
    /// CMD-1 承認を記録する。検査で使ったパスとハッシュ値の並びを、そのまま記録する。
    pub fn record(
        directory: InstancePath,
        errors: &[ValidationError],
        drifts: &[Drift],
        unfilled: &[Unfilled],
        instances: Vec<ApprovedInstance>,
    ) -> Result<Self, ApprovalReject> {
        if !errors.is_empty() {
            return Err(ApprovalReject::InvalidInstances);
        }
        if !drifts.is_empty() {
            return Err(ApprovalReject::Drift);
        }
        if !unfilled.is_empty() {
            return Err(ApprovalReject::Unfilled);
        }
        if instances.is_empty() {
            return Err(ApprovalReject::Empty);
        }
        Ok(Self {
            directory,
            instances,
        })
    }

    pub fn directory(&self) -> &InstancePath {
        &self.directory
    }
    pub fn instances(&self) -> &[ApprovedInstance] {
        &self.instances
    }

    /// 検査に渡す形（パス → ハッシュ値）。
    pub fn as_map(&self) -> Approved {
        self.instances
            .iter()
            .map(|instance| (instance.path().to_owned(), instance.hash().clone()))
            .collect()
    }

    /// 前の承認記録と、承認するインスタンスの並びが同じか（UC-8 EXT-2：変化が無い）。
    pub fn same_as(&self, instances: &[ApprovedInstance]) -> bool {
        self.instances == instances
    }

    pub fn to_json(&self) -> String {
        let list: Vec<Value> = self
            .instances
            .iter()
            .map(|instance| json!({"path": instance.path(), "hash": instance.hash().as_str()}))
            .collect();
        json!({"directory": self.directory.as_str(), "instances": list}).to_string()
    }

    pub fn from_json(text: &str) -> Option<Self> {
        let record: Value = serde_json::from_str(text).ok()?;
        let directory = InstancePath::new(record.get("directory")?.as_str()?).ok()?;
        let mut instances = Vec::new();
        for entry in record.get("instances")?.as_array()? {
            let hash = Hash::new(entry.get("hash")?.as_str()?).ok()?;
            instances.push(ApprovedInstance::new(entry.get("path")?.as_str()?, hash).ok()?);
        }
        Some(Self {
            directory,
            instances,
        })
    }
}
