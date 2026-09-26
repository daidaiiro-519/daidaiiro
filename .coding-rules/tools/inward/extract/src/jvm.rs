// SPDX-License-Identifier: MIT
//! JVM の類ファイルから辺を取る。**言語の実行時を使わない。**
//!
//! `jdeps -verbose:class` を呼び、`<参照する側> -> <参照される側> <在り処>` を読む
//! （実測 2026-09-26、Corretto 25）。**Java も Kotlin も、組んだあとは同じ形になる**
//! ので、抽出器は1つで足りる。
//!
//! **組んだものを見る。** 原文ではなく類ファイルなので、根の中に `.class` か `.jar` が
//! 要る ── 無ければ判定できていないと返す。

use std::io;
use std::path::{Path, PathBuf};

use inward_core::Edge;

use crate::tool::{self, Ran};
use crate::{Extracted, Extractor};

/// 見ない包み。**組んだ先は除外しない** ── JVM では類ファイルがまさにそこに在る
/// （実測 ── `out/` を除外して辺が0件になった）。
const SKIP: [&str; 2] = ["node_modules", ".gradle"];

/// JVM の抽出器。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Jvm {
    language: &'static str,
}

impl Jvm {
    /// 名前を受け取って組む ── `java` と `kotlin` で同じ実体を使う。
    #[must_use]
    pub const fn new(language: &'static str) -> Self {
        Self { language }
    }
}

fn find(dir: &Path, want: &str, out: &mut Vec<PathBuf>) {
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
                find(&p, want, out);
            }
        } else if p.extension().is_some_and(|x| x == want) {
            out.push(p);
        }
    }
}

/// 見る先を決める。**jar が在れば jar を、無ければ類ファイルの包みを渡す。**
fn artifacts(root: &Path) -> Vec<PathBuf> {
    let mut jars = Vec::new();
    find(root, "jar", &mut jars);
    if !jars.is_empty() {
        return jars;
    }
    let mut classes = Vec::new();
    find(root, "class", &mut classes);
    let mut dirs: Vec<PathBuf> = classes
        .iter()
        .filter_map(|p| p.parent().map(Path::to_path_buf))
        .collect();
    dirs.sort();
    dirs.dedup();
    dirs
}

/// `jdeps` の1行から辺を読む。**`->` を挟んで左が参照する側である。**
fn edge(line: &str) -> Option<Edge> {
    let (left, right) = line.split_once("->")?;
    let from = left.trim();
    let to = right.split_whitespace().next()?;
    if from.is_empty() || to.is_empty() {
        return None;
    }
    Some(Edge::new(from.to_owned(), to.to_owned(), from.to_owned()))
}

impl Extractor for Jvm {
    fn language(&self) -> &'static str {
        self.language
    }

    fn available(&self) -> bool {
        tool::exists("jdeps")
    }

    fn extract(&self, root: &Path) -> io::Result<Extracted> {
        let targets = artifacts(root);
        if targets.is_empty() {
            return Ok(Extracted {
                undecided: vec![format!(
                    "{} に .class も .jar が無いので判定していない ── 先に組む",
                    root.display()
                )],
                ..Extracted::default()
            });
        }
        let mut args = vec!["-verbose:class".to_owned(), "-filter:none".to_owned()];
        args.extend(targets.iter().map(|p| p.display().to_string()));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        match tool::run("jdeps", &refs, Path::new("."))? {
            Ran::Absent => Ok(Extracted {
                undecided: vec!["jdeps が無いので判定していない".to_owned()],
                ..Extracted::default()
            }),
            Ran::Output { stderr, code, .. } if code != Some(0) => Ok(Extracted {
                undecided: vec![format!(
                    "jdeps が終了コード {} を返した ── {}",
                    code.map_or_else(|| "（信号）".to_owned(), |c| c.to_string()),
                    stderr.trim()
                )],
                ..Extracted::default()
            }),
            Ran::Output { stdout, .. } => Ok(Extracted {
                edges: stdout.lines().filter_map(edge).collect(),
                ..Extracted::default()
            }),
        }
    }
}
