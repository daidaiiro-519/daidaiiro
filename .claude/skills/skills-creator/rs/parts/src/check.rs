// SPDX-License-Identifier: MIT
//! Skill が契約を満たしているかを検査する。**見つけるが、直さない。**
//!
//! 検査するのは3つである ── 層が crate に分かれていること、**許可辺が宣言どおりで
//! あること**、**節の構成が対応する雛形を満たすこと**。
//!
//! **層を crate に分ける。** 1つの crate の中の module では、内側が外側を参照しても
//! コンパイラが通す（実測 2026-09-26）── 層の境界を crate の境界に置いて初めて、
//! 宣言に無い依存が解決しなくなる。
//!
//! **許可辺は各 `Cargo.toml` が宣言する。** この検査は宣言を読み、並びと食い違って
//! いないかを見る ── 実際に守られているかはコンパイラが判定する。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::sections;

/// Skill の道具を置く場所。
pub const RS: &str = "rs";

/// 層の並び。**内から外である** ── 先頭が最も内側である。
pub const ORDER: [&str; 3] = ["parts", "declare", "entry"];

/// 入口の層に属する crate。**合成する側なので複数在ってよい。**
pub const ENTRIES: [&str; 2] = ["cli", "mcp"];

/// 事例を置く場所（`rs/` からの相対）。
pub const TESTS: &str = "parts/tests";

/// 助言の Skill を見分ける節の名前。**名前で分岐しない** ── Skill が増えるたびに直す。
pub const ADVISOR_MARK: &str = "相談種別と回答テンプレート";

/// 雛形の置き場所。**呼ぶ側が渡す** ── この crate が Skill の並びを認知しない。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Templates {
    /// 一般の Skill が満たす雛形。
    pub general: PathBuf,
    /// 助言の Skill が満たす雛形。
    pub advisor: PathBuf,
}

impl Templates {
    /// 雛形の置き場所を組む。**欄を足しても、呼ぶ側は壊れない。**
    #[must_use]
    pub const fn new(general: PathBuf, advisor: PathBuf) -> Self {
        Self { general, advisor }
    }
}

/// その層が参照してよい層を返す。**並びから導く** ── 表を別に持つと、並びとずれる。
#[must_use]
fn allowed(layer: &str) -> Vec<&'static str> {
    let at = ORDER.iter().position(|x| *x == layer);
    match at {
        Some(i) => ORDER[..i].to_vec(),
        None => Vec::new(),
    }
}

/// crate の名前から、属する層を決める。
#[must_use]
fn layer_of(name: &str) -> Option<&'static str> {
    let tail = name.rsplit('_').next().unwrap_or(name);
    if ENTRIES.contains(&tail) {
        return Some("entry");
    }
    ORDER.iter().copied().find(|x| *x == tail)
}

/// `Cargo.toml` が宣言している、同じ workspace の中の依存を読む。
///
/// **`path = "../…"` の形だけを見る** ── 外の crate は層の外である。
fn declared(manifest: &Path) -> Vec<String> {
    let Ok(body) = fs::read_to_string(manifest) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut in_deps = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_deps = trimmed == "[dependencies]";
            continue;
        }
        if !in_deps || !trimmed.contains("path") {
            continue;
        }
        let Some((name, _)) = trimmed.split_once('=') else {
            continue;
        };
        out.push(name.trim().to_owned());
    }
    out
}

/// workspace が並べている crate を読む。
fn members(manifest: &Path) -> Vec<String> {
    let Ok(body) = fs::read_to_string(manifest) else {
        return Vec::new();
    };
    let Some(head) = body.find("members") else {
        return Vec::new();
    };
    let rest = &body[head..];
    let Some(open) = rest.find('[') else {
        return Vec::new();
    };
    let Some(close) = rest[open..].find(']') else {
        return Vec::new();
    };
    rest[open + 1..open + close]
        .split(',')
        .map(|x| x.trim().trim_matches('"').to_owned())
        .filter(|x| !x.is_empty())
        .collect()
}

/// 節の構成が、対応する雛形を満たすかを見る。**文書が無ければ、そう返す。**
fn missing_sections(root: &Path, templates: &Templates) -> Vec<String> {
    let document = root.join("SKILL.md");
    let Ok(body) = fs::read_to_string(&document) else {
        return vec!["文書が無い: SKILL.md".to_owned()];
    };
    let is_advisor = sections::headings(&body).iter().any(|x| x == ADVISOR_MARK);
    let template = if is_advisor {
        &templates.advisor
    } else {
        &templates.general
    };
    let Ok(want) = fs::read_to_string(template) else {
        return Vec::new();
    };
    let name = template
        .file_name()
        .map_or_else(String::new, |x| x.to_string_lossy().into_owned());
    sections::missing(&body, &want)
        .into_iter()
        .map(|x| format!("節が無い: {x} ── {name} が要求する"))
        .collect()
}

/// 道具を持つ Skill かを返す。**助言と手順だけの Skill には、道具を要求しない。**
#[must_use]
pub fn has_tools(root: &Path) -> bool {
    root.join(RS).join("Cargo.toml").is_file() || root.join("scripts").is_dir()
}

/// 契約を満たしているかを検査し、食い違いを並べる。
///
/// # Errors
///
/// いまは返さない。呼ぶ側の形を変えずに、あとから検査を足せるようにしてある。
pub fn check(root: &Path, templates: &Templates) -> io::Result<Vec<String>> {
    let mut findings = Vec::new();
    let rs = root.join(RS);

    // **道具を持たない Skill に、層を要求しない。** 助言と手順だけの Skill が在る
    if has_tools(root) {
        if !rs.join("Cargo.toml").is_file() {
            findings.push(format!(
                "層が crate に分かれていない: {RS}/Cargo.toml が無い ── \
                 1つの単位の中の module では、内側が外側を参照してもコンパイラが通す"
            ));
        } else {
            let listed = members(&rs.join("Cargo.toml"));
            for need in ORDER.iter().copied().filter(|x| *x != "entry") {
                if !listed.iter().any(|x| x == need) {
                    findings.push(format!("層の crate が無い: {RS}/{need}"));
                }
            }
            if !ENTRIES.iter().any(|e| listed.iter().any(|x| x == e)) {
                findings.push(format!(
                    "入口の crate が無い: {RS}/ に {} のどれかを置く",
                    ENTRIES.join(" か ")
                ));
            }
            // **許可辺が宣言どおりかを見る。** 守られているかはコンパイラが判定する
            for member in &listed {
                let manifest = rs.join(member).join("Cargo.toml");
                if !manifest.is_file() {
                    findings.push(format!("宣言が無い: {RS}/{member}/Cargo.toml"));
                    continue;
                }
                let Some(here) = layer_of(member) else {
                    findings.push(format!(
                        "どの層か決まらない: {RS}/{member} ── 名前を {} か {} で終える",
                        ORDER.join(" ・ "),
                        ENTRIES.join(" ・ ")
                    ));
                    continue;
                };
                let may = allowed(here);
                for dep in declared(&manifest) {
                    let Some(to) = layer_of(&dep) else { continue };
                    if !may.contains(&to) {
                        findings.push(format!(
                            "許可していない辺を宣言している: {RS}/{member}/Cargo.toml ── \
                             {here} → {to}"
                        ));
                    }
                }
            }
            if !rs.join(TESTS).is_dir() {
                findings.push(format!("事例が無い: {RS}/{TESTS}/"));
            }
        }
        // **Python を残さない。** 移行が済んでいない箇所を、黙って通さない
        if root.join("scripts").is_dir() {
            findings.push("Python が残っている: scripts/ ── 道具は rs/ が持つ".to_owned());
        }
    }

    findings.extend(missing_sections(root, templates));
    Ok(findings)
}
