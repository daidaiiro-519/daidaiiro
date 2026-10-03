// SPDX-License-Identifier: MIT
//! リファレンス実装と Skill の型を読み、置く一式を決める（ボード skills-creator-contract ・ ACDR 0097）。
//!
//! **実装は Rust のリファレンス実装を1組だけ持つ**（`references/sample/rust/`）。ほかの言語で作る Skill には、
//! 言語に依存しない枠（`references/types/skeleton.json`）と型の一式だけを置き、道具は AI がリファレンス実装から
//! 移植して、契約のテストケースに合格させる。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::data_access::files;

/// リファレンス実装の言語。
pub const RUST: &str = "rust";

/// リファレンス実装の定義のファイル名。
pub const DEFINITION: &str = "sample.json";

/// 言語に依存しない枠の定義（`references/types/` からの相対）。
pub const SKELETON: &str = "skeleton.json";

/// リファレンス実装1組。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Sample {
    /// 言語の名前。
    pub name: String,
    /// Skill のフォルダからの相対の経路。**これが在れば、その Skill はリファレンス実装と同じ言語で作られている。**
    pub detect: String,
    /// 全ての型に置く一式 ── 雛形（リファレンス実装のフォルダからの相対）と、置く先（Skill のフォルダからの相対）。
    pub common: Vec<(String, String)>,
    /// 型ごとに足す一式。**ここに無い型は生めない。**
    pub types: BTreeMap<String, Vec<(String, String)>>,
    /// ソースのファイルの拡張子。
    pub extensions: Vec<String>,
    /// ソースを探すときに入らないフォルダの名前。
    pub skip: Vec<String>,
    /// 外部の道具を起動する書き方。**この直後に文字列が在れば、名前の直書きである。**
    pub spawn: Vec<String>,
    /// 雛形の構成の検査（`--layout 1`）を持つか。
    pub layout: bool,
}

fn strings(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|x| x.as_str().map(str::to_owned))
        .collect()
}

fn pairs(v: Option<&Value>) -> Vec<(String, String)> {
    v.and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|i| {
            let s = |k: &str| {
                i.get(k)
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned()
            };
            (s("from"), s("to"))
        })
        .collect()
}

fn read(at: &Path) -> Result<Value, String> {
    let body =
        files::read_to_string(at).map_err(|e| format!("{} を読めない ── {e}", at.display()))?;
    serde_json::from_str(&body).map_err(|e| format!("{} が JSON でない ── {e}", at.display()))
}

/// リファレンス実装の定義を読む。`dir` はリファレンス実装のフォルダ（`references/sample/rust/`）である。
///
/// # Errors
///
/// 定義が無いか、読めないときに返す。
pub fn load(dir: &Path) -> Result<Sample, String> {
    let v = read(&dir.join(DEFINITION))?;
    let text = |k: &str| {
        v.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    let types = v
        .get("types")
        .and_then(Value::as_object)
        .map(|m| m.iter().map(|(k, x)| (k.clone(), pairs(Some(x)))).collect())
        .unwrap_or_default();
    let sources = v.get("sources").cloned().unwrap_or(Value::Null);
    Ok(Sample {
        name: text("name"),
        detect: text("detect"),
        common: pairs(v.get("common")),
        types,
        extensions: strings(&sources, "extensions"),
        skip: strings(&sources, "skip"),
        spawn: strings(&v, "spawn"),
        layout: v.get("layout").and_then(Value::as_bool).unwrap_or(false),
    })
}

/// Skill がリファレンス実装と同じ言語で作られているか。**定義の `detect` が在るかで決める。**
#[must_use]
pub fn is_written_in(root: &Path, sample: &Sample) -> bool {
    !sample.detect.is_empty() && files::is_file(root.join(&sample.detect))
}

/// Skill の型1つ（ACDR 0060）。**言語に依存しない一式を持つ** ── スキーマ ・ SKILL.md の雛形。
/// 道具のコードはリファレンス実装の `types/<型>/` が持つ。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SkillType {
    /// 型の名前（work ・ generate ・ advisor）。
    pub name: String,
    /// 人が読む名前（作業型など）。
    pub label: String,
    /// 生んだあとに次にすること（人向けの1文）。
    pub next: String,
    /// 言語に依存しない一式 ── 雛形（型のフォルダからの相対）と、置く先。
    pub items: Vec<(String, String)>,
}

/// 名前で型を1つ読む。
///
/// # Errors
///
/// 定義が無いか、読めないときに返す。無いときは、在る型の名前を並べる。
pub fn load_type(dir: &Path, name: &str) -> Result<SkillType, String> {
    let at = dir.join(name).join("type.json");
    if !files::is_file(&at) {
        let known: Vec<String> = files::list(dir)
            .unwrap_or_default()
            .into_iter()
            .filter(|p| files::is_file(p.join("type.json")))
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect();
        return Err(format!(
            "型が無い: {name} ── 在るのは {}",
            known.join(" ・ ")
        ));
    }
    let v = read(&at)?;
    let text = |k: &str| {
        v.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    Ok(SkillType {
        name: text("name"),
        label: text("label"),
        next: text("next"),
        items: pairs(v.get("items")),
    })
}

/// 言語の名前の形。**英小文字で始まり、英小文字と数字だけ**である ── 置く一式には入らないが、案内に出す。
#[must_use]
pub fn is_language_name(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

/// 型と言語から、置く一式を決める。**雛形の経路は絶対の経路で返す。** 後に並ぶものが、同じ置き先の
/// 前のものを置き換える ── 言語に依存しない枠 → リファレンス実装の共通の一式 → リファレンス実装の
/// 型の一式 → 型の言語に依存しない一式の順。**リファレンス実装と違う言語なら、リファレンス実装の
/// 一式を置かない** ── 道具は移植して書く。
///
/// # Errors
///
/// リファレンス実装が型の一式を持たないとき（移植の元が無い）と、枠の定義を読めないときに返す。
pub fn plan(
    sample_dir: &Path,
    types: &Path,
    sample: &Sample,
    ty: &SkillType,
    language: &str,
) -> Result<Vec<(PathBuf, String)>, String> {
    let Some(own) = sample.types.get(&ty.name) else {
        let have: Vec<&str> = sample.types.keys().map(String::as_str).collect();
        return Err(format!(
            "リファレンス実装は{}の雛形をまだ持たない ── 生める型は {}",
            ty.label,
            have.join(" ・ ")
        ));
    };
    let skeleton = pairs(read(&types.join(SKELETON))?.get("items"));
    let mut out: Vec<(PathBuf, String)> = Vec::new();
    let mut put = |from: PathBuf, to: &str| {
        out.retain(|(_, t)| t != to);
        out.push((from, to.to_owned()));
    };
    for (from, to) in &skeleton {
        put(types.join(from), to);
    }
    if language == sample.name {
        for (from, to) in sample.common.iter().chain(own) {
            put(sample_dir.join(from), to);
        }
    }
    for (from, to) in &ty.items {
        put(types.join(&ty.name).join(from), to);
    }
    Ok(out)
}
