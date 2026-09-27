// SPDX-License-Identifier: MIT
//! 和語の一覧を読めないときの振る舞いを検証する。
//!
//!     cargo test -p dws_declare

use dws_declare::{tools, Given};

#[test]
fn an_unreadable_predicate_list_is_a_misuse() {
    // **空の一覧で続行しない。** 続行すると、和語を検査しないまま0件と報告する
    let dir = std::env::temp_dir().join("dws-words");
    std::fs::create_dir_all(&dir).expect("作れる");
    let target = dir.join("a.md");
    std::fs::write(&target, "形を揃える。\n").expect("書ける");
    let check = tools().into_iter().find(|t| t.name == "check").expect("在る");
    let mut given = Given::default();
    given.push("path", target.display().to_string());
    given.push("skill_root", dir.display().to_string());
    let out = (check.run)(&given);
    assert!(!out.ok, "誤用として止まらなかった: {:?}", out.findings);
    assert!(out.findings[0].contains("和語の一覧を読めない"));
}
