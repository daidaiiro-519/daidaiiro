// SPDX-License-Identifier: MIT
//! 見た目の複製の検査（ボード view-design-tokens）を事例で検証する。
//!
//!     cargo test -p sc_business_logic --test view

use std::path::{Path, PathBuf};

use sc_business_logic::view::{contrast, findings};

/// 見た目の正本（skills-creator の references/view/）。
fn canon() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../references/view")
}

/// 正本の複製を置いた Skill の references/ を作る。
fn skill(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("sc-view-{name}"));
    let _ = std::fs::remove_dir_all(&root);
    let refs = root.join("references");
    std::fs::create_dir_all(&refs).expect("作れる");
    for f in [
        "view.tokens.json",
        "view.tokens.schema.json",
        "view.css",
        "view.template.html",
    ] {
        std::fs::copy(canon().join(f), refs.join(f)).expect("写せる");
    }
    root
}

fn edit(root: &Path, file: &str, from: &str, to: &str) {
    let p = root.join("references").join(file);
    let s = std::fs::read_to_string(&p).expect("読める");
    assert!(s.contains(from), "{from} が {file} に無い");
    std::fs::write(&p, s.replacen(from, to, 1)).expect("書ける");
}

#[test]
fn a_faithful_copy_passes() {
    let root = skill("faithful");
    assert!(
        findings(&root, &canon()).is_empty(),
        "{:?}",
        findings(&root, &canon())
    );
}

#[test]
fn choosing_another_palette_is_not_a_difference() {
    let root = skill("palette");
    edit(
        &root,
        "view.tokens.json",
        "\"palette\": \"teal\"",
        "\"palette\": \"amber\"",
    );
    assert!(
        findings(&root, &canon()).is_empty(),
        "{:?}",
        findings(&root, &canon())
    );
}

#[test]
fn an_edited_copy_is_reported() {
    let root = skill("edited");
    edit(&root, "view.css", ".rv table{", ".rv table{outline:0;");
    assert!(
        findings(&root, &canon())
            .iter()
            .any(|f| f.contains("view.css") && f.contains("正本")),
        "{:?}",
        findings(&root, &canon())
    );
}

#[test]
fn a_missing_copy_is_reported() {
    let root = skill("missing");
    std::fs::remove_file(root.join("references/view.template.html")).expect("消せる");
    assert!(findings(&root, &canon())
        .iter()
        .any(|f| f.contains("view.template.html")));
}

#[test]
fn a_color_literal_in_the_skill_rules_is_reported() {
    let root = skill("literal");
    std::fs::write(
        root.join("references/view.skill.css"),
        ".rv .add{background:#e6ffed}",
    )
    .expect("書ける");
    assert!(
        findings(&root, &canon())
            .iter()
            .any(|f| f.contains("色の直値")),
        "{:?}",
        findings(&root, &canon())
    );
}

#[test]
fn an_undefined_variable_is_reported() {
    let root = skill("undefined");
    std::fs::write(
        root.join("references/view.skill.css"),
        ".rv .add{background:var(--add-bg)}",
    )
    .expect("書ける");
    assert!(findings(&root, &canon())
        .iter()
        .any(|f| f.contains("--add-bg")));
    // 固有のトークンで定めれば通る
    std::fs::write(
        root.join("references/view.skill.tokens.json"),
        r##"{"light":{"add-bg":"#e6ffed"},"dark":{"add-bg":"#12331c"}}"##,
    )
    .expect("書ける");
    assert!(
        findings(&root, &canon()).is_empty(),
        "{:?}",
        findings(&root, &canon())
    );
}

#[test]
fn contrast_follows_the_wcag_formula() {
    // 黒と白は 21:1、同じ色は 1:1
    assert!((contrast("#000000", "#ffffff").expect("計算できる") - 21.0).abs() < 0.01);
    assert!((contrast("#777777", "#777777").expect("計算できる") - 1.0).abs() < 0.01);
}

#[test]
fn a_low_contrast_palette_in_the_canon_is_reported() {
    let dir = std::env::temp_dir().join("sc-view-canon-low");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("作れる");
    for f in [
        "view.tokens.schema.json",
        "view.css",
        "view.template.html",
        "view.tokens.json",
    ] {
        std::fs::copy(canon().join(f), dir.join(f)).expect("写せる");
    }
    let p = dir.join("view.tokens.json");
    let s = std::fs::read_to_string(&p).expect("読める");
    std::fs::write(
        &p,
        s.replacen("\"muted\": \"#555f6a\"", "\"muted\": \"#cccccc\"", 1),
    )
    .expect("書ける");
    let root = skill("low");
    std::fs::copy(&p, root.join("references/view.tokens.json")).expect("写せる");
    assert!(
        findings(&root, &dir)
            .iter()
            .any(|f| f.contains("muted") && f.contains("比")),
        "{:?}",
        findings(&root, &dir)
    );
}
