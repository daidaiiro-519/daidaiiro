// SPDX-License-Identifier: MIT
//! 利用者への応答に当てる照合（reply）を検証する。
//!
//!     cargo test -p dws_service

use dws_service::{tools, Given, Outcome};

fn reply(message: &str) -> Outcome {
    let tool = tools()
        .into_iter()
        .find(|t| t.name == "reply")
        .expect("在る");
    let mut given = Given::default();
    // **Skill の置き場所を渡す** ── 試験の実行ファイルは target/ の下に在り、1つ上に references/ が無い
    given.push(
        "skill_root",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../..").to_owned(),
    );
    given.push("message", message.to_owned());
    (tool.run)(&given)
}

#[test]
fn a_retired_word_in_a_reply_is_blocked() {
    // 検出があれば、Stop フックが読む additionalContext を持つ ── **decision: block は使わない**
    // （原典は block を hook error として表示し、additionalContext を hook feedback として表示する）
    let out = reply("代償が大きい。");
    assert!(out.ok);
    assert_eq!(out.findings.len(), 1, "{:?}", out.findings);
    let hook = &out.data["hookSpecificOutput"];
    assert_eq!(hook["hookEventName"], "Stop");
    assert!(hook["additionalContext"]
        .as_str()
        .unwrap_or_default()
        .contains("代償"));
    assert!(out.data.get("decision").is_none());
}

#[test]
fn a_clean_reply_passes_without_output() {
    // 検出が無ければ何も出さない ── 空の出力は、フックに判定が無いことを意味する
    let out = reply("結果を統一します。");
    assert!(out.ok);
    assert!(out.findings.is_empty());
    assert!(out.data.get("decision").is_none());
    assert!(out.data.get("hookSpecificOutput").is_none());
}

#[test]
fn the_layout_checks_are_not_applied_to_a_reply() {
    // 見出しの階層や文体は、会話の応答には当てない ── 語彙表で決まる1つだけである
    let out = reply("# 一\n\n### 三\n\nこれは本文である。\n\nこれは本文です。\n");
    assert!(out.findings.is_empty(), "{:?}", out.findings);
}

fn review(message: &str, scope: &str) -> Outcome {
    let tool = tools()
        .into_iter()
        .find(|t| t.name == "review")
        .expect("在る");
    let mut given = Given::default();
    // **Skill の置き場所を渡す** ── 試験の実行ファイルは target/ の下に在り、1つ上に references/ が無い
    given.push(
        "skill_root",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../..").to_owned(),
    );
    given.push("message", message.to_owned());
    given.push("scope", scope.to_owned());
    (tool.run)(&given)
}

#[test]
fn a_review_prompt_holds_the_text_and_the_criteria() {
    let out = review("3言語は覆った。", "reply");
    assert!(out.ok, "{:?}", out.findings);
    let prompt = out.data["prompt"].as_str().unwrap_or_default();
    assert!(prompt.contains("3言語は覆った。"));
    assert!(prompt.contains("predicate-fit"));
}

#[test]
fn a_reply_review_omits_the_criteria_that_need_context() {
    // 会話の応答は前後の文脈を持たない ── 指し先の基準を適用すると、正しい文を違反と判定する
    let reply = review("本文。", "reply");
    let document = review("本文。", "document");
    assert!(!reply.data["prompt"]
        .as_str()
        .unwrap_or_default()
        .contains("missing-referent"));
    assert!(document.data["prompt"]
        .as_str()
        .unwrap_or_default()
        .contains("missing-referent"));
    assert!(reply.data["criteria"].as_u64() < document.data["criteria"].as_u64());
}
