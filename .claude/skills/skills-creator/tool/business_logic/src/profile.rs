// SPDX-License-Identifier: MIT
//! 言語の組の定義を読む。**組ごとの違いは `references/profiles/<名前>.profile.json` が持つ**
//! （ACDR 0060）── 何をどこへ置くか ・ どのファイルが在ればこの組か ・ 外部の道具を起動する書き方。
//! コードに組の名前で分岐を書くと、組を足すたびにここを直すことになる。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::data_access::files;

/// 定義のファイル名の末尾。
pub const TAIL: &str = ".profile.json";

/// 言語の組1つ。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Profile {
    /// 組の名前（例：rust ・ python）。
    pub name: String,
    /// Skill のフォルダからの相対の経路。**これが在れば、その Skill はこの組で作られている。**
    pub detect: String,
    /// 満たす契約の版。**型が要る版に届かなければ、その型は生めない。**
    pub contract: u64,
    /// 全ての型に置く共通の一式 ── 雛形（組のフォルダからの相対）と、置く先（Skill のフォルダからの相対）。
    pub common: Vec<(String, String)>,
    /// 型ごとに足す一式。**ここに無い型は、この組では生めない。**
    pub types: BTreeMap<String, Vec<(String, String)>>,
    /// 生んだあとに Skill のフォルダで実行する組み立てのコマンド。
    pub build: Vec<String>,
    /// ソースのファイルの拡張子。
    pub extensions: Vec<String>,
    /// ソースを探すときに入らないフォルダの名前。
    pub skip: Vec<String>,
    /// 外部の道具を起動する書き方。**この直後に文字列が在れば、名前の直書きである。**
    pub spawn: Vec<String>,
    /// 作った Skill の配布（dist）に対応するか。
    pub dist: bool,
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

fn parse(body: &str, at: &Path) -> Result<Profile, String> {
    let v: Value = serde_json::from_str(body)
        .map_err(|e| format!("{} が JSON でない ── {e}", at.display()))?;
    let text = |k: &str| {
        v.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    let flag = |k: &str| v.get(k).and_then(Value::as_bool).unwrap_or(false);
    let types = v
        .get("types")
        .and_then(Value::as_object)
        .map(|m| m.iter().map(|(k, x)| (k.clone(), pairs(Some(x)))).collect())
        .unwrap_or_default();
    let sources = v.get("sources").cloned().unwrap_or(Value::Null);
    Ok(Profile {
        name: text("name"),
        detect: text("detect"),
        contract: v.get("contract").and_then(Value::as_u64).unwrap_or(1),
        common: pairs(v.get("common")),
        types,
        build: strings(&v, "build"),
        extensions: strings(&sources, "extensions"),
        skip: strings(&sources, "skip"),
        spawn: strings(&v, "spawn"),
        dist: flag("dist"),
        layout: flag("layout"),
    })
}

/// 名前で組を1つ読む。
///
/// # Errors
///
/// 定義が無いか、読めないときに返す。無いときは、在る組の名前を並べる。
pub fn load(dir: &Path, name: &str) -> Result<Profile, String> {
    let at = dir.join(format!("{name}{TAIL}"));
    let Ok(body) = files::read_to_string(&at) else {
        let known: Vec<String> = all(dir).into_iter().map(|p| p.name).collect();
        return Err(format!(
            "言語の組が無い: {name} ── 在るのは {}",
            known.join(" ・ ")
        ));
    };
    parse(&body, &at)
}

/// 在る組をすべて読む。**名前の順に並べる** ── 判定の順を OS に任せない。
#[must_use]
pub fn all(dir: &Path) -> Vec<Profile> {
    files::list(dir)
        .unwrap_or_default()
        .into_iter()
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().ends_with(TAIL))
        })
        .filter_map(|p| {
            files::read_to_string(&p)
                .ok()
                .and_then(|body| parse(&body, &p).ok())
        })
        .collect()
}

/// Skill がどの組で作られているかを決める。**定義の `detect` が在る最初の組である。**
#[must_use]
pub fn of(root: &Path, dir: &Path) -> Option<Profile> {
    all(dir)
        .into_iter()
        .find(|p| !p.detect.is_empty() && files::is_file(root.join(&p.detect)))
}

/// 組の雛形の置き場所。
#[must_use]
pub fn templates_of(dir: &Path, name: &str) -> PathBuf {
    dir.join(name)
}

/// Skill の型1つ（ACDR 0060）。**言語に依存しない一式を持つ** ── スキーマ ・ SKILL.md の雛形。
/// 道具のコードは言語の組の `types/<型>/` が持つ。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SkillType {
    /// 型の名前（work ・ generate ・ advisor）。
    pub name: String,
    /// 人が読む名前（作業型など）。
    pub label: String,
    /// この型が要る契約の版。
    pub contract: u64,
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
    let body = files::read_to_string(&at).map_err(|_| {
        let known: Vec<String> = files::list(dir)
            .unwrap_or_default()
            .into_iter()
            .filter(|p| files::is_file(p.join("type.json")))
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect();
        format!("型が無い: {name} ── 在るのは {}", known.join(" ・ "))
    })?;
    let v: Value = serde_json::from_str(&body)
        .map_err(|e| format!("{} が JSON でない ── {e}", at.display()))?;
    let text = |k: &str| {
        v.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    Ok(SkillType {
        name: text("name"),
        label: text("label"),
        contract: v.get("contract").and_then(Value::as_u64).unwrap_or(1),
        items: pairs(v.get("items")),
    })
}

/// 型と言語の組から、置く一式を決める。**雛形の経路は絶対の経路で返す。** 後に並ぶものが、
/// 同じ置き先の前のものを置き換える ── 共通の一式 → 組の型の一式 → 型の言語に依存しない一式の順。
///
/// # Errors
///
/// 組が型の要る契約の版に届かないか、組が型の一式を持たないときに、対応している組を並べて返す。
pub fn plan(
    profiles: &Path,
    types: &Path,
    lang: &Profile,
    ty: &SkillType,
) -> Result<Vec<(PathBuf, String)>, String> {
    let supporting = || -> String {
        let names: Vec<String> = all(profiles)
            .into_iter()
            .filter(|p| p.contract >= ty.contract && p.types.contains_key(&ty.name))
            .map(|p| p.name)
            .collect();
        if names.is_empty() {
            "まだ無い".to_owned()
        } else {
            names.join(" ・ ")
        }
    };
    if lang.contract < ty.contract {
        return Err(format!(
            "{} の組は{}に対応しない ── {}は契約の版{}を要し、この組は版{}である。対応している組は {}",
            lang.name,
            ty.label,
            ty.label,
            ty.contract,
            lang.contract,
            supporting()
        ));
    }
    let Some(own) = lang.types.get(&ty.name) else {
        return Err(format!(
            "{} の組は{}の雛形をまだ持たない。対応している組は {}",
            lang.name,
            ty.label,
            supporting()
        ));
    };
    let base = templates_of(profiles, &lang.name);
    let mut out: Vec<(PathBuf, String)> = Vec::new();
    let mut put = |from: PathBuf, to: &str| {
        out.retain(|(_, t)| t != to);
        out.push((from, to.to_owned()));
    };
    for (from, to) in &lang.common {
        put(base.join(from), to);
    }
    for (from, to) in own {
        put(base.join(from), to);
    }
    for (from, to) in &ty.items {
        put(types.join(&ty.name).join(from), to);
    }
    Ok(out)
}
