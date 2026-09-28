// SPDX-License-Identifier: MIT
//! 成果物（`units`）ごとに、根 ・ 言語 ・ 層の並びを持つ規則ファイルを固定する。
//!
//! - 規則は、適用する成果物の名前を持ち、成果物ごとに実行1件へ展開する
//! - 実行する場所は「成果物の根 ＋ `check.target`」である
//! - 依存の向きは `check.inward` で書き、その成果物の言語と層の並びで測る
//!
//!     cargo test -p nms_parts --test units

use std::path::{Path, PathBuf};

use nms_parts::label::Verdict;
use nms_parts::{rules, run, validate};

fn scratch(name: &str, body: &str) -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!("nms-units-{name}"));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join(".coding-rules")).expect("作れる");
    let file = root.join(".coding-rules/rules.json");
    std::fs::write(&file, body).expect("書ける");
    (root, file)
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().expect("親が在る")).expect("作れる");
    std::fs::write(path, body).expect("書ける");
}

const TWO_UNITS: &str = r#"{
  "units": {
    "app": { "root": "app", "language": "typescript",
             "layers": [ { "name": "core", "where": ["src/core"] },
                         { "name": "adapter", "where": ["src/adapter"] } ] },
    "docs": { "root": "docs" }
  },
  "rules": [
    { "rule": "整形されている", "source": { "record": "x" }, "scope": "全体",
      "check": { "tool": ["true"], "target": "src" }, "units": ["app", "docs"] },
    { "rule": "依存の向きが、層の並びと一致する", "source": { "record": "x" }, "scope": "全体",
      "check": { "inward": true }, "units": ["*"] }
  ]
}"#;

#[test]
fn a_rule_is_expanded_for_each_unit_it_names() {
    let (_, file) = scratch("expand", TWO_UNITS);
    let got = rules::load(&file).expect("読める");
    let tidy: Vec<_> = got
        .iter()
        .filter(|r| r.name.starts_with("整形されている"))
        .collect();
    assert_eq!(tidy.len(), 2, "{got:?}");
    // 実行する場所は、成果物の根 ＋ target である
    assert_eq!(tidy[0].target, "app/src");
    assert_eq!(tidy[1].target, "docs/src");
    // 2つ以上の成果物へ展開した規則は、どの成果物の実行かを名前に持つ
    assert_eq!(tidy[0].name, "整形されている（app）");
    assert_eq!(tidy[1].name, "整形されている（docs）");
}

#[test]
fn every_unit_for_an_inward_rule_means_every_unit_with_layers() {
    // 層を持たない成果物（docs）は、依存の向きを測る対象にならない
    let (_, file) = scratch("star", TWO_UNITS);
    let got = rules::load(&file).expect("読める");
    let inward: Vec<_> = got.iter().filter(|r| r.inward.is_some()).collect();
    assert_eq!(inward.len(), 1, "{got:?}");
    assert_eq!(inward[0].target, "app");
    assert_eq!(inward[0].name, "依存の向きが、層の並びと一致する");
}

#[test]
fn a_rule_naming_a_missing_unit_is_not_run() {
    let body = r#"{ "units": { "app": { "root": "." } },
      "rules": [ { "rule": "r", "source": { "record": "x" }, "scope": "s",
                   "check": { "tool": ["true"] }, "units": ["nowhere"] } ] }"#;
    let (root, file) = scratch("missing", body);
    let report = run::check(&root, &file, 10).expect("実行できる");
    assert_eq!(report.rules.len(), 1);
    assert_eq!(report.rules[0].verdict, Verdict::Skip);
    assert!(
        report.rules[0].reason.contains("nowhere"),
        "{:?}",
        report.rules[0]
    );
}

#[test]
fn an_inward_rule_measures_the_unit_with_its_language_and_layers() {
    let (root, file) = scratch("inward", TWO_UNITS);
    write(&root, "app/src/core/name.ts", "export const n = 1;\n");
    write(
        &root,
        "app/src/adapter/use.ts",
        "import { n } from '../core/name';\nexport const m = n;\n",
    );
    write(&root, "docs/src/.keep", "");
    write(&root, "app/src/.keep", "");
    let report = run::check(&root, &file, 10).expect("実行できる");
    let got = report
        .rules
        .iter()
        .find(|r| r.name.starts_with("依存の向き"))
        .expect("在る");
    assert_eq!(got.verdict, Verdict::Pass, "{got:?}");

    // 逆の並びでは、同じコードが違反になる
    let reversed = TWO_UNITS.replace(
        r#"[ { "name": "core", "where": ["src/core"] },
                         { "name": "adapter", "where": ["src/adapter"] } ]"#,
        r#"[ { "name": "adapter", "where": ["src/adapter"] },
                         { "name": "core", "where": ["src/core"] } ]"#,
    );
    assert_ne!(reversed, TWO_UNITS);
    std::fs::write(&file, reversed).expect("書ける");
    let report = run::check(&root, &file, 10).expect("実行できる");
    let got = report
        .rules
        .iter()
        .find(|r| r.name.starts_with("依存の向き"))
        .expect("在る");
    assert_eq!(got.verdict, Verdict::Fail, "{got:?}");
    assert!(got.output.contains("use.ts"), "{}", got.output);
}

#[test]
fn the_units_are_checked_as_a_structure() {
    let body = r#"{
      "units": {
        "app": { "root": "app", "layers": [ { "name": "core", "where": ["core"] },
                                           { "name": "core", "where": ["x"] } ] },
        "docs": { "root": "docs" }
      },
      "rules": [
        { "rule": "a", "source": { "record": "x" }, "scope": "s", "check": { "tool": ["true"] }, "units": ["nowhere"] },
        { "rule": "b", "source": { "record": "x" }, "scope": "s", "check": { "inward": true }, "units": ["docs"] },
        { "rule": "c", "source": { "record": "x" }, "scope": "s", "check": { "tool": ["true"] } },
        { "rule": "d", "source": { "record": "x" }, "scope": "s", "check": { "inward": true }, "units": ["app"] }
      ]
    }"#;
    let (_, file) = scratch("structure", body);
    let found = validate::check_units(&file).expect("検査できる");
    let has = |needle: &str| found.iter().any(|f| f.contains(needle));
    assert!(has("nowhere"), "実在しない成果物: {found:?}");
    assert!(has("docs"), "層を持たない成果物への inward: {found:?}");
    assert!(has("c:"), "成果物を指さない規則: {found:?}");
    assert!(has("言語"), "言語を持たない成果物への inward: {found:?}");
    assert!(has("core"), "同じ名前の層が2つ: {found:?}");
}

#[test]
fn the_former_order_and_layers_are_refused() {
    // **並びと層を全体で1つに持つ形は、成果物ごとの形へ移った**
    let body = r#"{"order":["a"],"layers":{"a":"x"},"rules":[]}"#;
    let (_, file) = scratch("former", body);
    let found = validate::check_units(&file).expect("検査できる");
    assert!(found.iter().any(|f| f.contains("units")), "{found:?}");
}

fn contracts() -> validate::Contracts {
    let r = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references");
    validate::Contracts::new(
        r.join("rules.schema.json"),
        r.join("concepts.schema.json"),
        r.join("schema-meta.schema.json"),
    )
}

#[test]
fn an_inward_rule_needs_no_tool_and_meets_the_contract() {
    let (_, file) = scratch("contract-ok", TWO_UNITS);
    let found = validate::check_rules(&file, &contracts()).expect("検査できる");
    assert!(found.is_empty(), "{found:?}");
    assert!(validate::check_units(&file).expect("検査できる").is_empty());
}

#[test]
fn a_rule_with_both_a_tool_and_inward_is_refused() {
    // **検証方法は1つである** ── 両方あると、どちらで判定したかが読めない
    let body = TWO_UNITS.replace(
        r#""check": { "inward": true }"#,
        r#""check": { "inward": true, "tool": ["true"] }"#,
    );
    let (_, file) = scratch("contract-both", &body);
    let found = validate::check_rules(&file, &contracts()).expect("検査できる");
    assert!(!found.is_empty(), "{found:?}");
}
