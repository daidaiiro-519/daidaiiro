//! crate の依存の向きを、ワークスペースの根の architecture.json と照らす（SAS の同じ検査を写したもの）。
//!
//! cargo metadata が返すワークスペースの crate と依存を読み、次を確かめる。
//! - architecture.json の crate と、ワークスペースの members が一致する
//! - crate の名前がフォルダから決まる（<文脈>-<層> ・ <モジュール>-<層> ・ <inbound|outbound>-<技術> ・ test-support）
//! - 通常の依存（normal ・ build）は、ワークスペースの crate なら allow の to、外のライブラリなら allow の libraries に書いたものだけ
//! - dev_only の crate（試験の部品）は dev-dependencies にしか現れない

// 試験の補助の関数でも expect を使う（clippy.toml が #[test] の中で許すのと同じ考え）。
#![allow(clippy::expect_used)]

use serde_json::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_json(path: PathBuf) -> Value {
    let text = std::fs::read_to_string(&path).expect("architecture.json を読めること");
    serde_json::from_str(&text).expect("architecture.json が JSON であること")
}

fn metadata() -> Value {
    let out = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(root())
        .output()
        .expect("cargo metadata を実行できること");
    assert!(out.status.success(), "cargo metadata が失敗した");
    serde_json::from_slice(&out.stdout).expect("cargo metadata の出力が JSON であること")
}

fn names(v: &Value, key: &str) -> BTreeSet<String> {
    v[key]
        .as_array()
        .expect("配列であること")
        .iter()
        .map(|c| {
            c["name"]
                .as_str()
                .expect("name が文字列であること")
                .to_string()
        })
        .collect()
}

#[test]
fn crates_match_members() {
    let arch = read_json(root().join("architecture.json"));
    let meta = metadata();
    assert_eq!(names(&arch, "crates"), names(&meta, "packages"));
}

/// 文脈の中の経路から crate の名前を決める：文脈の直下の層（application ・ domain）→ <文脈>-<層>、
/// <モジュール>/<層> → <モジュール>-<層>、adapters/<inbound|outbound>/<技術> → <inbound|outbound>-<技術>、
/// それ以外は最後のフォルダの名前。
fn name_from_path(path: &str, context: &str) -> String {
    let inner = path
        .strip_prefix(&format!("{context}/"))
        .expect("crate は文脈のフォルダの中にあること");
    let parts: Vec<&str> = inner.split('/').collect();
    match parts.as_slice() {
        ["adapters", side, tech] => format!("{side}-{tech}"),
        [layer @ ("application" | "domain")] => format!("{context}-{layer}"),
        [module, layer] => format!("{module}-{layer}"),
        _ => (*parts.last().expect("経路があること")).to_string(),
    }
}

#[test]
fn crate_names_follow_their_folders() {
    let arch = read_json(root().join("architecture.json"));
    let context = arch["context"].as_str().expect("context があること");
    for c in arch["crates"].as_array().expect("crates が配列であること") {
        let name = c["name"].as_str().expect("name が文字列であること");
        let path = c["path"].as_str().expect("path が文字列であること");
        assert_eq!(
            name,
            name_from_path(path, context),
            "{path} の crate の名前"
        );
    }
}

#[test]
fn dependencies_follow_allow() {
    let arch = read_json(root().join("architecture.json"));
    let dev_only: BTreeSet<String> = arch["dev_only"]
        .as_array()
        .expect("dev_only が配列であること")
        .iter()
        .map(|v| v.as_str().expect("文字列であること").to_string())
        .collect();
    let allow = |from: &str, key: &str| -> BTreeSet<String> {
        arch["allow"]
            .as_array()
            .expect("allow が配列であること")
            .iter()
            .find(|a| a["from"] == from)
            .map(|a| {
                a[key]
                    .as_array()
                    .expect("to と libraries が配列であること")
                    .iter()
                    .map(|v| v.as_str().expect("文字列であること").to_string())
                    .collect()
            })
            .unwrap_or_default()
    };
    let meta = metadata();
    let workspace = names(&meta, "packages");
    let mut broken = Vec::new();
    for pkg in meta["packages"]
        .as_array()
        .expect("packages が配列であること")
    {
        let from = pkg["name"].as_str().expect("name が文字列であること");
        let allowed = allow(from, "to");
        let libraries = allow(from, "libraries");
        for dep in pkg["dependencies"]
            .as_array()
            .expect("dependencies が配列であること")
        {
            let to = dep["name"].as_str().expect("name が文字列であること");
            let is_dev = dep["kind"].as_str() == Some("dev");
            if dev_only.contains(to) && !is_dev {
                broken.push(format!("{from} → {to}（試験の部品を通常の依存にしている）"));
            } else if is_dev {
                continue;
            } else if workspace.contains(to) && !allowed.contains(to) {
                broken.push(format!("{from} → {to}（architecture.json に無い向き）"));
            } else if !workspace.contains(to) && !libraries.contains(to) {
                broken.push(format!(
                    "{from} → {to}（architecture.json の libraries に無い外のライブラリ）"
                ));
            }
        }
    }
    assert!(broken.is_empty(), "許していない依存: {broken:?}");
}
