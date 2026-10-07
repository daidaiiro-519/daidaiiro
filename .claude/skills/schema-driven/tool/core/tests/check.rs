//! ドメインサービス 検査する のテスト（テストレベル component）。
//! references/annotations.schema.json の各キーについて、合格とずれの両方を確かめる。

use schema_driven_core::domain::check::{check, Approved, Doc, Finding, Status};
use schema_driven_core::domain::schema::Schema;
use schema_driven_core::domain::values::JsonValue;
use serde_json::{json, Value};

fn doc<'schema>(path: &str, value: Value, schema: &'schema Schema) -> Doc<'schema> {
    let hash = JsonValue::new(&value.to_string()).hash();
    Doc {
        path: path.to_owned(),
        value,
        hash,
        schema,
    }
}

fn schema(name: &str, root: Value) -> Schema {
    Schema::new(name, root, Vec::new())
}

fn of<'findings>(findings: &'findings [Finding], check: &str) -> Vec<&'findings Finding> {
    findings
        .iter()
        .filter(|finding| finding.check == check)
        .collect()
}

fn statuses(findings: &[Finding], check: &str) -> Vec<Status> {
    of(findings, check)
        .iter()
        .map(|finding| finding.status)
        .collect()
}

// 指す先のスキーマ：kind が term の用語集（項目 terms）と、kind が rule のルール
fn glossary() -> Schema {
    schema("glossary.schema.json", json!({"type": "object"}))
}

#[test]
fn to_finds_instance_by_id_and_reports_missing() {
    let referring_schema = schema(
        "a.schema.json",
        json!({"properties": {"uses": {"x-ref": {"to": "rule"}}}}),
    );
    let target_schema = schema("rule.schema.json", json!({}));
    let docs = vec![
        doc(
            "a.json",
            json!({"id": "A-1", "kind": "a", "uses": ["R-1", "R-9"]}),
            &referring_schema,
        ),
        doc(
            "r.json",
            json!({"id": "R-1", "kind": "rule"}),
            &target_schema,
        ),
    ];
    let findings = check(&docs, None);
    assert_eq!(
        statuses(&findings, "指す先がある"),
        vec![Status::Pass, Status::Drift]
    );
    assert_eq!(of(&findings, "指す先がある")[1].message, "指す先が無い");
}

#[test]
fn item_finds_item_inside_instance() {
    let referring_schema = schema(
        "a.schema.json",
        json!({"properties": {"impl": {"x-ref": {"to": "req", "item": true, "in": "rules"}}}}),
    );
    let target_schema = schema("req.schema.json", json!({}));
    let docs = vec![
        doc(
            "a.json",
            json!({"id": "A-1", "kind": "a", "impl": ["REQ-1.BR-1", "REQ-1.BR-2", "REQ-1"]}),
            &referring_schema,
        ),
        doc(
            "r.json",
            json!({"id": "REQ-1", "kind": "req", "rules": [{"id": "BR-1"}]}),
            &target_schema,
        ),
    ];
    let findings = check(&docs, None);
    let found = of(&findings, "指す先がある");
    let get = |to: &str| found.iter().find(|finding| finding.to == to).unwrap();
    assert_eq!(get("REQ-1.BR-1").status, Status::Pass);
    assert_eq!(get("REQ-1.BR-2").message, "指す先が無い");
    assert!(get("REQ-1").message.contains("形が違う"));
}

#[test]
fn bare_rejects_ambiguous_item() {
    let referring_schema = schema(
        "a.schema.json",
        json!({"properties": {"v": {"x-ref": {"to": "dom", "bare": true, "in": "values"}}}}),
    );
    let target_schema = schema("dom.schema.json", json!({}));
    let docs = vec![
        doc(
            "a.json",
            json!({"id": "A-1", "kind": "a", "v": ["VAL-1", "VAL-2"]}),
            &referring_schema,
        ),
        doc(
            "d1.json",
            json!({"id": "D-1", "kind": "dom", "values": [{"id": "VAL-1"}, {"id": "VAL-2"}]}),
            &target_schema,
        ),
        doc(
            "d2.json",
            json!({"id": "D-2", "kind": "dom", "values": [{"id": "VAL-2"}]}),
            &target_schema,
        ),
    ];
    let findings = check(&docs, None);
    let found = of(&findings, "指す先がある");
    assert_eq!(
        found
            .iter()
            .map(|finding| finding.status)
            .collect::<Vec<_>>(),
        vec![Status::Pass, Status::Drift]
    );
    assert_eq!(found[1].message, "指す先が1つに決まらない");
}

#[test]
fn self_looks_inside_same_instance() {
    let referring_schema = schema(
        "a.schema.json",
        json!({"properties": {"invariants": {"items": {"properties": {"via": {"x-ref": {"to": "self", "in": "commands"}}}}}}}),
    );
    let docs = vec![doc(
        "a.json",
        json!({"id": "A-1", "kind": "a", "commands": [{"id": "CMD-1"}], "invariants": [{"id": "INV-1", "via": ["CMD-1", "CMD-2"]}]}),
        &referring_schema,
    )];
    let findings = check(&docs, None);
    assert_eq!(
        statuses(&findings, "指す先がある"),
        vec![Status::Pass, Status::Drift]
    );
}

#[test]
fn only_skips_values_that_are_not_references() {
    let referring_schema = schema(
        "a.schema.json",
        json!({"properties": {"type": {"x-ref": {"to": "vo", "only": "^VO-"}}}}),
    );
    let docs = vec![doc(
        "a.json",
        json!({"id": "A-1", "kind": "a", "type": "ID"}),
        &referring_schema,
    )];
    assert!(of(&check(&docs, None), "指す先がある").is_empty());
}

#[test]
fn accept_with_when_checks_target_kind() {
    let referring_schema = schema(
        "a.schema.json",
        json!({"properties": {"steps": {"items": {"properties": {"data": {"x-ref": {
            "to": "glossary", "bare": true, "in": "terms", "accept": {"at": "meanings/kind", "in": ["情報"]}, "when": {"kind": ["相互作用"]}}}}}}}}),
    );
    let glossary_schema = glossary();
    let docs = vec![
        doc(
            "a.json",
            json!({"id": "A-1", "kind": "a", "steps": [
                {"id": "S-1", "kind": "相互作用", "data": ["T-1"]},
                {"id": "S-2", "kind": "相互作用", "data": ["T-2"]},
                {"id": "S-3", "kind": "内部", "data": ["T-2"]}]}),
            &referring_schema,
        ),
        doc(
            "g.json",
            json!({"id": "G-1", "kind": "glossary", "terms": [
                {"id": "T-1", "meanings": [{"kind": "情報"}]},
                {"id": "T-2", "meanings": [{"kind": "動作"}]}]}),
            &glossary_schema,
        ),
    ];
    let findings = check(&docs, None);
    assert_eq!(
        statuses(&findings, "指す先が受け付ける値"),
        vec![Status::Pass, Status::Drift]
    );
}

#[test]
fn unique_rejects_repeated_value_in_same_array() {
    let referring_schema = schema(
        "a.schema.json",
        json!({"properties": {"uses": {"items": {"properties": {"term": {"x-ref": {"to": "glossary", "bare": true, "in": "terms", "unique": true}}}}}}}),
    );
    let glossary_schema = glossary();
    let docs = vec![
        doc(
            "a.json",
            json!({"id": "A-1", "kind": "a", "uses": [{"term": "T-1"}, {"term": "T-1"}]}),
            &referring_schema,
        ),
        doc(
            "g.json",
            json!({"id": "G-1", "kind": "glossary", "terms": [{"id": "T-1"}]}),
            &glossary_schema,
        ),
    ];
    assert_eq!(
        statuses(&check(&docs, None), "重ねて指さない"),
        vec![Status::Drift]
    );
}

#[test]
fn covered_by_requires_each_value_to_be_handled() {
    let referring_schema = schema(
        "a.schema.json",
        json!({"properties": {"steps": {"items": {"properties": {"checks": {"x-ref": {"to": "req", "item": true, "covered_by": "extensions/fails"}}}}}}}),
    );
    let target_schema = schema("req.schema.json", json!({}));
    let docs = vec![
        doc(
            "a.json",
            json!({"id": "A-1", "kind": "a", "steps": [{"id": "S-1", "checks": ["R-1.B-1", "R-1.B-2"], "extensions": [{"fails": ["R-1.B-1"]}]}]}),
            &referring_schema,
        ),
        doc(
            "r.json",
            json!({"id": "R-1", "kind": "req", "b": [{"id": "B-1"}, {"id": "B-2"}]}),
            &target_schema,
        ),
    ];
    assert_eq!(
        statuses(&check(&docs, None), "扱われている"),
        vec![Status::Pass, Status::Drift]
    );
}

#[test]
fn inverse_counts_min_max_and_shared_group_including_unreferenced() {
    let sd = schema(
        "sd.schema.json",
        json!({"properties": {"use_cases": {"x-ref": {"to": "uc",
        "inverse": {"group": "ユースケースを束ねるのは1つ", "min": 1, "max": 1, "where": {"level": ["ユーザー目的"]}}}}}}),
    );
    let uc = schema("uc.schema.json", json!({}));
    let docs = vec![
        doc(
            "sd1.json",
            json!({"id": "SD-1", "kind": "sd", "use_cases": ["UC-1", "UC-2"]}),
            &sd,
        ),
        doc(
            "sd2.json",
            json!({"id": "SD-2", "kind": "sd", "use_cases": ["UC-1"]}),
            &sd,
        ),
        doc(
            "uc1.json",
            json!({"id": "UC-1", "kind": "uc", "level": "ユーザー目的"}),
            &uc,
        ),
        doc(
            "uc2.json",
            json!({"id": "UC-2", "kind": "uc", "level": "ユーザー目的"}),
            &uc,
        ),
        doc(
            "uc3.json",
            json!({"id": "UC-3", "kind": "uc", "level": "ユーザー目的"}),
            &uc,
        ),
        doc(
            "uc0.json",
            json!({"id": "UC-0", "kind": "uc", "level": "要約"}),
            &uc,
        ),
    ];
    let findings = check(&docs, None);
    let group = of(&findings, "ユースケースを束ねるのは1つ");
    let by: Vec<(&str, Status)> = group
        .iter()
        .map(|finding| (finding.to.as_str(), finding.status))
        .collect();
    assert!(by.contains(&("UC-1", Status::Drift)), "{by:?}");
    assert!(by.contains(&("UC-2", Status::Pass)));
    assert!(
        by.contains(&("UC-3", Status::Drift)),
        "参照が0件の候補も数える"
    );
    assert!(
        !by.iter().any(|(target, _)| *target == "UC-0"),
        "where で外す"
    );
}

#[test]
fn inverse_max_instances_counts_instances_not_references() {
    let referring_schema = schema(
        "agg.schema.json",
        json!({"properties": {"rejects": {"x-ref": {"to": "glossary", "bare": true, "in": "terms",
        "inverse": {"group": "拒否の理由を使う集約は1つ", "max_instances": 1}}}}}),
    );
    let glossary_schema = glossary();
    let docs = vec![
        doc(
            "a1.json",
            json!({"id": "AGG-1", "kind": "agg", "rejects": ["T-1", "T-1"]}),
            &referring_schema,
        ),
        doc(
            "a2.json",
            json!({"id": "AGG-2", "kind": "agg", "rejects": ["T-2"]}),
            &referring_schema,
        ),
        doc(
            "a3.json",
            json!({"id": "AGG-3", "kind": "agg", "rejects": ["T-2"]}),
            &referring_schema,
        ),
        doc(
            "g.json",
            json!({"id": "G-1", "kind": "glossary", "terms": [{"id": "T-1"}, {"id": "T-2"}]}),
            &glossary_schema,
        ),
    ];
    let findings = check(&docs, None);
    let by: Vec<(&str, Status)> = of(&findings, "拒否の理由を使う集約は1つ")
        .iter()
        .map(|finding| (finding.to.as_str(), finding.status))
        .collect();
    assert!(by.contains(&("G-1.T-1", Status::Pass)), "{by:?}");
    assert!(by.contains(&("G-1.T-2", Status::Drift)));
}

#[test]
fn x_derive_compares_with_declared_and_expect() {
    let sd = schema(
        "sd.schema.json",
        json!({"properties": {
            "classification": {"x-derive": {"rules": [{"when": {"advantage": true}, "then": "中核"}], "otherwise": "補完", "declared": "category"}},
            "logic": {"x-derive": {"rules": [{"when": {"complex": true}, "then": "ドメインモデル"}], "otherwise": "トランザクションスクリプト",
                "expect": [{"name": "カテゴリーと実装方法", "when": {"classification/category": ["中核"]}, "in": ["ドメインモデル"]}]}}}}),
    );
    let docs = vec![
        doc(
            "1.json",
            json!({"id": "SD-1", "kind": "sd", "classification": {"advantage": true, "category": "中核"}, "logic": {"complex": true}}),
            &sd,
        ),
        doc(
            "2.json",
            json!({"id": "SD-2", "kind": "sd", "classification": {"advantage": false, "category": "中核"}, "logic": {"complex": false}}),
            &sd,
        ),
    ];
    let findings = check(&docs, None);
    assert_eq!(
        statuses(&findings, "導出値と宣言した値"),
        vec![Status::Pass, Status::Drift]
    );
    assert_eq!(
        statuses(&findings, "カテゴリーと実装方法"),
        vec![Status::Pass, Status::Drift]
    );
}

#[test]
fn inverse_where_derive_filters_by_derived_value_of_referenced_instance() {
    let sd = schema(
        "sd.schema.json",
        json!({"properties": {"logic": {"x-derive": {"rules": [{"when": {"complex": true}, "then": "DM"}], "otherwise": "TS"}}}}),
    );
    let bc = schema("bc.schema.json", json!({}));
    let agg = schema(
        "agg.schema.json",
        json!({"properties": {"context": {"x-ref": {"to": "bc",
        "inverse": {"group": "ドメインモデルの文脈に集約がある", "min": 1, "where_derive": {"via": "subdomains", "derive": "logic", "in": ["DM"]}}}}}}),
    );
    let docs = vec![
        doc(
            "sd1.json",
            json!({"id": "SD-1", "kind": "sd", "logic": {"complex": true}}),
            &sd,
        ),
        doc(
            "sd2.json",
            json!({"id": "SD-2", "kind": "sd", "logic": {"complex": false}}),
            &sd,
        ),
        doc(
            "bc1.json",
            json!({"id": "BC-1", "kind": "bc", "subdomains": ["SD-1"]}),
            &bc,
        ),
        doc(
            "bc2.json",
            json!({"id": "BC-2", "kind": "bc", "subdomains": ["SD-2"]}),
            &bc,
        ),
        doc("a.json", json!({"id": "AGG-0", "kind": "agg"}), &agg),
    ];
    let findings = check(&docs, None);
    let by: Vec<(&str, Status)> = of(&findings, "ドメインモデルの文脈に集約がある")
        .iter()
        .map(|finding| (finding.to.as_str(), finding.status))
        .collect();
    assert_eq!(
        by,
        vec![("BC-1", Status::Drift)],
        "BC-2 はトランザクションスクリプトなので数えない"
    );
}

#[test]
fn approval_reports_change_and_unapproved() {
    let referring_schema = schema(
        "a.schema.json",
        json!({"properties": {"uses": {"x-ref": {"to": "rule"}}}}),
    );
    let target_schema = schema("rule.schema.json", json!({}));
    let docs = vec![
        doc(
            "a.json",
            json!({"id": "A-1", "kind": "a", "uses": ["R-1", "R-2"]}),
            &referring_schema,
        ),
        doc(
            "r1.json",
            json!({"id": "R-1", "kind": "rule"}),
            &target_schema,
        ),
        doc(
            "r2.json",
            json!({"id": "R-2", "kind": "rule", "x": 1}),
            &target_schema,
        ),
    ];
    let mut approved = Approved::new();
    approved.insert("r1.json".into(), docs[1].hash.clone());
    let findings = check(&docs, Some(&approved));
    let changes = of(&findings, "承認のあとの変化");
    assert_eq!(changes[0].status, Status::Pass);
    assert_eq!(changes[1].status, Status::Recheck);
    assert_eq!(changes[1].message, "まだ承認していない");
    let mut changed = approved.clone();
    changed.insert("r1.json".into(), JsonValue::new("{}").hash());
    let findings = check(&docs, Some(&changed));
    assert_eq!(
        of(&findings, "承認のあとの変化")[0].message,
        "承認のあとで指す先が変わった"
    );
}
