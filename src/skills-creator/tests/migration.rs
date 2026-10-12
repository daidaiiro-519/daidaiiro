//! 移す試験。リポジトリの全 Skill の tool.json を読み、2つの check に振り分ける。
//! - 版3（contract が 3）：凍結した古い check（src/skills-creator-v3/）で検査し、結果が移す前の基準と同じことを見る
//! - 型 skills-creator/tool@v1：新しい check で検査し、不合格 0件 かつ 判定不能 0件を見る
//!
//! 配る skills-creator は版3 を知らない。振り分けは、この試験だけが持つ。
//! 最後の版3 の Skill を移したら、この試験の版3 の側と、src/skills-creator-v3/ と、tests/baseline-v3/ を消す。

mod common;

use common::{repo, skills};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 基盤の Skill。skills-creator が作るものではないので、tool.json を持たず、この試験の対象にしない
const BASE: &str = "schema-driven";

fn read_json(path: &Path) -> Value {
    serde_json::from_str(
        &fs::read_to_string(path).unwrap_or_else(|e| panic!("{}：{e}", path.display())),
    )
    .unwrap()
}

/// 凍結した古い skills-creator を、ソースと Cargo.lock からビルドする。凍結したフォルダには何も書かない
fn frozen_check() -> PathBuf {
    let frozen = repo().join("src/skills-creator-v3");
    let target = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("skills-creator-v3");
    let status = Command::new("cargo")
        .args([
            "build",
            "--quiet",
            "--locked",
            "-p",
            "sc_cli",
            "--manifest-path",
        ])
        .arg(frozen.join("tool/Cargo.toml"))
        .arg("--target-dir")
        .arg(&target)
        .status()
        .unwrap();
    assert!(
        status.success(),
        "凍結した古い skills-creator をビルドできない"
    );
    target.join("debug/skills-creator")
}

#[test]
fn every_skill_keeps_passing_while_moving() {
    let repo = repo();
    let skills = skills();
    let baselines = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/baseline-v3");
    let new_check = env!("CARGO_BIN_EXE_skills-creator");
    let engine = skills.join(BASE).join("scripts/schema-driven");
    let own_skill = skills.join("skills-creator");
    let frozen = frozen_check();
    let mut wrong = Vec::new();
    let mut v3 = Vec::new();

    let mut names: Vec<String> = fs::read_dir(&skills)
        .unwrap()
        .flatten()
        .filter(|e| e.path().join("SKILL.md").is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name != BASE)
        .collect();
    names.sort();
    for name in &names {
        let dir = skills.join(name);
        let tool = read_json(&dir.join("tool.json"));
        if tool["contract"] == 3 {
            v3.push(name.clone());
            let Ok(text) = fs::read_to_string(baselines.join(format!("{name}.json"))) else {
                wrong.push(format!("{name}：版3 なのに、移す前の基準が無い"));
                continue;
            };
            let baseline: Value = serde_json::from_str(&text).unwrap();
            let output = Command::new(&frozen)
                .args(["check", "--path", ".", "--skill_root"])
                .arg(repo.join("src/skills-creator-v3"))
                .arg("--json")
                .current_dir(&dir)
                .env("CLAUDE_PROJECT_DIR", &repo)
                .output()
                .unwrap();
            let got: Value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
            let mut want: Vec<&Value> = baseline["findings"].as_array().unwrap().iter().collect();
            let mut have: Vec<&Value> = got["findings"]
                .as_array()
                .map(|a| a.iter().collect())
                .unwrap_or_default();
            want.sort_by_key(|v| v.to_string());
            have.sort_by_key(|v| v.to_string());
            if want != have {
                wrong.push(format!(
                    "{name}：古い check の結果が基準と違う\n  基準 {want:?}\n  いま {have:?}"
                ));
            }
        } else if tool["$schema"] == "skills-creator/tool@v1" {
            let output = Command::new(new_check)
                .arg("check")
                .arg(&dir)
                .arg("--engine")
                .arg(&engine)
                .arg("--self")
                .arg(&own_skill)
                .arg("--json")
                .output()
                .unwrap();
            let got: Value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
            if got["ok"] != true {
                let bad: Vec<String> = got["results"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|r| r["status"] == "fail" || r["status"] == "unknown")
                    .map(|r| format!("{} {}", r["id"], r["detail"]))
                    .collect();
                wrong.push(format!("{name}：新しい check で合格しない：{bad:?}"));
            }
        } else {
            wrong.push(format!(
                "{name}：tool.json が版3 でも型 skills-creator/tool@v1 でもない"
            ));
        }
    }
    // 移した Skill の基準は消す
    for entry in fs::read_dir(&baselines).unwrap().flatten() {
        let name = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        if !v3.contains(&name) {
            wrong.push(format!(
                "{name}：版3 でなくなったのに、移す前の基準が残っている"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
