// SPDX-License-Identifier: MIT
//! meta-thinking-advisor の references を事例で検証する。**助言型の雛形の事例5件である**（ACDR 0061）。
//! 判断基準の件数や題に依存しない ── 全件に同じ性質を確かめる。
//!
//!     cargo test -p mta_business_logic

use std::path::PathBuf;

use mta_business_logic::refs;

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn the_references_match_their_schemas() {
    let found = refs::validate(&references()).expect("読める");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn every_criterion_is_taken_by_its_id() {
    // **判断基準は1件ずつ取り出せる** ── 回答の前に読む手順が、この取り出しに頼る
    let all = refs::get(&references(), "criteria", None).expect("読める");
    for item in all["items"].as_array().into_iter().flatten() {
        let id = item["id"].as_str().expect("id が在る");
        let one = refs::get(&references(), "criteria", Some(id)).expect("在る");
        assert_eq!(one["id"], item["id"]);
    }
}

#[test]
fn the_answer_example_passes_the_answer_schema() {
    let found = refs::validate_file(&references(), "answer", &fixture("answer.example.json"))
        .expect("読める");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_concept_answer_needs_its_definition() {
    // **概念相談は、定義と具体例を欠くと合格しない**
    let fx = std::env::temp_dir().join("meta-thinking-advisor-concept.json");
    std::fs::write(
        &fx,
        r#"{"kind": "concept", "question": "それは何か",
            "conclusion": {"stance": "adopt", "headline": "こうである"},
            "grounds": [{"criterion": "x", "quote": "x"}], "next": ["読む"]}"#,
    )
    .expect("書ける");
    let found = refs::validate_file(&references(), "answer", &fx).expect("読める");
    assert!(found.iter().any(|f| f.contains("definition")), "{found:?}");
}

#[test]
fn a_design_answer_needs_its_steps() {
    // **設計相談は、設計の手順を欠くと合格しない**
    let fx = std::env::temp_dir().join("meta-thinking-advisor-design.json");
    std::fs::write(
        &fx,
        r#"{"kind": "design", "question": "どう組み立てるか",
            "conclusion": {"stance": "adopt", "headline": "こう組む"},
            "grounds": [{"criterion": "x", "quote": "x"}], "next": ["組む"]}"#,
    )
    .expect("書ける");
    let found = refs::validate_file(&references(), "answer", &fx).expect("読める");
    assert!(found.iter().any(|f| f.contains("steps")), "{found:?}");
}
