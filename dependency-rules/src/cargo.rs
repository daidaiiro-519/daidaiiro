//! Rust の読み取り。公式の `cargo metadata --no-deps --format-version 1` を起動し、
//! workspace の member が Cargo.toml に書いた依存を、グラフ（node と edge）に直す。
//!
//! - node の名前は package の name（依存では dependencies の name。rename の名前は使わない）
//! - 辺の種類は dependencies の kind（"dev" ・ "build"、null は "normal"）
//! - optional の依存も feature に依らず入る。target（プラットフォームの条件）は見ない
//! - 内の node は workspace_members の package、外の node は内の node の直接の依存だけ

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::core::{Edge, Graph, Node};

/// `cargo metadata` を起動して JSON を受け取る
pub fn metadata(root: &Path) -> Result<String, String> {
    let manifest = root.join("Cargo.toml");
    let out = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()
        .map_err(|e| format!("cargo を起動できない：{e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo metadata が失敗した：{}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("cargo metadata の出力が UTF-8 でない：{e}"))
}

/// `cargo metadata --no-deps` の JSON をグラフに直す
pub fn to_graph(json: &str) -> Result<Graph, String> {
    let v: Value = serde_json::from_str(json)
        .map_err(|e| format!("cargo metadata の出力が JSON でない：{e}"))?;
    let members: BTreeSet<&str> = v["workspace_members"]
        .as_array()
        .ok_or("cargo metadata に workspace_members が無い")?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let packages = v["packages"]
        .as_array()
        .ok_or("cargo metadata に packages が無い")?;
    let inner: Vec<&Value> = packages
        .iter()
        .filter(|p| p["id"].as_str().is_some_and(|id| members.contains(id)))
        .collect();
    let inner_names: BTreeSet<&str> = inner.iter().filter_map(|p| p["name"].as_str()).collect();
    let mut nodes: BTreeMap<String, bool> =
        inner_names.iter().map(|n| (n.to_string(), false)).collect();
    let mut edges = BTreeSet::new();
    for p in &inner {
        let from = p["name"].as_str().ok_or("package に name が無い")?;
        for d in p["dependencies"].as_array().into_iter().flatten() {
            let to = d["name"].as_str().ok_or("依存に name が無い")?;
            let kind = d["kind"].as_str().unwrap_or("normal");
            nodes
                .entry(to.to_string())
                .or_insert(!inner_names.contains(to));
            edges.insert(Edge {
                from: from.to_string(),
                to: to.to_string(),
                kind: kind.to_string(),
            });
        }
    }
    Ok(Graph {
        nodes: nodes
            .into_iter()
            .map(|(name, external)| Node { name, external })
            .collect(),
        edges: edges.into_iter().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const META: &str = r#"{
      "packages": [
        {"name": "app", "id": "path+file:///w/app#0.1.0", "dependencies": [
          {"name": "domain", "kind": null, "rename": "d", "optional": false},
          {"name": "serde", "kind": null, "optional": true},
          {"name": "tempfile", "kind": "dev"},
          {"name": "cc", "kind": "build", "target": "cfg(unix)"}
        ]},
        {"name": "domain", "id": "path+file:///w/domain#0.1.0", "dependencies": []}
      ],
      "workspace_members": ["path+file:///w/app#0.1.0", "path+file:///w/domain#0.1.0"]
    }"#;

    #[test]
    fn 依存の_name_と_kind_から辺を作る() {
        let g = to_graph(META).unwrap();
        let es: Vec<(&str, &str, &str)> = g
            .edges
            .iter()
            .map(|e| (e.from.as_str(), e.to.as_str(), e.kind.as_str()))
            .collect();
        assert_eq!(
            es,
            vec![
                ("app", "cc", "build"),
                ("app", "domain", "normal"),
                ("app", "serde", "normal"),
                ("app", "tempfile", "dev")
            ],
            "rename ではなく name、null は normal、optional と target 付きも入る"
        );
    }

    #[test]
    fn member_は内の_node_それ以外は外の_node() {
        let g = to_graph(META).unwrap();
        let ext: Vec<(&str, bool)> = g
            .nodes
            .iter()
            .map(|n| (n.name.as_str(), n.external))
            .collect();
        assert_eq!(
            ext,
            vec![
                ("app", false),
                ("cc", true),
                ("domain", false),
                ("serde", true),
                ("tempfile", true)
            ]
        );
    }
}
