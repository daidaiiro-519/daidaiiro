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
use crate::profile::{self, Profile};
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
    /// 言語の組の定義の置き場所（`references/profiles/`）。**無ければ Rust の組だけを知る。**
    pub profiles: Option<PathBuf>,
    /// 見た目の正本の置き場所（`references/view/`）。**在れば、道具を持つ Skill の写しを突き合わせる。**
    pub view: Option<PathBuf>,
    /// テストケースの置き場所（`references/contract/cases/`）。**`--cases 1` のときだけ実行する。**
    pub cases: Option<PathBuf>,
}

impl Templates {
    /// 雛形の置き場所を組む。**欄を足しても、呼ぶ側は壊れない。**
    #[must_use]
    pub const fn new(general: PathBuf, advisor: PathBuf) -> Self {
        Self {
            general,
            advisor,
            refs: None,
            profiles: None,
            view: None,
            cases: None,
        }
    }

    /// 言語の組の定義の置き場所を足す。
    #[must_use]
    pub fn with_profiles(mut self, dir: PathBuf) -> Self {
        self.profiles = Some(dir);
        self
    }

    /// 見た目の正本の置き場所を足す。
    #[must_use]
    pub fn with_view(mut self, dir: PathBuf) -> Self {
        self.view = Some(dir);
        self
    }

    /// テストケースの置き場所を足す。
    #[must_use]
    pub fn with_cases(mut self, dir: PathBuf) -> Self {
        self.cases = Some(dir);
        self
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
    let mut found: Vec<String> = sections::missing(&body, &want)
        .into_iter()
        .map(|x| format!("節が無い: {x} ── {name} が要求する"))
        .collect();
    let left = placeholders(&body);
    if left > 0 {
        found.push(format!(
            "未記入の差し込み場所が在る: SKILL.md の {{{{…}}}} が {left} か所 ── 各 {{{{…}}}} の指示に従って記入する"
        ));
    }
    found
}

/// 未記入の差し込み場所（`{{…}}`）を数える。**コードの枠とコードの記法の中は数えない** ──
/// そこに在るのは、差し込み場所の書き方の説明である。
#[must_use]
pub fn placeholders(body: &str) -> usize {
    let mut count = 0;
    let mut fence = false;
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            fence = !fence;
            continue;
        }
        if fence {
            continue;
        }
        // コードの記法（`…`）の中を外す
        let plain: String = line
            .split('`')
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, x)| x)
            .collect();
        let mut rest = plain.as_str();
        while let Some(open) = rest.find("{{") {
            let Some(close) = rest[open..].find("}}") else {
                break;
            };
            count += 1;
            rest = &rest[open + close + 2..];
        }
    }
    count
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
    /// テストケース（ボード skills-creator-contract の論点2）。**渡したときだけ実行する。**
    Cases,
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
        if let Some(canon) = &templates.view {
            let found = crate::view::findings(root, canon);
            if found.is_empty() {
                report.lines.push(Line::new(
                    Stage::Source,
                    State::Pass,
                    "見た目の写しが正本と一致し、色の直値と定まらない変数が無く、文字と地の比が足りる".to_owned(),
                ));
            }
            report.lines.extend(
                found
                    .into_iter()
                    .map(|t| Line::new(Stage::Source, State::Fail, t)),
            );
        }
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

/// テストケースを実行する段。**Skill の tool.json の実行コマンドで呼ぶ** ── どの言語で書いた Skill にも、
/// 同じテストケースを実行できる。1件ごとに合格か不合格を返し、読めなければ1件の不合格にする。
#[must_use]
pub fn cases(root: &Path, dir: &Path) -> Vec<Line> {
    match crate::cases::run(root, dir) {
        Err(e) => vec![Line::new(
            Stage::Cases,
            State::Fail,
            format!("テストケースを実行できない ── {e}"),
        )],
        Ok(results) => results
            .into_iter()
            .map(|(name, why)| match why {
                None => Line::new(Stage::Cases, State::Pass, format!("テストケース {name}")),
                Some(w) => Line::new(
                    Stage::Cases,
                    State::Fail,
                    format!("テストケース {name} ── {w}"),
                ),
            })
            .collect(),
    }
}

/// 2段目。**言語の組を、ソースの置き方から選ぶ。** 組が無ければ「実行しない」と返す。
///
/// **既定では契約だけを見る**（ACDR 0059）── 外部の道具の名前を直書きしていないこと。
/// 道具の中の構成（層 ・ 依存の向き ・ 入出力の置き場所）は Skill ごとの設計であり、契約ではない。
/// `layout` が真のときだけ、雛形の構成を満たすかも見る ── 雛形の構成を採ると決めた
/// リポジトリが、自分で選んで有効にする。
#[must_use]
pub fn source(root: &Path, templates: &Templates, layout: bool) -> Vec<Line> {
    let found_profile = match &templates.profiles {
        Some(dir) => profile::of(root, dir),
        None => builtin_rust(root),
    };
    let Some(p) = found_profile else {
        if layout && files::is_dir(root.join("scripts")) {
            return vec![Line::new(
                Stage::Source,
                State::Fail,
                format!("Python が残っている: scripts/ ── 道具は {TOOL}/ が持つ"),
            )];
        }
        return vec![Line::new(
            Stage::Source,
            State::Skip,
            "実行しない ── この Skill の言語の組が無い（合格とは扱わない）".to_owned(),
        )];
    };
    let mut found = hardcoded(root, &p);
    let mut lines = Vec::new();
    if layout {
        if p.layout {
            found.extend(rust(root));
            if behavior::contract_version(root) >= 2 {
                found.extend(rust_refs(root, templates));
            }
        } else {
            lines.push(Line::new(
                Stage::Source,
                State::Skip,
                format!(
                    "実行しない ── {} の組は雛形の構成の検査を持たない（合格とは扱わない）",
                    p.name
                ),
            ));
        }
    }
    if found.is_empty() {
        let text = if layout && p.layout {
            format!("{} の組 ── 外部の道具の名前を直書きしていない。雛形の構成（層が crate に分かれ、依存の向きが直下の層だけで、入出力はデータアクセス層だけが持つ）も満たす", p.name)
        } else {
            format!("{} の組 ── 外部の道具の名前を直書きしていない", p.name)
        };
        lines.insert(0, Line::new(Stage::Source, State::Pass, text));
        return lines;
    }
    lines.extend(
        found
            .into_iter()
            .map(|t| Line::new(Stage::Source, State::Fail, t)),
    );
    lines
}

/// 組の定義を渡されなかったときの Rust の組。**`tool/Cargo.toml` か `rs/` が在れば Rust とみなす。**
fn builtin_rust(root: &Path) -> Option<Profile> {
    (files::is_file(root.join(TOOL).join("Cargo.toml")) || files::is_dir(root.join("rs"))).then(
        || Profile {
            name: "rust".to_owned(),
            detect: format!("{TOOL}/Cargo.toml"),
            contract: 2,
            common: Vec::new(),
            types: std::collections::BTreeMap::new(),
            build: Vec::new(),
            test: Vec::new(),
            format: Vec::new(),
            extensions: vec![".rs".to_owned()],
            skip: vec![
                "target".to_owned(),
                "tests".to_owned(),
                "examples".to_owned(),
            ],
            spawn: vec!["Command::new(".to_owned()],
            dist: true,
            layout: true,
            fixtures: format!("{TOOL}/business_logic/tests/fixtures"),
        },
    )
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
    let mut found = missing_sections(root, templates);
    if files::is_file(root.join("tool.json")) {
        found.extend(markdown_in_references(root));
    }
    found.extend(missing_targets(root));
    found.extend(placeholders_in_references(root));
    found
}

/// references の直下の JSON に残った差し込み場所。**スキーマの題と説明は、描画した頁の見出しになる** ──
/// SKILL.md だけを見ると、雛形の `{{…}}` が頁に出たまま検査を通る（実測 2026-10-02、qa-advisor）。
/// 雛形が差し込み場所を置くのはスキーマ（`*.schema.json`）だけなので、それだけを見る ── ほかの JSON
/// には、差し込み場所の書き方を説明する文が在る（skills-creator の document.json）。下位のフォルダ
/// （学習ノートの置き場など）も見ない。
fn placeholders_in_references(root: &Path) -> Vec<String> {
    let mut found: Vec<String> = files::list(root.join("references"))
        .unwrap_or_default()
        .into_iter()
        .filter(|p| {
            files::is_file(p)
                && p
                    .file_name()
                    .is_some_and(|x| x.to_string_lossy().ends_with(".schema.json"))
        })
        .filter_map(|p| {
            let left = placeholders(&files::read_to_string(&p).ok()?);
            (left > 0).then(|| {
                let rel = p.strip_prefix(root).unwrap_or(&p);
                format!(
                    "未記入の差し込み場所が在る: {} の {{{{…}}}} が {left} か所 ── 各 {{{{…}}}} の指示に従って記入する",
                    rel.display()
                )
            })
        })
        .collect();
    found.sort();
    found
}

/// テストケースの入力データの置き場所。**import が取り込む Markdown を入力として持つ** ── 中身は
/// references の内容ではなく、テストケースの入力である。
const CASE_FIXTURES: &str = "references/contract/cases/fixtures";

/// references に置いた Markdown。**道具を持つ Skill は、Markdown を SKILL.md だけにする**（契約の版2）──
/// references に置くと、スキーマで検査できず、`view` でも描画できない。テストケースの入力データは除く。
fn markdown_in_references(root: &Path) -> Vec<String> {
    let ignored = ignored_prefixes(root);
    let fixtures = root.join(CASE_FIXTURES);
    let is_ignored = |path: &Path| {
        path.starts_with(&fixtures)
            || ignored.iter().any(|(prefix, name)| match name {
                Some(name) => path
                    .strip_prefix(root)
                    .is_ok_and(|rel| rel.components().any(|c| c.as_os_str() == name.as_str())),
                None => path.starts_with(prefix),
            })
    };
    let mut found = Vec::new();
    let mut stack = vec![root.join("references")];
    while let Some(dir) = stack.pop() {
        for path in files::list(&dir).unwrap_or_default() {
            if is_ignored(&path) {
                continue;
            }
            if files::is_dir(&path) {
                stack.push(path);
            } else if path.extension().is_some_and(|x| x == "md") {
                let rel = path.strip_prefix(root).unwrap_or(&path);
                found.push(format!(
                    "references に Markdown が在る: {} ── 契約の版2 は JSON Schema と JSON で持つ（Markdown は SKILL.md だけ）",
                    rel.display()
                ));
            }
        }
    }
    found.sort();
    found
}

/// git が追跡しない場所。**配らないものは契約の外である** ── 手元だけに置く覚え書きを検出しない。
/// Skill のフォルダから上へ辿り、各 `.gitignore` の行のうち文字どおりの経路だけを読む
/// （`*` ・ `?` ・ `[` ・ `!` を含む行は読まない）。`/` を途中に含む行はその `.gitignore` の場所からの経路、
/// 含まない行はどの階層でも一致する名前として扱う。
fn ignored_prefixes(root: &Path) -> Vec<(PathBuf, Option<String>)> {
    let mut out = Vec::new();
    let start = files::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    for dir in start.ancestors() {
        let Ok(body) = files::read_to_string(dir.join(".gitignore")) else {
            continue;
        };
        for line in body.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') || line.contains(['*', '?', '[', '!']) {
                continue;
            }
            let bare = line.trim_start_matches('/').trim_end_matches('/');
            if bare.contains('/') || line.starts_with('/') {
                let full = dir.join(bare);
                let rel = full
                    .strip_prefix(&start)
                    .map(|r| root.join(r))
                    .unwrap_or(full);
                out.push((rel, None));
            } else {
                out.push((PathBuf::new(), Some(bare.to_owned())));
            }
        }
    }
    out
}

/// SKILL.md がコードの記法で指す、Skill の中のファイル。**指す先が無ければ検出する** ──
/// 改名や移動の後に旧い名前が残ると、読み手はその場所を開けない。
/// 書き方の説明（`<…>` ・ `{…}` ・ `*` を含むもの）は、ファイルを指していないので見ない。
fn missing_targets(root: &Path) -> Vec<String> {
    let Ok(body) = files::read_to_string(root.join("SKILL.md")) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for (i, part) in body.split('`').enumerate() {
        if i % 2 == 0 {
            continue;
        }
        let path = part.trim().trim_end_matches(':');
        let inside = [
            "tool/",
            "references/",
            "infra/",
            "examples/",
            "assets/",
            "agents/",
        ]
        .iter()
        .any(|p| path.starts_with(p));
        let pattern = path.contains(['<', '>', '{', '}', '*', ' ', '…']);
        if !inside || pattern {
            continue;
        }
        let text = format!("SKILL.md が指す先が無い: {path} ── 実在する経路へ直すか、記述を消す");
        if !files::exists(root.join(path)) && !found.contains(&text) {
            found.push(text);
        }
    }
    found
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
/// そこから読んで渡す**（ACDR 0036）── 直書きすると、利用者が差し替えられず、
/// 試験で偽物を渡せない。**起動の書き方は組の定義が持つ**（ACDR 0060）。
fn hardcoded(root: &Path, p: &Profile) -> Vec<String> {
    let mut sources = Vec::new();
    source_files(&root.join(TOOL), p, &mut sources);
    let mut findings = Vec::new();
    for file in sources {
        let Ok(body) = files::read_to_string(&file) else {
            continue;
        };
        let at = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .display()
            .to_string();
        for name in spawned(&body, &p.spawn).into_iter().flatten() {
            findings.push(format!(
                "外部の道具の名前を直書きしている: {at} ── \"{name}\"（tool.json の external に宣言し、読んで渡す）"
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

/// 外部の道具を起動する呼び出しを拾う。**直後が文字列なら名前、そうでなければ None。**
/// 配列で渡す書き方（`["git", …]`）も、先頭が文字列なら名前とみなす。注記の行（`//` と `#`
/// で始まる行）は見ない ── 説明の中の呼び出しの形を、呼び出しと数えない。
///
/// **呼び出しの書き方に `…` を含めると、そこまでの引数を飛ばす**（例：`exec.CommandContext(…,`）──
/// 名前が2つ目の引数に在る呼び出しがある。**識別子の途中の一致は数えない** ── `respawn(` の中の
/// `spawn(` は、別の関数である。
#[must_use]
pub fn spawned(body: &str, calls: &[String]) -> Vec<Option<String>> {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let mut out = Vec::new();
    for line in body.lines().filter(|l| {
        let t = l.trim_start();
        !t.starts_with("//") && !t.starts_with('#')
    }) {
        for call in calls {
            let (head, skip) = call.split_once('…').unwrap_or((call.as_str(), ""));
            let starts_word = head.chars().next().is_some_and(word);
            let mut rest = line;
            while let Some(at) = rest.find(head) {
                let before = &rest[..at];
                let tail = &rest[at + head.len()..];
                rest = tail;
                // **文字列の中に書かれたものは数えない** ── 直前が引用符なら、呼び出しではない
                if before.ends_with('"') || before.ends_with('\'') {
                    continue;
                }
                if starts_word && before.chars().next_back().is_some_and(word) {
                    continue;
                }
                let after = if skip.is_empty() {
                    tail
                } else {
                    match tail.find(skip) {
                        Some(k) => &tail[k + skip.len()..],
                        None => continue,
                    }
                };
                let after = after.trim_start();
                let after = after.strip_prefix('[').map_or(after, str::trim_start);
                let quote = after
                    .chars()
                    .next()
                    .filter(|c| *c == '"' || *c == '\'' || *c == '`');
                out.push(quote.map(|q| after[1..].chars().take_while(|c| *c != q).collect()));
            }
        }
    }
    out
}

/// 組のソースのファイルを集める。**組が入らないと決めたフォルダ（組み立ての出力 ・ 事例 ・
/// 開発用の例など）は見ない** ── どれも配布する道具に入らない。
fn source_files(dir: &Path, p: &Profile, out: &mut Vec<PathBuf>) {
    let Ok(entries) = files::list(dir) else {
        return;
    };
    for path in entries {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if files::is_dir(&path) {
            if !p.skip.contains(&name) {
                source_files(&path, p, out);
            }
        } else if p.extensions.iter().any(|e| name.ends_with(e.as_str())) {
            out.push(path);
        }
    }
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
