// SPDX-License-Identifier: MIT
//! 人が読む文書へ当てる8つの判定。
//!
//! **概念から導けるものと、媒体の決めを分ける。** どちらかが読めないと、直すべきか
//! 外すべきかを判定できない。
//!
//! **語の一覧を、この側に書かない** ── 和語は `references/predicates.json`、同義語と
//! 廃語は呼ぶ側が渡す。語を1つ足すたびに組み直すことになる。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use fancy_regex::Regex;

use crate::ending::{self, Ending};
use crate::finding::Finding;
use crate::unit::{Kind, Unit};

/// 言い換えの対。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Pair {
    /// 探す語。
    pub word: String,
    /// 言い換える先。
    pub use_instead: String,
    /// いつ、なぜ決めたか。
    pub whence: String,
    /// 活用形まで探す形。**無ければ `word` の文字列で探す。**
    pub pattern: Option<Regex>,
}

impl Pair {
    /// 対を組む。
    #[must_use]
    pub const fn new(word: String, use_instead: String, whence: String) -> Self {
        Self {
            word,
            use_instead,
            whence,
            pattern: None,
        }
    }

    /// 活用形まで探す形を持たせる。**動詞は終止形の文字列だけでは、〜ます の形が通過する。**
    #[must_use]
    pub fn with_pattern(mut self, pattern: Regex) -> Self {
        self.pattern = Some(pattern);
        self
    }
}

/// 鉤括弧の中を伏せる。**位置は変えない** ── 伏せたあとの位置で、原文から抜き出す。
fn mask_quotes(text: &str) -> String {
    let mut out = String::new();
    let mut depth = 0_i32;
    for c in text.chars() {
        if c == '「' {
            depth += 1;
        }
        out.push(if depth > 0 { '＿' } else { c });
        if c == '」' {
            depth = (depth - 1).max(0);
        }
    }
    out
}

/// 判定に渡す語の一覧。**この crate は一覧を持たない。**
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Words {
    /// 和語の述部と、その言い換え先。
    pub predicates: Vec<(String, String)>,
    /// 同じ意味の語の対。
    pub synonyms: Vec<(String, String)>,
    /// 一度破棄した語。
    pub retired: Vec<Pair>,
}

impl Words {
    /// 語の一覧を組む。**欄を足しても、呼ぶ側は壊れない。**
    #[must_use]
    pub const fn new(
        predicates: Vec<(String, String)>,
        synonyms: Vec<(String, String)>,
        retired: Vec<Pair>,
    ) -> Self {
        Self {
            predicates,
            synonyms,
            retired,
        }
    }
}

fn cut(text: &str, from: usize, len: usize) -> String {
    text.chars().skip(from).take(len).collect()
}

/// 抜粋を組む。**読み手がその場所を開ける粒度にする。**
///
/// 前をどれだけ取るかは判定ごとに違う ── 廃語は語だけを示せばよく、述部は文脈が要る。
fn excerpt_around(text: &str, at: usize, hit: &str, tail: &str, before: usize) -> String {
    let head: String = text
        .chars()
        .take(at)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .take(before)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("…{head}<{hit}>{tail}…")
}

// ── 概念4 ──────────────────────────────────────────────────

/// 見出しの階層を飛ばしていないかを見る。
///
/// **飛ばすと、間に何が在るはずだったかが分からない。**
#[must_use]
pub fn heading_skip(units: &[Unit]) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut prev = 0;
    for u in units.iter().filter(|u| u.kind == Kind::Heading) {
        if prev > 0 && u.level > prev + 1 {
            out.push(Finding::new(
                "見出しの階層が飛んでいる",
                "概念4",
                u.line,
                format!(
                    "見出し{prev} の次に 見出し{}：{}",
                    u.level,
                    cut(&u.text, 0, 24)
                ),
            ));
        }
        prev = u.level;
    }
    out
}

// ── 概念6 ──────────────────────────────────────────────────

/// 文へ切る。**鉤括弧の中の句点では切らない** ── 切ると、引用の文末（「〜ました。」）が
/// 書き手の文末として数えられる。
fn sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut depth = 0_i32;
    for c in text.chars() {
        buf.push(c);
        match c {
            '「' => depth += 1,
            '」' => depth = (depth - 1).max(0),
            '。' if depth == 0 => out.push(std::mem::take(&mut buf)),
            _ => {}
        }
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out
}

/// 文体が混ざっていないかを見る。
///
/// **引用は数えない。** 引用は書き手の文体ではなく、他人の文である ── 統一しようと
/// すれば原文を書き換えることになり、引用でなくなる。鉤括弧の中も同じ扱いにする。
#[must_use]
pub fn mixed_style(units: &[Unit]) -> Vec<Finding> {
    let mut count: BTreeMap<Ending, usize> = BTreeMap::new();
    let mut where_: BTreeMap<Ending, (usize, String)> = BTreeMap::new();
    for u in units
        .iter()
        .filter(|u| u.kind.prose() && u.kind != Kind::Quote)
    {
        // 鉤括弧は文をまたぐので、深さを持って追う
        let mut depth: i32 = 0;
        for s in sentences(&u.text) {
            let s = s.trim();
            let inside = depth > 0 || s.starts_with('「');
            depth += i32::try_from(s.matches('「').count()).unwrap_or(0)
                - i32::try_from(s.matches('」').count()).unwrap_or(0);
            if !s.ends_with('。') || inside {
                continue;
            }
            let e = ending::of(s);
            // **体言止めは、常体とも敬体とも決められない**
            if e == Ending::Plain && !ending::plain_form().is_match(s).unwrap_or(false) {
                continue;
            }
            *count.entry(e).or_default() += 1;
            where_.entry(e).or_insert_with(|| (u.line, cut(s, 0, 26)));
        }
    }
    if count.len() < 2 {
        return Vec::new();
    }
    let (Some(a), Some(b)) = (where_.get(&Ending::Polite), where_.get(&Ending::Plain)) else {
        return Vec::new();
    };
    vec![Finding::new(
        "文体が混ざっている",
        "概念6",
        a.0.min(b.0),
        format!(
            "敬体 {} 文（{}行「{}」）と 常体 {} 文（{}行「{}」）",
            count.get(&Ending::Polite).copied().unwrap_or(0),
            a.0,
            a.1,
            count.get(&Ending::Plain).copied().unwrap_or(0),
            b.0,
            b.1
        ),
    )]
}

/// 並んだ項目の語尾が統一されているかを見る。
///
/// **同じ立場のものは、同じ形で書く。** 形が違うと、対応が読めない。
#[must_use]
pub fn unparallel_items(units: &[Unit]) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut group: Vec<&Unit> = Vec::new();
    let mut prev_line: i64 = -9;

    fn flush(group: &[&Unit], out: &mut Vec<Finding>) {
        if group.len() < 2 {
            return;
        }
        let mut seen: BTreeMap<Ending, &Unit> = BTreeMap::new();
        for u in group {
            seen.entry(ending::of(&u.text)).or_insert(u);
        }
        if seen.len() > 1 {
            let names: Vec<String> = seen
                .iter()
                .map(|(k, v)| format!("{}（{}行）", k.label(), v.line))
                .collect();
            out.push(Finding::new(
                "並んだ項目の語尾が統一されていない",
                "概念6",
                group[0].line,
                format!("{} 項目に {} が混ざる", group.len(), names.join(" と ")),
            ));
        }
    }

    for u in units {
        let joins = u.kind == Kind::Item
            && (group.is_empty()
                || (i64::try_from(u.line).unwrap_or(0) - prev_line <= 1
                    && u.indent == group[0].indent));
        if joins {
            group.push(u);
        } else {
            flush(&group, &mut out);
            group = if u.kind == Kind::Item {
                vec![u]
            } else {
                Vec::new()
            };
        }
        if u.kind == Kind::Item {
            prev_line = i64::try_from(u.line).unwrap_or(0);
        }
    }
    flush(&group, &mut out);
    out
}

// ── 概念2 ──────────────────────────────────────────────────

fn box_corner() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"[┌┐┘┏┓┛╭╮╯┬┴┼┤]").expect("組める"))
}
fn box_rule() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"[─━]{4,}").expect("組める"))
}

/// 文字で図や表を描いていないかを見る。
///
/// **図と表は、図と表の形で置く。** 箱の角と、横罫の連なりを検出する ──
/// 二倍ダッシュと置き場所の一覧は図ではない。
#[must_use]
pub fn drawn_figure(u: &Unit) -> Vec<String> {
    if box_corner().is_match(&u.raw).unwrap_or(false)
        || box_rule().is_match(&u.raw).unwrap_or(false)
    {
        vec![cut(u.raw.trim(), 0, 28)]
    } else {
        Vec::new()
    }
}

// ── 概念7 ──────────────────────────────────────────────────

/// 記号で囲んだ中身を外す。**閉じの無いものは、そのまま残す。**
#[must_use]
pub fn without_code(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    loop {
        let Some(open) = rest.find('`') else {
            out.push_str(rest);
            return out;
        };
        let (head, tail) = rest.split_at(open);
        out.push_str(head);
        let body = &tail[1..];
        match body.find('`') {
            Some(close) => rest = &body[close + 1..],
            None => {
                out.push_str(tail);
                return out;
            }
        }
    }
}

/// 同じ意味の語が2つ在るかを見る。
///
/// **記号で囲んだ中は検査しない。** そこに在るのは識別子と原文の引用であり、書き手の
/// 言葉づかいではない ── 言い換えれば、その名前で参照するものが壊れる。
#[must_use]
pub fn synonym(units: &[Unit], words: &Words) -> Vec<Finding> {
    let body: String = units
        .iter()
        .filter(|u| u.kind != Kind::Code)
        .map(|u| without_code(&u.raw))
        .collect::<Vec<_>>()
        .join("\n");
    words
        .synonyms
        .iter()
        .filter(|(a, b)| body.contains(a.as_str()) && body.contains(b.as_str()))
        .map(|(a, b)| {
            Finding::new(
                "同じ意味の語が2つある",
                "概念7",
                0,
                format!("「{a}」と「{b}」"),
            )
        })
        .collect()
}

/// 一度破棄した語を、また使用していないかを見る。
///
/// **この判定は語の一覧を持たない。** 廃語も、いつ破棄したかもプロジェクトごとに
/// 相違する ── 一覧が渡されなければ、何も出ない。
#[must_use]
pub fn retired_word(u: &Unit, words: &Words) -> Vec<String> {
    // **鉤括弧の中は検査しない** ── 廃語を引用して説明する文は、廃語を使用していない
    let masked = mask_quotes(&u.text);
    let mut out = Vec::new();
    for pair in &words.retired {
        let found = match &pair.pattern {
            Some(rx) => rx
                .find(&masked)
                .ok()
                .flatten()
                .map(|m| (m.start(), m.as_str().chars().count())),
            None => masked
                .find(&pair.word)
                .map(|b| (b, pair.word.chars().count())),
        };
        if let Some((byte, len)) = found {
            let at = masked[..byte].chars().count();
            let hit: String = u.text.chars().skip(at).take(len).collect();
            let after: String = u.text.chars().skip(at + len).take(8).collect();
            out.push(format!(
                "{} → {}（{}）",
                excerpt_around(&u.text, at, &hit, &after, 10),
                pair.use_instead,
                pair.whence
            ));
        }
    }
    out
}

// ── 概念8 ──────────────────────────────────────────────────

/// 述部が和語かを見る。
///
/// 訓読みの動詞は意味の範囲が広い（公用文 Ⅲ－４ エ のただし書き）。技術文書は厳密に
/// 意味を特定しなければならない文書なので、ウ を必須として適用する ── **原典は禁止
/// していない。強度を上げたのは、このリポジトリの決定である。**
///
/// **引用は検査しない** ── 原文の形を変えないと決めている。原典の語を言い換えた
/// 時点で、それは引用ではなくなる。
#[must_use]
pub fn wago_predicate(u: &Unit, words: &Words) -> Vec<String> {
    static COMPILED: OnceLock<Vec<(Regex, String)>> = OnceLock::new();
    let patterns = COMPILED.get_or_init(|| {
        words
            .predicates
            .iter()
            .filter_map(|(p, to)| Regex::new(p).ok().map(|r| (r, to.clone())))
            .collect()
    });
    // 鉤括弧の中を伏せる ── 廃語の照合と同じ処理を使う
    let masked = mask_quotes(&u.text);
    let mut hits = Vec::new();
    for (rx, to) in patterns {
        if let Ok(Some(m)) = rx.find(&masked) {
            let at = masked[..m.start()].chars().count();
            let hit: String = u
                .text
                .chars()
                .skip(at)
                .take(m.as_str().chars().count())
                .collect();
            let after: String = u
                .text
                .chars()
                .skip(at + m.as_str().chars().count())
                .take(8)
                .collect();
            hits.push(format!(
                "{} → {to}",
                excerpt_around(&u.text, at, &hit, &after, 12)
            ));
        }
    }
    hits
}

// ── 媒体の決め ──────────────────────────────────────────────

/// 日本語の約物。**閉じの印がこの直後に在ると、記法として閉じない。**
const JA_PUNCT: [char; 8] = ['。', '、', '）', '」', '・', '：', '；', '！'];

/// 強調が描画されるかを見る。
///
/// **概念からは導けない。** CommonMark の記法に拠る ── 閉じの `**` が約物の直後に
/// 在ると、閉じ記号として読まれない。
#[must_use]
pub fn broken_emphasis(u: &Unit) -> Vec<String> {
    let parts: Vec<&str> = u.raw.split("**").collect();
    let mut hits = Vec::new();
    // 1つ目の区切りが開き、2つ目が閉じ ── 閉じの前後だけを見る
    for k in (2..parts.len()).step_by(2) {
        let prev = parts[k - 1];
        let next = parts[k];
        let Some(last) = prev.chars().last() else {
            continue;
        };
        let Some(first) = next.chars().next() else {
            continue;
        };
        if JA_PUNCT.contains(&last) && !JA_PUNCT.contains(&first) && !first.is_whitespace() {
            let head: String = prev
                .chars()
                .rev()
                .take(14)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let tail: String = next.chars().take(10).collect();
            hits.push(format!("…{head}**{tail}…"));
        }
    }
    hits
}
