// SPDX-License-Identifier: MIT
//! 成果物（`units`）ごとに、根 ・ 言語 ・ 層の並びを持つ規則ファイルを固定する。
//!
//! - 規則は、適用する成果物の名前を持ち、成果物ごとに実行1件へ展開する
//! - 実行する場所は「成果物の根 ＋ `check.target`」である
//! - 依存の向きは `check.inward` で書き、その成果物の言語と層の並びで測る
//!
//!     cargo test -p nms_business_logic --test units

use std::path::{Path, PathBuf};

use nms_business_logic::label::Verdict;
use nms_business_logic::{rules, run, validate};

fn scratch(name: &str, body: &str) -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!("nms-units-{name}-{}", std::process::id()));
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
    "lib": { "root": "lib", "language": "typescript",
             "layers": [ { "name": "core", "where": ["src/core"] } ] }
  },
  "rules": [
    { "rule": "整形されている", "source": { "record": "x" }, "scope": "全体",
      "check": { "tool": ["true"], "target": "src" }, "units": ["app", "lib"] },
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
    assert_eq!(tidy[1].target, "lib/src");
    // 2つ以上の成果物へ展開した規則は、どの成果物の実行かを名前に持つ
    assert_eq!(tidy[0].name, "整形されている（app）");
    assert_eq!(tidy[1].name, "整形されている（lib）");
}

#[test]
fn every_unit_for_an_inward_rule_means_every_unit() {
    // **単位はどれもコードの成果物で、層の並びを持つ**（言語と層は契約が必須にする）──
    // 「すべての成果物」は、その全部である
    let (_, file) = scratch("star", TWO_UNITS);
    let got = rules::load(&file).expect("読める");
    let inward: Vec<_> = got.iter().filter(|r| r.inward.is_some()).collect();
    assert_eq!(inward.len(), 2, "{got:?}");
    assert_eq!(inward[0].target, "app");
    assert_eq!(inward[1].target, "lib");
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
    write(&root, "lib/src/core/.keep", "");
    write(&root, "app/src/.keep", "");
    let report = run::check(&root, &file, 10).expect("実行できる");
    let got = report
        .rules
        .iter()
        .find(|r| r.name.starts_with("依存の向き") && r.target == "app")
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
        .find(|r| r.name.starts_with("依存の向き") && r.target == "app")
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

/// 生成物（src/gen/made.ts）を参照する成果物。**生成の手順が、inward の前にそれを置く。**
fn generated(generate: &str) -> String {
    format!(
        r#"{{
  "units": {{
    "app": {{ "root": "app", "language": "typescript", "generate": {generate},
             "layers": [ {{ "name": "core", "where": ["src/core"] }},
                         {{ "name": "gen", "where": ["src/gen"] }},
                         {{ "name": "adapter", "where": ["src/adapter"] }} ] }}
  }},
  "rules": [
    {{ "rule": "依存の向きが、層の並びと一致する", "source": {{ "record": "x" }}, "scope": "全体",
      "check": {{ "inward": true }}, "units": ["app"] }},
    {{ "rule": "依存の向きが、層の並びと一致する（2件目）", "source": {{ "record": "x" }}, "scope": "全体",
      "check": {{ "inward": true }}, "units": ["app"] }}
  ]
}}"#
    )
}

fn app_with_generated_reference(root: &Path) {
    write(root, "app/src/core/name.ts", "export const n = 1;\n");
    write(
        root,
        "app/src/adapter/use.ts",
        "import { m } from '../gen/made';\nexport const k = m;\n",
    );
    write(root, "app/made.ts.in", "export const m = 1;\n");
    std::fs::create_dir_all(root.join("app/src/gen")).expect("作れる");
}

fn verdicts(report: &run::Report) -> Vec<(String, Verdict, String)> {
    report
        .rules
        .iter()
        .map(|r| (r.name.clone(), r.verdict, r.reason.clone()))
        .collect()
}

#[test]
fn without_the_generation_the_generated_module_is_missing() {
    // 生成の手順が無ければ、生成物への参照は「参照先が実在しない」になる
    let (root, file) = scratch("gen-none", &generated("[]"));
    app_with_generated_reference(&root);
    let report = run::check(&root, &file, 10).expect("実行できる");
    let got = &report.rules[0];
    assert_eq!(got.verdict, Verdict::Fail, "{:?}", verdicts(&report));
    assert!(got.output.contains("実在しない"), "{}", got.output);
}

#[test]
fn the_generation_runs_once_before_inward() {
    let body = generated(r#"[["cp", "made.ts.in", "src/gen/made.ts"]]"#);
    let (root, file) = scratch("gen-ok", &body);
    app_with_generated_reference(&root);
    let report = run::check(&root, &file, 10).expect("実行できる");
    let all = verdicts(&report);
    // 生成の手順は1回だけで、inward の2件より前に並ぶ
    let steps: Vec<_> = all
        .iter()
        .filter(|(n, _, _)| n.starts_with("生成の手順"))
        .collect();
    assert_eq!(steps.len(), 1, "{all:?}");
    assert!(all[0].0.starts_with("生成の手順"), "{all:?}");
    assert_eq!(all[0].1, Verdict::Pass, "{all:?}");
    assert!(
        all[1..].iter().all(|(_, v, _)| *v == Verdict::Pass),
        "{all:?}"
    );
}

#[test]
fn a_failed_generation_leaves_inward_unrun() {
    let (root, file) = scratch("gen-fail", &generated(r#"[["false"]]"#));
    app_with_generated_reference(&root);
    let report = run::check(&root, &file, 10).expect("実行できる");
    let all = verdicts(&report);
    assert_eq!(all[0].1, Verdict::Fail, "生成の手順は不合格: {all:?}");
    for (_, verdict, reason) in &all[1..] {
        // 生成物が無いまま測ると、本当の違反と区別できない ── 合格にも不合格にもしない
        assert_eq!(*verdict, Verdict::Skip, "{all:?}");
        assert!(reason.contains("生成の手順"), "{all:?}");
    }
}

#[test]
fn the_generation_appears_in_the_plan() {
    let body = generated(r#"[["cp", "made.ts.in", "src/gen/made.ts"]]"#);
    let (_, file) = scratch("gen-plan", &body);
    let got = rules::load(&file).expect("読める");
    assert_eq!(
        got[0].tool,
        vec!["cp", "made.ts.in", "src/gen/made.ts"],
        "{got:?}"
    );
    assert_eq!(got[0].target, "app");
    assert_eq!(got[0].generates, "app");
}

#[test]
fn the_generation_is_a_list_of_commands_in_the_contract() {
    let ok = generated(r#"[["cp", "made.ts.in", "src/gen/made.ts"]]"#);
    let (_, file) = scratch("gen-contract-ok", &ok);
    let found = validate::check_rules(&file, &contracts()).expect("検査できる");
    assert!(found.is_empty(), "{found:?}");
    // **コマンドは配列で書く** ── 空の配列と文字列は、シェルを経由しない形にならない
    for bad in [r#"[[]]"#, r#"["cp made.ts.in src/gen/made.ts"]"#] {
        let (_, file) = scratch("gen-contract-bad", &generated(bad));
        let found = validate::check_rules(&file, &contracts()).expect("検査できる");
        assert!(!found.is_empty(), "{bad} を受け付けた");
    }
}

#[test]
fn the_root_mark_in_a_tool_is_the_repository_root() {
    // **道具の欄の ${root} は、リポジトリの根である** ── 道具は成果物の根（app）で起動するが、
    // 経路は根から書ける
    let body = r#"{ "units": { "app": { "root": "app", "language": "typescript",
                     "layers": [ { "name": "core", "where": ["src"] } ] } },
      "rules": [ { "rule": "根の印を見る", "source": { "record": "x" }, "scope": "s",
                   "check": { "tool": ["test", "-d", "${root}/marker"] }, "units": ["app"] } ] }"#;
    let (root, file) = scratch("root-mark", body);
    write(&root, "app/.keep", "");
    write(&root, "marker/.keep", "");
    let report = run::check(&root, &file, 10).expect("実行できる");
    assert_eq!(
        report.rules[0].verdict,
        Verdict::Pass,
        "{:?}",
        report.rules[0]
    );
}

#[test]
fn the_root_mark_is_replaced_in_every_argument() {
    let tool = vec![
        "${root}/bin/x".to_owned(),
        "--rules".to_owned(),
        "${root}/r.json".to_owned(),
        "plain".to_owned(),
    ];
    let got = run::with_root(&tool, std::path::Path::new("/repo"));
    assert_eq!(got, vec!["/repo/bin/x", "--rules", "/repo/r.json", "plain"]);
}
