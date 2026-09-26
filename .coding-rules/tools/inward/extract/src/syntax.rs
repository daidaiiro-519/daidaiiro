// SPDX-License-Identifier: MIT
//! 原文の構文木から辺を取る。**外の道具を1つも呼ばない。**
//!
//! 文法は tree-sitter が持つ ── **解析器を自作しない。** 書くのは、参照を指す問い
//! （query）1本と、参照の文字列を層の識別子へ直す規則である。
//!
//! **こちらが行う解決は2種類だけである** ── 経路の相対と、点の段数。それ以外は
//! 参照が既に完全修飾なので、解決が要らない。
//!
//! **取りこぼす範囲を申告する。** 字面を見るので、経路の別名（設定で付け替えるもの）と
//! 探索路の指定は解決しない ── 道具に出させる抽出器のほうが、そこは正確である。

use std::io;
use std::path::{Path, PathBuf};

use inward_core::Edge;
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator as _};

use crate::{Escape, Extracted, Extractor};

/// 参照の文字列を、層の識別子へ直す規則。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Resolve {
    /// 既に完全修飾である ── そのまま使う。
    Qualified,
    /// 経路の相対である ── 書かれたファイルの場所から畳む。
    Path,
    /// 点の段数である ── 書かれた点の包みから遡る。
    Dotted,
    /// crate の中の点である ── `crate::` は根、`super::` は1つ上、`self::` はここ。
    Crate,
}

/// 1つの言語の宣言。
#[derive(Clone)]
#[non_exhaustive]
pub struct Syntax {
    /// 言語の名前。
    pub language: &'static str,
    /// 見る拡張子。
    pub extensions: &'static [&'static str],
    /// 参照を指す問い。**捕まえる名前は `to` である。**
    pub imports: &'static str,
    /// その点の名前を指す問い。**捕まえる名前は `here` である。** 空なら経路を使う。
    pub here: &'static str,
    /// 参照の直し方。
    pub resolve: Resolve,
    /// 文法。
    pub grammar: fn() -> Language,
}

/// 扱う言語の一覧。**言語を足すときに触るのはここ1か所である。**
#[must_use]
pub fn table() -> Vec<Syntax> {
    vec![
        Syntax {
            language: "python",
            extensions: &["py"],
            imports: "[(import_statement name: (dotted_name) @to)
                        (import_statement name: (aliased_import name: (dotted_name) @to))
                        (import_from_statement module_name: (dotted_name) @to)
                        (import_from_statement module_name: (relative_import) @to)]",
            here: "",
            resolve: Resolve::Dotted,
            grammar: || tree_sitter_python::LANGUAGE.into(),
        },
        Syntax {
            language: "rust",
            extensions: &["rs"],
            imports: "(use_declaration argument: (_) @to)",
            here: "",
            resolve: Resolve::Crate,
            grammar: || tree_sitter_rust::LANGUAGE.into(),
        },
        Syntax {
            language: "typescript",
            extensions: &["ts", "tsx", "mts", "cts", "js", "mjs", "cjs", "jsx"],
            imports: "[(import_statement source: (string) @to)
                       (export_statement source: (string) @to)]",
            here: "",
            resolve: Resolve::Path,
            grammar: || tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        },
        Syntax {
            language: "java",
            extensions: &["java"],
            imports: "(import_declaration (scoped_identifier) @to)",
            here: "(package_declaration (scoped_identifier) @here)",
            resolve: Resolve::Qualified,
            grammar: || tree_sitter_java::LANGUAGE.into(),
        },
        Syntax {
            language: "kotlin",
            extensions: &["kt", "kts"],
            imports: "(import (qualified_identifier) @to)",
            here: "(package_header (qualified_identifier) @here)",
            resolve: Resolve::Qualified,
            grammar: || tree_sitter_kotlin_ng::LANGUAGE.into(),
        },
        Syntax {
            language: "csharp",
            extensions: &["cs"],
            imports: "(using_directive [(qualified_name) (identifier)] @to)",
            here: "[(file_scoped_namespace_declaration name: (_) @here)
                    (namespace_declaration name: (_) @here)]",
            resolve: Resolve::Qualified,
            grammar: || tree_sitter_c_sharp::LANGUAGE.into(),
        },
        Syntax {
            language: "php",
            extensions: &["php"],
            imports: "(namespace_use_declaration (namespace_use_clause (qualified_name) @to))",
            here: "(namespace_definition name: (namespace_name) @here)",
            resolve: Resolve::Qualified,
            grammar: || tree_sitter_php::LANGUAGE_PHP.into(),
        },
        Syntax {
            language: "go",
            extensions: &["go"],
            imports: "(import_spec path: (_) @to)",
            here: "",
            resolve: Resolve::Qualified,
            grammar: || tree_sitter_go::LANGUAGE.into(),
        },
        Syntax {
            language: "ruby",
            extensions: &["rb"],
            imports: "(call method: (identifier) @how arguments: (argument_list (string) @to))",
            here: "",
            resolve: Resolve::Path,
            grammar: || tree_sitter_ruby::LANGUAGE.into(),
        },
        Syntax {
            language: "cpp",
            extensions: &["cpp", "cc", "cxx", "c", "hpp", "hxx", "h"],
            imports: "(preproc_include path: (string_literal) @to)",
            here: "",
            resolve: Resolve::Path,
            grammar: || tree_sitter_cpp::LANGUAGE.into(),
        },
    ]
}

/// 引用符を外す。**文字列の節点は引用符を含む。**
fn unquote(raw: &str) -> &str {
    raw.trim_matches(|c| c == '"' || c == '\'' || c == '`')
}

/// 経路を畳む。**`..` と `.` を解いて、根からの相対にする。**
fn fold(root: &Path, here: &Path, spec: &str) -> String {
    let base = here.parent().unwrap_or(Path::new(""));
    let joined = base.join(spec);
    let mut parts: Vec<std::ffi::OsString> = Vec::new();
    for c in joined.components() {
        match c {
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::CurDir => {}
            other => parts.push(other.as_os_str().to_owned()),
        }
    }
    let cleaned: PathBuf = parts.iter().collect();
    cleaned.strip_prefix(root).unwrap_or(&cleaned).display().to_string()
}

/// 点の名前を、経路から組む。
fn dotted(root: &Path, file: &Path) -> String {
    let rel = file.strip_prefix(root).unwrap_or(file).with_extension("");
    let mut parts: Vec<String> =
        rel.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    if parts.last().is_some_and(|x| x == "__init__" || x == "mod" || x == "lib") {
        parts.pop();
    }
    parts.join(".")
}

/// 段数を解いて、点の名前へ直す。
fn relative_dotted(here: &str, spec: &str) -> String {
    let dots = spec.chars().take_while(|c| *c == '.').count();
    let tail = spec.trim_start_matches('.');
    if dots == 0 {
        return spec.to_owned();
    }
    let mut parts: Vec<&str> = if here.is_empty() { Vec::new() } else { here.split('.').collect() };
    // **点の段数は、書かれた点の包みから遡る。** 1つ目は包み自身である
    for _ in 0..dots.saturating_sub(1) {
        parts.pop();
    }
    if !tail.is_empty() {
        parts.push(tail);
    }
    parts.join(".")
}

/// crate の中の点を直す。
fn in_crate(here: &str, spec: &str) -> String {
    let body = spec.replace("::", ".");
    if let Some(tail) = body.strip_prefix("crate.") {
        return tail.to_owned();
    }
    if let Some(tail) = body.strip_prefix("self.") {
        return if here.is_empty() { tail.to_owned() } else { format!("{here}.{tail}") };
    }
    if let Some(tail) = body.strip_prefix("super.") {
        let mut parts: Vec<&str> = here.split('.').filter(|x| !x.is_empty()).collect();
        parts.pop();
        let head = parts.join(".");
        return if head.is_empty() { tail.to_owned() } else { format!("{head}.{tail}") };
    }
    body
}

/// 原文から辺を取る抽出器。
#[derive(Clone)]
#[non_exhaustive]
pub struct Tree {
    syntax: Syntax,
}

impl Tree {
    /// 言語の名前から組む。**知らない言語なら返さない。**
    #[must_use]
    pub fn of(language: &str) -> Option<Self> {
        table().into_iter().find(|s| s.language == language).map(|syntax| Self { syntax })
    }

    /// 扱える言語を並べる。
    #[must_use]
    pub fn languages() -> Vec<&'static str> {
        table().iter().map(|s| s.language).collect()
    }
}

/// 見ない包み。
const SKIP: [&str; 6] = ["node_modules", "target", "build", "dist", "__pycache__", "vendor"];

fn sources(dir: &Path, want: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        let name = p.file_name().unwrap_or_default().to_string_lossy().into_owned();
        if p.is_dir() {
            if !SKIP.contains(&name.as_str()) && !name.starts_with('.') {
                sources(&p, want, out);
            }
        } else if p.extension().is_some_and(|x| want.contains(&&*x.to_string_lossy())) {
            out.push(p);
        }
    }
}

/// 読み込みを起こす名前。**引数が文字列でなければ抜け道である。**
const RUBY_LOADERS: [&str; 4] = ["require", "require_relative", "load", "autoload"];

impl Extractor for Tree {
    fn language(&self) -> &'static str {
        self.syntax.language
    }

    fn available(&self) -> bool {
        true // **文法は焼き込まれている** ── 外の道具を要しない
    }

    fn extract(&self, root: &Path) -> io::Result<Extracted> {
        let s = &self.syntax;
        let mut files = Vec::new();
        sources(root, s.extensions, &mut files);
        if files.is_empty() {
            return Ok(Extracted {
                undecided: vec![format!(
                    "{} に {} のファイルが1件も無い",
                    root.display(),
                    s.extensions.join(" ・ ")
                )],
                ..Extracted::default()
            });
        }
        let lang = (s.grammar)();
        let mut parser = Parser::new();
        parser
            .set_language(&lang)
            .map_err(|e| io::Error::new(io::ErrorKind::Unsupported, e.to_string()))?;
        let imports = Query::new(&lang, s.imports)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
        let here_query = if s.here.is_empty() {
            None
        } else {
            Some(
                Query::new(&lang, s.here)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?,
            )
        };

        let mut edges = Vec::new();
        let mut escapes = Vec::new();
        let mut undecided = Vec::new();
        // **測り方の限界は、申告であって検出ではない。** 検出に混ぜると、取りこぼしの
        // 申告そのものが不合格の原因になる（実測 ── 向きの規則が常に不合格になった）
        let limits =
            vec!["字面で測っている ── 経路の別名と探索路の指定は解決していない".to_owned()];
        for file in &files {
            let Ok(body) = std::fs::read_to_string(file) else {
                undecided.push(format!("{} ── 読めない", file.display()));
                continue;
            };
            let Some(tree) = parser.parse(&body, None) else {
                undecided.push(format!("{} ── 解析できない", file.display()));
                continue;
            };
            let bytes = body.as_bytes();
            let rel = file.strip_prefix(root).unwrap_or(file).display().to_string();
            // 参照する側の名前を決める
            let here = match (&here_query, s.resolve) {
                (Some(q), _) => {
                    let mut cursor = QueryCursor::new();
                    let mut hits = cursor.matches(q, tree.root_node(), bytes);
                    let mut found = String::new();
                    if let Some(m) = hits.next() {
                        if let Some(cap) = m.captures.first() {
                            found = cap.node.utf8_text(bytes).unwrap_or("").to_owned();
                        }
                    }
                    if found.is_empty() {
                        undecided.push(format!("{rel} ── 点の名前を宣言していない"));
                        continue;
                    }
                    found
                }
                (None, Resolve::Dotted | Resolve::Crate) => dotted(root, file),
                (None, _) => rel.clone(),
            };
            let mut cursor = QueryCursor::new();
            let mut hits = cursor.matches(&imports, tree.root_node(), bytes);
            while let Some(m) = hits.next() {
                let mut how = String::new();
                let mut raw = None;
                let mut line = 1;
                for cap in m.captures {
                    let name = &imports.capture_names()[cap.index as usize];
                    let text = cap.node.utf8_text(bytes).unwrap_or("");
                    if *name == "how" {
                        how = text.to_owned();
                    } else {
                        raw = Some(text);
                        line = cap.node.start_position().row + 1;
                    }
                }
                let Some(raw) = raw else { continue };
                if s.language == "ruby" && !RUBY_LOADERS.contains(&how.as_str()) {
                    continue;
                }
                let spec = unquote(raw);
                let to = match s.resolve {
                    Resolve::Qualified => spec.to_owned(),
                    Resolve::Path => fold(root, file.strip_prefix(root).unwrap_or(file), spec),
                    Resolve::Dotted => relative_dotted(&here, spec),
                    Resolve::Crate => in_crate(&here, spec),
                };
                edges.push(Edge::new(here.clone(), to, format!("{rel}:{line}")));
            }
            // 読み込みが文字列でない箇所は、行き先が静的に決まらない
            if s.language == "ruby" {
                let dyn_query = Query::new(
                    &lang,
                    "(call method: (identifier) @how arguments: (argument_list . (_) @arg))",
                )
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
                let mut c2 = QueryCursor::new();
                let mut got = c2.matches(&dyn_query, tree.root_node(), bytes);
                while let Some(m) = got.next() {
                    let mut method = "";
                    let mut kind = "";
                    let mut line = 1;
                    for cap in m.captures {
                        let name = &dyn_query.capture_names()[cap.index as usize];
                        if *name == "how" {
                            method = cap.node.utf8_text(bytes).unwrap_or("");
                        } else {
                            kind = cap.node.kind();
                            line = cap.node.start_position().row + 1;
                        }
                    }
                    if RUBY_LOADERS.contains(&method) && kind != "string" {
                        escapes.push(Escape::new(
                            here.clone(),
                            format!("{rel}:{line}"),
                            format!("図に現れない読み込み ── {method} に文字列以外を渡している"),
                        ));
                    }
                }
            }
        }
        Ok(Extracted { edges, escapes, undecided, limits })
    }
}
