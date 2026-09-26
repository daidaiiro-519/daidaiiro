// SPDX-License-Identifier: MIT
//! skills-creator の道具の宣言。**能力の正本はここである。**
//!
//! 入口（CLI ・ MCP）はこの宣言から組む ── 能力を2回書くと、片方だけが古くなる。
//! 許可辺は `Cargo.toml` が宣言する ── この crate は部品だけを参照する。

pub mod contract;

use std::path::{Path, PathBuf};

use sc_parts::{check, scaffold};
use serde_json::json;

pub use contract::{Arg, Given, Outcome, Tool};

/// 雛形の置き場所。**この crate が Skill の並びを知る唯一の場所である。**
fn templates(here: &Path) -> check::Templates {
    let references = here.join("references");
    let advisor = here.parent().map_or_else(
        || references.join("skill-template-advisor.md"),
        |skills| skills.join("advisor-creator/references/skill-template-advisor.md"),
    );
    check::Templates::new(references.join("skill-template.md"), advisor)
}

/// この Skill が置かれている場所。**実行ファイルの位置から辿らない** ── build の
/// 置き場所に依存する。引数で受け取り、渡されなければいま作業している場所を使う。
fn skill_root(given: &Given) -> PathBuf {
    PathBuf::from(given.one("skill_root", "."))
}

fn run_check(given: &Given) -> Outcome {
    let root = PathBuf::from(given.one("path", ""));
    if root.as_os_str().is_empty() {
        return Outcome::misuse("Skill のフォルダを渡していない".to_owned());
    }
    match check::check(&root, &templates(&skill_root(given))) {
        Ok(findings) => Outcome::found(findings, json!({ "root": root.display().to_string() })),
        Err(e) => Outcome::misuse(format!("検査できない ── {e}")),
    }
}

fn human_check(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    if out.findings.is_empty() {
        return "契約を満たしている ── 入口は1つ、道具は宣言の中に在る".to_owned();
    }
    let head = format!("契約に合わない箇所　{} 件", out.findings.len());
    let body: Vec<String> = out.findings.iter().map(|x| format!("  ・{x}")).collect();
    std::iter::once(head)
        .chain(body)
        .collect::<Vec<_>>()
        .join("\n")
}

/// 置く一式を組む。**何を置くかはここが決める** ── 部品は並びを認知しない。
///
/// **層を crate に分ける。** 1つの crate の中の module では、内側が外側を参照しても
/// コンパイラが通す ── 層の境界を crate の境界に置いて初めて、宣言に無い依存が
/// 解決しなくなる。
fn items(skill: &str, here: &Path) -> std::io::Result<Vec<scaffold::Item>> {
    let tmpl = here.join("references/tool-contract");
    let read = |name: &str| std::fs::read_to_string(tmpl.join(name));
    // **接頭辞は名前から導く。** 別に受け取ると、名前と食い違う
    let prefix: String = skill.split('-').filter_map(|w| w.chars().next()).collect();
    let root = here
        .parent()
        .map_or_else(|| skill.to_owned(), |p| p.join(skill).display().to_string());
    let fill = |body: String| {
        body.replace("{{Skill名}}", skill)
            .replace("{{接頭辞}}", &prefix)
            .replace("{{Skillの絶対パス}}", &root)
    };
    let mut out = Vec::new();
    for (from, to) in [
        ("workspace.Cargo.toml.tmpl", "rs/Cargo.toml"),
        ("parts.Cargo.toml.tmpl", "rs/parts/Cargo.toml"),
        ("parts.lib.rs.tmpl", "rs/parts/src/lib.rs"),
        ("parts.tests.rs.tmpl", "rs/parts/tests/example.rs"),
        ("declare.Cargo.toml.tmpl", "rs/declare/Cargo.toml"),
        ("declare.lib.rs.tmpl", "rs/declare/src/lib.rs"),
        ("contract.rs.tmpl", "rs/declare/src/contract.rs"),
        ("cli.Cargo.toml.tmpl", "rs/cli/Cargo.toml"),
        ("cli.main.rs.tmpl", "rs/cli/src/main.rs"),
        ("mcp.Cargo.toml.tmpl", "rs/mcp/Cargo.toml"),
        ("mcp.main.rs.tmpl", "rs/mcp/src/main.rs"),
        ("mcp.json.tmpl", "mcp.json"),
    ] {
        out.push(scaffold::Item::keep(PathBuf::from(to), fill(read(from)?)));
    }
    Ok(out)
}

fn run_scaffold(given: &Given) -> Outcome {
    let skill = given.one("skill", "");
    if skill.is_empty() {
        return Outcome::misuse("Skill の名前を渡していない".to_owned());
    }
    let here = skill_root(given);
    let root = PathBuf::from(given.one("path", ".claude/skills")).join(skill);
    let items = match items(skill, &here) {
        Ok(items) => items,
        Err(e) => return Outcome::misuse(format!("雛形を読めない ── {e}")),
    };
    let placed = match scaffold::place(&root, &items) {
        Ok(placed) => placed,
        Err(e) => return Outcome::misuse(format!("置けない ── {e}")),
    };
    let findings = placed
        .kept
        .iter()
        .map(|k| format!("既に在るので残した: {k}"))
        .collect();
    Outcome::found(
        findings,
        json!({ "written": placed.written, "root": root.display().to_string() }),
    )
}

fn human_scaffold(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    let written = out
        .data
        .get("written")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    let mut lines: Vec<String> = written
        .iter()
        .filter_map(|x| x.as_str())
        .map(|p| format!("置いた: {p}"))
        .collect();
    lines.extend(out.findings.iter().cloned());
    lines.push(
        "次に書くもの ── 差し込む場所（{{…}}）を埋め、部品を rs/parts/src/ へ置く".to_owned(),
    );
    lines.push(
        "組む ── cd rs && cargo build --release。登録は mcp.json をホストの設定へ差し込む"
            .to_owned(),
    );
    lines.join("\n")
}

/// 道具の一覧。**能力の正本である。**
#[must_use]
pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "scaffold",
            summary: "道具の契約一式を、Skill のフォルダへ置く",
            args: vec![
                Arg::need("skill", "Skill の名前"),
                Arg::opt("path", "置き場所", Some(".claude/skills")),
                Arg::opt("skill_root", "この Skill の場所（雛形を読む先）", Some(".")),
            ],
            run: run_scaffold,
            human: human_scaffold,
        },
        Tool {
            name: "check",
            summary: "契約を満たしているかを検査する",
            args: vec![
                Arg::need("path", "Skill のフォルダ"),
                Arg::opt("skill_root", "この Skill の場所（雛形を読む先）", Some(".")),
            ],
            run: run_check,
            human: human_check,
        },
    ]
}
