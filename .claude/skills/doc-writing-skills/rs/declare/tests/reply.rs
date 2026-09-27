// SPDX-License-Identifier: MIT
//! 利用者への応答に当てる照合（reply）を検証する。
//!
//!     cargo test -p dws_declare

use dws_declare::{tools, Given, Outcome};

fn reply(message: &str) -> Outcome {
    let tool = tools().into_iter().find(|t| t.name == "reply").expect("在る");
    let mut given = Given::default();
    given.push("message", message.to_owned());
    (tool.run)(&given)
}

#[test]
fn a_wago_predicate_in_a_reply_is_blocked() {
    // 検出があれば、Stop フックが読む decision と reason を持つ
    let out = reply("結果を揃えます。");
    assert!(out.ok);
    assert_eq!(out.findings.len(), 1, "{:?}", out.findings);
    assert_eq!(out.data["decision"], "block");
    assert!(out.data["reason"].as_str().unwrap_or_default().contains("揃えま"));
}

#[test]
fn a_clean_reply_passes_without_output() {
    // 検出が無ければ何も出さない ── 空の出力は、フックに判定が無いことを意味する
    let out = reply("結果を統一します。");
    assert!(out.ok);
    assert!(out.findings.is_empty());
    assert!(out.data.get("decision").is_none());
}

#[test]
fn the_layout_checks_are_not_applied_to_a_reply() {
    // 見出しの階層や文体は、会話の応答には当てない ── 語彙表で決まる2つだけである
    let out = reply("# 一\n\n### 三\n\nこれは本文である。\n\nこれは本文です。\n");
    assert!(out.findings.is_empty(), "{:?}", out.findings);
}
