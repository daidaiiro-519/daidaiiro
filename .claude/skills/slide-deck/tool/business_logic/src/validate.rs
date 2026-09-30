// SPDX-License-Identifier: MIT
//! デッキの入力を検査する。**組み立てより前に、1件でも出れば止まる。**
//!
//! 形は `references/slide-deck.schema.json` が見る。ここが見るのは、
//! **形では書けない規則**である ── 設計規則（`references/document.json` の `design-rules`）のうち、
//! 数えれば判定できるものを機械に見させる。散文の規定は破れる。

use std::path::Path;

use serde_json::Value;

use crate::data_access::files;
use crate::theme;

/// 大きい要素。**4つ以上あると、どれが主張か消える**（設計規則 §2）
const BIG: [&str; 6] = ["stat", "figure", "punch", "card", "pair", "recap"];
const BIG_MAX: usize = 3;

/// 見出しは1〜2行に収める（§1）。h2 は 36px で、1枚の幅に約30字が入る
const HEADING_MAX: usize = 60;
/// 締めは1〜2行（§2）
const CLOSE_MAX: usize = 120;
/// 強調は1か所だけ（§2 ・ §8 の4）
const MARK_MAX: usize = 1;

/// 出典を示す語。**要素から離れた出典は、支える相手を失う。**
const SOURCES: [&str; 4] = ["出典", "arXiv", "http://", "https://"];

fn text_of(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_owned()
}

fn array_of<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(|x| x.as_array())
        .map_or(&[], |x| x.as_slice())
}

/// 枚が持つ要素を、列の内も外も同じ並びで返す。
fn blocks(slide: &Value) -> Vec<&Value> {
    let mut out: Vec<&Value> = array_of(slide, "blocks").iter().collect();
    for column in array_of(slide, "columns") {
        out.extend(array_of(column, "blocks").iter());
    }
    out
}

fn kind_of(block: &Value) -> String {
    text_of(block, "kind")
}

/// 枚の呼び名。**何枚目かと、こちらで付けた名前の両方を出す。**
fn at_of(no: usize, slide: &Value) -> String {
    let label = slide
        .get("label")
        .and_then(|x| x.as_str())
        .unwrap_or("名前なし");
    format!("{no}枚目（{label}）")
}

/// 形を検査する。
#[must_use]
pub fn shape(schema_path: &Path, deck: &Value) -> Vec<String> {
    let Ok(body) = files::read_to_string(schema_path) else {
        return vec![format!("形: 契約を読めない ── {}", schema_path.display())];
    };
    let Ok(schema) = serde_json::from_str::<Value>(&body) else {
        return vec!["形: 契約が JSON として読めない".to_owned()];
    };
    let built = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&schema);
    let Ok(validator) = built else {
        return vec!["形: 契約がスキーマとして無効である".to_owned()];
    };
    let mut found: Vec<String> = validator
        .iter_errors(deck)
        .map(|e| {
            let at = e.instance_path.to_string();
            format!("形: {} ── {e}", at.trim_start_matches('/'))
        })
        .collect();
    found.sort();
    found
}

/// 並べ方と、渡した中身が噛み合っているかを検査する。
#[must_use]
pub fn wiring(references: &Path, deck: &Value) -> Vec<String> {
    let mut bad = Vec::new();
    let wanted = text_of(deck, "theme");
    let names = theme::theme_names(references);
    if !wanted.is_empty() && !names.contains(&wanted) {
        bad.push(format!(
            "テーマ「{wanted}」が references/themes/ に無い ── 在るのは {} である",
            names.join(" ・ ")
        ));
    }
    for (i, slide) in array_of(deck, "slides").iter().enumerate() {
        let at = at_of(i + 1, slide);
        let layout = text_of(slide, "layout");
        let columns = array_of(slide, "columns");
        let has_blocks = !array_of(slide, "blocks").is_empty();
        if matches!(layout.as_str(), "cols" | "cols-3") && columns.is_empty() {
            bad.push(format!("{at}: {layout} なのに columns が無い"));
        }
        if matches!(layout.as_str(), "single" | "center") && !has_blocks {
            bad.push(format!("{at}: {layout} なのに blocks が無い"));
        }
        if layout == "cols-3" && columns.len() != 3 {
            bad.push(format!("{at}: cols-3 の列が {} 本である", columns.len()));
        }
        if layout == "cols" && columns.len() != 2 {
            bad.push(format!("{at}: cols の列が {} 本である", columns.len()));
        }
        if layout != "cover" && has_blocks && !columns.is_empty() {
            bad.push(format!(
                "{at}: blocks と columns の両方を渡している ── どちらか1つにする"
            ));
        }
        if layout != "cover" && text_of(slide, "heading").is_empty() {
            bad.push(format!("{at}: 見出しが無い ── 枚で言い切ることを1文で書く"));
        }
        if layout == "cover" && (has_blocks || !columns.is_empty()) {
            bad.push(format!("{at}: 表紙は要素を持たない"));
        }
    }
    bad
}

/// 見出しが問いの形かを判定する。
fn asks(heading: &str) -> bool {
    let tail = heading.trim_end();
    ["か", "か？", "?", "？"]
        .iter()
        .any(|mark| tail.ends_with(mark))
}

/// 設計規則のうち、数えれば判定できるものを検査する。
#[must_use]
pub fn limits(deck: &Value) -> Vec<String> {
    let mut bad = Vec::new();
    for (i, slide) in array_of(deck, "slides").iter().enumerate() {
        let at = at_of(i + 1, slide);
        let heading = text_of(slide, "heading");
        let length = heading.chars().count();
        if length > HEADING_MAX {
            bad.push(format!(
                "{at}: 見出しが {length} 字 ── {HEADING_MAX} 字（2行）に収める"
            ));
        }
        if asks(&heading) {
            bad.push(format!("{at}: 見出しが問いの形である ── 断定形で書く"));
        }
        let big: Vec<String> = blocks(slide)
            .iter()
            .map(|b| kind_of(b))
            .filter(|k| BIG.contains(&k.as_str()))
            .collect();
        if big.len() > BIG_MAX {
            bad.push(format!(
                "{at}: 大きい要素が {} つ（{}）── {BIG_MAX} つまでにする。どれが主張か消える",
                big.len(),
                big.join(" ・ ")
            ));
        }
        let mut marks = blocks(slide)
            .iter()
            .filter(|b| kind_of(b) == "flow")
            .flat_map(|b| array_of(b, "rows"))
            .filter(|r| r.get("mark").and_then(Value::as_bool) == Some(true))
            .count();
        if slide
            .get("kicker")
            .map(|k| text_of(k, "tone"))
            .is_some_and(|tone| tone == "warm")
        {
            marks += 1;
        }
        if marks > MARK_MAX {
            bad.push(format!("{at}: 強調が {marks} か所 ── 1か所だけにする"));
        }
        for column in array_of(slide, "columns") {
            let close = text_of(column, "close").chars().count();
            if close > CLOSE_MAX {
                bad.push(format!(
                    "{at}: 締めが {close} 字 ── {CLOSE_MAX} 字（2行）に収める"
                ));
            }
        }
        // **出典を、要素から離さない**（§2 ・ §8 の3）
        for block in blocks(slide) {
            if kind_of(block) != "text" {
                continue;
            }
            let body = text_of(block, "body");
            if SOURCES.iter().any(|mark| body.contains(mark)) {
                bad.push(format!(
                    "{at}: 出典が本文に在る ── 支える要素（card の note ・ stat の source）の直下へ置く"
                ));
            }
        }
    }
    bad
}

/// 列に与えた役割が、枚をまたいで同じかを検査する（§5）。
///
/// **役割を与えた枚どうしだけを比べる** ── 与えていない枚は、役割を持たない枚で
/// あって、違反ではない。
#[must_use]
pub fn roles(deck: &Value) -> Vec<String> {
    let mut seen: Vec<(usize, Vec<String>)> = Vec::new();
    let mut bad = Vec::new();
    for (i, slide) in array_of(deck, "slides").iter().enumerate() {
        let columns = array_of(slide, "columns");
        let got: Vec<String> = columns.iter().map(|c| text_of(c, "role")).collect();
        if got.iter().all(String::is_empty) {
            continue;
        }
        let count = columns.len();
        match seen.iter().find(|(n, _)| *n == count) {
            None => seen.push((count, got)),
            Some((_, first)) if *first != got => bad.push(format!(
                "{}: 列の役割が {} ── 他の枚は {} である。対応づけを崩さない",
                at_of(i + 1, slide),
                got.join(" ／ "),
                first.join(" ／ ")
            )),
            Some(_) => {}
        }
    }
    bad
}

/// すべての検査を、同じ並びで実行する。**形が通らなければ、その先は見ない。**
#[must_use]
pub fn check(references: &Path, schema_path: &Path, deck: &Value) -> Vec<String> {
    let mut bad = shape(schema_path, deck);
    if bad.iter().any(|e| e.starts_with("形:")) {
        return bad;
    }
    bad.extend(wiring(references, deck));
    bad.extend(limits(deck));
    bad.extend(roles(deck));
    bad
}
