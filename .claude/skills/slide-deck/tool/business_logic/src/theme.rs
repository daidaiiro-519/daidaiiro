// SPDX-License-Identifier: MIT
//! テーマ ── 配色の正本を読み、検査し、図を組ませる側へ渡す形へ写す。
//!
//! 検査で見るのは3つ ── キーがすべてのテーマで一致しているか、適合条件を満たすか、
//! テーマの外に16進の直書きが残っていないか。**見た目は見ない。**
//!
//! **この Skill は図を描かない。** 渡すのは配色だけである ── `as_roles` が、
//! テーマのキーを**図の中の役割の名前**へ複製する。誰に組ませるかは配線表が決める。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::data_access::files;

/// 適合条件。`(文字, 地, 名前, 要る比)`
///
/// `--line` は装飾専用である ── 情報を単独で担わせないので、比を課さない。
const RULES: [(&str, &str, &str, f64); 13] = [
    ("--ink", "--ground", "本文", 4.5),
    ("--dim", "--ground", "補助", 4.5),
    ("--faint", "--ground", "最薄", 4.5),
    ("--accent", "--ground", "強調", 4.5),
    ("--accent", "--surface", "強調（面の上）", 4.5),
    ("--on-accent", "--accent", "強調面の文字", 4.5),
    ("--on-accent-dim", "--accent-dim", "濃い強調面の文字", 4.5),
    ("--on-paper", "--paper", "明るい面の文字", 4.5),
    ("--on-paper-dim", "--paper", "明るい面の補足", 4.5),
    ("--on-paper", "--mark", "印の上の文字", 4.5),
    ("--ink", "--surface", "面の上の本文", 4.5),
    ("--diagram-line", "--canvas", "図の線（図の地）", 3.0),
    ("--diagram-line", "--ground", "図の線（地）", 3.0),
];

/// 図の中の役割と、テーマのどのキーから取るか。
///
/// **この表はこちら側の語彙である** ── 組ませる相手の名前も、相手のトークン名も、
/// ここは保持しない。
pub const FIGURE_ROLES: [(&str, &str); 8] = [
    ("ink", "--ink"),             // 図の中の見出し・強い文字
    ("dim", "--dim"),             // 図の中の補足
    ("faint", "--faint"),         // いちばん薄い文字
    ("line", "--diagram-line"),   // 図の線。**本文の文字色とは別のキーから取る**
    ("paper", "--paper"),         // 明るい面
    ("surface", "--surface"),     // 面
    ("accent", "--accent"),       // 強調
    ("on-accent", "--on-accent"), // 強調面の上に置く文字
];

/// テーマの区切り。**これが無いと、テーマの外の直書きを判定できない。**
const HEAD: &str = "▼ テーマ";
const TAIL: &str = "▲ テーマここまで";

/// 配色の正本の置き場所。
#[must_use]
pub fn dir(references: &Path) -> PathBuf {
    references.join("themes")
}

/// 相対輝度。**W3C の式である** ── 目で見た明るさではない。
fn luminance(hex: &str) -> f64 {
    let body = hex.trim_start_matches('#');
    let digits: String = if body.chars().count() == 3 {
        body.chars().flat_map(|c| [c, c]).collect()
    } else {
        body.to_owned()
    };
    let at = |i: usize| -> f64 {
        let Some(pair) = digits.get(i..i + 2) else {
            return 0.0;
        };
        let value = f64::from(u8::from_str_radix(pair, 16).unwrap_or(0)) / 255.0;
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * at(0) + 0.7152 * at(2) + 0.0722 * at(4)
}

/// 文字と地の比。
#[must_use]
pub fn ratio(fore: &str, back: &str) -> f64 {
    let (a, b) = (luminance(fore), luminance(back));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn is_hex(c: char) -> bool {
    c.is_ascii_hexdigit()
}

/// 16進の色を1つ読む。**長さは3〜8桁である。**
///
/// 返すのは、読んだ色と、次に見る位置である。
fn hex_at(chars: &[char], from: usize) -> Option<(String, usize)> {
    let mut end = from;
    while end < chars.len() && is_hex(chars[end]) {
        end += 1;
    }
    let len = end - from;
    if !(3..=8).contains(&len) {
        return None;
    }
    Some((chars[from..end].iter().collect(), end))
}

/// テーマが持つ色を読む。**同じキーが2度出たら、後のものが残る。**
#[must_use]
pub fn tokens(body: &str) -> BTreeMap<String, String> {
    let chars: Vec<char> = body.chars().collect();
    let mut out = BTreeMap::new();
    let mut i = 0;
    while i + 1 < chars.len() {
        if chars[i] != '-' || chars[i + 1] != '-' {
            i += 1;
            continue;
        }
        let mut end = i + 2;
        while end < chars.len()
            && (chars[end].is_ascii_lowercase() || chars[end].is_ascii_digit() || chars[end] == '-')
        {
            end += 1;
        }
        let key: String = chars[i..end].iter().collect();
        let mut at = end;
        while at < chars.len() && chars[at].is_whitespace() {
            at += 1;
        }
        if at >= chars.len() || chars[at] != ':' {
            i = end.max(i + 1);
            continue;
        }
        at += 1;
        while at < chars.len() && chars[at].is_whitespace() {
            at += 1;
        }
        if at >= chars.len() || chars[at] != '#' {
            i = end.max(i + 1);
            continue;
        }
        let Some((hex, mut after)) = hex_at(&chars, at + 1) else {
            i = end.max(i + 1);
            continue;
        };
        let stop = after;
        while after < chars.len() && chars[after].is_whitespace() {
            after += 1;
        }
        if after < chars.len() && chars[after] == ';' {
            out.insert(key, format!("#{hex}"));
            i = after + 1;
        } else {
            i = stop.max(i + 1);
        }
    }
    out
}

/// 置いてあるテーマの場所。**名前の並びは、置いてあるファイルが決める。**
#[must_use]
pub fn theme_files(references: &Path) -> Vec<PathBuf> {
    let Ok(entries) = files::list(dir(references)) else {
        return Vec::new();
    };
    entries
        .into_iter()
        .filter(|p| p.extension().is_some_and(|x| x == "css"))
        .collect()
}

/// 置いてあるテーマの名前。
#[must_use]
pub fn theme_names(references: &Path) -> Vec<String> {
    theme_files(references)
        .iter()
        .map(|p| {
            p.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

/// 名前から正本の場所を引く。
///
/// # Errors
///
/// 置いていない名前のとき、使える名前を添えて返す。
pub fn theme_path(references: &Path, name: &str) -> Result<PathBuf, String> {
    let path = dir(references).join(format!("{name}.css"));
    if !files::exists(&path) {
        return Err(format!(
            "知らないテーマ: {name}。使えるのは {} である",
            theme_names(references).join("／")
        ));
    }
    Ok(path)
}

/// テーマの中身を読む。
///
/// # Errors
///
/// 置いていない名前のときと、読めないときに返す。
pub fn read(references: &Path, name: &str) -> Result<String, String> {
    let path = theme_path(references, name)?;
    files::read_to_string(&path).map_err(|e| format!("{}: 読めない ── {e}", path.display()))
}

/// 図の中の役割ごとの色を書き出す。**書き出し先のフォルダは作らない。**
///
/// # Errors
///
/// 書けないときに返す。
pub fn save(path: &Path, body: &str) -> Result<(), String> {
    files::write(path, body).map_err(|e| format!("{}: 書けない ── {e}", path.display()))
}

/// テーマを、図の中の役割ごとの色へ複製する。
///
/// **色の正本は1つである** ── 図の側にも色を書くと、テーマを替えたときに
/// 図だけが前の配色のまま残る（実測 ── 4配色のうち1つで文字が消えた）。
///
/// # Errors
///
/// 置いていない名前のときと、図へ渡すキーが欠けているときに返す。
pub fn as_roles(references: &Path, name: &str) -> Result<Vec<(String, String)>, String> {
    let found = tokens(&read(references, name)?);
    let mut missing: Vec<&str> = FIGURE_ROLES
        .iter()
        .map(|(_, key)| *key)
        .filter(|key| !found.contains_key(*key))
        .collect();
    missing.sort_unstable();
    missing.dedup();
    if !missing.is_empty() {
        return Err(format!(
            "{name}: 図へ渡すキーが欠けている: {}",
            missing.join("、")
        ));
    }
    Ok(FIGURE_ROLES
        .iter()
        .map(|(role, key)| ((*role).to_owned(), found[*key].clone()))
        .collect())
}

/// テーマの外に残った色の直書きを集める。
fn written_outside(body: &str) -> Vec<String> {
    let (Some(head), Some(tail)) = (body.find(HEAD), body.find(TAIL)) else {
        return Vec::new();
    };
    let outside = format!("{}{}", &body[..head], &body[tail..]);
    let chars: Vec<char> = outside.chars().collect();
    let mut found: Vec<String> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '#' {
            i += 1;
            continue;
        }
        let Some((hex, after)) = hex_at(&chars, i + 1) else {
            i += 1;
            continue;
        };
        // **語の切れ目で終わっていなければ、色ではない** ── 9桁以上の並びの先頭を
        // 色として数えると、在らない直書きを報告することになる
        let bounded = chars
            .get(after)
            .is_none_or(|c| !(c.is_ascii_alphanumeric() || *c == '_'));
        if bounded {
            found.push(format!("#{hex}"));
        }
        i = after;
    }
    found.sort();
    found.dedup();
    found
}

/// 検査の検出を返す。**印字はしない** ── 印字はプレゼンテーション層が持つ。
#[must_use]
pub fn findings(references: &Path, decks: &[String]) -> Vec<String> {
    let mut bad = Vec::new();
    let files = theme_files(references);
    let read: Vec<(String, BTreeMap<String, String>)> = files
        .iter()
        .map(|p| {
            let name = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let body = files::read_to_string(p).unwrap_or_default();
            (name, tokens(&body))
        })
        .collect();
    let mut every: Vec<&String> = read.iter().flat_map(|(_, t)| t.keys()).collect();
    every.sort();
    every.dedup();

    for (name, found) in &read {
        for key in &every {
            if !found.contains_key(*key) {
                bad.push(format!("{name}: {key} が無い"));
            }
        }
        for (fore, back, label, need) in RULES {
            if let (Some(a), Some(b)) = (found.get(fore), found.get(back)) {
                let got = ratio(a, b);
                if got < need {
                    bad.push(format!(
                        "{name}: {label} {got:.2}（要 {need:.1}）  {a} / {b}"
                    ));
                }
            }
        }
    }

    for deck in decks {
        let Ok(body) = files::read_to_string(deck) else {
            bad.push(format!("{deck}: 読めない"));
            continue;
        };
        if !body.contains(HEAD) || !body.contains(TAIL) {
            bad.push(format!("{deck}: テーマの区切りが無い"));
            continue;
        }
        for hex in written_outside(&body) {
            bad.push(format!("{deck}: テーマの外に色の直書き {hex}"));
        }
    }
    bad
}
