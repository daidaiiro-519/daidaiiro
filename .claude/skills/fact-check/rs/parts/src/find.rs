// SPDX-License-Identifier: MIT
//! 原文の文字列で照合する。**3つの種類を分ける。**
//!
//! | 種類 | 何を照合するか |
//! |---|---|
//! | 識別子 | **語として照合する。** 前後が語の文字なら、それは別の名前である |
//! | 引用 | **引用として照合する。** 空白の圧縮規則だけを統一し、語は1文字も変えない |
//! | そのまま | **探索のための種類である。** 照合した証しにはならない |
//!
//! **種類を渡さなければ止まる** ── 道具の側で「たぶん識別子だろう」と決めない。
//!
//! **位置は文字で数える。** バイトで数えると、日本語を含む原文で読み手の見る位置と
//! 食い違う ── 報告を読んだ人は、その位置を開いて確認する。

/// 照合の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum How {
    /// 語として照合する。
    Identifier,
    /// 引用として照合する。
    Quote,
    /// そのまま探す。
    Text,
}

impl How {
    /// 機械が分岐する値。
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Identifier => "identifier",
            Self::Quote => "quote",
            Self::Text => "text",
        }
    }

    /// 名前から決める。**知らない種類は返さない。**
    #[must_use]
    pub fn of(key: &str) -> Option<Self> {
        match key {
            "identifier" => Some(Self::Identifier),
            "quote" => Some(Self::Quote),
            "text" => Some(Self::Text),
            _ => None,
        }
    }

    /// 扱える種類を並べる。
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::Identifier, Self::Quote, Self::Text]
    }
}

/// 語の文字。英数と、識別子で必ず語の内側になる区切り。
fn always_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// 語の区切りを決める。
///
/// **それ以外の区切り（`.` `/` `:` `~` など）は、照合する語自身が含んでいるときだけ
/// 内側とする** ── 語は、自分の記法を自分の中に持っている。
fn is_word_char(c: char, needle: &str) -> bool {
    always_word(c) || (!c.is_whitespace() && !c.is_alphanumeric() && needle.contains(c))
}

/// 日本語の文字か。**日本語は、行の折り返しに空白を持たない。**
#[must_use]
pub fn cjk(c: char) -> bool {
    let o = c as u32;
    (0x3000..=0x30FF).contains(&o)
        || (0x3400..=0x4DBF).contains(&o)
        || (0x4E00..=0x9FFF).contains(&o)
        || (0xF900..=0xFAFF).contains(&o)
        || (0xFF00..=0xFF60).contains(&o)
}

/// 語として照合する。**前後が語の文字なら、それは別の名前である。**
#[must_use]
pub fn identifier(text: &str, needle: &str) -> Vec<usize> {
    let mut out = Vec::new();
    if needle.is_empty() {
        return out;
    }
    let mut from = 0;
    while let Some(rel) = text[from..].find(needle) {
        let at = from + rel;
        let before = text[..at].chars().next_back();
        let after = text[at + needle.len()..].chars().next();
        let blocked = before.is_some_and(|c| is_word_char(c, needle))
            || after.is_some_and(|c| is_word_char(c, needle));
        if !blocked {
            out.push(text[..at].chars().count());
        }
        from = at + text[at..].chars().next().map_or(1, char::len_utf8);
    }
    out
}

/// 空白の連なりを圧縮し、圧縮した先から元の位置へ戻れるようにする。
///
/// **圧縮した先は、前後の文字で決まる。** 日本語どうしの間なら**無**に、そうで
/// なければ**空白1つ**に圧縮する ── 原文が行で折り返されているだけの箇所に、空白を
/// 作り出さないためである。英語は語の切れ目に空白を持つので、逆に空白1つが要る。
#[must_use]
pub fn fold(text: &str) -> (String, Vec<usize>) {
    let mut out = String::new();
    let mut index = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let (at, c) = (i, chars[i]);
        if !c.is_whitespace() {
            out.push(c);
            index.push(at);
            i += 1;
            continue;
        }
        let mut j = i;
        while j < chars.len() && chars[j].is_whitespace() {
            j += 1;
        }
        let before = out.chars().next_back();
        let after = chars.get(j).copied();
        let both_cjk = before.is_some_and(cjk) && after.is_some_and(cjk);
        if before.is_some() && after.is_some() && !both_cjk {
            out.push(' ');
            index.push(at);
        }
        i = j;
    }
    (out, index)
}

/// 引用として照合する。**空白の圧縮規則だけを統一し、語は1文字も変えない。**
#[must_use]
pub fn quote(text: &str, needle: &str) -> Vec<usize> {
    let (folded, index) = fold(text);
    let wanted = fold(needle).0.trim().to_owned();
    if wanted.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = folded[from..].find(&wanted) {
        let at = from + rel;
        // 圧縮した先の位置を、元の位置へ戻す
        let nth = folded[..at].chars().count();
        if let Some(original) = index.get(nth) {
            out.push(*original);
        }
        from = at + folded[at..].chars().next().map_or(1, char::len_utf8);
    }
    out
}

/// そのまま探す。**探索のための種類である。** 照合した証しにはならない。
#[must_use]
pub fn text(body: &str, needle: &str) -> Vec<usize> {
    let mut out = Vec::new();
    if needle.is_empty() {
        return out;
    }
    let mut from = 0;
    while let Some(rel) = body[from..].find(needle) {
        let at = from + rel;
        out.push(body[..at].chars().count());
        from = at + body[at..].chars().next().map_or(1, char::len_utf8);
    }
    out
}

/// 種類に応じて照合する。
#[must_use]
pub fn find(how: How, body: &str, needle: &str) -> Vec<usize> {
    match how {
        How::Identifier => identifier(body, needle),
        How::Quote => quote(body, needle),
        How::Text => text(body, needle),
    }
}
