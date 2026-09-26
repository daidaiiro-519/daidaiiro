// SPDX-License-Identifier: MIT
//! 宣言そのものから辺を取る。**言語の実行時を使わない。**
//!
//! 層の境界が、その言語の依存の宣言の境界と一致する場合である。**宣言に無い依存は
//! コンパイラが解決しない**ので、検査の主体はコンパイラである ── この抽出器は、
//! 宣言そのものが向きに違反していないかを見る。
//!
//! | 言語 | 宣言を出す道具 | 実測（2026-09-26） |
//! |---|---|---|
//! | Rust | `cargo metadata --no-deps` | `adapter_layer → ['core_layer']` |
//! | Go | `go list -json ./...`（**module ごとに聞く**） | `example.com/adapter → [...]` |
//! | C# | `dotnet msbuild -getItem:ProjectReference` | `Identity: ../Core/Core.csproj` |

use std::io;
use std::path::{Path, PathBuf};

use inward_core::Edge;
use serde_json::Value;

use crate::tool::{self, Ran};
use crate::{Extracted, Extractor};

/// 宣言の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Kind {
    /// crate の宣言。
    Rust,
    /// module の宣言。
    Go,
    /// 企画の宣言。
    CSharp,
}

impl Kind {
    /// 呼ぶ道具の名前。
    #[must_use]
    pub const fn program(self) -> &'static str {
        match self {
            Self::Rust => "cargo",
            Self::Go => "go",
            Self::CSharp => "dotnet",
        }
    }

    /// 画面へ出す言語の名前。
    #[must_use]
    pub const fn language(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Go => "go",
            Self::CSharp => "csharp",
        }
    }

    /// 宣言のファイル名。
    #[must_use]
    pub const fn manifest(self) -> &'static str {
        match self {
            Self::Rust => "Cargo.toml",
            Self::Go => "go.mod",
            Self::CSharp => "csproj",
        }
    }
}

/// 宣言から辺を取る抽出器。
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct Manifest {
    kind: Kind,
}

impl Manifest {
    /// 種類を受け取って組む。
    #[must_use]
    pub const fn new(kind: Kind) -> Self {
        Self { kind }
    }
}

fn find(dir: &Path, name: &str, by_extension: bool, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        let base = p
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if p.is_dir() {
            if !matches!(base.as_str(), "target" | "bin" | "obj" | "node_modules")
                && !base.starts_with('.')
            {
                find(&p, name, by_extension, out);
            }
        } else if by_extension {
            if p.extension().is_some_and(|x| x == name) {
                out.push(p);
            }
        } else if base == name {
            out.push(p);
        }
    }
}

/// 道具が起動できない環境を補う値。**判定の結果を変える値は置かない。**
const DOTNET_ENV: [(&str, &str); 3] = [
    ("DOTNET_SYSTEM_GLOBALIZATION_INVARIANT", "1"),
    ("DOTNET_CLI_TELEMETRY_OPTOUT", "1"),
    ("DOTNET_NOLOGO", "1"),
];

/// 道具を1回呼び、標準出力か、判定できなかった理由を返す。
fn ask(program: &str, args: &[&str], cwd: &Path) -> io::Result<Result<String, String>> {
    let env: &[(&str, &str)] = if program == "dotnet" {
        &DOTNET_ENV
    } else {
        &[]
    };
    Ok(match tool::run_with(program, args, cwd, env)? {
        Ran::Absent => Err(format!("{program} が無いので判定していない")),
        Ran::Output {
            stdout,
            stderr,
            code,
        } => {
            if code == Some(0) {
                Ok(stdout)
            } else {
                Err(format!(
                    "{program} が終了コード {} を返した ── {}",
                    code.map_or_else(|| "（信号）".to_owned(), |c| c.to_string()),
                    stderr.trim().chars().take(200).collect::<String>()
                ))
            }
        }
    })
}

fn rust(root: &Path) -> io::Result<Extracted> {
    let out = match ask(
        "cargo",
        &["metadata", "--no-deps", "--format-version", "1"],
        root,
    )? {
        Ok(out) => out,
        Err(why) => {
            return Ok(Extracted {
                undecided: vec![why],
                ..Extracted::default()
            })
        }
    };
    let Ok(parsed) = serde_json::from_str::<Value>(&out) else {
        return Ok(Extracted {
            undecided: vec!["cargo metadata の返りを読めない".to_owned()],
            ..Extracted::default()
        });
    };
    let mut edges = Vec::new();
    for package in parsed
        .get("packages")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        let name = package
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let at = package
            .get("manifest_path")
            .and_then(Value::as_str)
            .unwrap_or("Cargo.toml");
        for dep in package
            .get("dependencies")
            .and_then(Value::as_array)
            .unwrap_or(&vec![])
        {
            if let Some(to) = dep.get("name").and_then(Value::as_str) {
                edges.push(Edge::new(name.to_owned(), to.to_owned(), at.to_owned()));
            }
        }
    }
    Ok(Extracted {
        edges,
        ..Extracted::default()
    })
}

fn go(root: &Path) -> io::Result<Extracted> {
    // **module ごとに聞く。** 根が module でないとき `./...` は解決しない（実測）
    let mut manifests = Vec::new();
    find(root, "go.mod", false, &mut manifests);
    if manifests.is_empty() {
        return Ok(Extracted {
            undecided: vec![format!("{} に go.mod が無い", root.display())],
            ..Extracted::default()
        });
    }
    let mut edges = Vec::new();
    let mut undecided = Vec::new();
    for manifest in manifests {
        let Some(dir) = manifest.parent() else {
            continue;
        };
        let out = match ask("go", &["list", "-json", "./..."], dir)? {
            Ok(out) => out,
            Err(why) => {
                undecided.push(why);
                continue;
            }
        };
        let stream = serde_json::Deserializer::from_str(&out).into_iter::<Value>();
        for package in stream.flatten() {
            let here = package
                .get("ImportPath")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let at = package.get("Dir").and_then(Value::as_str).unwrap_or(here);
            for to in package
                .get("Imports")
                .and_then(Value::as_array)
                .unwrap_or(&vec![])
            {
                if let Some(to) = to.as_str() {
                    edges.push(Edge::new(here.to_owned(), to.to_owned(), at.to_owned()));
                }
            }
        }
    }
    Ok(Extracted {
        edges,
        undecided,
        ..Extracted::default()
    })
}

fn csharp(root: &Path) -> io::Result<Extracted> {
    let mut projects = Vec::new();
    find(root, "csproj", true, &mut projects);
    if projects.is_empty() {
        return Ok(Extracted {
            undecided: vec![format!("{} に .csproj が無い", root.display())],
            ..Extracted::default()
        });
    }
    let mut edges = Vec::new();
    let mut undecided = Vec::new();
    for project in projects {
        let path = project.display().to_string();
        let out = match ask(
            "dotnet",
            &["msbuild", &path, "-getItem:ProjectReference", "-nologo"],
            root,
        )? {
            Ok(out) => out,
            Err(why) => {
                undecided.push(why);
                continue;
            }
        };
        let Ok(parsed) = serde_json::from_str::<Value>(&out) else {
            undecided.push(format!("{path} の返りを読めない"));
            continue;
        };
        let from = project
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let items = parsed
            .get("Items")
            .and_then(|x| x.get("ProjectReference"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for item in items {
            if let Some(identity) = item.get("Identity").and_then(Value::as_str) {
                let to = Path::new(identity)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                edges.push(Edge::new(from.clone(), to, path.clone()));
            }
        }
    }
    Ok(Extracted {
        edges,
        undecided,
        ..Extracted::default()
    })
}

impl Extractor for Manifest {
    fn language(&self) -> &'static str {
        self.kind.language()
    }

    fn available(&self) -> bool {
        tool::exists(self.kind.program())
    }

    fn extract(&self, root: &Path) -> io::Result<Extracted> {
        match self.kind {
            Kind::Rust => rust(root),
            Kind::Go => go(root),
            Kind::CSharp => csharp(root),
        }
    }
}
