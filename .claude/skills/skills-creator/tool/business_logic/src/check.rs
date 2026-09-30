// SPDX-License-Identifier: MIT
//! Skill が契約を満たしているかを検査する。**見つけるが、直さない。**
//!
//! **2段で検査する**（ACDR 0036）。1段目は実行ファイルを起動して振る舞いを確認する ── どの言語でも
//! 同じである（`behavior`）。2段目はソースを読む検査で、**言語の組が持つ**。組が在るのは
//! いま Rust だけで、組が無い言語では「実行しない」と出す ── 合格とは扱わない。
//! 最後に、節の構成が対応する雛形を満たすかを見る。
//!
//! Rust の組の2段目が見るのは、層が crate に分かれていること、**依存の向きが契約どおりで
//! あること**、外部の道具の名前を直書きしていないことである。
//!
//! **層を crate に分ける。** 1つの crate の中の module では、内側が外側を参照しても
//! コンパイラが通す（実測 2026-09-26）── 層の境界を crate の境界に置いて初めて、
//! `Cargo.toml` に書いていない依存が解決しなくなる。
//!
//! **依存の向きは各 `Cargo.toml` が宣言する。** この検査は `Cargo.toml` を読み、層の並びと
//! 食い違っていないかを見る ── 実際に守られているかはコンパイラが判定する。

use std::path::{Path, PathBuf};

use crate::behavior::{self, Verdict};
use crate::data_access::files;
use crate::sections;

/// Skill の道具のソースを置く場所。**配布しない。** 名前は中身の役割（道具）で付ける ──
/// 以前の rs/ は言語の名前で、役割を示さなかった。
pub const TOOL: &str = "tool";

/// Rust の組が、組み立てた実行ファイルを置く場所。**git で追跡しない。**
pub const BIN: &str = "bin";

/// 層の並び。**下から上である** ── 先頭が最も下（データアクセス層）である。
/// 名前はレイヤードアーキテクチャの層の正式名に合わせる（ACDR 0056 ・ 0058）。
pub const ORDER: [&str; 4] = ["data_access", "business_logic", "service", "presentation"];

/// 入出力を禁じた層のソース（`tool/` からの相対）。**入出力はデータアクセス層だけが持つ**（ACDR 0058）。
pub const NO_IO: [&str; 2] = ["business_logic/src", "service/src"];

/// 入出力の禁止から外すファイル。**`contract.rs` は起動の設定（tool.json）と
/// Skill のフォルダを求める** ── 全 Skill 共通の複製であり、呼ぶ側との契約そのものである。
pub const NO_IO_EXEMPT: [&str; 1] = ["contract.rs"];

/// 入出力の印。**この文字列を含む行は、入出力を直接扱っている** ── `files::` ・ `process::` を通す。
pub const IO_MARKS: [&str; 12] = [
    "std::fs",
    "fs::",
    "std::process",
    "Command::new",
    "std::net",
    "TcpListener",
    "TcpStream",
    "File::",
    "std::env::temp_dir",
    ".is_file()",
    ".is_dir()",
    ".exists()",
];

/// プレゼンテーション層に属する crate。**呼び出し方ごとに1つなので、複数在ってよい。**
pub const ENTRIES: [&str; 2] = ["cli", "mcp"];

/// 以前の層の名前。**残っていれば、改名が済んでいない**（ACDR 0056）。
pub const RETIRED: [(&str, &str); 2] = [("parts", "business_logic"), ("declare", "service")];

/// 事例を置く場所（`tool/` からの相対）。
pub const TESTS: &str = "business_logic/tests";

/// 助言の Skill を見分ける節の名前。**名前で分岐しない** ── Skill が増えるたびに直す。
pub const ADVISOR_MARK: &str = "相談種別と回答テンプレート";

/// 雛形の置き場所。**呼ぶ側が渡す** ── この crate が Skill の並びを認知しない。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Templates {
    /// 一般の Skill が満たす雛形。
    pub general: PathBuf,
    /// 助言の Skill が満たす雛形。
    pub advisor: PathBuf,
    /// references の実装の雛形（Rust の組）。**版2 の Skill の複製と突き合わせる。**
    pub refs: Option<PathBuf>,
}

impl Templates {
    /// 雛形の置き場所を組む。**欄を足しても、呼ぶ側は壊れない。**
    #[must_use]
    pub const fn new(general: PathBuf, advisor: PathBuf) -> Self {
        Self {
            general,
            advisor,
            refs: None,
        }
    }

    /// references の実装の雛形を足す。
    #[must_use]
    pub fn with_refs(mut self, refs: PathBuf) -> Self {
        self.refs = Some(refs);
        self
    }
}

/// その層が依存してよい層を返す。**直下の1つだけである** ── 層を飛ばすと、サービス層の
/// 道具の一覧を経ない呼び出し方ができる。**並びから導く** ── 表を別に持つと、並びとずれる。
#[must_use]
fn allowed(layer: &str) -> Vec<&'static str> {
    match ORDER.iter().position(|x| *x == layer) {
        Some(i) if i > 0 => vec![ORDER[i - 1]],
        _ => Vec::new(),
    }
}

/// crate の名前から、属する層を決める。**名前の末尾で見る** ── crate は `<接頭辞>_<層>` の形で、
/// 層の名前そのものが `_` を含む（`business_logic`）ので、区切りで割らない。
#[must_use]
fn layer_of(name: &str) -> Option<&'static str> {
    let ends = |x: &str| name == x || name.ends_with(&format!("_{x}"));
    if ENTRIES.iter().any(|e| ends(e)) {
        return Some("presentation");
    }
    ORDER.iter().copied().find(|x| ends(x))
}

/// `Cargo.toml` が宣言している、同じ workspace の中の依存を読む。
///
/// **`path = "../…"` の形だけを見る** ── 外の crate は層の外である。
fn declared(manifest: &Path) -> Vec<String> {
    let Ok(body) = files::read_to_string(manifest) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut in_deps = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_deps = trimmed == "[dependencies]";
            continue;
        }
        if !in_deps || !trimmed.contains("path") {
            continue;
        }
        let Some((name, _)) = trimmed.split_once('=') else {
            continue;
        };
        out.push(name.trim().to_owned());
    }
    out
}

/// workspace が並べている crate を読む。
fn members(manifest: &Path) -> Vec<String> {
    let Ok(body) = files::read_to_string(manifest) else {
        return Vec::new();
    };
    let Some(head) = body.find("members") else {
        return Vec::new();
    };
    let rest = &body[head..];
    let Some(open) = rest.find('[') else {
        return Vec::new();
    };
    let Some(close) = rest[open..].find(']') else {
        return Vec::new();
    };
    rest[open + 1..open + close]
        .split(',')
        .map(|x| x.trim().trim_matches('"').to_owned())
        .filter(|x| !x.is_empty())
        .collect()
}

/// 節の構成が、対応する雛形を満たすかを見る。**文書が無ければ、そう返す。**
fn missing_sections(root: &Path, templates: &Templates) -> Vec<String> {
    let document = root.join("SKILL.md");
    let Ok(body) = files::read_to_string(&document) else {
        return vec!["文書が無い: SKILL.md".to_owned()];
    };
    let is_advisor = sections::headings(&body).iter().any(|x| x == ADVISOR_MARK);
    let template = if is_advisor {
        &templates.advisor
    } else {
        &templates.general
    };
    let Ok(want) = files::read_to_string(template) else {
        return Vec::new();
    };
    let name = template
        .file_name()
        .map_or_else(String::new, |x| x.to_string_lossy().into_owned());
    sections::missing(&body, &want)
        .into_iter()
        .map(|x| format!("節が無い: {x} ── {name} が要求する"))
        .collect()
}

/// 道具を持つ Skill かを返す。**助言と手順だけの Skill には、道具を要求しない。**
#[must_use]
pub fn has_tools(root: &Path) -> bool {
    files::is_file(root.join("tool.json"))
        || files::is_file(root.join("mcp.json"))
        || files::is_dir(root.join(TOOL))
        || files::is_dir(root.join("rs"))
        || files::is_dir(root.join("scripts"))
}

/// 検査の段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Stage {
    /// 実行ファイルを起動する検査。全言語で共通である。
    Behavior,
    /// ソースを読む検査。言語の組が持つ。
    Source,
    /// 文書の節の構成。
    Document,
}

/// 検査1件の状態。**「実行しない」を合格と同じにしない。**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum State {
    /// 満たしている。
    Pass,
    /// 満たしていない。
    Fail,
    /// 実行しない（言語の組が無い）。
    Skip,
}

/// 検査1件。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Line {
    /// どの段か。
    pub stage: Stage,
    /// 状態。
    pub state: State,
    /// 人が読む文。
    pub text: String,
}

impl Line {
    fn new(stage: Stage, state: State, text: String) -> Self {
        Self { stage, state, text }
    }
}

/// 検査の結果。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Report {
    /// 検査1件ずつ。段の順に並ぶ。
    pub lines: Vec<Line>,
}

impl Report {
    /// 満たしていないものの文。
    #[must_use]
    pub fn findings(&self) -> Vec<String> {
        self.lines
            .iter()
            .filter(|l| l.state == State::Fail)
            .map(|l| l.text.clone())
            .collect()
    }
}

/// 契約を満たしているかを、2段で検査する。
#[must_use]
pub fn check(root: &Path, templates: &Templates, layout: bool) -> Report {
    let mut report = Report::default();
    // **道具を持たない Skill に、実行ファイルも層も要求しない。** 助言と手順だけの Skill が在る
    if has_tools(root) {
        for v in behavior::run(root) {
            report.lines.push(match v {
                Verdict::Pass(t) => Line::new(Stage::Behavior, State::Pass, t),
                Verdict::Fail(t) => Line::new(Stage::Behavior, State::Fail, t),
            });
        }
        report.lines.extend(source(root, templates, layout));
    }
    let missing = document(root, templates);
    if missing.is_empty() {
        report.lines.push(Line::new(
            Stage::Document,
            State::Pass,
            "節の構成が雛形を満たす".to_owned(),
        ));
    }
    report.lines.extend(
        missing
            .into_iter()
            .map(|t| Line::new(Stage::Document, State::Fail, t)),
    );
    report
}

/// 2段目。**言語の組を、ソースの置き方から選ぶ。** 組が無ければ「実行しない」と返す。
///
/// **既定では契約だけを見る**（ACDR 0059）── 外部の道具の名前を直書きしていないこと。
/// 道具の中の構成（層 ・ 依存の向き ・ 入出力の置き場所）は Skill ごとの設計であり、契約ではない。
/// `layout` が真のときだけ、雛形の構成を満たすかも見る ── 雛形の構成を採ると決めた
/// リポジトリが、自分で選んで有効にする。
#[must_use]
pub fn source(root: &Path, templates: &Templates, layout: bool) -> Vec<Line> {
    if files::is_file(root.join(TOOL).join("Cargo.toml")) || files::is_dir(root.join("rs")) {
        let mut found = hardcoded(root);
        if layout {
            found.extend(rust(root));
            if behavior::contract_version(root) >= 2 {
                found.extend(rust_refs(root, templates));
            }
        }
        if found.is_empty() {
            let text = if layout {
                "Rust の組 ── 外部の道具の名前を直書きしていない。雛形の構成（層が crate に分かれ、依存の向きが直下の層だけで、入出力はデータアクセス層だけが持つ）も満たす"
            } else {
                "Rust の組 ── 外部の道具の名前を直書きしていない"
            };
            return vec![Line::new(Stage::Source, State::Pass, text.to_owned())];
        }
        return found
            .into_iter()
            .map(|t| Line::new(Stage::Source, State::Fail, t))
            .collect();
    }
    if layout && files::is_dir(root.join("scripts")) {
        return vec![Line::new(
            Stage::Source,
            State::Fail,
            format!("Python が残っている: scripts/ ── 道具は {TOOL}/ が持つ"),
        )];
    }
    vec![Line::new(
        Stage::Source,
        State::Skip,
        "実行しない ── この Skill の言語の組が無い（合格とは扱わない）".to_owned(),
    )]
}

/// Rust の組の、版2 の規則。**references の実装は雛形の複製である** ── Skill ごとに書き換えると、
/// どの Skill でも同じ get ・ validate ・ view ・ import になるという契約が崩れる。
#[must_use]
pub fn rust_refs(root: &Path, templates: &Templates) -> Vec<String> {
    let mut findings = Vec::new();
    for need in ["business_logic/src/refs.rs", "service/src/refs.rs"] {
        if !files::is_file(root.join(TOOL).join(need)) {
            findings.push(format!(
                "references の実装が無い: {TOOL}/{need} ── 契約の版2 は雛形の複製を置く"
            ));
        }
    }
    if let Some(tmpl) = &templates.refs {
        let want = files::read_to_string(tmpl).unwrap_or_default();
        let have = files::read_to_string(root.join(TOOL).join("business_logic/src/refs.rs"))
            .unwrap_or_default();
        if !have.is_empty() && !want.is_empty() && have != want {
            findings.push(format!(
                "references の実装が雛形と違う: {TOOL}/business_logic/src/refs.rs ── 雛形（refs.rs.tmpl）から複製し直す"
            ));
        }
    }
    findings
}

/// 節の構成が、対応する雛形を満たすかを見る。
#[must_use]
pub fn document(root: &Path, templates: &Templates) -> Vec<String> {
    missing_sections(root, templates)
}

/// Rust の組の2段目。**層 ・ 依存の向き ・ 事例 ・ bin/ の追跡 ・ 外部の道具の直書き**を見る。
#[must_use]
pub fn rust(root: &Path) -> Vec<String> {
    let mut findings = Vec::new();
    let rs = root.join(TOOL);
    if !files::is_file(rs.join("Cargo.toml")) {
        findings.push(format!(
            "層が crate に分かれていない: {TOOL}/Cargo.toml が無い ── \
             1つの単位の中の module では、内側が外側を参照してもコンパイラが通す"
        ));
    } else {
        let listed = members(&rs.join("Cargo.toml"));
        for need in ORDER.iter().copied().filter(|x| *x != "presentation") {
            if !listed.iter().any(|x| x == need) {
                findings.push(format!("層の crate が無い: {TOOL}/{need}"));
            }
        }
        if !ENTRIES.iter().any(|e| listed.iter().any(|x| x == e)) {
            findings.push(format!(
                "プレゼンテーション層の crate が無い: {TOOL}/ に {} のどれかを置く",
                ENTRIES.join(" か ")
            ));
        }
        // **依存の向きが契約どおりかを見る。** 守られているかはコンパイラが判定する
        for member in &listed {
            let manifest = rs.join(member).join("Cargo.toml");
            if !files::is_file(&manifest) {
                findings.push(format!("Cargo.toml が無い: {TOOL}/{member}/Cargo.toml"));
                continue;
            }
            let Some(here) = layer_of(member) else {
                findings.push(format!(
                    "どの層か決まらない: {TOOL}/{member} ── 名前を {} ・ {} ・ {} か {} で終える",
                    ORDER[0],
                    ORDER[1],
                    ORDER[2],
                    ENTRIES.join(" ・ ")
                ));
                continue;
            };
            let may = allowed(here);
            for dep in declared(&manifest) {
                let Some(to) = layer_of(&dep) else { continue };
                if !may.contains(&to) {
                    findings.push(format!(
                        "依存の向きに違反している: {TOOL}/{member}/Cargo.toml ── \
                         {here} → {to}（依存してよいのは直下の層だけである）"
                    ));
                }
            }
        }
        if !files::is_dir(rs.join(TESTS)) {
            findings.push(format!("事例が無い: {TOOL}/{TESTS}/"));
        }
    }
    // **Python を残さない。** 移行が済んでいない箇所を、黙って通さない
    if files::is_dir(root.join("scripts")) {
        findings.push(format!(
            "Python が残っている: scripts/ ── 道具は {TOOL}/ が持つ"
        ));
    }
    if files::is_dir(root.join("rs")) {
        findings.push(format!(
            "rs/ が残っている ── 道具のソースは {TOOL}/ に置く（名前は中身の役割で付ける）"
        ));
    }
    for (old, new) in RETIRED {
        if files::is_dir(rs.join(old)) {
            findings.push(format!(
                "{old}/ が残っている: {TOOL}/{old} ── 層の正式名 {TOOL}/{new} へ改める"
            ));
        }
    }
    let ignored = files::read_to_string(root.join(".gitignore")).unwrap_or_default();
    if !ignored
        .lines()
        .map(str::trim)
        .any(|l| l == "bin/" || l == "/bin/" || l == "bin")
    {
        findings.push(format!(
            "{BIN}/ を git の追跡から外していない: .gitignore に {BIN}/ を書く ── \
             組み立てた実行ファイルは追跡しない"
        ));
    }
    findings.extend(io_leaks(root));
    findings
}

/// 入出力を禁じた層が、入出力を直接扱っていないかを見る。**見つけた行ごとに出す**
/// ── どこを移せばよいかを、そのまま示すためである。注記の行（`//`）は見ない。
fn io_leaks(root: &Path) -> Vec<String> {
    let mut findings = Vec::new();
    for dir in NO_IO {
        let mut sources = Vec::new();
        rust_files(&root.join(TOOL).join(dir), &mut sources);
        for file in sources {
            let name = file
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if NO_IO_EXEMPT.contains(&name.as_str()) {
                continue;
            }
            let Ok(body) = files::read_to_string(&file) else {
                continue;
            };
            let at = file
                .strip_prefix(root)
                .unwrap_or(&file)
                .display()
                .to_string();
            for (i, line) in body.lines().enumerate() {
                if line.trim_start().starts_with("//") {
                    continue;
                }
                let code = without_strings(line);
                if let Some(mark) = IO_MARKS.iter().find(|m| has_mark(&code, m)) {
                    findings.push(format!(
                        "入出力を禁じた層が入出力を直接扱っている: {at}:{} ── `{mark}`（{TOOL}/data_access の files ・ process を通す）",
                        i + 1
                    ));
                }
            }
        }
    }
    findings
}

/// 外部の道具の名前を直書きしていないかを見る。**外部の道具は tool.json に宣言し、
/// サービス層から注入する**（ACDR 0036）── 直書きすると、利用者が差し替えられず、
/// 試験で偽物を渡せない。
fn hardcoded(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    rust_files(&root.join(TOOL), &mut files);
    let mut findings = Vec::new();
    for file in files {
        let Ok(body) = files::read_to_string(&file) else {
            continue;
        };
        let at = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .display()
            .to_string();
        for name in spawned(&body).into_iter().flatten() {
            findings.push(format!(
                "外部の道具の名前を直書きしている: {at} ── \"{name}\"（tool.json の external に宣言し、given.external で受け取って渡す）"
            ));
        }
    }
    findings
}

/// 印が、識別子の途中ではない位置に在るか。**`refs::` の中の `fs::` を数えない。**
fn has_mark(code: &str, mark: &str) -> bool {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let starts_word = mark.chars().next().is_some_and(word);
    code.match_indices(mark)
        .any(|(at, _)| !starts_word || !code[..at].chars().next_back().is_some_and(word))
}

/// 行から文字列の中身を除く。**文字列の中に書かれたものは、呼び出しではない** ── 検出の
/// 印そのものを定数に持つと、それを入出力として数えてしまう。
fn without_strings(line: &str) -> String {
    let mut out = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for c in line.chars() {
        if quoted {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                quoted = false;
                out.push(c);
            }
            continue;
        }
        if c == '"' {
            quoted = true;
        }
        out.push(c);
    }
    out
}

/// `Command::new` の呼び出しを拾う。**文字列で書かれていれば名前、そうでなければ None。**
/// 注記の行（`//` で始まる行）は見ない ── 説明の中の呼び出しの形を、呼び出しと数えない。
fn spawned(body: &str) -> Vec<Option<String>> {
    const CALL: &str = "Command::new(";
    let mut out = Vec::new();
    for line in body.lines().filter(|l| !l.trim_start().starts_with("//")) {
        let mut rest = line;
        while let Some(at) = rest.find(CALL) {
            // **文字列の中に書かれたものは数えない** ── 直前が引用符なら、呼び出しではない
            if rest[..at].ends_with('"') {
                rest = &rest[at + CALL.len()..];
                continue;
            }
            let after = rest[at + CALL.len()..].trim_start();
            if let Some(stripped) = after.strip_prefix('"') {
                out.push(Some(stripped.chars().take_while(|c| *c != '"').collect()));
            } else {
                out.push(None);
            }
            rest = &rest[at + CALL.len()..];
        }
    }
    out
}

/// Rust のファイルを集める。**組み立ての出力（target/）・ 事例（tests/）・ 開発用の例（examples/）は
/// 見ない** ── どれも配布する実行ファイルに入らない。
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = files::list(dir) else {
        return;
    };
    for path in entries {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if files::is_dir(&path) {
            if name != "target" && name != "tests" && name != "examples" {
                rust_files(&path, out);
            }
        } else if name.ends_with(".rs") {
            out.push(path);
        }
    }
}
