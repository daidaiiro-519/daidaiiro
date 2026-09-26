// SPDX-License-Identifier: MIT
//! 脚本で辺を取る抽出器。**言語ごとに Rust の型を増やさない。**
//!
//! 言語が増えるたびに実装を1つ書くと、同じ手順が言語の数だけ並ぶ。差が在るのは
//! 3つだけである ── **呼ぶ道具 ・ 脚本の名前 ・ 探す拡張子**。それを値として持つ。
//!
//! 脚本は、その言語自身の解析器に出させる。**こちらは解析しない。**

use std::io;
use std::path::{Path, PathBuf};

use crate::{probe, tool, Extracted, Extractor};

/// 見ない包み。**生成物と外から持ってきたものは、層の宣言の外である。**
const SKIP: [&str; 8] = [
    "__pycache__",
    ".venv",
    "venv",
    "node_modules",
    "target",
    "vendor",
    "build",
    "dist",
];

/// 脚本で辺を取る抽出器。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Scripted {
    language: &'static str,
    program: &'static str,
    script: PathBuf,
    /// 探す拡張子。**空なら根だけを渡す** ── 道具の側が全体を走査する場合である。
    extensions: &'static [&'static str],
}

impl Scripted {
    /// 抽出器を組む。
    #[must_use]
    pub const fn new(
        language: &'static str,
        program: &'static str,
        script: PathBuf,
        extensions: &'static [&'static str],
    ) -> Self {
        Self {
            language,
            program,
            script,
            extensions,
        }
    }
}

fn sources(dir: &Path, want: &[&str], out: &mut Vec<PathBuf>) {
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
                sources(&p, want, out);
            }
        } else if p
            .extension()
            .is_some_and(|x| want.contains(&&*x.to_string_lossy()))
        {
            out.push(p);
        }
    }
}

impl Extractor for Scripted {
    fn language(&self) -> &'static str {
        self.language
    }

    fn available(&self) -> bool {
        tool::exists(self.program) && self.script.is_file()
    }

    fn extract(&self, root: &Path) -> io::Result<Extracted> {
        if !self.script.is_file() {
            return Ok(Extracted {
                undecided: vec![format!(
                    "探査の脚本が無いので判定していない ── {}",
                    self.script.display()
                )],
                ..Extracted::default()
            });
        }
        let mut args: Vec<String> = vec![
            self.script.display().to_string(),
            root.display().to_string(),
        ];
        if !self.extensions.is_empty() {
            let mut files = Vec::new();
            sources(root, self.extensions, &mut files);
            if files.is_empty() {
                // **空振りを合格にしない。**
                return Ok(Extracted {
                    undecided: vec![format!(
                        "{} に {} のファイルが1件も無い",
                        root.display(),
                        self.extensions.join(" ・ ")
                    )],
                    ..Extracted::default()
                });
            }
            args.extend(files.iter().map(|p| p.display().to_string()));
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        probe::ask(self.program, &refs, Path::new("."))
    }
}
