// SPDX-License-Identifier: MIT
//! 図のテンプレート ── **references/exemplars.json の索引と、exemplars/*.svg が食い違わないこと。**
//!
//! テンプレートは作成者が複製して編集する出発点なので、そのままで検査1〜4を通る。索引の
//! `classes` は SVG が実際に使う class と一致する ── 用途を読んで選ぶ作成者が、どの class を
//! 使う図かを索引だけで知れる。

use std::collections::BTreeSet;
use std::path::PathBuf;

use ds_business_logic::publish::authored;
use regex::Regex;
use serde_json::Value;

fn refs() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

fn index() -> Vec<Value> {
    let v: Value = serde_json::from_str(
        &std::fs::read_to_string(refs().join("exemplars.json")).expect("読める"),
    )
    .expect("JSON");
    v["items"].as_array().expect("並び").clone()
}

fn s(v: &Value, k: &str) -> String {
    v[k].as_str().unwrap_or_default().to_owned()
}

#[test]
fn there_are_17_templates_and_4_were_added() {
    let all = index();
    assert_eq!(all.len(), 17);
    let added: Vec<String> = all
        .iter()
        .filter(|x| x["added"].as_bool() == Some(true))
        .map(|x| s(x, "id"))
        .collect();
    assert_eq!(
        added,
        ["compare-before-after", "gantt", "er-diagram", "venn"]
    );
}

#[test]
fn every_template_has_its_file_and_a_unique_id_and_name() {
    let all = index();
    let ids: BTreeSet<String> = all.iter().map(|x| s(x, "id")).collect();
    let names: BTreeSet<String> = all.iter().map(|x| s(x, "name")).collect();
    assert_eq!(ids.len(), all.len(), "id が重なる");
    assert_eq!(names.len(), all.len(), "名前が重なる");
    for x in &all {
        assert_eq!(s(x, "file"), format!("exemplars/{}.svg", s(x, "id")));
        assert!(
            refs().join(s(x, "file")).is_file(),
            "{} が無い",
            s(x, "file")
        );
    }
}

#[test]
fn every_template_passes_checks_1_to_4() {
    let mut bad = Vec::new();
    for x in index() {
        let body = std::fs::read_to_string(refs().join(s(&x, "file"))).expect("読める");
        let done = authored(&body, None, true).expect("解決できる");
        if !done.findings.is_empty() {
            bad.push(format!("{}: {:?}", s(&x, "id"), done.findings));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn the_index_lists_the_classes_each_template_uses() {
    let re = Regex::new(r#"class="([^"]*)""#).expect("式");
    let mut bad = Vec::new();
    for x in index() {
        let body = std::fs::read_to_string(refs().join(s(&x, "file"))).expect("読める");
        let used: BTreeSet<String> = re
            .captures_iter(&body)
            .flat_map(|c| {
                c[1].split_whitespace()
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .collect();
        let listed: BTreeSet<String> = x["classes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect();
        if used != listed {
            bad.push(format!("{}: SVG {used:?} ／ 索引 {listed:?}", s(&x, "id")));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn the_catalog_lists_the_templates() {
    let c = ds_business_logic::catalog::catalog().expect("目録");
    let ids: Vec<&str> = c["exemplars"]
        .as_array()
        .expect("並び")
        .iter()
        .filter_map(|x| x["id"].as_str())
        .collect();
    assert_eq!(ids.len(), 17);
    assert!(ids.contains(&"state-transition"));
}
