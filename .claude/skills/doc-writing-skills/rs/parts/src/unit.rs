// SPDX-License-Identifier: MIT
//! 文書を、位置を保ったまま単位へ切る。
//!
//! **位置を保つ。** 切ったあとに行を数え直すと、報告が原文と食い違う ── 読み手は
//! その行を開いて直す。
//!
//! **囲みの中は判定しない。** 記法の中身は人が読む文ではない。

use fancy_regex::Regex;
use std::sync::OnceLock;

/// 単位の種類。**機械が分岐する値は ASCII である。**
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Kind {
    /// 段落。
    Body,
    /// 見出し。
    Heading,
    /// 箇条書きの1項目。
    Item,
    /// 表の1つの欄。
    Cell,
    /// 引用。
    Quote,
    /// 囲みの中。
    Code,
}

impl Kind {
    /// 画面へ出す語。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Body => "本文",
            Self::Heading => "見出し",
            Self::Item => "箇条書き",
            Self::Cell => "表のセル",
            Self::Quote => "引用",
            Self::Code => "コード",
        }
    }

    /// 描画されるか。**囲みの中は、記法がそのまま出る。**
    #[must_use]
    pub const fn rendered(self) -> bool {
        !matches!(self, Self::Code)
    }

    /// 散文か。**見出しと表の欄は、文の形を取らない。**
    #[must_use]
    pub const fn prose(self) -> bool {
        matches!(self, Self::Body | Self::Item | Self::Quote)
    }
}

/// 切り出した単位1つ。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Unit {
    /// 種類。
    pub kind: Kind,
    /// 原文の行（1から数える）。
    pub line: usize,
    /// 原文の行そのもの。
    pub raw: String,
    /// 記法を除いた中身。
    pub text: String,
    /// 見出しの深さ。見出し以外は0。
    pub level: usize,
    /// 箇条書きの字下げ。
    pub indent: usize,
}

fn re(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).expect("この正規表現は組める"))
}

fn heading() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^(\s*)(#{1,6})(\s|$)")
}
fn list_mark() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^(\s*)([-*+]|\d+[.)])(\s|$)")
}
fn quote() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^\s*>")
}
fn fence() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^\s*(`{3,}|~{3,})")
}
fn table() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^\s*\|")
}
fn table_sep() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"^\s*\|?[\s:|-]+\|[\s:|-]*\|?\s*$")
}
fn inline() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, r"\*\*|`|\[|\]\([^)]*\)|<br\s*/?>")
}

/// 記法を除いて、人が読む文だけを返す。
#[must_use]
pub fn plain(s: &str) -> String {
    inline().replace_all(s, "").trim().to_owned()
}

fn matched(r: &Regex, s: &str) -> bool {
    r.is_match(s).unwrap_or(false)
}

/// 文書を単位へ切る。
#[must_use]
pub fn split(text: &str) -> Vec<Unit> {
    let mut units = Vec::new();
    // 囲みの印と長さ。**同じ印で、同じ長さ以上のときだけ閉じる**
    let mut fenced: Option<(char, usize)> = None;
    let mut in_front = false;
    for (i, raw) in text.split('\n').enumerate() {
        let line = i + 1;
        if line == 1 && raw.trim() == "---" {
            in_front = true;
            continue;
        }
        if in_front {
            if raw.trim() == "---" {
                in_front = false;
            }
            continue;
        }
        if let Ok(Some(m)) = fence().captures(raw) {
            let run = m.get(1).map_or("", |x| x.as_str());
            let head = run.chars().next().unwrap_or('`');
            match fenced {
                None => fenced = Some((head, run.len())),
                Some((c, n)) if head == c && run.len() >= n => fenced = None,
                Some(_) => {}
            }
            continue;
        }
        if fenced.is_some() {
            units.push(Unit {
                kind: Kind::Code,
                line,
                raw: raw.to_owned(),
                text: raw.trim().to_owned(),
                level: 0,
                indent: 0,
            });
            continue;
        }
        if raw.trim().is_empty() {
            continue;
        }
        if let Ok(Some(m)) = heading().captures(raw) {
            let level = m.get(2).map_or(0, |x| x.as_str().len());
            let body = raw.trim_start().trim_start_matches('#').trim_start();
            units.push(Unit {
                kind: Kind::Heading,
                line,
                raw: raw.to_owned(),
                text: plain(body),
                level,
                indent: 0,
            });
            continue;
        }
        let s = raw.trim_start();
        let indent = raw.len() - s.len();
        if matched(quote(), s) {
            let body = s.trim_start_matches(|c: char| c == '>' || c.is_whitespace());
            units.push(Unit {
                kind: Kind::Quote,
                line,
                raw: raw.to_owned(),
                text: plain(body),
                level: 0,
                indent: 0,
            });
        } else if matched(table(), s) {
            if matched(table_sep(), s) {
                continue;
            }
            for cell in s.trim().trim_matches('|').split('|') {
                if !cell.trim().is_empty() {
                    units.push(Unit {
                        kind: Kind::Cell,
                        line,
                        raw: raw.to_owned(),
                        text: plain(cell),
                        level: 0,
                        indent: 0,
                    });
                }
            }
        } else if matched(list_mark(), s) {
            let body = list_mark().replace(s, "");
            units.push(Unit {
                kind: Kind::Item,
                line,
                raw: raw.to_owned(),
                text: plain(&body),
                level: 0,
                indent,
            });
        } else {
            units.push(Unit {
                kind: Kind::Body,
                line,
                raw: raw.to_owned(),
                text: plain(raw),
                level: 0,
                indent: 0,
            });
        }
    }
    units
}
