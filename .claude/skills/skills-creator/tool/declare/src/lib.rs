// SPDX-License-Identifier: MIT
//! skills-creator の道具の宣言。**能力の正本はここである。**
//!
//! 入口（CLI ・ MCP）はこの宣言から組む ── 能力を2回書くと、片方だけが古くなる。
//! 許可辺は `Cargo.toml` が宣言する ── この crate は部品だけを参照する。

pub mod contract;

use std::path::{Path, PathBuf};

use sc_parts::{check, scaffold};
use serde_json::json;

pub use contract::{catalog, Arg, Given, Outcome, Tool};

/// Skill の置き場所が見つからなければ、誤用として返す。**黙って「.」へ寄せない** ──
/// 実行した場所で結果が変わり、契約を読めずに止まる（ACDR 0019 ・ 0029）。
macro_rules! or_misuse {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(why) => return Outcome::misuse(why),
        }
    };
}

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
fn skill_root(given: &Given) -> Result<PathBuf, String> {
    given.skill_root()
}

fn run_check(given: &Given) -> Outcome {
    let root = PathBuf::from(given.one("path", ""));
    if root.as_os_str().is_empty() {
        return Outcome::misuse("Skill のフォルダを渡していない".to_owned());
    }
    let report = check::check(&root, &templates(&or_misuse!(skill_root(given))));
    let lines: Vec<serde_json::Value> = report
        .lines
        .iter()
        .map(|l| {
            json!({
                "stage": match l.stage {
                    check::Stage::Behavior => "behavior",
                    check::Stage::Source => "source",
                    _ => "document",
                },
                "state": match l.state {
                    check::State::Pass => "pass",
                    check::State::Fail => "fail",
                    _ => "skip",
                },
                "text": l.text,
            })
        })
        .collect();
    Outcome::found(
        report.findings(),
        json!({ "root": root.display().to_string(), "lines": lines }),
    )
}

/// 段ごとに、1件1行で並べる。**「実行しない」を合格の印で出さない。**
fn human_check(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    let lines = out
        .data
        .get("lines")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    let field = |l: &serde_json::Value, k: &str| {
        l.get(k)
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_owned()
    };
    let mut text = Vec::new();
    for (stage, head) in [
        ("behavior", "1段目（振る舞い）"),
        ("source", "2段目（ソース）"),
        ("document", "文書"),
    ] {
        let here: Vec<&serde_json::Value> = lines
            .iter()
            .filter(|l| field(l, "stage") == stage)
            .collect();
        if here.is_empty() {
            continue;
        }
        text.push(head.to_owned());
        for l in here {
            let mark = match field(l, "state").as_str() {
                "pass" => "OK",
                "fail" => "×",
                _ => "－",
            };
            text.push(format!("  {mark}  {}", field(l, "text")));
        }
    }
    text.join("\n")
}

/// 置く一式を組む。**何を置くかはここが決める** ── 部品は並びを認知しない。
///
/// **層を crate に分ける。** 1つの crate の中の module では、内側が外側を参照しても
/// コンパイラが通す ── 層の境界を crate の境界に置いて初めて、宣言に無い依存が
/// 解決しなくなる。
fn items(skill: &str, here: &Path) -> std::io::Result<Vec<scaffold::Item>> {
    let tmpl = here.join("references/profiles/rust");
    let read = |name: &str| std::fs::read_to_string(tmpl.join(name));
    // **接頭辞は名前から導く。** 別に受け取ると、名前と食い違う。
    // 語が1つなら頭の3文字を採る ── 1文字では、他の crate と見分けがつかない
    let words: Vec<&str> = skill.split('-').filter(|w| !w.is_empty()).collect();
    let prefix: String = if words.len() >= 2 {
        words.iter().filter_map(|w| w.chars().next()).collect()
    } else {
        words
            .first()
            .map_or_else(String::new, |w| w.chars().take(3).collect())
    };
    let fill = |body: String| {
        body.replace("{{Skill名}}", skill)
            .replace("{{接頭辞}}", &prefix)
    };
    let mut out = Vec::new();
    // **道具のソースは tool/ に置く。** 実行ファイルは bin/ に置き、git で追跡しない
    for (from, to) in [
        ("workspace.Cargo.toml.tmpl", "tool/Cargo.toml"),
        ("parts.Cargo.toml.tmpl", "tool/parts/Cargo.toml"),
        ("parts.lib.rs.tmpl", "tool/parts/src/lib.rs"),
        ("parts.tests.rs.tmpl", "tool/parts/tests/example.rs"),
        ("declare.Cargo.toml.tmpl", "tool/declare/Cargo.toml"),
        ("declare.lib.rs.tmpl", "tool/declare/src/lib.rs"),
        ("contract.rs.tmpl", "tool/declare/src/contract.rs"),
        ("cli.Cargo.toml.tmpl", "tool/cli/Cargo.toml"),
        ("cli.main.rs.tmpl", "tool/cli/src/main.rs"),
        ("mcp.Cargo.toml.tmpl", "tool/mcp/Cargo.toml"),
        ("mcp.main.rs.tmpl", "tool/mcp/src/main.rs"),
        ("mcp.json.tmpl", "mcp.json"),
        ("tool.json.tmpl", "tool.json"),
        ("gitignore.tmpl", ".gitignore"),
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
    let here = or_misuse!(skill_root(given));
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
        "次に書くもの ── 差し込む場所（{{…}}）を埋め、部品を tool/parts/src/ へ置く".to_owned(),
    );
    lines.push(
        "組む ── Skill のフォルダで cargo install --path tool/cli --root . --target-dir tool/target（mcp も同じ）。bin/ に置かれる"
            .to_owned(),
    );
    lines.join("\n")
}

/// 配布元のリポジトリに置く一式。**導入スクリプトと組み立ての定義は、配布元に1つずつ置く**
/// ── Skill ごとに複製すると、直しても既存の Skill に反映されない（ACDR 0029）。
fn dist_items(repo: &str, here: &Path) -> std::io::Result<Vec<scaffold::Item>> {
    let tmpl = here.join("references/distribution");
    let mut out = Vec::new();
    for (from, to) in [
        ("install.sh.tmpl", "install.sh"),
        ("install.ps1.tmpl", "install.ps1"),
        ("release.yml.tmpl", ".github/workflows/release.yml"),
    ] {
        let body = std::fs::read_to_string(tmpl.join(from))?.replace("{{配布元}}", repo);
        out.push(scaffold::Item::keep(PathBuf::from(to), body));
    }
    Ok(out)
}

fn run_dist(given: &Given) -> Outcome {
    let repo = given.one("repo", "");
    if repo.split('/').filter(|x| !x.is_empty()).count() != 2 {
        return Outcome::misuse(format!("配布元を 所有者/リポジトリ の形で渡す ── {repo}"));
    }
    let here = or_misuse!(skill_root(given));
    let root = PathBuf::from(given.one("path", "."));
    let items = match dist_items(repo, &here) {
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

fn human_dist(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    let mut lines: Vec<String> = out
        .data
        .get("written")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|x| x.as_str())
        .map(|p| format!("置いた: {p}"))
        .collect();
    lines.extend(out.findings.iter().cloned());
    lines.push(
        "公開 ── tag <Skill の名前>-v<版> を push すると、その Skill だけを組み立て ・ 試験 ・ 公開する（v<版> は全部をまとめて公開する）".to_owned(),
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
                Arg::opt(
                    "skill_root",
                    "この Skill の置き場所（既定は、実行ファイルの1つ上）",
                    None,
                ),
            ],
            run: run_scaffold,
            human: human_scaffold,
        },
        Tool {
            name: "check",
            summary: "契約を満たしているかを検査する",
            args: vec![
                Arg::need("path", "Skill のフォルダ"),
                Arg::opt(
                    "skill_root",
                    "この Skill の置き場所（既定は、実行ファイルの1つ上）",
                    None,
                ),
            ],
            run: run_check,
            human: human_check,
        },
        Tool {
            name: "dist",
            summary: "配布元のリポジトリに、導入スクリプトと組み立ての定義を置く",
            args: vec![
                Arg::need("repo", "配布元（所有者/リポジトリ）"),
                Arg::opt("path", "配布元のリポジトリの場所", Some(".")),
                Arg::opt(
                    "skill_root",
                    "この Skill の置き場所（既定は、実行ファイルの1つ上）",
                    None,
                ),
            ],
            run: run_dist,
            human: human_dist,
        },
    ]
}
