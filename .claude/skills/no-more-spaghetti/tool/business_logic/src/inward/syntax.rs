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
//! 探索路の指定は解決しない ── 毎回そう申告する（`limits`）。

use std::io;
use std::path::{Path, PathBuf};

use crate::data_access::files;

use super::judge::Edge;
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator as _};

use super::{Escape, Extracted, Extractor};

/// 参照の文字列を、層の識別子へ直す規則。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Resolve {
    /// 既に完全修飾である ── そのまま使う。
    Qualified,
    /// 経路の相対である ── 書かれたファイルの場所から畳む。
    Path,
    /// 先頭の . の数である ── 書かれたファイルのディレクトリから遡る。
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
    /// **契約の欄** ── 名前空間をつなぐ情報の出どころ。`None` は、ソースが自分の名前を
    /// 宣言する言語である（参照する側も参照される側も、宣言の名前空間で書かれる）。
    pub bridge: Option<fn(&Path, &mut super::names::Bridge)>,
    /// **契約の欄** ── 名前を実行時に決める読み込み。無い言語は `Absent` に理由を書く。
    pub dynamic: Dynamic,
    /// **契約の欄** ── 条件で分かれる読み込み。すべての分岐を依存として数える。無い言語は `Absent`。
    pub branches: Branches,
    /// 名前の区切り方。向きの判定へ渡す。
    pub naming: Naming,
}

/// 名前の区切り方。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Naming {
    /// ファイルの経路で書く（`/`）。
    Paths,
    /// . 区切りの名前で書く（`.` ・ `::`）。
    Dotted,
    /// 名前空間を `\` で区切る。
    Backslash,
}

/// 名前を実行時に決める読み込み。
///
/// **文字列で書かれていれば依存、そうでなければ静的に追跡できない読み込みである** ── どの言語にも同じ規則を課す。
#[derive(Debug, Clone, Copy)]
pub enum Dynamic {
    /// 読み込みの呼び出しを指す問い。捕まえる名前は `how`（呼ぶ名前。無くてもよい）と `arg`（最初の引数）。
    Calls {
        /// 問い。
        query: &'static str,
        /// 読み込みを起こす名前（末尾で一致させる）。空なら、問いに一致したものすべて。
        names: &'static [&'static str],
        /// 文字列として扱う節の種類。**埋め込みを含めば、文字列として扱わない。**
        literal: &'static [&'static str],
        /// 文字列で書かれた読み込みを、依存にするか ── 読み込みの問いが既に捕まえていれば依存にしない。
        edges: bool,
    },
    /// その言語には無い。**理由を書く。**
    Absent(&'static str),
}

/// 条件で分かれる読み込み。
#[derive(Debug, Clone, Copy)]
pub enum Branches {
    /// 分岐の中の読み込みも、構文木に現れる ── すべての分岐を依存にしている。書き方を添える。
    AllBranches(&'static str),
    /// その言語には無い。**理由を書く。**
    Absent(&'static str),
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
            bridge: Some(super::names::python),
            dynamic: Dynamic::Calls {
                query: "(call function: (_) @how arguments: (argument_list . (_) @arg))",
                names: &["__import__", "import_module"],
                literal: &["string"],
                edges: true,
            },
            branches: Branches::AllBranches("try ・ if の中の import"),
            naming: Naming::Dotted,
        },
        Syntax {
            language: "rust",
            extensions: &["rs"],
            imports: "(use_declaration argument: (_) @to)",
            here: "",
            resolve: Resolve::Crate,
            grammar: || tree_sitter_rust::LANGUAGE.into(),
            bridge: Some(super::names::rust),
            dynamic: Dynamic::Calls {
                query: "(macro_invocation macro: (identifier) @how (token_tree . (_) @arg))",
                names: &["include"],
                literal: &[],
                edges: false,
            },
            branches: Branches::AllBranches("#[cfg] の付いた use"),
            naming: Naming::Dotted,
        },
        Syntax {
            language: "typescript",
            extensions: &["ts", "tsx", "mts", "cts", "js", "mjs", "cjs", "jsx"],
            imports: "[(import_statement source: (string) @to)
                       (export_statement source: (string) @to)]",
            here: "",
            resolve: Resolve::Path,
            grammar: || tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            bridge: Some(super::names::typescript),
            dynamic: Dynamic::Calls {
                query: "[(call_expression function: (import) @how arguments: (arguments . (_) @arg))
                         (call_expression function: (identifier) @how arguments: (arguments . (_) @arg))]",
                names: &["import", "require"],
                literal: &["string"],
                edges: true,
            },
            branches: Branches::AllBranches("if の中の import() ・ require()"),
            naming: Naming::Paths,
        },
        Syntax {
            language: "java",
            extensions: &["java"],
            imports: "(import_declaration [(scoped_identifier) (identifier)] @to)",
            // **単一の識別子の宣言も受ける** ── `package core;` は点で繋がない
            here: "(package_declaration [(scoped_identifier) (identifier)] @here)",
            resolve: Resolve::Qualified,
            grammar: || tree_sitter_java::LANGUAGE.into(),
            bridge: None,
            dynamic: Dynamic::Calls {
                query: "(method_invocation name: (identifier) @how arguments: (argument_list . (_) @arg))",
                names: &["forName", "loadClass"],
                literal: &["string_literal"],
                edges: true,
            },
            branches: Branches::Absent("import に条件を付ける書き方が無い"),
            naming: Naming::Dotted,
        },
        Syntax {
            language: "kotlin",
            extensions: &["kt", "kts"],
            imports: "(import (qualified_identifier) @to)",
            here: "(package_header (qualified_identifier) @here)",
            resolve: Resolve::Qualified,
            grammar: || tree_sitter_kotlin_ng::LANGUAGE.into(),
            bridge: None,
            dynamic: Dynamic::Calls {
                query: "(call_expression (navigation_expression (_) (identifier) @how) (value_arguments . (value_argument (_) @arg)))",
                names: &["forName", "loadClass"],
                literal: &["string_literal"],
                edges: true,
            },
            branches: Branches::Absent("import に条件を付ける書き方が無い"),
            naming: Naming::Dotted,
        },
        Syntax {
            language: "csharp",
            extensions: &["cs"],
            imports: "(using_directive [(qualified_name) (identifier)] @to)",
            here: "[(file_scoped_namespace_declaration name: (_) @here)
                    (namespace_declaration name: (_) @here)]",
            resolve: Resolve::Qualified,
            grammar: || tree_sitter_c_sharp::LANGUAGE.into(),
            bridge: None,
            dynamic: Dynamic::Calls {
                query: "(invocation_expression function: (member_access_expression name: (identifier) @how) arguments: (argument_list . (argument (_) @arg)))",
                names: &["GetType", "Load", "LoadFrom", "CreateInstance"],
                literal: &["string_literal"],
                edges: true,
            },
            branches: Branches::AllBranches("#if の中の using"),
            naming: Naming::Dotted,
        },
        Syntax {
            language: "php",
            extensions: &["php"],
            imports: "(namespace_use_declaration (namespace_use_clause (qualified_name) @to))",
            here: "(namespace_definition name: (namespace_name) @here)",
            resolve: Resolve::Qualified,
            grammar: || tree_sitter_php::LANGUAGE_PHP.into(),
            bridge: None,
            dynamic: Dynamic::Calls {
                query: "[(include_expression (_) @arg) (include_once_expression (_) @arg)
                         (require_expression (_) @arg) (require_once_expression (_) @arg)]",
                names: &[],
                literal: &[],
                edges: false,
            },
            branches: Branches::Absent("use に条件を付ける書き方が無い"),
            naming: Naming::Backslash,
        },
        Syntax {
            language: "go",
            extensions: &["go"],
            imports: "(import_spec path: (_) @to)",
            here: "",
            resolve: Resolve::Qualified,
            grammar: || tree_sitter_go::LANGUAGE.into(),
            bridge: Some(super::names::go),
            dynamic: Dynamic::Calls {
                query: "(call_expression function: (selector_expression) @how arguments: (argument_list . (_) @arg))",
                names: &["plugin.Open"],
                literal: &[],
                edges: false,
            },
            branches: Branches::AllBranches("//go:build の付いたファイル"),
            naming: Naming::Paths,
        },
        Syntax {
            language: "ruby",
            extensions: &["rb"],
            imports: "(call method: (identifier) @how arguments: (argument_list (string) @to))",
            here: "",
            resolve: Resolve::Path,
            grammar: || tree_sitter_ruby::LANGUAGE.into(),
            bridge: Some(super::names::ruby),
            dynamic: Dynamic::Calls {
                // 受け手の無い呼び出しだけ ── `FeatureLoader.load(...)` は Kernel の load ではない（実測 ── rubocop）
                // autoload は2つ目が経路である ── 1つ目の記号を見ると、静的な autoload を、静的に追跡できない読み込みと誤る（実測 ── rubocop）
                query: "(call !receiver method: (identifier) @how arguments: (argument_list . (_) @arg)
                          (#match? @how \"^(require|require_relative|load)$\"))
                        (call !receiver method: (identifier) @how arguments: (argument_list (_) . (_) @arg)
                          (#eq? @how \"autoload\"))",
                names: &["require", "require_relative", "load", "autoload"],
                literal: &["string"],
                edges: false,
            },
            branches: Branches::AllBranches("if の中の require"),
            naming: Naming::Paths,
        },
        Syntax {
            language: "cpp",
            extensions: &["cpp", "cc", "cxx", "c", "hpp", "hxx", "h"],
            imports: "(preproc_include path: (string_literal) @to)",
            here: "",
            resolve: Resolve::Path,
            grammar: || tree_sitter_cpp::LANGUAGE.into(),
            bridge: Some(super::names::cpp),
            dynamic: Dynamic::Calls {
                query: "[(preproc_include path: [(identifier) (call_expression)] @arg)
                         (call_expression function: (identifier) @how arguments: (argument_list . (_) @arg))]",
                names: &["dlopen", "LoadLibraryA", "LoadLibraryW"],
                literal: &[],
                edges: false,
            },
            branches: Branches::AllBranches("#if の中の #include"),
            naming: Naming::Paths,
        },
    ]
}

/// Ruby の `"#{__dir__}/経路"` を、そのファイルからの相対経路へ直す。**`__dir__` は、
/// そのファイルのディレクトリである** ── 埋め込みはそれ1つだけのときに限る（実測 ── rubocop）。
fn ruby_dir_relative(raw: &str) -> Option<String> {
    let body = raw.trim_matches('"');
    let rest = body.strip_prefix("#{__dir__}/")?;
    if rest.contains("#{") {
        return None;
    }
    Some(format!("./{rest}"))
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
    cleaned
        .strip_prefix(root)
        .unwrap_or(&cleaned)
        .display()
        .to_string()
}

/// 点の名前を、経路から組む。
fn dotted(root: &Path, file: &Path) -> String {
    let rel = file.strip_prefix(root).unwrap_or(file).with_extension("");
    let mut parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if parts
        .last()
        .is_some_and(|x| x == "__init__" || x == "mod" || x == "lib")
    {
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
    let mut parts: Vec<&str> = if here.is_empty() {
        Vec::new()
    } else {
        here.split('.').collect()
    };
    // **先頭の . の数は、書かれたファイルのディレクトリから遡る。** 1つ目はディレクトリ自身である
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
        return if here.is_empty() {
            tail.to_owned()
        } else {
            format!("{here}.{tail}")
        };
    }
    if let Some(tail) = body.strip_prefix("super.") {
        let mut parts: Vec<&str> = here.split('.').filter(|x| !x.is_empty()).collect();
        parts.pop();
        let head = parts.join(".");
        return if head.is_empty() {
            tail.to_owned()
        } else {
            format!("{head}.{tail}")
        };
    }
    body
}

/// 参照の文字列を、置き場所の名前空間の識別子へ直す。
///
/// **別名と検索パスで2つの名前空間をつなぐ**（`names`）。つなげない参照は、書かれたまま返す
/// ── 作業領域の外（標準の部品 ・ 外の部品）を指すものである。
fn resolve(
    s: &Syntax,
    bridge: &super::names::Bridge,
    root: &Path,
    file: &Path,
    here: &str,
    spec: &str,
    how: &str,
) -> String {
    let rel_file = file.strip_prefix(root).unwrap_or(file);
    match s.resolve {
        Resolve::Qualified => bridge
            .alias(spec, "/", &rel_file.display().to_string())
            .unwrap_or_else(|| spec.to_owned()),
        Resolve::Crate => {
            let body = in_crate(here, spec);
            // `crate::` は、その crate の根から数える ── 作業領域に crate が複数在る
            if spec.starts_with("crate::") {
                if let Some(head) = bridge
                    .aliases
                    .iter()
                    .map(|a| a.to.as_str())
                    .filter(|t| here == *t || here.starts_with(&format!("{t}.")))
                    .max_by_key(|t| t.len())
                {
                    return format!("{head}.{body}");
                }
            }
            bridge
                .alias(&body, ".", &rel_file.display().to_string())
                .unwrap_or(body)
        }
        Resolve::Dotted => {
            if spec.starts_with('.') {
                return relative_dotted(here, spec);
            }
            let first = spec.split('.').next().unwrap_or(spec);
            // 根は最後に試す ── source root の下に同じ名前が在れば、そちらが先に見つかる
            let order = bridge
                .search
                .iter()
                .filter(|x| !x.is_empty())
                .chain(bridge.search.iter().filter(|x| x.is_empty()));
            for sr in order {
                let base = root.join(sr);
                if files::is_dir(base.join(first))
                    || files::is_file(base.join(format!("{first}.py")))
                {
                    return if sr.is_empty() {
                        spec.to_owned()
                    } else {
                        format!("{}.{spec}", sr.replace('/', "."))
                    };
                }
            }
            spec.to_owned()
        }
        Resolve::Path => {
            let relative = spec.starts_with("./") || spec.starts_with("../");
            match s.language {
                "typescript" if !relative => {
                    let Some(to) = bridge.alias(spec, "/", &rel_file.display().to_string()) else {
                        return spec.to_owned();
                    };
                    // package の subpath は、組み立ての出力（dist）を指すことが多い ── 置き場所に無ければ、
                    // その package の source root（tsconfig の rootDir、無ければ src）の下を探す（実測 ── trpc）
                    let exists = |x: &str| {
                        ["", ".ts", ".tsx", "/index.ts", "/index.tsx"]
                            .iter()
                            .any(|e| files::exists(root.join(format!("{x}{e}"))))
                    };
                    if exists(&to) {
                        return to;
                    }
                    for a in &bridge.aliases {
                        if let Some(tail) = to.strip_prefix(&format!("{}/", a.to)) {
                            for src in &bridge.search {
                                if let Some(inner) = src.strip_prefix(&format!("{}/", a.to)) {
                                    let cand = format!("{}/{inner}/{tail}", a.to);
                                    if exists(&cand) {
                                        return cand;
                                    }
                                }
                            }
                        }
                    }
                    to
                }
                "ruby" if how != "require_relative" && !relative => {
                    for lp in &bridge.search {
                        let base = root.join(lp);
                        if files::is_file(base.join(format!("{spec}.rb")))
                            || files::is_dir(base.join(spec))
                        {
                            return format!("{lp}/{spec}");
                        }
                    }
                    // 行き先が無くても、頭のディレクトリが検索パスの下に在れば、そこを指している（生成物か、名前の誤り）
                    let first = spec.split('/').next().unwrap_or(spec);
                    for lp in &bridge.search {
                        if files::exists(root.join(lp).join(first))
                            || files::is_file(root.join(lp).join(format!("{first}.rb")))
                        {
                            return format!("{lp}/{spec}");
                        }
                    }
                    spec.to_owned()
                }
                "cpp" => {
                    let folded = fold(root, rel_file, spec);
                    if files::exists(root.join(&folded)) {
                        return folded;
                    }
                    for inc in &bridge.search {
                        let cand = if inc.is_empty() {
                            spec.to_owned()
                        } else {
                            format!("{inc}/{spec}")
                        };
                        if files::exists(root.join(&cand)) {
                            return cand;
                        }
                    }
                    // 行き先が無くても、頭のディレクトリが検索パスの下に在れば、そこを指している
                    let first = spec.split('/').next().unwrap_or(spec);
                    for inc in &bridge.search {
                        let base = if inc.is_empty() {
                            root.to_path_buf()
                        } else {
                            root.join(inc)
                        };
                        if spec.contains('/') && files::is_dir(base.join(first)) {
                            return if inc.is_empty() {
                                spec.to_owned()
                            } else {
                                format!("{inc}/{spec}")
                            };
                        }
                    }
                    spec.to_owned()
                }
                _ => fold(root, rel_file, spec),
            }
        }
    }
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
        table()
            .into_iter()
            .find(|s| s.language == language)
            .map(|syntax| Self { syntax })
    }

    /// その言語のソースの拡張子。**名前の末尾を外すとき、これだけを拡張子として扱う**
    /// ── 版の番号（`v1.43.0`）や名前の一部（`types.gen`）を、拡張子として外さない（実測）。
    #[must_use]
    pub fn extensions(&self) -> &'static [&'static str] {
        self.syntax.extensions
    }

    /// 名前の区切り方を、向きの判定が読む形で返す。
    #[must_use]
    pub fn names(&self) -> super::judge::Names {
        match self.syntax.naming {
            Naming::Paths => super::judge::Names::paths(self.syntax.extensions),
            Naming::Dotted => super::judge::Names::dotted(),
            Naming::Backslash => super::judge::Names::backslash(),
        }
    }

    /// 扱える言語を並べる。
    #[must_use]
    pub fn languages() -> Vec<&'static str> {
        table().iter().map(|s| s.language).collect()
    }
}

/// 見ないディレクトリ。
const SKIP: [&str; 6] = [
    "node_modules",
    "target",
    "build",
    "dist",
    "__pycache__",
    "vendor",
];

fn sources(dir: &Path, want: &[&str], out: &mut Vec<PathBuf>) {
    // **名前の順に並べて返る** ── 並びを OS に任せると、同じ入力から別の結果が出る
    let Ok(paths) = files::list(dir) else {
        return;
    };
    for p in paths {
        let name = p
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if files::is_dir(&p) {
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

/// 読み込みを起こす名前。**引数が文字列でなければ抜け道である。**
const RUBY_LOADERS: [&str; 4] = ["require", "require_relative", "load", "autoload"];

impl Extractor for Tree {
    fn language(&self) -> &'static str {
        self.syntax.language
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
        let bridge = super::names::from(s.bridge, root);
        let limits = if matches!(s.language, "java" | "kotlin" | "csharp" | "php") {
            vec![
                "字面で測っている ── 名前は、ソースが宣言する package ・ namespace から取っている"
                    .to_owned(),
            ]
        } else if bridge.sources.is_empty() {
            vec!["字面で測っている ── 別名と検索パスの設定が見つからなかった。置き場所の名前だけで照合している".to_owned()]
        } else {
            // 設定の名前ごとに数える ── 全件を並べると、1行が読めない長さになる（実測 ── 100件）
            let mut count: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();
            for src in &bridge.sources {
                let name = src.rsplit('/').next().unwrap_or(src).to_owned();
                *count.entry(name).or_default() += 1;
            }
            vec![format!(
                "字面で測っている ── 別名と検索パスは、次の設定から解決した：{}",
                count
                    .iter()
                    .map(|(k, v)| format!("{k} {v}件"))
                    .collect::<Vec<_>>()
                    .join(" ・ ")
            )]
        };
        let mut limits = limits;
        limits.push(format!("走査しないディレクトリ ── {}", SKIP.join(" ・ ")));
        let mut points = Vec::new();

        for file in &files {
            let Ok(body) = files::read_to_string(file) else {
                undecided.push(format!("{} ── 読めない", file.display()));
                continue;
            };
            let Some(tree) = parser.parse(&body, None) else {
                undecided.push(format!("{} ── 解析できない", file.display()));
                continue;
            };
            let bytes = body.as_bytes();
            let rel = file
                .strip_prefix(root)
                .unwrap_or(file)
                .display()
                .to_string();
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
            points.push(here.clone());
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
                let owned;
                let (spec, how) = if s.language == "ruby" && raw.contains("#{") {
                    match ruby_dir_relative(raw) {
                        Some(r) => {
                            owned = r;
                            (owned.as_str(), "require_relative".to_owned())
                        }
                        None => continue, // 静的に追跡できない読み込みとして別に出す
                    }
                } else {
                    (unquote(raw), how)
                };
                // Rust の入れ子の mod ── `mod tests { use super::x; }` の super は、その mod から数える（実測 ── ripgrep）
                let inner_here = if s.language == "rust" {
                    let mut names = Vec::new();
                    let mut node = m.captures.first().map(|c| c.node);
                    while let Some(n) = node {
                        if n.kind() == "mod_item" {
                            if let Some(name) = n.child_by_field_name("name") {
                                names.push(name.utf8_text(bytes).unwrap_or("").to_owned());
                            }
                        }
                        node = n.parent();
                    }
                    names.reverse();
                    if names.is_empty() {
                        here.clone()
                    } else {
                        format!("{here}.{}", names.join("."))
                    }
                } else {
                    here.clone()
                };
                let to = resolve(s, &bridge, root, file, &inner_here, spec, &how);
                // 経路で書く言語は、走査しないディレクトリ（vendor など）の中でも、ファイルが在れば
                // 作業領域の中のモジュールである（実測 ── trpc）
                if s.naming == Naming::Paths
                    && s.extensions.iter().chain(std::iter::once(&"")).any(|x| {
                        let f = if x.is_empty() {
                            root.join(&to)
                        } else {
                            root.join(format!("{to}.{x}"))
                        };
                        files::is_file(&f)
                            || files::is_file(root.join(&to).join(format!("index.{x}")))
                    })
                {
                    points.push(to.clone());
                }
                edges.push(Edge::new(here.clone(), to, format!("{rel}:{line}")));
            }
            // **名前を実行時に決める読み込み** ── 文字列で書かれていれば依存、そうでなければ静的に追跡できない読み込み
            if let Dynamic::Calls {
                query,
                names,
                literal,
                edges: make_edges,
            } = s.dynamic
            {
                let dyn_query = Query::new(&lang, query)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
                let mut c2 = QueryCursor::new();
                let mut got = c2.matches(&dyn_query, tree.root_node(), bytes);
                while let Some(m) = got.next() {
                    let mut how = String::new();
                    let mut arg = None;
                    for cap in m.captures {
                        let name = &dyn_query.capture_names()[cap.index as usize];
                        if *name == "how" {
                            how = cap.node.utf8_text(bytes).unwrap_or("").to_owned();
                        } else if *name == "arg" {
                            arg = Some(cap.node);
                        }
                    }
                    let Some(arg) = arg else { continue };
                    let named = how.is_empty()
                        || names.is_empty()
                        || names.iter().any(|n| {
                            how == *n
                                || how.ends_with(&format!(".{n}"))
                                || how.ends_with(&format!("::{n}"))
                        });
                    if !named {
                        continue;
                    }
                    let line = arg.start_position().row + 1;
                    let embedded = (0..arg.child_count()).any(|k| {
                        arg.child(k).is_some_and(|c| {
                            matches!(c.kind(), "interpolation" | "template_substitution")
                        })
                    }) && !(s.language == "ruby"
                        && ruby_dir_relative(arg.utf8_text(bytes).unwrap_or("")).is_some());
                    if literal.contains(&arg.kind()) && !embedded {
                        if make_edges {
                            let spec = unquote(arg.utf8_text(bytes).unwrap_or(""));
                            let to = resolve(s, &bridge, root, file, &here, spec, &how);
                            edges.push(Edge::new(here.clone(), to, format!("{rel}:{line}")));
                        }
                    } else {
                        let what = if how.is_empty() {
                            arg.kind().to_owned()
                        } else {
                            how.clone()
                        };
                        escapes.push(Escape::new(
                            here.clone(),
                            format!("{rel}:{line}"),
                            format!(
                                "静的に追跡できない読み込み ── {what} の行き先が、実行時に決まる"
                            ),
                        ));
                    }
                }
            }
        }
        Ok(Extracted {
            edges,
            escapes,
            undecided,
            points,
            unreadable: bridge.unreadable.clone(),
            limits,
        })
    }
}
