// SPDX-License-Identifier: MIT
//! 提供者の検証 ── リファレンス実装を直したときに、Skill を生成して検証する。**見つけるが、直さない。**
//!
//! 空の作業場所に、利用者と同じ道具（`bin/skills-creator` の scaffold）でリファレンス実装から作業型と助言型を
//! 生み、生んだ Skill の `tool.json` が持つ組み立て ・ テスト ・ フォーマットのコマンドを起動する。生んだ Skill に
//! check（テストケースを含む）と accept（助言型）を当てる。リファレンス実装のテストとテストケースの対応を照合し、
//! ほかの言語の枠が置けることも確かめる（ACDR 0097）。
//!
//! **シェルを経由しない** ── コマンドは語の並びとして起動し、検出は JSON で受け取る。どの OS でも、
//! 同じ道具で検証できる。

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

use crate::behavior;
use crate::data_access::{files, process};

/// 生む型。
pub const TYPES: [&str; 2] = ["work", "advisor"];

/// 枠だけを生む言語の例。**リファレンス実装と違う言語なら、どれでも同じ枠が置かれる。**
pub const PORT_LANGUAGE: &str = "go";

/// ほかの言語の枠が持つファイル（Skill のフォルダからの相対）。
pub const SKELETON_FILES: [&str; 4] = [
    "SKILL.md",
    "tool.json",
    "mcp.json",
    "references/document.schema.json",
];

/// 1回の起動の制限時間。**組み立ては、依存を取り寄せると数分かかる。**
const LIMIT: Duration = Duration::from_secs(1800);

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

/// 1つの Skill を生み、組み立て、試験し、検査する。**コマンドは生んだ Skill の tool.json から読む。**
fn one(
    bin: &Path,
    here: &Path,
    skills: &Path,
    ty: &str,
    shared: Option<&str>,
    v: &mut Verified,
) -> Result<(), String> {
    let name = format!("v-rust-{ty}");
    let root = here.display().to_string();
    let sc = |args: &[&str]| -> Result<process::Ran, String> {
        let mut all: Vec<String> = vec![bin.display().to_string()];
        all.extend(args.iter().map(|a| (*a).to_owned()));
        all.extend(["--skill_root".to_owned(), root.clone(), "--json".to_owned()]);
        launch(&all, skills)
    };
    let made = sc(&["scaffold", &name, "--type", ty, "--path", "."])?;
    if made.code != 0 {
        v.failures
            .push(format!("[{name}] scaffold: {}", tail(&made)));
        return Ok(());
    }
    let dir = skills.join(&name);
    for command in behavior::commands(&dir, "build")? {
        let ran = launch(&shared_target(command.clone(), shared), &dir)?;
        if ran.code != 0 {
            v.failures.push(format!(
                "[{name}] 組み立て: {} ── {}",
                command.join(" "),
                tail(&ran)
            ));
            return Ok(());
        }
    }
    // **生んだ直後のコードが整形されているか** ── 整形されていない雛形は、生んだ Skill をリポジトリの
    // 整形の規則にその場で不合格にする（実測 2026-10-01、Rust の雛形で7か所）
    for (key, what) in [("test", "試験"), ("format", "整形")] {
        for command in behavior::commands(&dir, key)? {
            let ran = launch(&command, &dir)?;
            if ran.code != 0 {
                v.failures
                    .push(format!("[{name}] {what} ── {}", tail(&ran)));
            }
        }
    }
    let checked = sc(&["check", &dir.display().to_string(), "--cases", "1"])?;
    let checked = json_of(&checked);
    for f in blocking_check(&checked) {
        v.failures.push(format!("[{name}] check: {f}"));
    }
    let cases: Vec<&Value> = checked
        .pointer("/data/lines")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|l| l["stage"] == "cases")
        .collect();
    v.lines.push(format!(
        "[{name}] テストケース {} 件 ／ 不合格 {} 件",
        cases.len(),
        cases.iter().filter(|l| l["state"] != "pass").count()
    ));
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

/// ほかの言語の枠を生み、置いたものを確かめる。**道具のソースは置かない** ── AI が移植して書く。
fn skeleton(bin: &Path, here: &Path, skills: &Path, v: &mut Verified) -> Result<(), String> {
    let name = format!("v-{PORT_LANGUAGE}-advisor");
    let mut all: Vec<String> = vec![bin.display().to_string()];
    all.extend(
        [
            "scaffold",
            &name,
            "--type",
            "advisor",
            "--language",
            PORT_LANGUAGE,
            "--path",
            ".",
            "--skill_root",
        ]
        .map(str::to_owned),
    );
    all.extend([here.display().to_string(), "--json".to_owned()]);
    let made = launch(&all, skills)?;
    if made.code != 0 {
        v.failures
            .push(format!("[{name}] scaffold: {}", tail(&made)));
        return Ok(());
    }
    let dir = skills.join(&name);
    for need in SKELETON_FILES {
        if !files::is_file(dir.join(need)) {
            v.failures.push(format!("[{name}] 枠に {need} が無い"));
        }
    }
    if files::exists(dir.join("tool")) {
        v.failures
            .push(format!("[{name}] 枠に tool/ が在る ── 道具は移植して書く"));
    }
    if behavior::contract_version(&dir) != 3 {
        v.failures
            .push(format!("[{name}] 枠の tool.json が契約の版3 でない"));
    }
    v.lines.push(format!("[{name}] 済"));
    Ok(())
}

/// リファレンス実装から作業型と助言型を生成して検証し、ほかの言語の枠を確かめる。`bin` は skills-creator の
/// 実行ファイル、`here` は skills-creator のフォルダ、`work` は作業場所（その下に新しい置き場所を作る）である。
/// `shared` は Rust の組み立ての共有の出力先で、無ければ利用者の手順のまま組み立てる。
///
/// # Errors
///
/// 作業場所を作れないとき、道具を起動できないとき、生んだ Skill の tool.json を読めないときに返す。
pub fn verify(
    bin: &Path,
    here: &Path,
    work: &Path,
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
    for ty in TYPES {
        one(bin, here, &skills, ty, shared.as_deref(), &mut v)?;
    }
    skeleton(bin, here, &skills, &mut v)?;
    // **テストケース（ボード skills-creator-contract の論点2 ・ 3）**：リファレンス実装のテスト1件ごとに
    // 同名のテストケースが在るかを照合する
    let cases_dir = here.join(crate::cases::DIR);
    let tests: String = crate::cases::REFERENCE_TESTS
        .iter()
        .map(|t| files::read_to_string(here.join(t)).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    match crate::cases::unmatched(&tests, &cases_dir) {
        Ok(found) => {
            v.lines.push(format!(
                "[cases] テストとテストケースの対応 ── 不一致 {} 件",
                found.len()
            ));
            v.failures
                .extend(found.into_iter().map(|f| format!("[cases] {f}")));
        }
        Err(e) => v.failures.push(format!("[cases] {e}")),
    }
    Ok(v)
}
