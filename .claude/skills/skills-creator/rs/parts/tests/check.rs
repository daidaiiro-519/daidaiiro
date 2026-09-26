// SPDX-License-Identifier: MIT
//! 契約の検査を事例で検証する。
//!
//!     cargo test -p sc_parts

use std::path::{Path, PathBuf};

use sc_parts::check::{self, Templates};

fn templates() -> Templates {
    let skills = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    Templates::new(
        skills.join("skills-creator/references/skill-template.md"),
        skills.join("advisor-creator/references/skill-template-advisor.md"),
    )
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sc-check-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("作れる");
    dir
}

/// 雛形が要求する節だけを持つ文書を置く。
fn write_document(root: &Path) {
    let template = std::fs::read_to_string(templates().general).expect("読める");
    let body: String = sc_parts::sections::headings(&template)
        .iter()
        .filter(|x| !x.contains("{{"))
        .map(|x| format!("## {x}\n\n本文\n\n"))
        .collect();
    std::fs::write(root.join("SKILL.md"), body).expect("書ける");
}

/// 層を crate に分けた、契約を満たす形を置く。
fn write_layers(root: &Path, edges: &[(&str, &[&str])]) {
    let rs = root.join("rs");
    let members: Vec<String> = edges
        .iter()
        .map(|(name, _)| format!("\"{name}\""))
        .collect();
    std::fs::create_dir_all(&rs).expect("作れる");
    std::fs::write(
        rs.join("Cargo.toml"),
        format!("[workspace]\nmembers = [{}]\n", members.join(", ")),
    )
    .expect("書ける");
    for (name, deps) in edges {
        let dir = rs.join(name);
        std::fs::create_dir_all(&dir).expect("作れる");
        let listed: String = deps
            .iter()
            .map(|d| format!("{d} = {{ path = \"../{d}\" }}\n"))
            .collect();
        std::fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\n\n[dependencies]\n{listed}"),
        )
        .expect("書ける");
    }
    std::fs::create_dir_all(rs.join(check::TESTS)).expect("作れる");
}

const GOOD: [(&str, &[&str]); 3] = [
    ("parts", &[]),
    ("declare", &["parts"]),
    ("cli", &["declare"]),
];

#[test]
fn a_skill_with_layers_as_crates_passes() {
    let root = scratch("ok");
    write_document(&root);
    write_layers(&root, &GOOD);
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.is_empty(), "食い違いが出た ── {found:?}");
}

#[test]
fn a_skill_without_tools_needs_no_layers() {
    let root = scratch("advice");
    write_document(&root);
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found.is_empty(),
        "助言だけの Skill に層を要求しない ── {found:?}"
    );
}

#[test]
fn python_that_remains_is_reported() {
    let root = scratch("python");
    write_document(&root);
    write_layers(&root, &GOOD);
    std::fs::create_dir_all(root.join("scripts")).expect("作れる");
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found.iter().any(|x| x.contains("Python が残っている")),
        "{found:?}"
    );
}

#[test]
fn layers_that_are_not_crates_are_reported() {
    let root = scratch("flat");
    write_document(&root);
    std::fs::create_dir_all(root.join("scripts")).expect("作れる");
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found.iter().any(|x| x.contains("crate に分かれていない")),
        "1つの単位の中の module では、内側が外側を参照してもコンパイラが通す ── {found:?}"
    );
}

#[test]
fn an_edge_that_goes_outward_is_reported() {
    let root = scratch("outward");
    write_document(&root);
    // **部品が入口を参照する宣言** ── 内から外である
    write_layers(
        &root,
        &[
            ("parts", &["cli"]),
            ("declare", &["parts"]),
            ("cli", &["declare"]),
        ],
    );
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found
            .iter()
            .any(|x| x.contains("許可していない辺") && x.contains("parts → entry")),
        "{found:?}"
    );
}

#[test]
fn an_edge_that_skips_inward_is_allowed() {
    let root = scratch("skip");
    write_document(&root);
    // 入口が部品を直に参照する ── 外から内なので許す
    write_layers(
        &root,
        &[
            ("parts", &[]),
            ("declare", &["parts"]),
            ("cli", &["declare", "parts"]),
        ],
    );
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.is_empty(), "外から内は許す ── {found:?}");
}

#[test]
fn a_missing_layer_is_reported() {
    let root = scratch("missing");
    write_document(&root);
    write_layers(&root, &[("parts", &[]), ("cli", &["parts"])]);
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found
            .iter()
            .any(|x| x.contains("層の crate が無い") && x.contains("declare")),
        "{found:?}"
    );
}

#[test]
fn a_missing_entry_is_reported() {
    let root = scratch("noentry");
    write_document(&root);
    write_layers(&root, &[("parts", &[]), ("declare", &["parts"])]);
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(
        found.iter().any(|x| x.contains("入口の crate が無い")),
        "{found:?}"
    );
}

#[test]
fn absent_examples_are_reported() {
    let root = scratch("notests");
    write_document(&root);
    write_layers(&root, &GOOD);
    std::fs::remove_dir_all(root.join("rs").join(check::TESTS)).expect("消せる");
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.iter().any(|x| x.contains("事例が無い")), "{found:?}");
}

#[test]
fn a_missing_section_is_reported() {
    let root = scratch("section");
    write_layers(&root, &GOOD);
    std::fs::write(root.join("SKILL.md"), "## 目的\n\n本文\n").expect("書ける");
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.iter().any(|x| x.contains("節が無い")), "{found:?}");
}

#[test]
fn an_absent_document_is_reported() {
    let root = scratch("nodoc");
    write_layers(&root, &GOOD);
    let found = check::check(&root, &templates()).expect("検査できる");
    assert!(found.iter().any(|x| x.contains("文書が無い")), "{found:?}");
}
