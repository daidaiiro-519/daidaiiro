// SPDX-License-Identifier: MIT
//! 参照の名前空間と、置き場所の名前空間をつなぐ。
//!
//! 読み込みの文に書かれる名前（参照の名前空間）と、ファイルの在る場所（置き場所の
//! 名前空間）は、言語によって食い違う。**食い違いを生むものは2種類である。**
//!
//! | 種類 | 何か | 例 |
//! |---|---|---|
//! | 別名 | 置き場所に付けた別の名前 | crate 名 ・ module のパス ・ package 名 ・ tsconfig の paths |
//! | 検索パス | 読み込みの名前を探し始める場所 | Python の source root ・ Ruby の load path ・ C++ の include ディレクトリ |
//!
//! **ソースが自分の名前を宣言する言語（Java ・ Kotlin ・ C# ・ PHP）は食い違わない** ──
//! 参照する側も参照される側も、宣言の名前空間で書かれる。
//!
//! **その言語の道具は実行しない。** 設定のファイルを読むだけである。
//! **読めなかった設定は、`unreadable` へ入れる** ── 黙って飛ばすと、つなげなかった参照が
//! 作業領域の外として扱われ、検査を素通りする。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// 別名1件。**参照の名前の頭を、置き場所の名前の頭へ置き換える。**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alias {
    /// 参照の名前空間での頭（crate 名 ・ module のパス ・ paths の型など）。
    pub from: String,
    /// 置き場所の名前空間での頭。
    pub to: String,
    /// 効く範囲（根からの相対のディレクトリ）。**空なら作業領域の全体である** ── tsconfig の paths は、
    /// その tsconfig の下のファイルにだけ効く（実測 ── 例ごとに同じ `~` を別のディレクトリへ向けていた）。
    pub scope: String,
}

/// 2つの名前空間をつなぐ対応表。
#[derive(Debug, Clone, Default)]
pub struct Bridge {
    /// 別名。**長いものから試す** ── 入れ子の別名で、短いほうへ吸われない。
    pub aliases: Vec<Alias>,
    /// 検索パス（根からの相対）。
    pub search: Vec<String>,
    /// どこから組んだか。**申告として出す。**
    pub sources: Vec<String>,
    /// 読めなかった設定。**（その設定が効くディレクトリ, 文面）**
    pub unreadable: Vec<(String, String)>,
}

/// 見ないディレクトリ。
const SKIP: [&str; 7] = [
    "node_modules",
    "target",
    "build",
    "dist",
    "__pycache__",
    "vendor",
    "testdata",
];

/// 名前が条件に合うファイルを、根の下から集める。
fn find(dir: &Path, want: &dyn Fn(&str) -> bool, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        let file = p
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if p.is_dir() {
            if !SKIP.contains(&file.as_str()) && !file.starts_with('.') {
                find(&p, want, out);
            }
        } else if want(&file) {
            out.push(p);
        }
    }
}

fn rel(root: &Path, p: &Path) -> String {
    normalize(&p.strip_prefix(root).unwrap_or(p).display().to_string())
}

/// `a/./b` ・ `a/../b` ・ 末尾の `/` を畳む。
fn normalize(p: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for x in p.split('/') {
        match x {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

/// . 区切りの名前へ直す。
fn dotted(path: &str) -> String {
    path.replace(['/', '\\'], ".")
}

/// JSON にコメントと末尾のカンマを許した形（tsconfig）を、JSON へ直す。
/// **文字列の中は触らない。**
fn jsonc(body: &str) -> String {
    let b: Vec<char> = body.chars().collect();
    let mut out = String::with_capacity(body.len());
    let mut i = 0;
    let mut in_str = false;
    while i < b.len() {
        let c = b[i];
        if in_str {
            out.push(c);
            if c == '\\' && i + 1 < b.len() {
                out.push(b[i + 1]);
                i += 2;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            i += 1;
        } else if c == '/' && b.get(i + 1) == Some(&'/') {
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
        } else if c == '/' && b.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < b.len() && !(b[i] == '*' && b[i + 1] == '/') {
                i += 1;
            }
            i += 2;
        } else {
            out.push(c);
            i += 1;
        }
    }
    // 末尾のカンマ ── `,` の直後に空白を挟んで `]` か `}` が来るもの
    let mut cleaned = String::with_capacity(out.len());
    let chars: Vec<char> = out.chars().collect();
    let mut in_str = false;
    for (k, &c) in chars.iter().enumerate() {
        if c == '"' && (k == 0 || chars[k - 1] != '\\') {
            in_str = !in_str;
        }
        if !in_str && c == ',' {
            let next = chars[k + 1..].iter().find(|x| !x.is_whitespace());
            if matches!(next, Some(']' | '}')) {
                continue;
            }
        }
        cleaned.push(c);
    }
    cleaned
}

fn read_jsonc(p: &Path) -> Result<serde_json::Value, String> {
    let body = std::fs::read_to_string(p).map_err(|e| e.to_string())?;
    serde_json::from_str(&jsonc(&body)).map_err(|e| e.to_string())
}

fn read_toml(p: &Path) -> Result<toml::Value, String> {
    let body = std::fs::read_to_string(p).map_err(|e| e.to_string())?;
    body.parse::<toml::Value>().map_err(|e| e.to_string())
}

/// 言語の表の契約の欄（`bridge`）から、対応表を組む。
#[must_use]
pub fn from(read: Option<fn(&Path, &mut Bridge)>, root: &Path) -> Bridge {
    let mut b = Bridge::default();
    if let Some(read) = read {
        read(root, &mut b);
    }
    b.aliases.sort_by_key(|a| std::cmp::Reverse(a.from.len()));
    b.search.sort();
    b.search.dedup();
    b
}

/// Rust ── crate 名（Cargo.toml の `[package] name`）を、crate の根へ。
pub fn rust(root: &Path, b: &mut Bridge) {
    let mut found = Vec::new();
    find(root, &|f| f == "Cargo.toml", &mut found);
    for f in found {
        let v = match read_toml(&f) {
            Ok(v) => v,
            Err(e) => {
                b.unreadable.push((
                    rel(root, f.parent().unwrap_or(root)),
                    format!("{} ── {e}", rel(root, &f)),
                ));
                continue;
            }
        };
        let Some(pkg) = v.get("package") else {
            continue;
        }; // 作業領域の根
        let Some(name) = pkg.get("name").and_then(|x| x.as_str()) else {
            b.unreadable.push((
                rel(root, f.parent().unwrap_or(root)),
                format!("{} ── [package] に name が無い", rel(root, &f)),
            ));
            continue;
        };
        let dir = f.parent().unwrap_or(root);
        // `[lib] path` が在れば、そのディレクトリが crate の根である
        let lib = v
            .get("lib")
            .and_then(|l| l.get("path"))
            .and_then(|p| p.as_str())
            .map_or_else(
                || dir.join("src"),
                |p| dir.join(p).parent().unwrap_or(dir).to_path_buf(),
            );
        let lib_name = v
            .get("lib")
            .and_then(|l| l.get("name"))
            .and_then(|x| x.as_str())
            .unwrap_or(name);
        b.aliases.push(Alias {
            from: lib_name.replace('-', "_"),
            to: dotted(&rel(root, &lib)),
            scope: String::new(),
        });
        b.sources.push(rel(root, &f));
    }
}

/// Go ── module のパス（go.mod の `module`）を、そのディレクトリへ。
pub fn go(root: &Path, b: &mut Bridge) {
    let mut found = Vec::new();
    find(root, &|f| f == "go.mod", &mut found);
    for f in found {
        let Ok(body) = std::fs::read_to_string(&f) else {
            b.unreadable.push((
                rel(root, f.parent().unwrap_or(root)),
                format!("{} ── 読めない", rel(root, &f)),
            ));
            continue;
        };
        let module = body.lines().find_map(|l| {
            let l = l.split("//").next().unwrap_or("").trim();
            l.strip_prefix("module")
                .map(|m| m.trim().trim_matches('"').to_owned())
        });
        match module {
            Some(m) if !m.is_empty() => {
                let dir = rel(root, f.parent().unwrap_or(root));
                b.aliases.push(Alias {
                    from: m,
                    to: dir,
                    scope: String::new(),
                });
                b.sources.push(rel(root, &f));
            }
            _ => b.unreadable.push((
                rel(root, f.parent().unwrap_or(root)),
                format!("{} ── module を宣言していない", rel(root, &f)),
            )),
        }
    }
}

/// tsconfig の `extends` をたどり、`paths` と `baseUrl` を集める。**近いほうが勝つ。**
fn ts_paths(
    root: &Path,
    f: &Path,
    depth: usize,
    b: &mut Bridge,
) -> (Option<PathBuf>, BTreeMap<String, Vec<String>>) {
    if depth > 8 {
        return (None, BTreeMap::new());
    }
    let v = match read_jsonc(f) {
        Ok(v) => v,
        Err(e) => {
            b.unreadable.push((
                rel(root, f.parent().unwrap_or(root)),
                format!("{} ── {e}", rel(root, f)),
            ));
            return (None, BTreeMap::new());
        }
    };
    let dir = f.parent().unwrap_or(root);
    let (mut base, mut paths) = match v.get("extends").and_then(|x| x.as_str()) {
        Some(e) if e.starts_with('.') => {
            let mut p = dir.join(e);
            if p.extension().is_none() {
                p.set_extension("json");
            }
            ts_paths(root, &p, depth + 1, b)
        }
        _ => (None, BTreeMap::new()),
    };
    let opts = &v["compilerOptions"];
    if let Some(u) = opts["baseUrl"].as_str() {
        base = Some(dir.join(u));
    }
    if let Some(ps) = opts["paths"].as_object() {
        let at = base.clone().unwrap_or_else(|| dir.to_path_buf());
        for (k, targets) in ps {
            let list: Vec<String> = targets
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str())
                        .map(|t| rel(root, &at.join(t)))
                        .collect()
                })
                .unwrap_or_default();
            paths.insert(k.clone(), list);
        }
    }
    (base, paths)
}

/// TypeScript ── package 名（package.json の `name`）と、tsconfig の `paths`。
pub fn typescript(root: &Path, b: &mut Bridge) {
    let mut found = Vec::new();
    find(root, &|f| f == "package.json", &mut found);
    for f in found {
        match read_jsonc(&f) {
            Ok(v) => {
                if let Some(name) = v.get("name").and_then(|x| x.as_str()) {
                    let dir = rel(root, f.parent().unwrap_or(root));
                    b.aliases.push(Alias {
                        from: name.to_owned(),
                        to: dir,
                        scope: String::new(),
                    });
                    b.sources.push(rel(root, &f));
                }
            }
            Err(e) => b.unreadable.push((
                rel(root, f.parent().unwrap_or(root)),
                format!("{} ── {e}", rel(root, &f)),
            )),
        }
    }
    // package の source root ── tsconfig の rootDir、無ければ src
    let dirs: Vec<String> = b.aliases.iter().map(|a| a.to.clone()).collect();
    for d in dirs {
        let pkg = root.join(&d);
        let mut src = None;
        for name in ["tsconfig.build.json", "tsconfig.json"] {
            if let Ok(v) = read_jsonc(&pkg.join(name)) {
                if let Some(r) = v["compilerOptions"]["rootDir"].as_str() {
                    src = Some(rel(root, &pkg.join(r)));
                    break;
                }
            }
        }
        if src.is_none() && pkg.join("src").is_dir() {
            src = Some(rel(root, &pkg.join("src")));
        }
        if let Some(s) = src {
            if s != d {
                b.search.push(s);
            }
        }
    }
    let mut ts = Vec::new();
    find(
        root,
        &|f| f.starts_with("tsconfig") && f.ends_with(".json"),
        &mut ts,
    );
    for f in ts {
        let (_, paths) = ts_paths(root, &f, 0, b);
        for (pat, targets) in paths {
            let Some(target) = targets.first() else {
                continue;
            };
            let from = pat.trim_end_matches('*').trim_end_matches('/').to_owned();
            let to = target
                .trim_end_matches('*')
                .trim_end_matches('/')
                .to_owned();
            let scope = rel(root, f.parent().unwrap_or(root));
            if !b.aliases.iter().any(|a| a.from == from && a.scope == scope) {
                b.aliases.push(Alias {
                    from,
                    to: normalize(&to),
                    scope,
                });
            }
        }
        b.sources.push(rel(root, &f));
    }
}

/// Python ── source root（pyproject.toml の各道具の書き方、無ければ `src`）。
pub fn python(root: &Path, b: &mut Bridge) {
    let mut found = Vec::new();
    find(root, &|f| f == "pyproject.toml", &mut found);
    for f in &found {
        let v = match read_toml(f) {
            Ok(v) => v,
            Err(e) => {
                b.unreadable.push((
                    rel(root, f.parent().unwrap_or(root)),
                    format!("{} ── {e}", rel(root, f)),
                ));
                continue;
            }
        };
        let dir = f.parent().unwrap_or(root);
        let mut roots: Vec<String> = Vec::new();
        let tool = v.get("tool");
        let strs = |x: Option<&toml::Value>| -> Vec<String> {
            x.and_then(|a| a.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|s| s.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default()
        };
        // setuptools
        if let Some(st) = tool.and_then(|t| t.get("setuptools")) {
            roots.extend(strs(
                st.get("packages")
                    .and_then(|p| p.get("find"))
                    .and_then(|f| f.get("where")),
            ));
            if let Some(pd) = st
                .get("package-dir")
                .and_then(|x| x.get(""))
                .and_then(|x| x.as_str())
            {
                roots.push(pd.to_owned());
            }
        }
        // hatch
        if let Some(w) = tool
            .and_then(|t| t.get("hatch"))
            .and_then(|h| h.get("build"))
            .and_then(|x| x.get("targets"))
            .and_then(|x| x.get("wheel"))
        {
            roots.extend(strs(w.get("sources")));
            for p in strs(w.get("packages")) {
                if let Some((parent, _)) = p.trim_end_matches('/').rsplit_once('/') {
                    roots.push(parent.to_owned());
                }
            }
        }
        // poetry
        if let Some(ps) = tool
            .and_then(|t| t.get("poetry"))
            .and_then(|p| p.get("packages"))
            .and_then(|x| x.as_array())
        {
            for p in ps {
                if let Some(from) = p.get("from").and_then(|x| x.as_str()) {
                    roots.push(from.to_owned());
                }
            }
        }
        for r in roots {
            b.search.push(rel(root, &dir.join(r)));
        }
        b.sources.push(rel(root, f));
    }
    if b.search.is_empty() && root.join("src").is_dir() {
        b.search.push("src".to_owned());
        b.sources.push("src（既定の source root）".to_owned());
    }
    // 根も検索パスである ── 根から始まる . 区切りの名前がそのまま通る
    b.search.push(String::new());
}

/// Ruby ── load path（gemspec の `require_paths`、無ければ `lib`）。
pub fn ruby(root: &Path, b: &mut Bridge) {
    let mut found = Vec::new();
    find(root, &|f| f.ends_with(".gemspec"), &mut found);
    for f in &found {
        let Ok(body) = std::fs::read_to_string(f) else {
            b.unreadable.push((
                rel(root, f.parent().unwrap_or(root)),
                format!("{} ── 読めない", rel(root, f)),
            ));
            continue;
        };
        let dir = f.parent().unwrap_or(root);
        let mut got = false;
        for line in body.lines().filter(|l| l.contains("require_paths")) {
            for piece in line.split(['"', '\'']).skip(1).step_by(2) {
                b.search.push(rel(root, &dir.join(piece)));
                got = true;
            }
        }
        if !got && dir.join("lib").is_dir() {
            b.search.push(rel(root, &dir.join("lib")));
        }
        b.sources.push(rel(root, f));
    }
    if found.is_empty() && root.join("lib").is_dir() {
        b.search.push("lib".to_owned());
        b.sources.push("lib（既定の load path）".to_owned());
    }
}

/// CMake の引数を、変数と生成式を解いて並べる。解けない変数を含む経路は `None`。
fn cmake_args(args: &str, vars: &BTreeMap<String, String>) -> Vec<Option<String>> {
    let mut out = Vec::new();
    for raw in args.split_whitespace() {
        let mut a = raw.trim_matches('"').to_owned();
        // 生成式 ── 組み立てのときの経路だけを採る
        if let Some(x) = a.strip_prefix("$<BUILD_INTERFACE:") {
            a = x.trim_end_matches('>').to_owned();
        } else if a.starts_with("$<") {
            continue;
        }
        // 変数を展開する
        let mut unknown = false;
        while let Some(s) = a.find("${") {
            let Some(e) = a[s..].find('}') else { break };
            let name = &a[s + 2..s + e];
            match vars.get(name) {
                Some(v) => a = format!("{}{}{}", &a[..s], v, &a[s + e + 1..]),
                None => {
                    unknown = true;
                    break;
                }
            }
        }
        if unknown {
            // 変数だけの語は、目標の名前か修飾語である ── 経路として扱わない
            if raw.starts_with("${") && raw.ends_with('}') && raw.matches("${").count() == 1 {
                continue;
            }
            out.push(None);
            continue;
        }
        out.push(Some(a));
    }
    out
}

/// C++ ── include ディレクトリ（CMakeLists.txt、compile_commands.json）。
pub fn cpp(root: &Path, b: &mut Bridge) {
    const WORDS: [&str; 7] = [
        "PUBLIC",
        "PRIVATE",
        "INTERFACE",
        "SYSTEM",
        "BEFORE",
        "AFTER",
        "",
    ];
    let mut found = Vec::new();
    find(root, &|f| f == "CMakeLists.txt", &mut found);
    for f in &found {
        let Ok(body) = std::fs::read_to_string(f) else {
            b.unreadable.push((
                rel(root, f.parent().unwrap_or(root)),
                format!("{} ── 読めない", rel(root, f)),
            ));
            continue;
        };
        let dir = f.parent().unwrap_or(root);
        let mut vars = BTreeMap::new();
        vars.insert(
            "CMAKE_CURRENT_SOURCE_DIR".to_owned(),
            dir.display().to_string(),
        );
        vars.insert(
            "CMAKE_CURRENT_LIST_DIR".to_owned(),
            dir.display().to_string(),
        );
        vars.insert("PROJECT_SOURCE_DIR".to_owned(), root.display().to_string());
        vars.insert("CMAKE_SOURCE_DIR".to_owned(), root.display().to_string());
        // 同じファイルの set(名前 値) ── 値が1語のものだけ
        for chunk in body.split("set(").skip(1) {
            let args = chunk.split(')').next().unwrap_or("");
            let mut it = args.split_whitespace();
            if let (Some(k), Some(v), None) = (it.next(), it.next(), it.next()) {
                vars.entry(k.to_owned())
                    .or_insert_with(|| v.trim_matches('"').to_owned());
            }
        }
        for call in ["target_include_directories(", "include_directories("] {
            let mut rest = body.as_str();
            while let Some(pos) = rest.find(call) {
                // target_include_directories の中の include_directories( を二重に数えない
                let before = &rest[..pos];
                rest = &rest[pos + call.len()..];
                if call == "include_directories(" && before.ends_with("target_") {
                    continue;
                }
                let args = rest.split(')').next().unwrap_or("");
                let mut list = cmake_args(args, &vars);
                if call.starts_with("target") && !list.is_empty() {
                    list.remove(0); // 目標の名前
                }
                for a in list {
                    match a {
                        Some(a) if WORDS.contains(&a.as_str()) => {}
                        Some(a) => {
                            let p = Path::new(&a);
                            let full = if p.is_absolute() {
                                p.to_path_buf()
                            } else {
                                dir.join(p)
                            };
                            b.search.push(rel(root, &full));
                        }
                        None => b.unreadable.push((
                            rel(root, dir),
                            format!(
                                "{} ── include の経路に、展開できない変数が在る",
                                rel(root, f)
                            ),
                        )),
                    }
                }
            }
        }
        b.sources.push(rel(root, f));
    }
    let cc = root.join("compile_commands.json");
    if cc.is_file() {
        if let Ok(v) = read_jsonc(&cc) {
            for e in v.as_array().into_iter().flatten() {
                let cmd = e["command"].as_str().unwrap_or("");
                for tok in cmd.split_whitespace() {
                    if let Some(p) = tok.strip_prefix("-I") {
                        b.search.push(rel(root, Path::new(p)));
                    }
                }
            }
            b.sources.push("compile_commands.json".to_owned());
        }
    }
}

impl Bridge {
    /// 別名を当てる。**頭が一致したものだけを置き換える。** 効く範囲の中で、
    /// いちばん近い範囲 ・ いちばん長い頭のものを採る。
    #[must_use]
    pub fn alias(&self, name: &str, sep: &str, file: &str) -> Option<String> {
        let mut best: Option<(&Alias, String)> = None;
        for a in &self.aliases {
            let covers =
                a.scope.is_empty() || file == a.scope || file.starts_with(&format!("{}/", a.scope));
            if !covers {
                continue;
            }
            let got = if name == a.from {
                Some(a.to.clone())
            } else {
                name.strip_prefix(&format!("{}{sep}", a.from)).map(|tail| {
                    if a.to.is_empty() {
                        tail.to_owned()
                    } else {
                        format!("{}{sep}{tail}", a.to)
                    }
                })
            };
            if let Some(g) = got {
                let better = best.as_ref().is_none_or(|(b, _)| {
                    (a.scope.len(), a.from.len()) > (b.scope.len(), b.from.len())
                });
                if better {
                    best = Some((a, g));
                }
            }
        }
        best.map(|(_, g)| g)
    }
}
