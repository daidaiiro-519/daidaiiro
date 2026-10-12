//! 集約 承認記録。ディレクトリと、承認したインスタンス（パスとハッシュ値の組）を持つ。
//! 不変条件：承認したインスタンスは1件以上。

use crate::domain::check::Approved;
use crate::domain::values::{
    ApprovedInstance, Drift, Hash, InstancePath, Unfilled, ValidationError,
};
use serde_json::{json, Value};
use std::fmt;

/// 承認を拒んだ理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalReject {
    /// 検証を通過しないインスタンスがある
    InvalidInstances,
    /// 参照と導出値のずれがある
    Drift,
    /// 未記入のプロパティがある
    Unfilled,
    /// 承認するインスタンスが無い
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

impl fmt::Display for ApprovalReject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason())
    }
}

impl std::error::Error for ApprovalReject {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalRecord {
    directory: InstancePath,
    instances: Vec<ApprovedInstance>,
}

impl ApprovalRecord {
    /// 承認を記録する。検査で使ったパスとハッシュ値の並びを、そのまま記録する。
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

    /// 検査に渡す形（パス → ハッシュ値）。
    pub fn as_map(&self) -> Approved {
        self.instances
            .iter()
            .map(|instance| (instance.path().to_owned(), instance.hash().clone()))
            .collect()
    }

    /// 前の承認記録と、承認するインスタンスの並びが同じか（変化が無い）。
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

#[cfg(test)]
mod tests {
    // 集約 承認記録のテスト。テストの名前は、テスト条件の ID を含む。

    use crate::domain::approval::{ApprovalRecord, ApprovalReject};
    use crate::domain::check::Approved;
    use crate::domain::values::{
        ApprovedInstance, Drift, InstancePath, JsonValue, Unfilled, ValidationError,
    };

    fn approved(path: &str) -> ApprovedInstance {
        ApprovedInstance::new(path, JsonValue::new("{}").hash()).unwrap()
    }

    fn dir() -> InstancePath {
        InstancePath::new("declarations").unwrap()
    }

    // ── 集約 承認記録

    #[test]
    fn agg_2_cmd_1_ok_1_records_given_instances() {
        let record =
            ApprovalRecord::record(dir(), &[], &[], &[], vec![approved("declarations/a.json")])
                .unwrap();
        assert!(record.same_as(&[approved("declarations/a.json")]));
        assert!(!record.same_as(&[]));
    }

    #[test]
    fn agg_2_cmd_1_br_1_rejects_validation_errors() {
        let errors = vec![ValidationError::new("/name", "空である").unwrap()];
        let recorded = ApprovalRecord::record(
            dir(),
            &errors,
            &[],
            &[],
            vec![approved("declarations/a.json")],
        );
        assert_eq!(recorded.unwrap_err(), ApprovalReject::InvalidInstances);
        assert_eq!(
            ApprovalReject::InvalidInstances.reason(),
            "検証を通過しないインスタンスがある"
        );
    }

    #[test]
    fn agg_2_cmd_1_br_2_rejects_drift() {
        let drifts = vec![Drift::new("指す先がある", "UC-9")];
        let recorded = ApprovalRecord::record(
            dir(),
            &[],
            &drifts,
            &[],
            vec![approved("declarations/a.json")],
        );
        assert_eq!(recorded.unwrap_err(), ApprovalReject::Drift);
        assert_eq!(ApprovalReject::Drift.reason(), "参照と導出値のずれがある");
    }

    #[test]
    fn agg_2_cmd_1_br_3_rejects_unfilled() {
        let unfilled = vec![Unfilled::new("/name").unwrap()];
        let recorded = ApprovalRecord::record(
            dir(),
            &[],
            &[],
            &unfilled,
            vec![approved("declarations/a.json")],
        );
        assert_eq!(recorded.unwrap_err(), ApprovalReject::Unfilled);
        assert_eq!(
            ApprovalReject::Unfilled.reason(),
            "未記入のプロパティがある"
        );
    }

    #[test]
    fn agg_2_inv_1_needs_at_least_one_instance() {
        let recorded = ApprovalRecord::record(dir(), &[], &[], &[], vec![]);
        assert_eq!(recorded.unwrap_err(), ApprovalReject::Empty);
    }

    #[test]
    fn approval_record_round_trips_as_json() {
        let record =
            ApprovalRecord::record(dir(), &[], &[], &[], vec![approved("declarations/a.json")])
                .unwrap();
        let back = ApprovalRecord::from_json(&record.to_json()).unwrap();
        assert_eq!(back, record);
        let map: Approved = back.as_map();
        assert!(map.contains_key("declarations/a.json"));
    }

    // 基盤のエラー型は、Display と std::error::Error を実装し、句点で終えない（C-GOOD-ERR）。
    #[test]
    fn approval_rejects_describe_themselves_without_punctuation() {
        assert_eq!(
            Box::<dyn std::error::Error>::from(ApprovalReject::Drift).to_string(),
            "参照と導出値のずれがある"
        );
        let empty = Box::<dyn std::error::Error>::from(ApprovalReject::Empty).to_string();
        assert!(!empty.ends_with(['。', '.']), "{empty}");
    }
}
