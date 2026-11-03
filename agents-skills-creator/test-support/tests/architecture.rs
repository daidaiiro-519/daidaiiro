//! crate の依存の向きを、ワークスペースの根の architecture.json（architecture-lint の規則）と照らす（SAS の同じ検査を写したもの）。
//!
//! - 依存の辺が規則の allow にあり、どの crate と外のライブラリも component に入る（architecture-lint check）
//! - 規則の component の node は、どれもワークスペースのグラフにある（消した crate を規則に残さない）
//! - crate の名前がフォルダから決まる（SAS と同じ付け方。文脈の中は <文脈>-<層>（domain ・ application）か <モジュール>-<層>、
//!   文脈の外の adapters/<inbound|outbound>/<技術> は <inbound|outbound>-<技術>、根の test-support/ は test-support）
//!
//! architecture-lint は PATH から探し、無ければリポジトリの中の ../architecture-lint/target/debug から使う。

// 試験の補助の関数でも expect を使う（clippy.toml が #[test] の中で許すのと同じ考え）。
#![allow(clippy::expect_used, clippy::panic)]

use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 文脈のフォルダ。文脈の crate（domain ・ application）はどれもこのどれかの中にある。
const CONTEXTS: [&str; 2] = ["skills", "agents"];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// architecture-lint を起動する。PATH に無ければ、隣のフォルダで作った実行ファイルを使う。
fn architecture_lint(args: &[&str]) -> Output {
    let run = |program: &Path| {
        Command::new(program)
            .args(args)
            .current_dir(root())
            .output()
    };
    match run(Path::new("architecture-lint")) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let local = root().join("../architecture-lint/target/debug/architecture-lint");
            run(&local).unwrap_or_else(|e| {
                panic!(
                    "architecture-lint が PATH にも {} にも無い（cargo build を architecture-lint で打つ）: {e}",
                    local.display()
                )
            })
        }
        other => other.expect("architecture-lint を起動できること"),
    }
}

#[test]
fn dependencies_follow_the_rules() {
    let out = architecture_lint(&["check", "--rules", "architecture.json", "--cargo", "."]);
    assert!(
        out.status.success(),
        "architecture-lint check が違反か誤りを返した（終了コード {:?}）\n{}{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn components_name_nodes_in_the_graph() {
    let out = architecture_lint(&["graph", "--cargo", "."]);
    assert!(out.status.success(), "architecture-lint graph が失敗した");
    let graph: Value = serde_json::from_slice(&out.stdout).expect("グラフが JSON であること");
    let nodes: BTreeSet<&str> = graph["nodes"]
        .as_array()
        .expect("nodes が配列であること")
        .iter()
        .map(|n| n["name"].as_str().expect("name が文字列であること"))
        .collect();
    let text = std::fs::read_to_string(root().join("architecture.json"))
        .expect("architecture.json を読めること");
    let rules: Value = serde_json::from_str(&text).expect("architecture.json が JSON であること");
    let mut missing = Vec::new();
    for c in rules["components"]
        .as_array()
        .expect("components が配列であること")
    {
        for node in c["nodes"].as_array().expect("nodes が配列であること") {
            let node = node.as_str().expect("文字列であること");
            if !nodes.contains(node) {
                missing.push(node.to_string());
            }
        }
    }
    assert!(missing.is_empty(), "グラフに無い node: {missing:?}");
}

/// ワークスペースの根からの経路で crate の名前を決める：<文脈>/<層> → <文脈>-<層>、<文脈>/<モジュール>/<層> →
/// <モジュール>-<層>、adapters/<inbound|outbound>/<技術> → <inbound|outbound>-<技術>、test-support → test-support。
/// 決まらない経路は None。
fn name_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    match parts.as_slice() {
        ["adapters", side @ ("inbound" | "outbound"), tech] => Some(format!("{side}-{tech}")),
        ["test-support"] => Some("test-support".to_string()),
        [context, layer @ ("application" | "domain")] if CONTEXTS.contains(context) => {
            Some(format!("{context}-{layer}"))
        }
        [context, module, layer @ ("application" | "domain")] if CONTEXTS.contains(context) => {
            Some(format!("{module}-{layer}"))
        }
        _ => None,
    }
}

#[test]
fn crate_names_follow_their_folders() {
    let out = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(root())
        .output()
        .expect("cargo metadata を実行できること");
    assert!(out.status.success(), "cargo metadata が失敗した");
    let meta: Value = serde_json::from_slice(&out.stdout).expect("JSON であること");
    let workspace = PathBuf::from(
        meta["workspace_root"]
            .as_str()
            .expect("workspace_root が文字列であること"),
    );
    for pkg in meta["packages"]
        .as_array()
        .expect("packages が配列であること")
    {
        let name = pkg["name"].as_str().expect("name が文字列であること");
        let manifest = Path::new(
            pkg["manifest_path"]
                .as_str()
                .expect("manifest_path が文字列であること"),
        );
        let parent = manifest
            .parent()
            .expect("Cargo.toml はフォルダの中にあること");
        let inner = parent
            .strip_prefix(&workspace)
            .unwrap_or_else(|_| panic!("{name} はワークスペースの中にあること"))
            .to_str()
            .expect("経路が UTF-8 であること")
            .replace('\\', "/");
        assert_eq!(
            Some(name.to_string()),
            name_from_path(&inner),
            "{inner} の crate の名前"
        );
    }
}
