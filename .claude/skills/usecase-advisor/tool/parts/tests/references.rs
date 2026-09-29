// SPDX-License-Identifier: MIT
//! usecase-advisor の references を事例で検証する。**判断基準と回答の形が、スキーマに合う。**
//!
//!     cargo test -p ua_parts

use std::path::PathBuf;

use ua_parts::refs;

fn references() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references")
}

#[test]
fn the_references_match_their_schemas() {
    let found = refs::validate(&references()).expect("読める");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_criterion_is_taken_by_its_id() {
    let one = refs::get(&references(), "criteria", Some("goal-levels")).expect("在る");
    assert!(one["title"]
        .as_str()
        .unwrap_or_default()
        .contains("目的レベル"));
    assert!(
        one["answers"].as_array().is_some_and(|a| !a.is_empty()),
        "答える問いが在る"
    );
}

#[test]
fn every_chapter_is_a_criterion() {
    // **章を1件の判断基準にする** ── 第1〜22章と、付録A（UML）・付録C（用語集）の24件
    let all = refs::get(&references(), "criteria", None).expect("読める");
    let n = all["items"].as_array().map_or(0, Vec::len);
    assert_eq!(n, 24);
}

#[test]
fn the_answer_example_passes_the_answer_schema() {
    let fx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/answer.judgment.json");
    let found = refs::validate_file(&references(), "answer", &fx).expect("読める");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_writing_answer_needs_its_steps() {
    // **記述相談は、記述の手順を欠くと合格しない**
    let fx = std::env::temp_dir().join("usecase-advisor-writing.json");
    std::fs::write(
        &fx,
        r#"{"kind": "writing", "question": "拡張条件はどう書くか",
            "conclusion": {"stance": "adopt", "headline": "条件を書く"},
            "grounds": [{"criterion": "extensions", "quote": "拡張"}], "next": ["書く"]}"#,
    )
    .expect("書ける");
    let found = refs::validate_file(&references(), "answer", &fx).expect("読める");
    assert!(found.iter().any(|f| f.contains("steps")), "{found:?}");
}
