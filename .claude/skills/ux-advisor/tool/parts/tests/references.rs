// SPDX-License-Identifier: MIT
//! ux-advisor の references を事例で検証する。**判断基準と回答の形が、スキーマに合う。**
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
    let one = refs::get(&references(), "criteria", Some("design-system-tokens")).expect("在る");
    assert!(one["title"]
        .as_str()
        .unwrap_or_default()
        .contains("デザイントークン"));
    assert!(
        one["answers"].as_array().is_some_and(|a| !a.is_empty()),
        "答える判断が在る"
    );
}

#[test]
fn the_answer_example_passes_the_answer_schema() {
    let fx = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/answer.judgment.json");
    let found = refs::validate_file(&references(), "answer", &fx).expect("読める");
    assert!(found.is_empty(), "{found:?}");
}
