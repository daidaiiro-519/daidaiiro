// SPDX-License-Identifier: MIT
//! Skill が契約を満たしているかを検査する。**見つけるが、直さない。**
//!
//! 検査するのは3つである ── 層が crate に分かれていること、**許可辺が宣言どおりで
//! あること**、**節の構成が対応する雛形を満たすこと**。
//!
//! **層を crate に分ける。** 1つの crate の中の module では、内側が外側を参照しても
//! コンパイラが通す（実測 2026-09-26）── 層の境界を crate の境界に置いて初めて、
//! 宣言に無い依存が解決しなくなる。
//!
//! **許可辺は各 `Cargo.toml` が宣言する。** この検査は宣言を読み、並びと食い違って
//! いないかを見る ── 実際に守られているかはコンパイラが判定する。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::sections;

/// Skill の道具のソースを置く場所。**配布しない。** 名前は中身の役割（道具）で付ける ──
/// 以前の rs/ は言語の名前で、役割を示さなかった。
pub const TOOL: &str = "tool";

/// 組み立てた実行ファイルを置く場所。**配布物だけが持ち、git で追跡しない。**
pub const BIN: &str = "bin";

/// 名前を実行時に決める外部の道具を、宣言（EXTERNAL）で表す印。
pub const USER_CHOSEN: &str = "*";

/// OS によって無いコマンド。**呼ばない** ── date は Windows に実行ファイルとして無く、
/// timeout は macOS の標準に無い（ACDR 0029）。日付の計算と時間の制限は Rust の中で行う。
pub const NON_PORTABLE: [&str; 2] = ["date", "timeout"];

/// 層の並び。**内から外である** ── 先頭が最も内側である。
pub const ORDER: [&str; 3] = ["parts", "declare", "entry"];

/// 入口の層に属する crate。**合成する側なので複数在ってよい。**
pub const ENTRIES: [&str; 2] = ["cli", "mcp"];

/// 事例を置く場所（`tool/` からの相対）。
pub const TESTS: &str = "parts/tests";

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
}

impl Templates {
    /// 雛形の置き場所を組む。**欄を足しても、呼ぶ側は壊れない。**
    #[must_use]
    pub const fn new(general: PathBuf, advisor: PathBuf) -> Self {
        Self { general, advisor }
    }
}

/// その層が参照してよい層を返す。**並びから導く** ── 表を別に持つと、並びとずれる。
#[must_use]
fn allowed(layer: &str) -> Vec<&'static str> {
    let at = ORDER.iter().position(|x| *x == layer);
    match at {
        Some(i) => ORDER[..i].to_vec(),
        None => Vec::new(),
    }
}

/// crate の名前から、属する層を決める。
#[must_use]
fn layer_of(name: &str) -> Option<&'static str> {
    let tail = name.rsplit('_').next().unwrap_or(name);
    if ENTRIES.contains(&tail) {
        return Some("entry");
    }
    ORDER.iter().copied().find(|x| *x == tail)
}

/// `Cargo.toml` が宣言している、同じ workspace の中の依存を読む。
///
/// **`path = "../…"` の形だけを見る** ── 外の crate は層の外である。
fn declared(manifest: &Path) -> Vec<String> {
    let Ok(body) = fs::read_to_string(manifest) else {
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
    let Ok(body) = fs::read_to_string(manifest) else {
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
    let Ok(body) = fs::read_to_string(&document) else {
        return vec!["文書が無い: SKILL.md".to_owned()];
    };
    let is_advisor = sections::headings(&body).iter().any(|x| x == ADVISOR_MARK);
    let template = if is_advisor {
        &templates.advisor
    } else {
        &templates.general
    };
    let Ok(want) = fs::read_to_string(template) else {
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
    root.join(TOOL).join("Cargo.toml").is_file()
        || root.join("rs").is_dir()
        || root.join("scripts").is_dir()
}

/// 契約を満たしているかを検査し、食い違いを並べる。
///
/// # Errors
///
/// いまは返さない。呼ぶ側の形を変えずに、あとから検査を足せるようにしてある。
pub fn check(root: &Path, templates: &Templates) -> io::Result<Vec<String>> {
    let mut findings = Vec::new();
    let rs = root.join(TOOL);

    // **道具を持たない Skill に、層を要求しない。** 助言と手順だけの Skill が在る
    if has_tools(root) {
        if !rs.join("Cargo.toml").is_file() {
            findings.push(format!(
                "層が crate に分かれていない: {TOOL}/Cargo.toml が無い ── \
                 1つの単位の中の module では、内側が外側を参照してもコンパイラが通す"
            ));
        } else {
            let listed = members(&rs.join("Cargo.toml"));
            for need in ORDER.iter().copied().filter(|x| *x != "entry") {
                if !listed.iter().any(|x| x == need) {
                    findings.push(format!("層の crate が無い: {TOOL}/{need}"));
                }
            }
            if !ENTRIES.iter().any(|e| listed.iter().any(|x| x == e)) {
                findings.push(format!(
                    "入口の crate が無い: {TOOL}/ に {} のどれかを置く",
                    ENTRIES.join(" か ")
                ));
            }
            // **許可辺が宣言どおりかを見る。** 守られているかはコンパイラが判定する
            for member in &listed {
                let manifest = rs.join(member).join("Cargo.toml");
                if !manifest.is_file() {
                    findings.push(format!("宣言が無い: {TOOL}/{member}/Cargo.toml"));
                    continue;
                }
                let Some(here) = layer_of(member) else {
                    findings.push(format!(
                        "どの層か決まらない: {TOOL}/{member} ── 名前を {} か {} で終える",
                        ORDER.join(" ・ "),
                        ENTRIES.join(" ・ ")
                    ));
                    continue;
                };
                let may = allowed(here);
                for dep in declared(&manifest) {
                    let Some(to) = layer_of(&dep) else { continue };
                    if !may.contains(&to) {
                        findings.push(format!(
                            "許可していない辺を宣言している: {TOOL}/{member}/Cargo.toml ── \
                             {here} → {to}"
                        ));
                    }
                }
            }
            if !rs.join(TESTS).is_dir() {
                findings.push(format!("事例が無い: {TOOL}/{TESTS}/"));
            }
        }
        // **Python を残さない。** 移行が済んでいない箇所を、黙って通さない
        if root.join("scripts").is_dir() {
            findings.push(format!(
                "Python が残っている: scripts/ ── 道具は {TOOL}/ が持つ"
            ));
        }
        if root.join("rs").is_dir() {
            findings.push(format!(
                "rs/ が残っている ── 道具のソースは {TOOL}/ に置く（名前は中身の役割で付ける）"
            ));
        }
        findings.extend(distribution(root));
    }

    findings.extend(missing_sections(root, templates));
    Ok(findings)
}

/// 配布の形を見る。**実行ファイルは配布物だけが持ち、登録は置き場所に依存しない。**
fn distribution(root: &Path) -> Vec<String> {
    let mut findings = Vec::new();
    let ignored = fs::read_to_string(root.join(".gitignore")).unwrap_or_default();
    if !ignored
        .lines()
        .map(str::trim)
        .any(|l| l == "bin/" || l == "/bin/" || l == "bin")
    {
        findings.push(format!(
            "{BIN}/ を git の追跡から外していない: .gitignore に {BIN}/ を書く ── \
             実行ファイルは配布物だけが持つ"
        ));
    }
    if let Ok(body) = fs::read_to_string(root.join("mcp.json")) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) {
            let servers = value
                .get("mcpServers")
                .and_then(serde_json::Value::as_object)
                .cloned()
                .unwrap_or_default();
            for (name, server) in servers {
                let command = server
                    .get("command")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                if is_absolute(command) {
                    findings.push(format!(
                        "登録に絶対パスがある: mcp.json の {name} ── {command}（配布先では存在しない場所を指す）"
                    ));
                }
            }
        }
    }
    findings.extend(external_tools(root));
    findings
}

/// 絶対パスか。**Windows の書き方も含める** ── `C:\\…` ・ `C:/…`
fn is_absolute(command: &str) -> bool {
    let bytes = command.as_bytes();
    command.starts_with('/')
        || command.starts_with('~')
        || (bytes.len() > 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
}

/// 部品が呼ぶ外部の道具を、宣言と照合する。**外部コマンドは例外である** ── 呼んでよいのは、
/// Skill の目的に不可欠な道具だけで、declare の `EXTERNAL` に名前と理由を書く（ACDR 0034）。
fn external_tools(root: &Path) -> Vec<String> {
    let tool = root.join(TOOL);
    let declared = requires(&tool.join("declare/src/lib.rs"));
    let mut findings: Vec<String> = declared
        .iter()
        .filter(|(_, why)| why.trim().is_empty())
        .map(|(name, _)| {
            format!("外部の道具に理由が無い: {name} ── 目的に不可欠である理由を EXTERNAL に書く")
        })
        .collect();
    let named = |n: &str| declared.iter().any(|(d, _)| d == n);
    let mut files = Vec::new();
    rust_files(&tool, &mut files);
    for file in files {
        let Ok(body) = fs::read_to_string(&file) else {
            continue;
        };
        let at = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .display()
            .to_string();
        for called in spawned(&body) {
            match called {
                Some(name) => {
                    if NON_PORTABLE.contains(&name.as_str()) {
                        findings.push(format!(
                            "OS によって無いコマンドを呼んでいる: {at} ── {name}（Rust の中で行う）"
                        ));
                    } else if !named(&name) {
                        findings.push(format!(
                            "宣言に無い外部の道具を呼んでいる: {at} ── {name}（外部コマンドは例外である。目的に不可欠なら EXTERNAL に名前と理由を書き、そうでなければ Rust の中で行う）"
                        ));
                    }
                }
                None => {
                    if !named(USER_CHOSEN) {
                        findings.push(format!(
                            "名前を実行時に決める道具を呼んでいる: {at} ── 利用者が指定する道具（\"{USER_CHOSEN}\"）を、理由と一緒に EXTERNAL に書く"
                        ));
                    }
                }
            }
        }
    }
    findings
}

/// 宣言（declare の `EXTERNAL`）に並ぶ、名前と理由の組。**無ければ空である。**
fn requires(lib: &Path) -> Vec<(String, String)> {
    let body = fs::read_to_string(lib).unwrap_or_default();
    let Some(at) = body.find("EXTERNAL") else {
        return Vec::new();
    };
    let rest = &body[at..];
    // **型の `&[(&str, &str)]` ではなく、値の並び（`= &[`）から読む**
    let Some(open) = rest.find("= &[").map(|at| at + 4) else {
        return Vec::new();
    };
    let Some(close) = rest[open..].find("];") else {
        return Vec::new();
    };
    // 引用符で囲まれた文字列を順に拾い、2つずつ組にする
    let quoted: Vec<String> = rest[open..open + close]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect();
    quoted
        .chunks(2)
        .map(|pair| {
            (
                pair.first().cloned().unwrap_or_default(),
                pair.get(1).cloned().unwrap_or_default(),
            )
        })
        .collect()
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
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if name != "target" && name != "tests" && name != "examples" {
                rust_files(&path, out);
            }
        } else if name.ends_with(".rs") {
            out.push(path);
        }
    }
}
