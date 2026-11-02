//! system-tests/trace.json ・ untested.json と場面の突き合わせ。
//! - 範囲（scope）の id は、どれも trace か untested のどちらか一方にある
//! - trace の場面の並びは、その id を traces に書いた場面と一致する
//! - 公開された言葉（published-language/v1 のファイル）のどれにも、場面で確かめた約束がある
//! - 範囲にあるユースケースのどれにも、成功時保証と最低保証があり、場面で確かめている（どちらも untested に理由があるユースケース、
//!   つまりまだ作っていないものは除く）
//!
//! SAS の trace.rs を写したもの。

#![allow(clippy::expect_used)]

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

fn suite() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../system-tests")
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

/// 範囲にあり、場面で確かめた id。
fn traced() -> BTreeSet<String> {
    let trace = load("trace.json");
    let scope: BTreeSet<String> = strings(&trace["scope"]).into_iter().collect();
    trace["traces"]
        .as_array()
        .expect("traces が配列であること")
        .iter()
        .filter(|t| !strings(&t["scenarios"]).is_empty())
        .map(|t| t["id"].as_str().expect("id がある").to_string())
        .filter(|id| scope.contains(id))
        .collect()
}

#[test]
fn every_published_language_has_a_traced_promise() {
    let traced = traced();
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../published-language/v1");
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .expect("公開された言葉の置き場所を読めること")
        .map(|e| {
            e.expect("項目を読めること")
                .file_name()
                .to_string_lossy()
                .to_string()
        })
        .filter_map(|n| {
            n.strip_suffix(".schema.json")
                .or_else(|| n.strip_suffix(".json"))
                .map(ToString::to_string)
        })
        .collect();
    names.sort();
    for name in names {
        let prefix = format!("約束:{name}");
        assert!(
            traced
                .iter()
                .any(|id| id == &prefix || id.starts_with(&format!("{prefix}:"))),
            "{name} の約束を場面で確かめていない"
        );
    }
}

#[test]
fn every_use_case_in_scope_has_both_guarantees_traced() {
    let traced = traced();
    let scope = strings(&load("trace.json")["scope"]);
    let usecases: BTreeSet<String> = scope
        .iter()
        .filter_map(|id| id.split_once(':').map(|(uc, _)| uc.to_string()))
        .filter(|uc| uc.starts_with("UC"))
        .collect();
    assert!(!usecases.is_empty(), "範囲にユースケースが無い");
    // 成功時保証と最低保証のどちらも untested に理由がある（まだ作っていない）ユースケースは除く
    let explained: BTreeSet<String> = load("untested.json")["untested"]
        .as_array()
        .expect("untested が配列であること")
        .iter()
        .map(|t| t["id"].as_str().expect("id がある").to_string())
        .collect();
    for uc in usecases {
        if ["成功時保証", "最低保証"]
            .iter()
            .all(|g| explained.contains(&format!("{uc}:{g}")))
        {
            continue;
        }
        for g in ["成功時保証", "最低保証"] {
            let id = format!("{uc}:{g}");
            assert!(traced.contains(&id), "{id} を場面で確かめていない");
        }
    }
}
