// SPDX-License-Identifier: MIT
//! 提供者の検証（雛形の検証）のうち、起動しない部分を事例で検証する。
//!
//!     cargo test -p sc_business_logic --test provider

use sc_business_logic::provider::{blocking_accept, blocking_check, shared_target};
use serde_json::json;

#[test]
fn only_the_unfilled_placeholders_are_allowed_after_scaffolding() {
    // 生んだ直後に出てよいのは、SKILL.md の未記入の差し込み場所だけである
    let out = json!({"findings": [
        "SKILL.md に未記入の差し込み場所が 3 か所ある",
        "references/view.css が正本と違う"
    ]});
    assert_eq!(
        blocking_check(&out),
        vec!["references/view.css が正本と違う"]
    );
}

#[test]
fn accept_may_fail_only_the_notes_check_after_scaffolding() {
    // 生んだ直後は、学習ノートが無いので2番（複製と語彙と出典）が不合格になる ── 正しい結果である
    let out = json!({"data": {"checks": [
        {"no": 1, "pass": true},
        {"no": 2, "pass": false},
        {"no": 5, "pass": false}
    ]}});
    assert_eq!(blocking_accept(&out), vec![5]);
}

#[test]
fn the_build_output_goes_to_the_shared_place_when_one_is_given() {
    // **組み立ての出力先だけを置換する** ── 他の語は利用者の手順のまま起動する
    let words = |s: &str| s.split(' ').map(str::to_owned).collect::<Vec<_>>();
    let command = words("cargo install --path tool/cli --root . --target-dir tool/target");
    assert_eq!(
        shared_target(command.clone(), Some("/cache/cargo")),
        words("cargo install --path tool/cli --root . --target-dir /cache/cargo")
    );
    // 共有の出力先が無ければ、そのまま起動する
    assert_eq!(shared_target(command.clone(), None), command);
    // --target-dir を保持しないコマンドは変更しない
    let go = words("go build -C tool ./cli");
    assert_eq!(shared_target(go.clone(), Some("/cache/cargo")), go);
}
