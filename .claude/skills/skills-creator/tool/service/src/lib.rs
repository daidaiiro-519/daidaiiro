// SPDX-License-Identifier: MIT
//! skills-creator のサービス層。道具の一覧を持ち、**能力の正本はここである。**
//!
//! プレゼンテーション層（CLI ・ MCP）はこの一覧から組む ── 能力を2回書くと、片方だけが古くなる。
//! 依存の向きは `Cargo.toml` が宣言する ── この crate は業務ロジック層だけを参照する。

pub mod contract;
mod refs;

use std::path::{Path, PathBuf};

use sc_business_logic::{accept, check, profile, scaffold};
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
    // **助言型の SKILL.md の雛形は、型の置き場所が持つ**（ACDR 0061 ── advisor-creator を廃止した）
    check::Templates::new(
        references.join("skill-template.md.tmpl"),
        references.join("types/advisor/skill-template.md.tmpl"),
    )
    .with_refs(references.join("profiles/rust/common/refs.rs.tmpl"))
    .with_profiles(references.join("profiles"))
    .with_view(references.join("view"))
}

/// この Skill が置かれている場所。**実行ファイルの位置から辿らない** ── build の
/// 置き場所に依存する。引数で受け取り、渡されなければいま作業している場所を使う。
fn skill_root(given: &Given) -> Result<PathBuf, String> {
    given.skill_root()
}

fn run_accept(given: &Given) -> Outcome {
    let root = PathBuf::from(given.one("path", ""));
    if root.as_os_str().is_empty() {
        return Outcome::misuse("advisor のフォルダを渡していない".to_owned());
    }
    let here = or_misuse!(skill_root(given));
    let dir = here.join("references/profiles");
    let Some(found) = profile::of(&root, &dir) else {
        return Outcome::misuse(format!(
            "言語の組が決まらない: {} ── 組の定義の detect が在る Skill を渡す",
            root.display()
        ));
    };
    // **同じ置き場所の他の Skill の名前を集める** ── 助言型は、それらを名指ししない
    let own = root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let others = scaffold::siblings(&root)
        .into_iter()
        .filter(|n| *n != own)
        .collect::<Vec<_>>();
    let checks = accept::accept(&root, &others, &found);
    let findings: Vec<String> = checks
        .iter()
        .flat_map(|c| c.findings.iter().map(move |f| format!("{} {f}", c.no)))
        .collect();
    let data: Vec<serde_json::Value> = checks
        .iter()
        .map(|c| json!({ "no": c.no, "what": c.what, "pass": c.findings.is_empty(), "findings": c.findings }))
        .collect();
    Outcome::found(findings, json!({ "checks": data }))
}

fn human_accept(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    let mut lines = Vec::new();
    for c in out
        .data
        .get("checks")
        .and_then(|x| x.as_array())
        .into_iter()
        .flatten()
    {
        let mark = if c["pass"].as_bool().unwrap_or(false) {
            "OK"
        } else {
            "×"
        };
        lines.push(format!(
            "  {mark}  {} {}",
            c["no"],
            c["what"].as_str().unwrap_or_default()
        ));
        for f in c["findings"].as_array().into_iter().flatten() {
            lines.push(format!("       {}", f.as_str().unwrap_or_default()));
        }
    }
    lines.push("8 目視（人）── 判断基準と回答の頁を、1200px と 390px で描画して読む".to_owned());
    lines.join("\n")
}

fn run_check(given: &Given) -> Outcome {
    let root = PathBuf::from(given.one("path", ""));
    if root.as_os_str().is_empty() {
        return Outcome::misuse("Skill のフォルダを渡していない".to_owned());
    }
    let layout = given.one("layout", "0") == "1";
    let report = check::check(&root, &templates(&or_misuse!(skill_root(given))), layout);
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

/// 置く一式を組む。**何を置くかはここが決める** ── 業務ロジック層は並びを認知しない。
///
/// **層を crate に分ける。** 1つの crate の中の module では、内側が外側を参照しても
/// コンパイラが通す ── 層の境界を crate の境界に置いて初めて、`Cargo.toml` に書いていない依存が
/// 解決しなくなる。
fn items(skill: &str, plan: &[(PathBuf, String)]) -> std::io::Result<Vec<scaffold::Item>> {
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
    // **パッケージの名前は、Skill の名前の区切りを _ にしたもの**（- を使えない言語のため）
    let package = skill.replace('-', "_");
    let fill = |body: String| {
        body.replace("{{Skill名}}", skill)
            .replace("{{接頭辞}}", &prefix)
            .replace("{{パッケージ名}}", &package)
    };
    let mut out = Vec::new();
    // **何をどこへ置くかは、型と言語の組の定義が決める**（ACDR 0060）
    for (from, to) in plan {
        let dir = from.parent().unwrap_or_else(|| Path::new("."));
        let name = from
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        out.push(scaffold::Item::keep(
            PathBuf::from(fill(to.clone())),
            fill(scaffold::read_template(dir, &name)?),
        ));
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
    let dir = here.join("references/profiles");
    let types = here.join("references/types");
    let found = or_misuse!(profile::load(&dir, given.one("language", "rust")));
    let ty = or_misuse!(profile::load_type(&types, given.one("type", "work")));
    let plan = or_misuse!(profile::plan(&dir, &types, &found, &ty));
    let items = match items(skill, &plan) {
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
        json!({
            "written": placed.written,
            "root": root.display().to_string(),
            "language": found.name,
            "type": ty.name,
            "next": ty.next,
            // **組み立てのコマンドにも Skill の名前を差し込む** ── 実行ファイルの名前を持つ組が在る
            "build": found
                .build
                .iter()
                .map(|b| b.replace("{{Skill名}}", skill))
                .collect::<Vec<_>>(),
        }),
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
    if let Some(next) = out.data.get("next").and_then(|x| x.as_str()) {
        lines.push(format!("次に書くもの ── {next}"));
    }
    let build: Vec<String> = out
        .data
        .get("build")
        .and_then(|x| x.as_array())
        .into_iter()
        .flatten()
        .filter_map(|x| x.as_str().map(str::to_owned))
        .collect();
    if !build.is_empty() {
        lines.push(format!("組む ── Skill のフォルダで {}", build.join(" ／ ")));
    }
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
        let body = scaffold::read_template(&tmpl, from)?.replace("{{配布元}}", repo);
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
    let mut all = vec![
        Tool {
            name: "scaffold",
            summary: "道具の契約一式を、Skill のフォルダへ置く",
            args: vec![
                Arg::need("skill", "Skill の名前"),
                Arg::opt("path", "置き場所", Some(".claude/skills")),
                Arg::opt(
                    "type",
                    "Skill の型（work ＝作業型 ・ generate ＝生成型 ・ advisor ＝助言型）",
                    Some("work"),
                ),
                Arg::opt(
                    "language",
                    "雛形の言語の組（references/profiles/ に定義が在るもの）",
                    Some("rust"),
                ),
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
                    "layout",
                    "1 なら、雛形の構成（層 ・ 依存の向き ・ 入出力の置き場所）も検査する。既定は契約だけ",
                    Some("0"),
                ),
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
            name: "accept",
            summary: "助言型の受け入れの検査（機械の7件）を行う",
            args: vec![
                Arg::need("path", "advisor のフォルダ"),
                Arg::opt(
                    "skill_root",
                    "この Skill の置き場所（既定は、実行ファイルの1つ上）",
                    None,
                ),
            ],
            run: run_accept,
            human: human_accept,
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
    ];
    // **references の4つの道具（get ・ validate ・ view ・ import）は、どの Skill も同じものを足す**（契約の版2）
    all.extend(refs::tools());
    all
}
