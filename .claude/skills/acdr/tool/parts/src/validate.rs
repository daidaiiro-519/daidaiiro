// SPDX-License-Identifier: MIT
//! 入力（acdr.json）を検査する。**生成物ではなく、入力を検査する。**
//!
//! 生成物を検査する形では、複製が正しく生成されたことしか判明しない。しかも不合格の
//! 時点で HTML は既に出ている ── それはゲートではなく注記である。
//! **不合格なら HTML を1バイトも出さない。**
//!
//! 検査は4系統である ── 形（スキーマ）・ 欄の完備（節と3つ組）・ 散文 ・ 参照の解決。

use std::path::{Path, PathBuf};

use regex::Regex;
use serde_json::Value;

/// 承認の状態。**これ以外を書かせない** ── 状態が自由文になると、承認を得ているかを
/// 読む側が判定することになる。
pub const STATUS: [&str; 3] = ["proposed", "accepted", "superseded"];

/// 節。**欠けたら止まる** ── 欠けた記録は、あとから誰も補完できない。
pub const SECTIONS: [&str; 2] = ["decision", "why"];

/// 1つの欄に置ける主張の数。
const COHABIT: usize = 1;
/// 1文の字数の上限。
const LONGEST: usize = 120;
/// 主張の区切り。
const SEP: &str = "──";

/// 形の契約の場所。
#[must_use]
pub fn schema_path(references: &Path) -> PathBuf {
    references.join("acdr.schema.json")
}

fn value_text(value: &Value, key: &str) -> String {
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

/// 欄の完備を検査する。**3つ組が完備していない変更を、記録に包含しない。**
#[must_use]
pub fn fields(spec: &Value) -> Vec<String> {
    let mut bad = Vec::new();
    for key in ["no", "title", "date", "status"]
        .iter()
        .chain(SECTIONS.iter())
    {
        if value_text(spec, key).trim().is_empty() {
            bad.push(format!("必須の欄が欠けている: {key}"));
        }
    }
    let status = value_text(spec, "status");
    if !status.is_empty() && !STATUS.contains(&status.as_str()) {
        bad.push(format!(
            "状態が「{status}」。使えるのは {} である",
            STATUS.join("／")
        ));
    }
    for (i, row) in array_of(spec, "shift").iter().enumerate() {
        for key in ["what", "from", "to"] {
            if row.get(key).is_none() {
                bad.push(format!("shift[{i}] に {key} が無い"));
            }
        }
    }
    for (i, row) in array_of(spec, "alternatives").iter().enumerate() {
        for key in ["option", "why_not"] {
            if row.get(key).is_none() {
                bad.push(format!("alternatives[{i}] に {key} が無い"));
            }
        }
    }
    for doc in array_of(spec, "docs") {
        for key in ["key", "tab", "file"] {
            if doc.get(key).is_none() {
                bad.push(format!("docs の項目に {key} が無い"));
            }
        }
        let name = doc
            .get("key")
            .and_then(|x| x.as_str())
            .unwrap_or("?")
            .to_owned();
        for (i, change) in array_of(doc, "marks").iter().enumerate() {
            for key in ["find", "before", "why"] {
                if value_text(change, key).trim().is_empty() {
                    bad.push(format!(
                        "{name} の変更[{i}] に {key} が無い ── 3つ組が完備していない変更は、記録に含めない"
                    ));
                }
            }
        }
    }
    bad
}

/// 検査する欄を、場所の名前つきで全部列挙する。**取りこぼしを作らない。**
///
/// **原文は列挙しない** ── 変更前（`before`）は原典であり、形を変えない。照合する
/// 文字列（`find`）も、散文ではない。
#[must_use]
pub fn cells(spec: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for key in [
        "decision",
        "why",
        "how",
        "applies_to",
        "supersedes",
        "figure_caption",
    ] {
        let body = value_text(spec, key);
        if !body.is_empty() {
            out.push((key.to_owned(), body));
        }
    }
    for (i, row) in array_of(spec, "shift").iter().enumerate() {
        for key in ["what", "from", "to"] {
            out.push((format!("shift[{i}]/{key}"), value_text(row, key)));
        }
    }
    for (i, row) in array_of(spec, "alternatives").iter().enumerate() {
        for key in ["option", "why_not"] {
            let body = value_text(row, key);
            if !body.is_empty() {
                out.push((format!("alternatives[{i}]/{key}"), body));
            }
        }
    }
    for (i, item) in array_of(spec, "after_approval").iter().enumerate() {
        let body = item
            .as_str()
            .map_or_else(|| item.to_string(), std::borrow::ToOwned::to_owned);
        out.push((format!("after_approval[{i}]"), body));
    }
    for doc in array_of(spec, "docs") {
        let name = doc.get("key").and_then(|x| x.as_str()).unwrap_or("?");
        for (i, change) in array_of(doc, "marks").iter().enumerate() {
            out.push((format!("{name} の変更[{i}]/why"), value_text(change, "why")));
        }
    }
    out
}

/// 印と引用を落とした、素の文。**引用は除外する** ── 原文の形を変えないためである。
fn plain(body: &str) -> String {
    let tag = Regex::new(r"<[^>]+>").expect("式である");
    let quote = Regex::new(r"(?s)「[^」]*」|<code>.*?</code>").expect("式である");
    quote
        .replace_all(&tag.replace_all(body, ""), "")
        .into_owned()
}

/// 1つの欄に2つのことが入っていないか、1文が長すぎないかを見る。
#[must_use]
pub fn prose(place: &str, body: &str) -> Vec<String> {
    let mut bad = Vec::new();
    let raw = plain(body);
    let count = raw.matches(SEP).count();
    if count > COHABIT {
        bad.push(format!(
            "{place}: 1つの欄に区切り「{SEP}」が {count} 個ある"
        ));
    }
    for sentence in split_sentences(&raw) {
        let sentence = sentence.trim();
        let length = sentence.chars().count();
        if length > LONGEST {
            bad.push(format!(
                "{place}: 1文が {length} 字ある（上限 {LONGEST}）── {}…",
                sentence.chars().take(34).collect::<String>()
            ));
        }
    }
    bad
}

/// 「。」のあとで文を割る。**区切りは前の文に残す。**
fn split_sentences(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut now = String::new();
    for c in body.chars() {
        now.push(c);
        if c == '。' {
            out.push(std::mem::take(&mut now));
        }
    }
    if !now.is_empty() {
        out.push(now);
    }
    out
}

/// **入力は宣言だけを保持する。** 見出しや表が入っていれば、宣言の外に構造がある。
#[must_use]
pub fn level_mix(place: &str, body: &str) -> Vec<String> {
    let items = Regex::new(
        r"(?i)</?(p|h[1-6]|ul|ol|li|pre|div|table|tr|td|th|details|summary|figure|blockquote)\b",
    )
    .expect("式である");
    items.find(body).map_or_else(Vec::new, |m| {
        vec![format!(
            "{place}: 見出しや表「{}」が欄の中に在る ── 宣言へ割る",
            m.as_str()
        )]
    })
}

/// 参照先が実在するかを検査する。**図と、対象の文書である。**
#[must_use]
pub fn refs(folder: &Path, spec: &Value, root: Option<&Path>) -> Vec<String> {
    let mut bad = Vec::new();
    if let Some(name) = spec
        .get("figure")
        .and_then(|x| x.as_str())
        .filter(|name| !name.is_empty())
    {
        let at = folder.join(name);
        if !at.exists() {
            bad.push(format!(
                "図が無い: {} ── design-svg に組ませて置くか、\"figure\" の欄を削除する",
                at.display()
            ));
        }
    }
    for doc in array_of(spec, "docs") {
        let file = value_text(doc, "file");
        let path = root.map_or_else(|| PathBuf::from(&file), |root| root.join(&file));
        if !path.exists() {
            bad.push(format!("対象の文書が無い: {}", path.display()));
        }
    }
    bad
}

/// 入力1つを、4系統すべてで検査する。**呼ぶ側は、0件のときだけ組む。**
#[must_use]
pub fn inspect(
    references: &Path,
    spec: &Value,
    folder: Option<&Path>,
    root: Option<&Path>,
) -> Vec<String> {
    let mut bad = fields(spec);
    bad.extend(crate::shape::against(&schema_path(references), spec, "形"));
    for (place, cell) in cells(spec) {
        bad.extend(prose(&place, &cell));
        bad.extend(level_mix(&place, &cell));
    }
    if let Some(folder) = folder {
        bad.extend(refs(folder, spec, root));
    }
    bad
}

/// 記録のフォルダ1つを検査する。
///
/// # Errors
///
/// 入力を読めないときに返す。
pub fn check(references: &Path, folder: &Path, root: Option<&Path>) -> Result<Vec<String>, String> {
    let path = folder.join("acdr.json");
    let body = std::fs::read_to_string(&path)
        .map_err(|e| format!("{}: 読めない ── {e}", path.display()))?;
    let spec: Value = serde_json::from_str(&body)
        .map_err(|e| format!("{}: JSON として読めない ── {e}", path.display()))?;
    Ok(inspect(references, &spec, Some(folder), root))
}
