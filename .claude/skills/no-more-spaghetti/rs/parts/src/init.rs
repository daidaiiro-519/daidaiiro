// SPDX-License-Identifier: MIT
//! 規則ファイルの雛形を出す。**中身は呼ぶ側が書く。**
//!
//! 置き場所は**リポジトリの `.coding-rules/rules.json`** である ── 管理する対象を1つに
//! する。成果物が複数在るときは、規則1件ごとに `check.target` でどこを検査するかを書く。
//!
//! **道具の名前を1つも持たない。** 雛形は契約の形から組み、何を入れるかは各項目の
//! `x-prompt.write` が案内する。

use std::io;
use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

/// 規則ファイルを置く包み。
pub const RULES_DIR: &str = ".coding-rules";

/// 内を指す規則。**出典はモデルであり、原典を要さない。**
pub const INNER_RULES: [&str; 2] = [
    "層の場所が、宣言した対応と一致する",
    "依存の向きが、内から外へ出ていない",
];

/// 層の宣言1件。
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Layer {
    /// 層の名前。
    pub name: String,
    /// その言語での識別子。
    pub id: String,
}

impl Layer {
    /// 層を組む。
    #[must_use]
    pub const fn new(name: String, id: String) -> Self {
        Self { name, id }
    }
}

/// `名前=識別子` の並びを読む。**識別子の形は問わない** ── 言語ごとに違う。
///
/// # Errors
///
/// 形が `名前=識別子` でないときに返す。
pub fn parse_layers(pairs: &[String]) -> Result<Vec<Layer>, String> {
    let mut out = Vec::new();
    for item in pairs {
        let Some((name, id)) = item.split_once('=') else {
            return Err(format!("層は 名前=識別子 で渡す ── {item}"));
        };
        let (name, id) = (name.trim(), id.trim());
        if name.is_empty() || id.is_empty() {
            return Err(format!("層は 名前=識別子 で渡す ── {item}"));
        }
        out.push(Layer::new(name.to_owned(), id.to_owned()));
    }
    Ok(out)
}

/// 契約の形から雛形を組む。**項目の一覧を、この側に書かない。**
///
/// # Errors
///
/// 契約を読めないときに返す。
pub fn skeleton(contract: &Path, layers: &[Layer], target: &str) -> io::Result<Value> {
    let body = std::fs::read_to_string(contract)?;
    let schema: Value = serde_json::from_str(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let shape = schema
        .get("$defs")
        .and_then(|d| d.get("rule"))
        .and_then(|r| r.get("properties"))
        .and_then(Value::as_object)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "契約が規則の形を持たない"))?;

    let rule = |name: &str| -> Value {
        let mut out = Map::new();
        for (key, spec) in shape {
            let value = match key.as_str() {
                "rule" => json!(name),
                "check" => {
                    let mut check = Map::new();
                    check.insert("tool".to_owned(), json!([]));
                    if !target.is_empty() {
                        check.insert("target".to_owned(), json!(target));
                    }
                    Value::Object(check)
                }
                _ => {
                    if spec.get("type").and_then(Value::as_str) == Some("string") {
                        json!("")
                    } else {
                        json!({})
                    }
                }
            };
            out.insert(key.clone(), value);
        }
        Value::Object(out)
    };

    let mut mapped = Map::new();
    for layer in layers {
        let id = if target.is_empty() {
            layer.id.clone()
        } else {
            format!("{}/{}", target.trim_end_matches('/'), layer.id)
        };
        mapped.insert(layer.name.clone(), json!(id));
    }
    Ok(json!({
        "$schema": "…/no-more-spaghetti/references/rules.schema.json",
        "order": layers.iter().map(|l| l.name.clone()).collect::<Vec<_>>(),
        "layers": Value::Object(mapped),
        "rules": INNER_RULES.iter().map(|x| rule(x)).collect::<Vec<_>>(),
    }))
}

/// リポジトリの `.coding-rules/` へ、成果物のファイルを置く。
///
/// **既に在れば作り直さない** ── 書いた規則が消える。
///
/// # Errors
///
/// 層が空のとき、既に在るとき、または書けないときに返す。
pub fn create(root: &Path, target: &str, layers: &[Layer], contract: &Path) -> io::Result<PathBuf> {
    if layers.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "層を1つ以上渡す ── 層の無い規則ファイルは、何も検査できない",
        ));
    }
    let path = root.join(RULES_DIR).join("rules.json");
    if path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("既に在る: {} ── 作り直さない", path.display()),
        ));
    }
    let repo_wide = target.is_empty() || target == ".";
    let body = skeleton(contract, layers, if repo_wide { "" } else { target })?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    std::fs::write(&path, text + "\n")?;
    Ok(path)
}
