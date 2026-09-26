// SPDX-License-Identifier: MIT
//! 前処理器から辺を取る。**言語の実行時を使わない。**
//!
//! `g++ -MM` は make の形で並べる ── `<目的>.o: <原文> <含めた表題…>`。
//! **先頭が参照する側、以降が参照される側である**（実測 2026-09-26）。
//! 包みの外の表題（角括弧で書くもの）は `-MM` が既に落としている。

use std::io;
use std::path::{Path, PathBuf};

use inward_core::Edge;

use crate::tool::{self, Ran};
use crate::{Extracted, Extractor};

/// 見る拡張子。
const EXTENSIONS: [&str; 7] = ["cpp", "cc", "cxx", "c", "hpp", "hxx", "h"];
/// 見ない包み。
const SKIP: [&str; 4] = ["build", "out", "target", "node_modules"];

/// C ・ C++ の抽出器。
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct Cpp;

impl Cpp {
    /// 抽出器を組む。
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        let name = p
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if p.is_dir() {
            if !SKIP.contains(&name.as_str()) && !name.starts_with('.') {
                sources(&p, out);
            }
        } else if p
            .extension()
            .is_some_and(|x| EXTENSIONS.contains(&&*x.to_string_lossy()))
        {
            out.push(p);
        }
    }
}

/// 根からの相対に直す。**直せなければそのまま返す。**
fn near(root: &Path, raw: &str) -> String {
    let full = if Path::new(raw).is_absolute() {
        PathBuf::from(raw)
    } else {
        root.join(raw)
    };
    let cleaned = full.components().fold(PathBuf::new(), |mut acc, c| {
        match c {
            std::path::Component::ParentDir => {
                acc.pop();
            }
            std::path::Component::CurDir => {}
            other => acc.push(other),
        }
        acc
    });
    cleaned
        .strip_prefix(root)
        .map_or_else(|_| raw.to_owned(), |p| p.display().to_string())
}

impl Extractor for Cpp {
    fn language(&self) -> &'static str {
        "cpp"
    }

    fn available(&self) -> bool {
        tool::exists("g++")
    }

    fn extract(&self, root: &Path) -> io::Result<Extracted> {
        let mut files = Vec::new();
        sources(root, &mut files);
        if files.is_empty() {
            return Ok(Extracted {
                undecided: vec![format!("{} に原文が1件も無い", root.display())],
                ..Extracted::default()
            });
        }
        let mut args = vec![
            "-MM".to_owned(),
            "-I".to_owned(),
            root.display().to_string(),
        ];
        args.extend(files.iter().map(|p| p.display().to_string()));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let (stdout, stderr, code) = match tool::run("g++", &refs, Path::new("."))? {
            Ran::Absent => {
                return Ok(Extracted {
                    undecided: vec!["g++ が無いので判定していない".to_owned()],
                    ..Extracted::default()
                })
            }
            Ran::Output {
                stdout,
                stderr,
                code,
            } => (stdout, stderr, code),
        };
        if code != Some(0) {
            return Ok(Extracted {
                undecided: vec![format!(
                    "g++ が終了コード {} を返した ── {}",
                    code.map_or_else(|| "（信号）".to_owned(), |c| c.to_string()),
                    stderr.trim()
                )],
                ..Extracted::default()
            });
        }
        // 行の継続（末尾の `\`）を畳んでから読む
        let body = stdout.replace("\\\n", " ");
        let mut edges = Vec::new();
        for line in body.lines() {
            let Some((_, rest)) = line.split_once(':') else {
                continue;
            };
            let mut items = rest.split_whitespace();
            let Some(source) = items.next() else { continue };
            let from = near(root, source);
            for header in items {
                edges.push(Edge::new(from.clone(), near(root, header), from.clone()));
            }
        }
        Ok(Extracted {
            edges,
            ..Extracted::default()
        })
    }
}
