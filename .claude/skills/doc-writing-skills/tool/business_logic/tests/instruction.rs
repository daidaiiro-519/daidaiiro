// SPDX-License-Identifier: MIT
//! document の1件を、`review` の依頼文へ入れる本文へ変換する操作を、事例で検証する。
//!
//!     cargo test -p dws_business_logic

use dws_business_logic::instruction::text;
use dws_business_logic::refs::import_markdown;

#[test]
fn an_imported_procedure_returns_to_headings_and_blocks() {
    let md = "# 手順書\n\n前文である。\n\n## 手順\n\n1. 用意する\n2. 適用する\n\n- 注意\n\n## 出力\n\n```\n{\"ok\": true}\n```\n";
    let doc = import_markdown("procedure", "procedure.md", "2026-10-01", md);
    let got = text(&doc);
    assert!(
        got.starts_with("# 手順書\n\n前文である。\n"),
        "題を二重に出さない"
    );
    assert!(got.contains("## 手順\n\n1. 用意する\n2. 適用する\n"));
    assert!(got.contains("- 注意\n"));
    assert!(got.contains("## 出力\n\n```\n{\"ok\": true}\n```\n"));
}

#[test]
fn a_table_keeps_its_head_and_rows() {
    let md = "# 表\n\n| 甲 | 乙 |\n|---|---|\n| 1 | 2 |\n";
    let doc = import_markdown("table", "table.md", "", md);
    let got = text(&doc);
    assert!(got.contains("| 甲 | 乙 |\n|---|---|\n| 1 | 2 |\n"));
}
