//! 型 v1 の試験。試験データ（tests/data/skill-v1/）を check に通し、
//! 終了コードと、落ちた項目 id の集合の両方を期待と照らす。

mod common;

use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 基盤のエンジン。SC_ENGINE で替えられる
fn engine() -> PathBuf {
    std::env::var("SC_ENGINE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| common::skills().join("schema-driven/scripts/schema-driven"))
}

/// 型のスキーマを持つ skills-creator の Skill のフォルダ。SC_SELF で替えられる
fn own_skill() -> PathBuf {
    std::env::var("SC_SELF")
        .map(PathBuf::from)
        .unwrap_or_else(|_| common::skills().join("skills-creator"))
}

pub fn run(args: &[&str]) -> (i32, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_skills-creator"))
        .args(args)
        .arg("--engine")
        .arg(engine())
        .arg("--self")
        .arg(own_skill())
        .output()
        .unwrap();
    let value = serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
        serde_json::json!({"unparsed": String::from_utf8_lossy(&output.stdout).into_owned(),
                           "stderr": String::from_utf8_lossy(&output.stderr).into_owned()})
    });
    (output.status.code().unwrap_or(-1), value)
}

fn ids_with(out: &Value, statuses: &[&str]) -> BTreeSet<String> {
    out["results"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter(|r| statuses.contains(&r["status"].as_str().unwrap_or("")))
        .map(|r| r["id"].as_str().unwrap().to_owned())
        .collect()
}

fn cases() -> Vec<(String, BTreeSet<String>)> {
    let root = manifest().join("tests/data/skill-v1");
    let expected: Value =
        serde_json::from_str(&fs::read_to_string(root.join("expected.json")).unwrap()).unwrap();
    expected["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let dir = root.join(c["skill"].as_str().unwrap());
            let fails = c["fails"]
                .as_array()
                .unwrap()
                .iter()
                .map(|f| f.as_str().unwrap().to_owned())
                .collect();
            (dir.to_string_lossy().into_owned(), fails)
        })
        .collect()
}

#[test]
fn every_case_fails_exactly_the_expected_items() {
    let mut wrong = Vec::new();
    for (dir, fails) in cases() {
        let (code, out) = run(&["check", &dir, "--json"]);
        let got = ids_with(&out, &["fail", "unknown"]);
        let want_code = if fails.is_empty() { 0 } else { 1 };
        if got != fails || code != want_code {
            wrong.push(format!(
                "{dir}: 期待 {fails:?}（{want_code}）、実際 {got:?}（{code}）{out}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn a_warning_does_not_fail() {
    let dir = manifest().join("tests/data/skill-v1/warning/body-ai-name/glossary-keeper");
    let (code, out) = run(&["check", dir.to_str().unwrap(), "--json"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(
        ids_with(&out, &["warn"]),
        BTreeSet::from(["skill.body.ai-names".to_owned()]),
        "{out}"
    );
}

/// 不合格になりうる項目には、不合格例が1つ以上ある。項目を足して不合格例を忘れると、ここで落ちる
#[test]
fn every_item_that_can_fail_has_an_invalid_case() {
    let dir = manifest().join("tests/data/skill-v1/valid/minimal/glossary-keeper");
    let (_, out) = run(&["check", dir.to_str().unwrap(), "--json"]);
    let all: BTreeSet<String> = out["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap().to_owned())
        .collect();
    // 警告だけの項目と、別の試験が見る項目
    let elsewhere = [
        "skill.body.ai-names",
        "tools.declared",
        "dist.command",
        "dist.sha256",
        "dist.tools-listed",
    ];
    let covered: BTreeSet<String> = cases().into_iter().flat_map(|(_, f)| f).collect();
    let missing: Vec<&String> = all
        .iter()
        .filter(|id| !elsewhere.contains(&id.as_str()) && !covered.contains(*id))
        .collect();
    assert!(missing.is_empty(), "不合格例の無い項目：{missing:?}");
}

/// 試験データの不合格例は、それぞれ1か所だけを破る
#[test]
fn each_invalid_case_breaks_one_item() {
    for (dir, fails) in cases() {
        if dir.contains("/invalid/") {
            assert_eq!(fails.len(), 1, "{dir}");
        }
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn scaffold_leaves_only_the_fields_to_fill() {
    let root = scratch("scaffold");
    let dir = root.join("glossary-keeper");
    let (code, out) = run(&[
        "scaffold",
        dir.to_str().unwrap(),
        "--kind",
        "term",
        "--json",
    ]);
    assert_eq!(code, 0, "{out}");
    for file in ["SKILL.md", "tool.json", "assets/term.schema.json"] {
        assert!(dir.join(file).is_file(), "{file} が無い");
    }
    assert!(
        !dir.join("references").exists(),
        "求めていない references/ がある"
    );
    assert!(
        !dir.join("tools.json").exists(),
        "求めていない tools.json がある"
    );
    let (code, out) = run(&["check", dir.to_str().unwrap(), "--json"]);
    assert_eq!(code, 1, "{out}");
    assert_eq!(
        ids_with(&out, &["fail", "unknown"]),
        BTreeSet::from(["skill.placeholders".to_owned()]),
        "{out}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn scaffold_refuses_an_existing_folder() {
    let root = scratch("scaffold-exists");
    let (code, out) = run(&[
        "scaffold",
        root.to_str().unwrap(),
        "--kind",
        "term",
        "--json",
    ]);
    assert_eq!(code, 1, "{out}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn the_dist_level_is_skipped_until_the_engine_is_in() {
    let root = scratch("dist-skip");
    let dir = root.join("glossary-keeper");
    copy_dir(
        &manifest().join("tests/data/skill-v1/valid/minimal/glossary-keeper"),
        &dir,
    );
    let (code, out) = run(&["check", dir.to_str().unwrap(), "--level", "dist", "--json"]);
    assert_eq!(code, 0, "{out}");
    let skipped = ids_with(&out, &["skip"]);
    for id in ["dist.command", "dist.sha256", "dist.tools-listed"] {
        assert!(skipped.contains(id), "{id} が対象外でない：{out}");
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn the_dist_level_checks_the_engine_and_declared_tools() {
    let root = scratch("dist");
    let dir = root.join("glossary-keeper");
    copy_dir(
        &manifest().join("tests/data/skill-v1/valid/minimal/glossary-keeper"),
        &dir,
    );
    fs::create_dir_all(dir.join("scripts")).unwrap();
    fs::copy(engine(), dir.join("scripts/schema-driven")).unwrap();
    let (code, out) = run(&["check", dir.to_str().unwrap(), "--level", "dist", "--json"]);
    assert_eq!(code, 0, "{out}");
    assert!(ids_with(&out, &["pass"]).contains("dist.command"), "{out}");

    // 宣言した実行ファイルの SHA-256 が合わない
    fs::write(
        dir.join("tools.json"),
        serde_json::json!({"$schema": "schema-driven/tools@v1", "kind": "tools",
            "tools": [{"name": "count", "description": "語を数える", "args": [],
                       "command": ["scripts/schema-driven"], "sha256": "0".repeat(64)}]})
        .to_string(),
    )
    .unwrap();
    let (code, out) = run(&["check", dir.to_str().unwrap(), "--level", "dist", "--json"]);
    assert_eq!(code, 1, "{out}");
    assert!(ids_with(&out, &["fail"]).contains("dist.sha256"), "{out}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_unknown_flag_is_refused() {
    let (code, _) = run(&["check", ".", "--nope"]);
    assert_eq!(code, 2);
}
