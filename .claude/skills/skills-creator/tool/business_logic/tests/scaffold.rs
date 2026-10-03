// SPDX-License-Identifier: MIT
//! 置く操作を事例で検証する。
//!
//!     cargo test -p sc_business_logic

use std::path::PathBuf;

use sc_business_logic::sample;
use sc_business_logic::scaffold::{place, Item};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sc-scaffold-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("作れる");
    dir
}

#[test]
fn the_items_are_written_where_they_are_told() {
    let root = scratch("write");
    let items = vec![
        Item::keep(
            PathBuf::from("tool/business_logic/src/lib.rs"),
            "業務ロジック層".to_owned(),
        ),
        Item::keep(PathBuf::from("mcp.json"), "登録".to_owned()),
    ];
    let placed = place(&root, &items).expect("置ける");
    assert_eq!(placed.written.len(), 2);
    assert!(placed.kept.is_empty());
    // **深い場所でも、包みごと作る**
    assert_eq!(
        std::fs::read_to_string(root.join("tool/business_logic/src/lib.rs")).expect("読める"),
        "業務ロジック層"
    );
}

#[test]
fn what_is_already_there_is_kept() {
    let root = scratch("keep");
    let items = vec![Item::keep(PathBuf::from("mcp.json"), "はじめ".to_owned())];
    place(&root, &items).expect("置ける");
    let again = vec![Item::keep(PathBuf::from("mcp.json"), "あと".to_owned())];
    let placed = place(&root, &again).expect("置ける");
    // **既に在るものを上書きしない** ── 書いたものが消える
    assert!(placed.written.is_empty());
    assert_eq!(placed.kept.len(), 1);
    assert_eq!(
        std::fs::read_to_string(root.join("mcp.json")).expect("読める"),
        "はじめ"
    );
}

/// リファレンス実装と型の定義（ACDR 0060 ・ 0097）。**事例は skills-creator の references を読む。**
fn defs() -> (PathBuf, PathBuf, sample::Sample) {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references");
    let dir = here.join("sample/rust");
    let rust = sample::load(&dir).expect("リファレンス実装の定義が在る");
    (dir, here.join("types"), rust)
}

fn plan_of(ty: &str, language: &str) -> Vec<(PathBuf, String)> {
    let (dir, types, rust) = defs();
    let ty = sample::load_type(&types, ty).expect("型が在る");
    sample::plan(&dir, &types, &rust, &ty, language).expect("組み合わせられる")
}

#[test]
fn a_work_skill_can_be_planned_in_rust() {
    let plan = plan_of("work", "rust");
    // **共通の一式と型の一式が両方入る** ── 見本の道具は作業型の一式から来る
    assert!(plan
        .iter()
        .any(|(_, to)| to == "tool/service/src/contract.rs"));
    assert!(plan
        .iter()
        .any(|(_, to)| to == "tool/business_logic/src/hello.rs"));
    // **作業型も SKILL.md の雛形を置く**
    assert!(plan.iter().any(|(_, to)| to == "SKILL.md"));
    // **CLI の共通の決まりのテストも置く** ── 契約のテストケースと同じ名前で確かめる
    assert!(plan.iter().any(|(_, to)| to == "tool/cli/tests/cli.rs"));
    // **同じ置き先は1回だけ**
    let mut tos: Vec<&String> = plan.iter().map(|(_, to)| to).collect();
    let n = tos.len();
    tos.sort();
    tos.dedup();
    assert_eq!(n, tos.len());
}

#[test]
fn the_rust_tool_json_replaces_the_skeleton() {
    // **リファレンス実装の tool.json が、枠の tool.json を置き換える** ── 組み立て ・ テスト ・ フォーマットの
    // コマンドを持つのは、リファレンス実装の側である
    let plan = plan_of("work", "rust");
    let (from, _) = plan
        .iter()
        .find(|(_, to)| to == "tool.json")
        .expect("tool.json を置く");
    assert!(
        from.ends_with("sample/rust/common/tool.json.tmpl"),
        "{}",
        from.display()
    );
    let body = std::fs::read_to_string(from).expect("読める");
    let v: serde_json::Value = serde_json::from_str(&body).expect("JSON");
    assert_eq!(v["contract"], 3);
    assert_eq!(
        v["format"][0],
        serde_json::json!([
            "cargo",
            "fmt",
            "--manifest-path",
            "tool/Cargo.toml",
            "--all",
            "--",
            "--check"
        ])
    );
}

#[test]
fn another_language_gets_only_the_skeleton() {
    // **リファレンス実装と違う言語なら、道具のソースを置かない** ── AI がリファレンス実装から移植する
    let plan = plan_of("advisor", "go");
    assert!(
        !plan.iter().any(|(_, to)| to.starts_with("tool/")),
        "{plan:?}"
    );
    for to in [
        "SKILL.md",
        "tool.json",
        "mcp.json",
        ".gitignore",
        "references/document.schema.json",
        "references/criteria.schema.json",
        "references/answer.schema.json",
    ] {
        assert!(plan.iter().any(|(_, t)| t == to), "{to} が無い");
    }
    // 枠の tool.json は、組み立て ・ テスト ・ フォーマットを空で持つ ── 移植した言語のコマンドを書く
    let (from, _) = plan
        .iter()
        .find(|(_, to)| to == "tool.json")
        .expect("tool.json を置く");
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(from).expect("読める")).expect("JSON");
    assert_eq!(v["contract"], 3);
    for key in ["build", "test", "format"] {
        assert_eq!(v[key], serde_json::json!([]), "{key}");
    }
}

#[test]
fn a_type_without_templates_in_the_sample_is_refused() {
    // **リファレンス実装が型の一式を持たなければ生まない** ── 移植の元も無い
    let (dir, types, rust) = defs();
    let generate = sample::load_type(&types, "generate").expect("生成型が在る");
    for language in ["rust", "go"] {
        let why = sample::plan(&dir, &types, &rust, &generate, language).expect_err("まだ断る");
        assert!(
            why.contains("生成型") && why.contains("雛形をまだ持たない"),
            "{why}"
        );
    }
}

#[test]
fn an_unknown_type_lists_the_known_ones() {
    let (_, types, _) = defs();
    let why = sample::load_type(&types, "nope").expect_err("無い型");
    assert!(why.contains("work") && why.contains("advisor"), "{why}");
}

#[test]
fn an_advisor_skill_can_be_planned_in_rust() {
    // **助言型は、スキーマ ・ SKILL.md（型の側）と、references の道具と事例（リファレンス実装の側）を置く**
    let plan = plan_of("advisor", "rust");
    for to in [
        "references/criteria.schema.json",
        "references/answer.schema.json",
        "SKILL.md",
        "tool/business_logic/tests/references.rs",
    ] {
        assert!(plan.iter().any(|(_, t)| t == to), "{to} が無い");
    }
    // **見本の道具は置かない** ── 助言型の道具は references の4つだけである
    assert!(!plan.iter().any(|(_, t)| t.ends_with("hello.rs")));
}

#[test]
fn every_template_in_the_plan_exists() {
    for (ty, language) in [
        ("work", "rust"),
        ("advisor", "rust"),
        ("work", "go"),
        ("advisor", "go"),
    ] {
        for (from, _) in plan_of(ty, language) {
            assert!(from.is_file(), "{} が無い", from.display());
        }
    }
}

#[test]
fn a_language_name_is_lowercase_letters_and_digits() {
    assert!(sample::is_language_name("go"));
    assert!(sample::is_language_name("python3"));
    assert!(!sample::is_language_name("Go"));
    assert!(!sample::is_language_name("../x"));
    assert!(!sample::is_language_name(""));
}
