// SPDX-License-Identifier: MIT
//! 置く操作を事例で検証する。
//!
//!     cargo test -p sc_business_logic

use std::path::PathBuf;

use sc_business_logic::profile;
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

/// 型と言語の組の定義（ACDR 0060）。**事例は skills-creator の references を読む。**
fn defs() -> (PathBuf, PathBuf) {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references");
    (here.join("profiles"), here.join("types"))
}

#[test]
fn a_work_skill_can_be_planned_in_rust() {
    let (profiles, types) = defs();
    let rust = profile::load(&profiles, "rust").expect("Rust の組が在る");
    let work = profile::load_type(&types, "work").expect("作業型が在る");
    let plan = profile::plan(&profiles, &types, &rust, &work).expect("組み合わせられる");
    // **共通の一式と型の一式が両方入る** ── 見本の道具は作業型の一式から来る
    assert!(plan
        .iter()
        .any(|(_, to)| to == "tool/service/src/contract.rs"));
    assert!(plan
        .iter()
        .any(|(_, to)| to == "tool/business_logic/src/hello.rs"));
    // **同じ置き先は1回だけ**
    let mut tos: Vec<&String> = plan.iter().map(|(_, to)| to).collect();
    let n = tos.len();
    tos.dedup();
    assert_eq!(n, tos.len());
}

#[test]
fn a_type_without_templates_in_the_language_is_refused() {
    // **型の一式を持たない組では生まない** ── 生んでから壊れていると分かる形にしない
    let (profiles, types) = defs();
    let rust = profile::load(&profiles, "rust").expect("Rust の組が在る");
    let generate = profile::load_type(&types, "generate").expect("生成型が在る");
    let why = profile::plan(&profiles, &types, &rust, &generate).expect_err("まだ断る");
    assert!(
        why.contains("生成型") && why.contains("雛形をまだ持たない"),
        "{why}"
    );
}

#[test]
fn an_unknown_type_lists_the_known_ones() {
    let (_, types) = defs();
    let why = profile::load_type(&types, "nope").expect_err("無い型");
    assert!(why.contains("work") && why.contains("advisor"), "{why}");
}

#[test]
fn an_advisor_skill_can_be_planned_in_rust() {
    // **助言型は、スキーマ ・ SKILL.md（型の側）と、references の道具と事例（組の側）を置く**
    let (profiles, types) = defs();
    let rust = profile::load(&profiles, "rust").expect("Rust の組が在る");
    let advisor = profile::load_type(&types, "advisor").expect("助言型が在る");
    let plan = profile::plan(&profiles, &types, &rust, &advisor).expect("組み合わせられる");
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
fn a_work_skill_can_be_planned_in_python() {
    // **置き先の中のパッケージ名は、差し込む前の形で返る** ── 差し込みはサービス層が行う
    let (profiles, types) = defs();
    let python = profile::load(&profiles, "python").expect("Python の組が在る");
    let work = profile::load_type(&types, "work").expect("作業型が在る");
    let plan = profile::plan(&profiles, &types, &python, &work).expect("組み合わせられる");
    for to in [
        "tool/pyproject.toml",
        "tool/cli.py",
        "tool/mcp_server.py",
        "tool/{{パッケージ名}}/contract.py",
        "tool/{{パッケージ名}}/tools.py",
        "tool.json",
        "mcp.json",
    ] {
        assert!(plan.iter().any(|(_, t)| t == to), "{to} が無い");
    }
    // **雛形はすべて実在する** ── 定義にだけ在る行を、生んでから見つける形にしない
    for (from, _) in &plan {
        assert!(from.is_file(), "{} が無い", from.display());
    }
}

#[test]
fn an_advisor_skill_is_refused_in_python() {
    // **契約の版1 の組は、版2 を要る助言型を生まない** ── 対応している組を並べて断る
    let (profiles, types) = defs();
    let python = profile::load(&profiles, "python").expect("Python の組が在る");
    let advisor = profile::load_type(&types, "advisor").expect("助言型が在る");
    let why = profile::plan(&profiles, &types, &python, &advisor).expect_err("断る");
    assert!(why.contains("版2") && why.contains("rust"), "{why}");
}
