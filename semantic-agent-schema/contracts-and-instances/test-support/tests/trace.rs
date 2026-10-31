//! test-suite/trace.json ・ untested.json と場面の突き合わせ。
//! - 範囲（scope）の id は、どれも trace か untested のどちらか一方にある
//! - trace の場面の並びは、その id を traces に書いた場面と一致する

#![allow(clippy::expect_used)]

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

fn suite() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-suite")
}

fn load(name: &str) -> Value {
    let text = std::fs::read_to_string(suite().join(name)).expect("読めること");
    serde_json::from_str(&text).expect("JSON であること")
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .expect("配列であること")
        .iter()
        .map(|s| s.as_str().expect("文字列であること").to_string())
        .collect()
}

#[test]
fn every_id_in_scope_is_traced_or_explained() {
    let trace = load("trace.json");
    let untested = load("untested.json");
    let traced: BTreeSet<String> = trace["traces"]
        .as_array()
        .expect("traces が配列であること")
        .iter()
        .map(|t| t["id"].as_str().expect("id がある").to_string())
        .collect();
    let explained: BTreeSet<String> = untested["untested"]
        .as_array()
        .expect("untested が配列であること")
        .iter()
        .map(|t| t["id"].as_str().expect("id がある").to_string())
        .collect();
    assert!(
        traced.is_disjoint(&explained),
        "trace と untested の両方にある id がある"
    );
    for id in strings(&trace["scope"]) {
        assert!(
            traced.contains(&id) || explained.contains(&id),
            "{id} が試験にも untested にも無い"
        );
    }
}

#[test]
fn trace_matches_scenarios() {
    let mut from_scenarios: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut paths: Vec<PathBuf> = std::fs::read_dir(suite().join("scenarios"))
        .expect("場面の置き場所を読めること")
        .map(|e| e.expect("項目を読めること").path())
        .collect();
    paths.sort();
    for path in paths {
        let s: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("読めること"))
            .expect("JSON であること");
        for t in strings(&s["traces"]) {
            from_scenarios
                .entry(t)
                .or_default()
                .push(s["id"].as_str().expect("id がある").to_string());
        }
    }
    let from_trace: BTreeMap<String, Vec<String>> = load("trace.json")["traces"]
        .as_array()
        .expect("traces が配列であること")
        .iter()
        .map(|t| {
            (
                t["id"].as_str().expect("id がある").to_string(),
                strings(&t["scenarios"]),
            )
        })
        .collect();
    assert_eq!(from_trace, from_scenarios);
}
