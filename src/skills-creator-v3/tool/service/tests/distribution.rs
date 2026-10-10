// SPDX-License-Identifier: MIT
//! 配布の形を置く操作を、事例で検証する。
//!
//!     cargo test -p sc_service --test distribution

use std::path::PathBuf;

use sc_service::{tools, Given, Outcome};

fn here() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .display()
        .to_string()
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sc-dist-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("作れる");
    dir
}

fn run(name: &str, pairs: &[(&str, String)]) -> Outcome {
    let tool = tools().into_iter().find(|t| t.name == name).expect("在る");
    let mut given = Given::default();
    for (k, v) in pairs {
        given.push(k, v.clone());
    }
    (tool.run)(&given)
}

#[test]
fn scaffold_places_the_tool_under_tool_and_ignores_bin() {
    // **道具のソースは tool/ に置き、bin/ は git で追跡しない**
    let root = scratch("scaffold");
    let out = run(
        "scaffold",
        &[
            ("skill", "sample".to_owned()),
            ("path", root.display().to_string()),
            ("skill_root", here()),
        ],
    );
    assert!(out.ok, "{:?}", out.findings);
    let skill = root.join("sample");
    assert!(skill.join("tool/Cargo.toml").is_file());
    assert!(!skill.join("rs").exists(), "rs/ を作らない");
    let ignore = std::fs::read_to_string(skill.join(".gitignore")).expect("在る");
    assert!(ignore.lines().any(|l| l == "bin/"), "{ignore}");
    let mcp = std::fs::read_to_string(skill.join("mcp.json")).expect("在る");
    assert!(
        mcp.contains("${CLAUDE_PROJECT_DIR:-.}/.claude/skills/sample/bin/sample-mcp"),
        "登録に絶対パスを書かない ── {mcp}"
    );
}

#[test]
fn dist_places_the_installers_and_the_workflow_once() {
    // **導入スクリプトと組み立ての定義は、配布元のリポジトリに1つずつ置く**
    let repo = scratch("dist");
    let args = [
        ("path", repo.display().to_string()),
        ("repo", "owner/name".to_owned()),
        ("skill_root", here()),
    ];
    let out = run("dist", &args);
    assert!(out.ok, "{:?}", out.findings);
    for file in ["install.sh", "install.ps1", ".github/workflows/release.yml"] {
        let body =
            std::fs::read_to_string(repo.join(file)).unwrap_or_else(|e| panic!("{file} ── {e}"));
        assert!(!body.contains("{{配布元}}"), "{file} に差し込み忘れがある");
    }
    let sh = std::fs::read_to_string(repo.join("install.sh")).expect("在る");
    assert!(
        sh.contains("github.com/owner/name/releases"),
        "配布元を差し込む"
    );
    // **既に在るものを上書きしない** ── 配布元が手を入れたものが消える
    std::fs::write(repo.join("install.sh"), "手を入れた").expect("書ける");
    let again = run("dist", &args);
    assert!(again.ok);
    assert_eq!(
        std::fs::read_to_string(repo.join("install.sh")).expect("在る"),
        "手を入れた"
    );
}

#[test]
fn the_text_of_check_shows_the_cases_stage() {
    // `--json` だけでなく、人向けの文にもテストケースの段を出す
    let tool = tools()
        .into_iter()
        .find(|t| t.name == "check")
        .expect("在る");
    let out = Outcome::found(
        Vec::new(),
        serde_json::json!({ "lines": [
            { "stage": "cases", "state": "pass", "text": "テストケース one_item_is_taken_by_its_id" }
        ] }),
    );
    let text = (tool.human)(&out);
    assert!(text.contains("テストケース\n"), "{text}");
    assert!(text.contains("one_item_is_taken_by_its_id"), "{text}");
}
