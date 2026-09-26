// SPDX-License-Identifier: MIT
//! 句の末尾を全部拾って並べる。**語の一覧を使わない。**
//!
//! 「述部が和語である」の判定は一覧で照合するので、**一覧に無い和語は通過する**。
//! この道具は逆で、句の末尾を機械的に全部拾い、漢語の述部だけを除外して残りを並べる
//! ── 判定は人が実施するが、**拾い残しが発生しない**。
//!
//! 確定したものは `references/predicates.json` へ追加する ── そうすると次からは機械が
//! 検出する。**この道具は「未知を洗い出す2周目」であって、判定の代替ではない。**

use std::collections::BTreeMap;
use std::io;
use std::path::Path;
use std::sync::OnceLock;

use fancy_regex::Regex;
use serde_json::Value;

fn kango() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"[一-鿿]{1,6}(する|した|しない|される|された|できる|できない|である|でない|になる|による|とする|しうる|し、|する。)$",
        )
        .expect("組める")
    })
}

fn tail() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"([一-鿿ぁ-んァ-ヶー]{2,8})(?=[。、）」\n]|$)").expect("組める"))
}

fn predicate_like() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(る|た|い|う|く|す|つ|ぶ|む|ぬ|ぐ|ず|ない|なる|れる|られる|ある|いる)$")
            .expect("組める")
    })
}

fn strip(pattern: &str, text: &str) -> String {
    Regex::new(pattern).map_or_else(
        |_| text.to_owned(),
        |r| r.replace_all(text, "").into_owned(),
    )
}

/// 1つの文から、述部らしい末尾を拾う。
#[must_use]
pub fn of(text: &str) -> Vec<String> {
    let text = strip(r"`[^`]*`", &strip(r"<[^>]+>", text));
    let mut out = Vec::new();
    let mut at = 0;
    while at < text.len() {
        let Ok(Some(m)) = tail().captures_from_pos(&text, at) else {
            break;
        };
        let Some(word) = m.get(1) else { break };
        let s = word.as_str().to_owned();
        at = word.end().max(at + 1);
        if kango().is_match(&s).unwrap_or(false) {
            continue;
        }
        if predicate_like().is_match(&s).unwrap_or(false) {
            out.push(s);
        }
    }
    out
}

fn prose_of_json(raw: &str) -> String {
    fn walk(v: &Value, out: &mut Vec<String>) {
        match v {
            Value::String(s) => out.push(s.clone()),
            Value::Array(items) => items.iter().for_each(|x| walk(x, out)),
            Value::Object(map) => {
                for (k, v) in map {
                    if !k.starts_with('_') {
                        walk(v, out);
                    }
                }
            }
            _ => {}
        }
    }
    let Ok(parsed) = serde_json::from_str::<Value>(raw) else {
        return raw.to_owned();
    };
    let mut out = Vec::new();
    walk(&parsed, &mut out);
    out.join("。")
}

/// 渡したファイルの述部の末尾を数える。**多い順、同数なら名前の順で返す。**
///
/// # Errors
///
/// 読めないときに返す。
pub fn counted(paths: &[&Path]) -> io::Result<Vec<(String, usize)>> {
    let mut count: BTreeMap<String, usize> = BTreeMap::new();
    for path in paths {
        let mut raw = std::fs::read_to_string(path)?;
        if path.extension().is_some_and(|x| x == "json") {
            raw = prose_of_json(&raw);
        }
        for word in of(&raw) {
            *count.entry(word).or_default() += 1;
        }
    }
    let mut out: Vec<(String, usize)> = count.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    Ok(out)
}

/// 一覧の行。**印字する側と、機械へ返す側が、同じ文字列を使う。**
///
/// # Errors
///
/// 読めないときに返す。
pub fn lines(paths: &[&Path]) -> io::Result<Vec<String>> {
    Ok(counted(paths)?
        .into_iter()
        .map(|(word, n)| format!("{n:3}  {word}"))
        .collect())
}
