// SPDX-License-Identifier: MIT
//! 突き合わせ（conform）を、偽の CLI で検証する。**実物の言語の組を組まずに確かめる。**
//! 偽の CLI はシェルのスクリプトで、渡された引数に関わらず決まった JSON を返す。

#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use sc_business_logic::conform;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sc-conform-test-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("作れる");
    dir
}

/// 決まった JSON を返す偽の CLI を持つ Skill を置く。
fn fake_skill(root: &Path, name: &str, answer: &str) -> PathBuf {
    let skill = root.join(name);
    std::fs::create_dir_all(skill.join("bin")).expect("作れる");
    let script = skill.join("bin/cli");
    std::fs::write(&script, format!("#!/bin/sh\ncat <<'EOF'\n{answer}\nEOF\n")).expect("書ける");
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).expect("変えられる");
    std::fs::write(
        skill.join("tool.json"),
        format!(
            r#"{{"cli":{{"command":"{}","args":[]}},"external":[]}}"#,
            script.display()
        ),
    )
    .expect("書ける");
    skill
}

/// references を1種類だけ持つ Skill の置き場所を作る。
fn corpus(root: &Path) -> PathBuf {
    let refs = root.join("corpus/one/references");
    std::fs::create_dir_all(&refs).expect("作れる");
    std::fs::write(refs.join("k.schema.json"), r#"{"title":"k"}"#).expect("書ける");
    std::fs::write(
        refs.join("k.json"),
        r#"{"$schema":"k.schema.json","items":[]}"#,
    )
    .expect("書ける");
    root.join("corpus")
}

fn schema() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../references/profiles/shared/document.schema.json.tmpl")
}

const OK: &str = r#"{"ok":true,"findings":[],"data":{"html":"<p>同じ</p>","kinds":["k"]}}"#;

#[test]
fn the_same_output_is_counted_as_a_match() {
    let root = scratch("same");
    let a = fake_skill(&root, "a", OK);
    let b = fake_skill(&root, "b", OK);
    let report = conform::conform(&a, &b, &corpus(&root), &schema()).expect("比べられる");
    assert!(report.cases > 0);
    assert!(report.mismatches.is_empty(), "{:?}", report.mismatches);
}

#[test]
fn a_different_output_is_reported() {
    let root = scratch("different");
    let a = fake_skill(&root, "a", OK);
    let b = fake_skill(
        &root,
        "b",
        r#"{"ok":true,"findings":[],"data":{"html":"<p>違う</p>","kinds":["k"]}}"#,
    );
    let report = conform::conform(&a, &b, &corpus(&root), &schema()).expect("比べられる");
    assert!(
        report.mismatches.iter().any(|m| m.contains("基準と違う")),
        "{:?}",
        report.mismatches
    );
}

#[test]
fn a_failing_base_is_not_counted_as_a_match() {
    // **両方が同じ誤りを返しても、一致と数えない** ── 比べていないのに合格に見える（実際に起きた）
    let root = scratch("failing");
    let failed = r#"{"ok":false,"findings":["読めない"],"data":{}}"#;
    let a = fake_skill(&root, "a", failed);
    let b = fake_skill(&root, "b", failed);
    let report = conform::conform(&a, &b, &corpus(&root), &schema()).expect("比べられる");
    assert!(
        report
            .mismatches
            .iter()
            .any(|m| m.contains("基準が道具を実行できない")),
        "{:?}",
        report.mismatches
    );
}
