//! 集約 承認記録（AGG-2）・ 値オブジェクト VO-3 ・ 4 ・ 11 ・ ドメインサービス 検査する（DS-1）のテスト（テストレベル component）。
//! テストの名前は、テスト条件の ID を含む。

use schema_driven_core::domain::approval::{ApprovalRecord, ApprovalReject};
use schema_driven_core::domain::check::{check, Approved, Doc};
use schema_driven_core::domain::schema::Schema;
use schema_driven_core::domain::values::{
    ApprovedInstance, Derived, Drift, InstancePath, JsonValue, Reference, Status, Unfilled,
    ValidationError,
};
use serde_json::{json, Value};

fn approved(path: &str) -> ApprovedInstance {
    ApprovedInstance::new(path, JsonValue::new("{}").hash()).unwrap()
}

fn dir() -> InstancePath {
    InstancePath::new("decls").unwrap()
}

// ── 値オブジェクト

#[test]
fn vo_3_inv_1_rejects_empty_target() {
    assert!(Reference::new("", "UC-1", "use_case").is_err());
    assert!(Reference::new("UC-1", "UC-1", "use_case").is_ok());
}

#[test]
fn vo_4_op_1_ok_1_same_values_pass() {
    assert_eq!(
        Derived::new(json!("中核"), json!("中核")).compare(),
        Status::Pass
    );
}

#[test]
fn vo_4_op_1_ok_2_different_values_drift() {
    assert_eq!(
        Derived::new(json!("中核"), json!("補完")).compare(),
        Status::Drift
    );
}

#[test]
fn vo_11_inv_1_rejects_empty_path() {
    assert!(ApprovedInstance::new("", JsonValue::new("{}").hash()).is_err());
}

// ── 集約 承認記録

#[test]
fn agg_2_cmd_1_ok_1_records_given_instances() {
    let record =
        ApprovalRecord::record(dir(), &[], &[], &[], vec![approved("decls/a.json")]).unwrap();
    assert_eq!(record.instances().len(), 1);
    assert_eq!(record.instances()[0].path(), "decls/a.json");
}

#[test]
fn agg_2_cmd_1_br_1_rejects_validation_errors() {
    let errors = vec![ValidationError::new("/name", "空である").unwrap()];
    let r = ApprovalRecord::record(dir(), &errors, &[], &[], vec![approved("decls/a.json")]);
    assert_eq!(r.unwrap_err(), ApprovalReject::InvalidInstances);
    assert_eq!(
        ApprovalReject::InvalidInstances.reason(),
        "検証を通過しないインスタンスがある"
    );
}

#[test]
fn agg_2_cmd_1_br_2_rejects_drift() {
    let drifts = vec![Drift::new("指す先がある", "UC-9")];
    let r = ApprovalRecord::record(dir(), &[], &drifts, &[], vec![approved("decls/a.json")]);
    assert_eq!(r.unwrap_err(), ApprovalReject::Drift);
    assert_eq!(ApprovalReject::Drift.reason(), "参照と導出値のずれがある");
}

#[test]
fn agg_2_cmd_1_br_3_rejects_unfilled() {
    let unfilled = vec![Unfilled::new("/name").unwrap()];
    let r = ApprovalRecord::record(dir(), &[], &[], &unfilled, vec![approved("decls/a.json")]);
    assert_eq!(r.unwrap_err(), ApprovalReject::Unfilled);
    assert_eq!(
        ApprovalReject::Unfilled.reason(),
        "未記入のプロパティがある"
    );
}

#[test]
fn agg_2_inv_1_needs_at_least_one_instance() {
    let r = ApprovalRecord::record(dir(), &[], &[], &[], vec![]);
    assert_eq!(r.unwrap_err(), ApprovalReject::Empty);
}

#[test]
fn approval_record_round_trips_as_json() {
    let record =
        ApprovalRecord::record(dir(), &[], &[], &[], vec![approved("decls/a.json")]).unwrap();
    let back = ApprovalRecord::from_json(&record.to_json()).unwrap();
    assert_eq!(back, record);
    let map: Approved = back.as_map();
    assert!(map.contains_key("decls/a.json"));
}

// ── ドメインサービス 検査する

fn docs_with<'a>(s: &'a Schema, t: &'a Schema, refs: Value) -> Vec<Doc<'a>> {
    let a = json!({"id": "A-1", "kind": "a", "refs": refs});
    let b = json!({"id": "B-1", "kind": "b", "tag": "x"});
    vec![
        Doc {
            path: "a.json".into(),
            hash: JsonValue::new(&a.to_string()).hash(),
            value: a,
            schema: s,
        },
        Doc {
            path: "b.json".into(),
            hash: JsonValue::new(&b.to_string()).hash(),
            value: b,
            schema: t,
        },
    ]
}

fn schemas() -> (Schema, Schema) {
    (
        Schema::new(
            "a.schema.json",
            json!({"properties": {"refs": {"x-ref": {"to": "b", "accept": {"at": "tag", "in": ["x"]}}}}}),
            vec![],
        ),
        Schema::new("b.schema.json", json!({}), vec![]),
    )
}

#[test]
fn ds_1_op_1_res_3_missing_target_is_drift() {
    let (s, t) = schemas();
    let f = check(&docs_with(&s, &t, json!(["B-9"])), None);
    assert!(f
        .iter()
        .any(|x| x.check == "指す先がある" && x.status == Status::Drift));
}

#[test]
fn ds_1_op_1_res_4_wrong_target_kind_is_drift() {
    let (_, t) = schemas();
    let s = Schema::new(
        "a.schema.json",
        json!({"properties": {"refs": {"x-ref": {"to": "b", "accept": {"at": "tag", "in": ["y"]}}}}}),
        vec![],
    );
    let f = check(&docs_with(&s, &t, json!(["B-1"])), None);
    assert!(f
        .iter()
        .any(|x| x.check == "指す先が受け付ける値" && x.status == Status::Drift));
}

#[test]
fn ds_1_op_3_res_2_changed_since_approval_is_recheck() {
    let (s, t) = schemas();
    let docs = docs_with(&s, &t, json!(["B-1"]));
    let mut a = Approved::new();
    a.insert("b.json".into(), JsonValue::new("{}").hash());
    let f = check(&docs, Some(&a));
    assert!(f
        .iter()
        .any(|x| x.check == "承認のあとの変化" && x.status == Status::Recheck));
}

#[test]
fn ds_1_op_3_res_3_unchanged_since_approval_passes() {
    let (s, t) = schemas();
    let docs = docs_with(&s, &t, json!(["B-1"]));
    let mut a = Approved::new();
    a.insert("b.json".into(), docs[1].hash.clone());
    let f = check(&docs, Some(&a));
    assert!(f
        .iter()
        .any(|x| x.check == "承認のあとの変化" && x.status == Status::Pass));
}
