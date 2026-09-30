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

/// 中身を読まない要素。**頁の見た目と動きであり、人が読む文ではない。**
const SKIPPED: [&str; 3] = ["script", "style", "template"];

/// 中身がコードである要素。
const CODE: [&str; 2] = ["pre", "code"];

/// 単位を区切る要素。**開きでも閉じでも、そこで単位が終わる。**
const BLOCKS: [&str; 34] = [
    "address",
    "article",
    "aside",
    "blockquote",
    "body",
    "br",
    "caption",
    "dd",
    "details",
    "div",
    "dl",
    "dt",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "li",
    "main",
    "nav",
    "ol",
    "p",
    "section",
    "summary",
    "table",
    "td",
    "th",
];

/// 文字参照を解く。**知らない参照は、そのまま残す。**
fn decode(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let Some(end) = tail.find(';').filter(|e| *e <= 10) else {
            out.push('&');
            rest = &tail[1..];
            continue;
        };
        let name = &tail[1..end];
        let ch = match name {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some(' '),
            _ => name
                .strip_prefix("#x")
                .or_else(|| name.strip_prefix("#X"))
                .and_then(|h| u32::from_str_radix(h, 16).ok())
                .or_else(|| name.strip_prefix('#').and_then(|d| d.parse().ok()))
                .and_then(char::from_u32),
        };
        match ch {
            Some(c) => {
                out.push(c);
                rest = &tail[end + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// 組み立て中の単位。
#[derive(Default)]
struct Pending {
    line: usize,
    /// 記号で囲んだコードを含む中身。
    raw: String,
    /// コードの外に文字が在るか。
    prose: bool,
    /// コードを含むか。
    code: bool,
}

/// いま開いている要素から、単位の種類を決める。**内側の要素が優先する。**
fn kind_of(open: &[String]) -> (Kind, usize) {
    for name in open.iter().rev() {
        match name.as_str() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                return (Kind::Heading, usize::from(name.as_bytes()[1] - b'0'));
            }
            "li" | "dt" | "dd" => return (Kind::Item, 0),
            "td" | "th" => return (Kind::Cell, 0),
            "blockquote" => return (Kind::Quote, 0),
            _ => {}
        }
    }
    (Kind::Body, 0)
}

fn flush(units: &mut Vec<Unit>, p: &mut Pending, open: &[String]) {
    let taken = std::mem::take(p);
    let raw = decode(taken.raw.trim());
    if raw.is_empty() {
        return;
    }
    let (kind, level) = if taken.code && !taken.prose {
        (Kind::Code, 0)
    } else {
        kind_of(open)
    };
    let text = if kind == Kind::Code {
        raw.trim_matches('`').to_owned()
    } else {
        plain(&raw)
    };
    units.push(Unit {
        kind,
        line: taken.line,
        raw,
        text,
        level,
        indent: 0,
    });
}

/// HTML の頁を単位へ切る。**タグではなく要素で区切り、タグを本文として読まない。**
///
/// Markdown として読むと、タグの文字が本文に混ざる ── 閉じの `**` の直後の `<` を文字と数えて
/// 強調の判定が誤り、表に並べたコードの行を本文として判定する。コードの要素（pre ・ code）の中は
/// 記号で囲んだ中身と同じ扱いにし、中身が全部コードの単位はコードの単位にする。
#[must_use]
pub fn split_html(text: &str) -> Vec<Unit> {
    let mut units = Vec::new();
    let mut open: Vec<String> = Vec::new();
    let mut p = Pending::default();
    let mut code_depth = 0usize;
    let mut line = 1usize;
    let mut rest = text;
    while !rest.is_empty() {
        let Some(lt) = rest.find('<') else {
            for c in rest.chars() {
                if c == '\n' {
                    line += 1;
                }
            }
            push_text(&mut p, rest, line, code_depth > 0);
            break;
        };
        let (head, tail) = rest.split_at(lt);
        let mut at = line;
        for piece in head.split_inclusive('\n') {
            push_text(&mut p, piece, at, code_depth > 0);
            if piece.ends_with('\n') {
                at += 1;
            }
        }
        line = at;
        // 注釈と宣言は読まない
        let skip_to = if tail.starts_with("<!--") { "-->" } else { ">" };
        let Some(gt) = tail.find(skip_to) else {
            break;
        };
        let tag = &tail[..gt + skip_to.len()];
        line += tag.matches('\n').count();
        rest = &tail[gt + skip_to.len()..];
        if tag.starts_with("<!") || tag.starts_with("<?") {
            continue;
        }
        let closing = tag.starts_with("</");
        let name: String = tag
            .trim_start_matches('<')
            .trim_start_matches('/')
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        if !closing && SKIPPED.contains(&name.as_str()) {
            // **閉じまで読み飛ばす** ── 中身の行も数える
            let close = format!("</{name}");
            let end = rest.to_ascii_lowercase().find(&close).unwrap_or(rest.len());
            line += rest[..end].matches('\n').count();
            rest = &rest[end..];
            continue;
        }
        if CODE.contains(&name.as_str()) {
            if closing {
                code_depth = code_depth.saturating_sub(1);
                p.raw.push('`');
            } else {
                code_depth += 1;
                p.raw.push('`');
            }
            continue;
        }
        if BLOCKS.contains(&name.as_str()) {
            flush(&mut units, &mut p, &open);
            if closing {
                if let Some(i) = open.iter().rposition(|n| *n == name) {
                    open.truncate(i);
                }
            } else if !matches!(name.as_str(), "br" | "hr") && !tag.ends_with("/>") {
                open.push(name);
            }
        }
    }
    flush(&mut units, &mut p, &open);
    units
}

fn push_text(p: &mut Pending, piece: &str, line: usize, in_code: bool) {
    if piece.trim().is_empty() {
        if !p.raw.is_empty() {
            p.raw.push(' ');
        }
        return;
    }
    if !p.prose && !p.code {
        p.line = line;
    }
    if in_code {
        p.code = true;
    } else {
        p.prose = true;
    }
    p.raw.push_str(piece.trim_end_matches('\n'));
}
