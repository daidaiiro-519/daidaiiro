// SPDX-License-Identifier: MIT
//! 契約の検査を事例で検証する。
//!
//!     cargo test -p nms_business_logic

use std::path::PathBuf;

use nms_business_logic::validate::{self, Contracts, Kind};

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

fn contracts() -> Contracts {
    let r = references();
    Contracts::new(
        r.join("rules.schema.json"),
        r.join("concepts.schema.json"),
        r.join("schema-meta.schema.json"),
    )
}

fn temp(name: &str, body: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nms-validate-{name}"));
    std::fs::create_dir_all(&dir).expect("作れる");
    let path = dir.join("x.json");
    std::fs::write(&path, body).expect("書ける");
    path
}

const OUTER: &str = r#"{"rules":[{"rule":"外を指す","source":{"meta":"x","authority":"spec",
 "quote":"y","fetched":{"url":"https://example.invalid/","date":"2026-09-26",
 "sha256":"0000000000000000000000000000000000000000000000000000000000000000","line":1}},
 "scope":"x","check":{"tool":["true"]}}]}"#;

#[test]
fn a_complete_outer_rule_passes() {
    let path = temp("ok", OUTER);
    let found = validate::check_rules(&path, &contracts()).expect("検査できる");
    assert!(found.is_empty(), "食い違いが出た ── {found:?}");
}

#[test]
fn an_outer_rule_needs_the_record_of_the_fetch() {
    let body = r#"{"rules":[{"rule":"外","source":{"meta":"x","authority":"spec","quote":"y"},
     "scope":"x","check":{"tool":["true"]}}]}"#;
    let path = temp("fetched", body);
    let found = validate::check_rules(&path, &contracts()).expect("検査できる");
    assert!(
        found.iter().any(|x| x.contains("fetched")),
        "取得の記録を要求する ── {found:?}"
    );
}

#[test]
fn an_outer_rule_needs_the_standing_of_the_source() {
    let body = OUTER.replace(r#""authority":"spec","#, "");
    let path = temp("authority", &body);
    let found = validate::check_rules(&path, &contracts()).expect("検査できる");
    assert!(
        found.iter().any(|x| x.contains("authority")),
        "立場を要求する ── {found:?}"
    );
}

#[test]
fn a_standing_outside_the_allowed_values_is_refused() {
    let body = OUTER.replace(r#""authority":"spec""#, r#""authority":"公式""#);
    let path = temp("authority-bad", &body);
    let found = validate::check_rules(&path, &contracts()).expect("検査できる");
    assert!(
        found.iter().any(|x| x.contains("authority")),
        "決められた値の外を断る ── {found:?}"
    );
}

#[test]
fn an_inner_rule_does_not_need_the_fetch() {
    let body = r#"{"rules":[{"rule":"内","source":{"record":"model"},"scope":"x",
     "check":{"tool":["true"]}}]}"#;
    let path = temp("inner", body);
    let found = validate::check_rules(&path, &contracts()).expect("検査できる");
    assert!(found.is_empty(), "内を指す規則には要求しない ── {found:?}");
}

#[test]
fn a_rule_without_a_tool_is_refused() {
    let body = r#"{"rules":[{"rule":"道具なし","source":{"record":"x"},"scope":"x","check":{}}]}"#;
    let path = temp("no-tool", body);
    let found = validate::check_rules(&path, &contracts()).expect("検査できる");
    assert!(found.iter().any(|x| x.contains("道具が無い")), "{found:?}");
}

#[test]
fn a_tool_that_is_not_a_list_is_refused() {
    let body = r#"{"rules":[{"rule":"文字列","source":{"record":"x"},"scope":"x",
     "check":{"tool":"true"}}]}"#;
    let path = temp("tool-str", body);
    let found = validate::check_rules(&path, &contracts()).expect("検査できる");
    assert!(
        found.iter().any(|x| x.contains("配列ではない")),
        "{found:?}"
    );
}

#[test]
fn a_rule_without_a_source_is_refused() {
    let body = r#"{"rules":[{"rule":"出典なし","scope":"x","check":{"tool":["true"]}}]}"#;
    let path = temp("no-source", body);
    let found = validate::check_rules(&path, &contracts()).expect("検査できる");
    assert!(found.iter().any(|x| x.contains("出典が無い")), "{found:?}");
}

#[test]
fn the_kind_is_decided_by_the_contents() {
    let rules = temp("kind-rules", r#"{"rules":[]}"#);
    assert_eq!(validate::kind_of(&rules).expect("読める"), Kind::Rules);
    let concepts = temp("kind-concepts", r#"{"concepts":[]}"#);
    assert_eq!(
        validate::kind_of(&concepts).expect("読める"),
        Kind::Concepts
    );
    let schema = temp("kind-schema", r#"{"properties":{}}"#);
    assert_eq!(validate::kind_of(&schema).expect("読める"), Kind::Schema);
}

#[test]
fn an_unreadable_file_is_a_misuse() {
    let broken = temp("broken", "{ not json");
    assert!(
        validate::unreadable(&broken).is_some(),
        "読めないことを返す"
    );
    let fine = temp("fine", r#"{"rules":[]}"#);
    assert!(validate::unreadable(&fine).is_none());
}

#[test]
fn an_unresolved_schema_reference_is_found() {
    let path = temp("schema-ref", r#"{"$schema":"../無い.json","rules":[]}"#);
    assert!(
        validate::unresolved_schema(&path).is_some(),
        "解決しない参照を検出する"
    );
    let path = temp(
        "schema-http",
        r#"{"$schema":"https://example.invalid/x","rules":[]}"#,
    );
    assert!(
        validate::unresolved_schema(&path).is_none(),
        "外を指す参照は見ない"
    );
}

#[test]
fn the_real_contracts_pass_their_own_checks() {
    let r = references();
    let c = contracts();
    for name in [
        "rules.schema.json",
        "concepts.schema.json",
        "schema-meta.schema.json",
    ] {
        let found = validate::check_schema(&r.join(name), &c).expect("検査できる");
        assert!(
            found.is_empty(),
            "{name} が案内の契約を満たさない ── {found:?}"
        );
    }
    let found = validate::check_concepts(&r.join("concepts.json"), &c).expect("検査できる");
    assert!(
        found.is_empty(),
        "概念の出典が契約を満たさない ── {found:?}"
    );
    let found = validate::check_rules(&r.join("self-rules.json"), &c).expect("検査できる");
    assert!(
        found.is_empty(),
        "自己の規則が契約を満たさない ── {found:?}"
    );
}
