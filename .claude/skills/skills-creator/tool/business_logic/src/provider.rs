// SPDX-License-Identifier: MIT
//! 提供者の検証 ── 言語の組の雛形を直したときに、5言語 × 2型を生成して検証する。**見つけるが、直さない。**
//!
//! 空の作業場所に、利用者と同じ道具（`bin/skills-creator` の scaffold）で Skill を生み、scaffold が案内する
//! 組み立てのコマンドと、言語の組の試験を起動する。生んだ Skill に check と accept（助言型）を当て、最後に
//! Rust の助言型を基準に、他の言語の references の道具の出力を突き合わせる（conform）。
//!
//! **シェルを経由しない**（以前の tool/verify-profiles.sh を置き換えた）── 組み立てのコマンドは語の並びとして
//! 起動し、検出は JSON で受け取る。どの OS でも、同じ道具で検証できる。

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

use crate::data_access::{files, process};
use crate::{conform, profile};

/// 生む型。
pub const TYPES: [&str; 2] = ["work", "advisor"];

/// 1回の起動の制限時間。**組み立ては、依存を取り寄せると数分かかる。**
const LIMIT: Duration = Duration::from_secs(1800);

/// 組み立てのコマンドを、語の並びにする。**シェルを経由しない** ── 言語の組の定義は、演算子を使わない。
#[must_use]
pub fn argv(command: &str) -> Vec<String> {
    command.split_whitespace().map(str::to_owned).collect()
}

/// 組み立ての出力先（`--target-dir` の値）を、共有の出力先へ置換する。**他の語は利用者の手順のまま起動する。**
///
/// 生んだ Skill ごとの出力先へ組み立てると、同じ依存を型ごと ・ 実行ごとにコンパイルし直す（実測 2026-10-01、
/// Rust の2つの型で1回あたり約 2.5G）。共有の出力先が無ければ、コマンドを変更しない。
#[must_use]
pub fn shared_target(mut command: Vec<String>, shared: Option<&str>) -> Vec<String> {
    if let Some(dir) = shared {
        if let Some(i) = command.iter().position(|w| w == "--target-dir") {
            if let Some(value) = command.get_mut(i + 1) {
                dir.clone_into(value);
            }
        }
    }
    command
}

/// check の検出のうち、生んだ直後に出てはならないもの。**未記入の差し込み場所だけは出てよい。**
#[must_use]
pub fn blocking_check(out: &Value) -> Vec<String> {
    out.get("findings")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|f| !f.contains("差し込み場所"))
        .map(str::to_owned)
        .collect()
}

/// accept の不合格のうち、生んだ直後に出てはならないもの。**学習ノートが無いので2番は出てよい。**
#[must_use]
pub fn blocking_accept(out: &Value) -> Vec<u64> {
    out.get("data")
        .and_then(|d| d.get("checks"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|c| c.get("pass").and_then(Value::as_bool) != Some(true))
        .filter_map(|c| c.get("no").and_then(Value::as_u64))
        .filter(|no| *no != 2)
        .collect()
}

/// 検証した結果。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Verified {
    /// 生んだ Skill の置き場所。
    pub work: PathBuf,
    /// 経過（「[v-rust-work] 済」など）。
    pub lines: Vec<String>,
    /// 不合格。**空なら、すべて合格した。**
    pub failures: Vec<String>,
}

fn launch(command: &[String], cwd: &Path) -> Result<process::Ran, String> {
    let (head, rest) = command
        .split_first()
        .ok_or_else(|| "起動するコマンドが空である".to_owned())?;
    process::run(head, rest, cwd, LIMIT).map_err(|e| format!("{head} を起動できない ── {e:?}"))
}

/// 出力の末尾の数行。**全文を載せない** ── 何が起きたかの見当が付けば足りる。
fn tail(ran: &process::Ran) -> String {
    let all = format!("{}{}", ran.stdout, ran.stderr);
    let lines: Vec<&str> = all.lines().collect();
    lines[lines.len().saturating_sub(5)..].join(" ／ ")
}

fn json_of(ran: &process::Ran) -> Value {
    serde_json::from_str(&ran.stdout).unwrap_or(Value::Null)
}

/// 空の作業場所を1つ用意する。**既に在るものを削除しない** ── 番号を進めて、新しい置き場所を作る。
fn fresh(work: &Path) -> Result<PathBuf, String> {
    let mut n = 1;
    loop {
        let dir = work.join(format!("run-{n}"));
        if !files::exists(&dir) {
            files::create_dir_all(dir.join(".claude/skills"))
                .map_err(|e| format!("{} を作れない ── {e}", dir.display()))?;
            return Ok(dir);
        }
        n += 1;
    }
}

/// 1つの Skill を生み、組み立て、試験し、検査する。
fn one(
    bin: &Path,
    here: &Path,
    skills: &Path,
    (lang, ty): (&str, &str),
    shared: Option<&str>,
    v: &mut Verified,
) -> Result<(), String> {
    let name = format!("v-{lang}-{ty}");
    let root = here.display().to_string();
    let sc = |args: &[&str]| -> Result<process::Ran, String> {
        let mut all: Vec<String> = vec![bin.display().to_string()];
        all.extend(args.iter().map(|a| (*a).to_owned()));
        all.extend(["--skill_root".to_owned(), root.clone(), "--json".to_owned()]);
        launch(&all, skills)
    };
    let made = sc(&[
        "scaffold",
        &name,
        "--language",
        lang,
        "--type",
        ty,
        "--path",
        ".",
    ])?;
    let made_json = json_of(&made);
    if made.code != 0 {
        v.failures
            .push(format!("[{name}] scaffold: {}", tail(&made)));
        return Ok(());
    }
    let dir = skills.join(&name);
    for command in made_json
        .get("data")
        .and_then(|d| d.get("build"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        let ran = launch(&shared_target(argv(command), shared), &dir)?;
        if ran.code != 0 {
            v.failures
                .push(format!("[{name}] 組み立て: {command} ── {}", tail(&ran)));
            return Ok(());
        }
    }
    let defs = profile::load(&here.join("references/profiles"), lang)?;
    let ran = launch(&defs.test, &dir)?;
    if ran.code != 0 {
        v.failures.push(format!("[{name}] 試験 ── {}", tail(&ran)));
    }
    // **生んだ直後のコードが整形されているか** ── 整形されていない雛形は、生んだ Skill をリポジトリの
    // 整形の規則にその場で不合格にする（実測 2026-10-01、Rust の雛形で7か所）
    if !defs.format.is_empty() {
        let ran = launch(&defs.format, &dir)?;
        if ran.code != 0 {
            v.failures.push(format!("[{name}] 整形 ── {}", tail(&ran)));
        }
    }
    let checked = sc(&["check", &dir.display().to_string()])?;
    for f in blocking_check(&json_of(&checked)) {
        v.failures.push(format!("[{name}] check: {f}"));
    }
    if ty == "advisor" {
        let accepted = sc(&["accept", &dir.display().to_string()])?;
        let bad = blocking_accept(&json_of(&accepted));
        if !bad.is_empty() {
            let nos: Vec<String> = bad.iter().map(u64::to_string).collect();
            v.failures
                .push(format!("[{name}] accept: {} 番", nos.join(" ・ ")));
        }
    }
    v.lines.push(format!("[{name}] 済"));
    Ok(())
}

/// 5言語 × 2型を生成して検証する。`bin` は skills-creator の実行ファイル、`here` は skills-creator の
/// フォルダ、`work` は作業場所（その下に新しい置き場所を作る）、`corpus` は突き合わせに使う references を
/// 持つ Skill の置き場所である。`shared` は Rust の組み立ての共有の出力先で、無ければ利用者の手順のまま組み立てる。
///
/// # Errors
///
/// 作業場所を作れないとき、道具を起動できないとき、言語の組の定義を読めないときに返す。
pub fn verify(
    bin: &Path,
    here: &Path,
    work: &Path,
    languages: &[String],
    corpus: &Path,
    shared: Option<&Path>,
) -> Result<Verified, String> {
    // **絶対の経路にする** ── 生んだ Skill のフォルダから起動するので、相対の経路では届かない
    let abs =
        |p: &Path| files::canonicalize(p).map_err(|e| format!("{} を読めない ── {e}", p.display()));
    let (bin, here) = (abs(bin)?, abs(here)?);
    let (bin, here) = (bin.as_path(), here.as_path());
    let shared = shared.map(|p| p.display().to_string());
    let dir = fresh(work)?;
    let skills = dir.join(".claude/skills");
    let mut v = Verified {
        work: dir,
        ..Verified::default()
    };
    for lang in languages {
        for ty in TYPES {
            one(bin, here, &skills, (lang, ty), shared.as_deref(), &mut v)?;
        }
    }
    let base = skills.join("v-rust-advisor");
    let schema = here.join("references/profiles/shared/document.schema.json.tmpl");
    for lang in languages.iter().filter(|l| l.as_str() != "rust") {
        let other = skills.join(format!("v-{lang}-advisor"));
        match conform::conform(&base, &other, corpus, &schema) {
            Ok(report) => {
                v.lines.push(format!(
                    "[conform {lang}] 事例 {} 件 ／ 不一致 {} 件",
                    report.cases,
                    report.mismatches.len()
                ));
                v.failures.extend(
                    report
                        .mismatches
                        .into_iter()
                        .map(|m| format!("[conform {lang}] {m}")),
                );
            }
            Err(e) => v.failures.push(format!("[conform {lang}] {e}")),
        }
    }
    Ok(v)
}
