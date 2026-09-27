// SPDX-License-Identifier: MIT
//! JSON を正本として、ブレストボードを組む。
//!
//! **JSON が正本で、HTML は生成物である。** 具体の側は実装を1行も保持しない。
//!
//! **この道具は入力を書き換えない。** 前の回の基準は `rounds/<番号>.json` から読むだけで、
//! 書き出さない ── 書き出すと、1回目と2回目で出力が相違する（実測 ── 2,209 行相違した）。
//! 基準の前進は `freeze` が担当する。
//!
//! **節の構造は、生成の時点で確定させる。** 閲覧する側の script で組み直すと、保存した
//! HTML の中に構造が存在しない。

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::blocks::{build, prepare};
use crate::deck::{self, Deck};
use crate::panel::Labels;
use crate::snapshot::snapshot;
use crate::style::{drop_numbering, Style};
use crate::template::Parts;
use crate::topic::LETTERS;

/// ブレストボードの置き場所の親。**init と同じ既定である。**
pub const BOARDS: &str = ".brainstorming-board";

/// 記号の並び。**除外した案の記号は、通過した案の次から振る。**
const ALPHABET: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

fn text_of(value: &Value, key: &str) -> String {
    match value.get(key) {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

fn array_of<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(|x| x.as_array())
        .map_or(&[], |x| x.as_slice())
}

/// 名前でも道でも、同じ1つのフォルダへ解決する。
///
/// **解決を1か所に置く** ── 道具ごとに違う解決をしていたので、名前で渡すと組み立てだけが
/// 黙って何もしないことがあった（実測 ── 実際にそうなった）。
///
/// # Errors
///
/// どちらにも `board.json` が無いときに返す。
pub fn board_dir(board: &str) -> Result<PathBuf, String> {
    let given = PathBuf::from(board);
    if given.join("board.json").exists() {
        return std::fs::canonicalize(&given).map_err(|e| format!("{board}: 開けない ── {e}"));
    }
    let alt = PathBuf::from(BOARDS).join(board);
    if alt.join("board.json").exists() {
        return std::fs::canonicalize(&alt).map_err(|e| format!("{board}: 開けない ── {e}"));
    }
    Err(format!(
        "board.json が無い: {board} ── {} にも {} にも見つからない",
        given.display(),
        alt.display()
    ))
}

/// 入力を読む。
///
/// # Errors
///
/// 読めないときと、JSON として読めないときに返す。
pub fn load(dir: &Path) -> Result<Value, String> {
    let path = dir.join("board.json");
    let body = std::fs::read_to_string(&path)
        .map_err(|e| format!("{}: 読めない ── {e}", path.display()))?;
    serde_json::from_str(&body)
        .map_err(|e| format!("{}: JSON として読めない ── {e}", path.display()))
}

/// 除外した案の記号が始まる位置を決める。
fn origin(kept: &[String]) -> usize {
    let highest = kept
        .iter()
        .filter_map(|c| ALPHABET.iter().position(|a| a == c))
        .max();
    let from = highest.map_or(0, |x| x + 1);
    ALPHABET
        .iter()
        .skip(from)
        .position(|c| !kept.contains(&(*c).to_owned()))
        .map_or(0, |x| from + x)
}

/// ブレストボードを1枚へ組む。
///
/// # Errors
///
/// 入力の検査が通らないときと、型と噛み合わないときに返す。
pub fn render(references: &Path, dir: &Path, verify: bool) -> Result<deck::Made, String> {
    if verify {
        let bad = crate::validate::check(references, dir)?;
        if !bad.is_empty() {
            return Err(format!(
                "入力の検査が通っていない ── HTML は書き出さない:\n  {}",
                bad.iter()
                    .map(|e| format!("× {e}"))
                    .collect::<Vec<_>>()
                    .join("\n  ")
            ));
        }
    }
    let d = load(dir)?;
    let parts = Parts::load(&references.join("board.template.html"))?;
    let figures = dir.join("figures");
    let topics = prepare(&parts, &d, &figures)?;

    let mut origins = Vec::new();
    for t in &topics {
        let used: Vec<String> = t.kept.iter().map(|o| o.name.clone()).collect();
        origins.push((t.no, origin(&used)));
    }

    // **トークンはここで置かない** ── 定義は page() が1回だけ置く。2か所から出すと、
    // どちらが勝つかを文書の順序に委ねることになる
    let style = Style::load(references)?;
    let round = d.get("round").and_then(Value::as_u64).unwrap_or(1) as usize;
    let mut prev = None;
    if round > 0 {
        let before = dir.join("rounds").join(format!("{}.json", round - 1));
        if before.exists() {
            let body = std::fs::read_to_string(&before)
                .map_err(|e| format!("{}: 読めない ── {e}", before.display()))?;
            let value: Value = serde_json::from_str(&body)
                .map_err(|e| format!("{}: JSON として読めない ── {e}", before.display()))?;
            prev = value.get("snap").cloned();
        }
    }
    let labels = match d.get("section_names") {
        Some(names) if !names.is_null() => Labels {
            history: text_of(names, "history"),
            progress: text_of(names, "progress"),
        },
        _ => Labels::default(),
    };
    let wanted = Deck {
        theme: text_of(&d, "title"),
        board: text_of(&d, "board"),
        round,
        intro: build(&parts, array_of(&d, "intro"), &figures)?,
        queue: array_of(&d, "queue")
            .iter()
            .map(|q| {
                (
                    q.get("no").and_then(Value::as_u64).unwrap_or(0) as usize,
                    text_of(q, "why"),
                )
            })
            .collect(),
        extras: {
            let mut out = Vec::new();
            for e in array_of(&d, "panels") {
                out.push((
                    text_of(e, "heading"),
                    build(&parts, array_of(e, "body"), &figures)?,
                ));
            }
            out
        },
        prev,
        // 見た目は page() が正本から置く。ここで置くのは、入力の中身から計算した値だけである
        style: format!("<style>{}</style>", drop_numbering(&origins)),
        labels,
        js: style.js.clone(),
    };
    deck::build(&parts, &topics, &wanted)
}

/// 組んだものを、そのまま公開できる形で書き出す。
///
/// # Errors
///
/// 読めないときと、書けないときに返す。
pub fn write(references: &Path, dir: &Path, body: &str) -> Result<usize, String> {
    let d = load(dir)?;
    let parts = Parts::load(&references.join("board.template.html"))?;
    let style = Style::load(references)?;
    let page = deck::page(&parts, &text_of(&d, "title"), &style.css, body)?;
    let path = dir.join("board.html");
    std::fs::write(&path, &page).map_err(|e| format!("{}: 書けない ── {e}", path.display()))?;
    Ok(body.len())
}

/// いまの入力を、次の回の基準として保存する。
///
/// **組み立てと分離する** ── 組み立てが基準を書き出すと、1回目と2回目で出力が相違する。
///
/// # Errors
///
/// 読めないときと、書けないときに返す。
pub fn freeze(references: &Path, dir: &Path) -> Result<String, String> {
    let d = load(dir)?;
    let parts = Parts::load(&references.join("board.template.html"))?;
    let topics = prepare(&parts, &d, &dir.join("figures"))?;
    let round = d.get("round").and_then(Value::as_u64).unwrap_or(1) as usize;
    let out = dir.join("rounds").join(format!("{round}.json"));
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: 作れない ── {e}", parent.display()))?;
    }
    let body = serde_json::json!({ "round": round, "snap": snapshot(&topics) });
    // **末尾に改行を足さない** ── 機械が読む記録であり、移す前と1バイトも変えない
    std::fs::write(&out, flat_json(&body))
        .map_err(|e| format!("{}: 書けない ── {e}", out.display()))?;
    Ok(format!(
        "基準を保存: {} ── 次は board.json の「回」を {} へ進める",
        out.strip_prefix(dir).unwrap_or(&out).display(),
        round + 1
    ))
}

/// 平らな JSON を、1字下げで組む。
fn flat_json(value: &Value) -> String {
    let mut out = Vec::new();
    let mut writer = serde_json::Serializer::with_formatter(
        &mut out,
        serde_json::ser::PrettyFormatter::with_indent(b" "),
    );
    use serde::Serialize as _;
    if value.serialize(&mut writer).is_err() {
        return String::new();
    }
    String::from_utf8(out).unwrap_or_default()
}

/// 冪等の検査の結果。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Same {
    /// 見つけたこと。
    pub findings: Vec<String>,
    /// 人が読む1行。
    pub message: String,
}

fn sha256_of(data: &[u8]) -> String {
    use sha2::{Digest as _, Sha256};
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// 入力のファイルの姿を集める。**生成物は入力ではない。**
fn inputs(dir: &Path) -> Vec<(PathBuf, String)> {
    fn walk(at: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(at) else {
            return;
        };
        let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            if p.is_dir() {
                walk(&p, out);
            } else if p.file_name().is_some_and(|n| n != "board.html") {
                out.push(p);
            }
        }
    }
    let mut paths = Vec::new();
    walk(dir, &mut paths);
    paths
        .into_iter()
        .map(|p| {
            let digest = std::fs::read(&p).map(|b| sha256_of(&b)).unwrap_or_default();
            (p, digest)
        })
        .collect()
}

/// **2回の一致では不足する。** 入力が変化しないことも、あわせて検査する。
///
/// # Errors
///
/// 組めないときに返す。
pub fn check_idempotent(references: &Path, dir: &Path, first: &str) -> Result<Same, String> {
    let before = inputs(dir);
    let second = render(references, dir, true)?;
    let after = inputs(dir);
    let mut bad = Vec::new();
    if first != second.body {
        bad.push(format!(
            "2回の生成物が相違する（{} と {} バイト）",
            first.len(),
            second.body.len()
        ));
    }
    let mut names: Vec<&PathBuf> = before.iter().map(|(p, _)| p).collect();
    for (p, _) in &after {
        if !names.contains(&p) {
            names.push(p);
        }
    }
    names.sort();
    for p in names {
        let was = before.iter().find(|(q, _)| q == p).map(|(_, d)| d);
        let now = after.iter().find(|(q, _)| q == p).map(|(_, d)| d);
        if was != now {
            bad.push(format!(
                "入力が書き換わった: {}",
                p.strip_prefix(dir).unwrap_or(p).display()
            ));
        }
    }
    // **読み取り専用でも通ることを、あわせて検査する** ── 同じ中身を書き直す実装は、
    // 要約の比較では素通りする。書けない場所で走らせて初めて判明する
    if let Err(why) = read_only_run(references, dir) {
        bad.push(why);
    }
    let message = if bad.is_empty() {
        "冪等の検査　通った".to_owned()
    } else {
        format!("冪等の検査　通っていない（{} 件）", bad.len())
    };
    Ok(Same {
        findings: bad,
        message,
    })
}

/// 案の記号の並び。**面が使う。**
#[must_use]
pub const fn letters() -> [&'static str; 8] {
    LETTERS
}

/// 入力を読み取り専用にした複製で、組めるかを検査する。
///
/// **生成物は入力ではない** ── 消してから、入力だけを読み取り専用にする。
fn read_only_run(references: &Path, dir: &Path) -> Result<(), String> {
    let name = dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let many = std::env::temp_dir()
        .join("bb-readonly")
        .join(format!("{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&many);
    copy_tree(dir, &many).map_err(|e| format!("複製を作れない ── {e}"))?;
    let _ = std::fs::remove_file(many.join("board.html"));
    set_read_only(&many, true).map_err(|e| format!("読み取り専用にできない ── {e}"))?;
    let got = render(references, &many, true);
    // 消せるように、書ける状態へ戻す
    let _ = set_read_only(&many, false);
    let _ = std::fs::remove_dir_all(&many);
    got.map(|_| ()).map_err(|why| {
        format!(
            "読み取り専用の複製で異常終了した: {}",
            why.lines().last().unwrap_or("(出力無し)")
        )
    })
}

/// 木ごと写す。
fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let at = entry.path();
        let into = to.join(entry.file_name());
        if at.is_dir() {
            copy_tree(&at, &into)?;
        } else {
            std::fs::copy(&at, &into)?;
        }
    }
    Ok(())
}

/// ファイルを読み取り専用にする（戻すこともできる）。
fn set_read_only(at: &Path, on: bool) -> std::io::Result<()> {
    for entry in std::fs::read_dir(at)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            set_read_only(&path, on)?;
            continue;
        }
        let mut perm = std::fs::metadata(&path)?.permissions();
        perm.set_readonly(on);
        std::fs::set_permissions(&path, perm)?;
    }
    Ok(())
}
