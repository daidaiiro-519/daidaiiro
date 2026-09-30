// SPDX-License-Identifier: MIT
//! 文字幅の見積り ── 部品とラベルの両方が使うので、ここへ独立させる。
//!
//! **この見積りは必ず実物以上でなければならない。** 用途は「場所を取り置くこと」（箱の幅 ・
//! 欄の幅 ・ 重なりの判定）で、少なく見積もれば文字が枠から出る。だから比は平均ではなく、
//! 実測した中の最大を使う。

/// CJK の文字が始まる符号位置（Unicode が決めている境目。選んだ値ではない）。
const CJK_START: u32 = 0x2E80;

/// テーマから引いた既定の比。**同じ数を2か所に書かない** ── 実測 ── 0.58 が2か所に在り、
/// 全角大文字を10%見誤っていた。
#[must_use]
pub fn latin_ratio() -> f64 {
    // **既定値をここに書かない** ── 書くと、正本と同じ数が2か所になる。正本は組み立ての時点で
    // 埋め込むので、欠けていれば組み立てた者の誤りである
    crate::theme::num(crate::theme::default_theme(), "font.latin-width-ratio")
        .expect("theme.json は font.latin-width-ratio を持つ")
}

/// CJK は全角、それ以外は半角相当として幅を見積もる。
#[must_use]
pub fn width_with(s: &str, size: f64, latin: f64) -> f64 {
    // **移す前の足し方で足す** ── 順に足すと、最後の桁が違う幅が出る
    crate::py::sum(s.chars().map(|c| {
        if c as u32 > CJK_START {
            size
        } else {
            size * latin
        }
    }))
}

/// 既定の比で幅を見積もる。
#[must_use]
pub fn width(s: &str, size: f64) -> f64 {
    width_with(s, size, latin_ratio())
}

/// 文字が並ぶ欄の幅を、実際に入る文字から決める。**空なら余白だけ。**
///
/// 欄の幅を決め打ちにすると、中身が短いときは無駄な空白が空き、長いときははみ出す ── 同じ
/// 「文字の欄」が5か所で別々の定数を持っていたので、決め方をここへ1つにまとめる。
#[must_use]
pub fn column(texts: &[String], size: f64, pad: f64) -> f64 {
    let widest = texts.iter().map(|t| width(t, size)).fold(0.0_f64, f64::max);
    widest + pad
}

/// 行頭に置かない文字（行頭の禁則）。閉じ括弧 ・ 句読点 ・ 長音
const NO_LINE_START: &str = "、。，．・：；？！）」』】〕〉》ー’”)]},.:;!?";

/// 行末に置かない文字（行末の禁則）。開き括弧
const NO_LINE_END: &str = "（「『【〔〈《‘“([{";

/// 折り返しの単位へ分ける。**英数字の続きは1つの単位にする**（語の途中で折り返さない）。
/// 行頭の禁則の文字は前の単位へ、行末の禁則の文字は後ろの単位へ付ける。
fn units(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut hold = String::new();
    for c in s.chars() {
        let word = c.is_ascii_alphanumeric() || c == '_' || c == '-';
        let glue_prev = NO_LINE_START.contains(c)
            || (word
                && out
                    .last()
                    .and_then(|u| u.chars().last())
                    .is_some_and(|l| l.is_ascii_alphanumeric() || l == '_' || l == '-')
                && hold.is_empty());
        if glue_prev && !out.is_empty() && hold.is_empty() {
            out.last_mut().expect("在る").push(c);
            continue;
        }
        hold.push(c);
        if NO_LINE_END.contains(c) {
            continue;
        }
        out.push(std::mem::take(&mut hold));
    }
    if !hold.is_empty() {
        out.push(hold);
    }
    out
}

/// 幅の上限までで、貪欲に折り返す。
fn greedy(parts: &[String], size: f64, latin: f64, max: f64) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for u in parts {
        let next = format!("{cur}{u}");
        if !cur.is_empty() && width_with(&next, size, latin) > max {
            lines.push(std::mem::take(&mut cur));
            cur = u.trim_start().to_owned();
            continue;
        }
        cur = next;
    }
    if !cur.is_empty() || lines.is_empty() {
        lines.push(cur);
    }
    lines
}

/// 幅の上限までで折り返した行の並び。**単位（英数字の語 ・ 禁則でつないだ文字）の途中では分けない**
/// （単位が上限をわずかに超えることを許す ── 語の途中や行頭の句読点よりも読みやすい）。
/// **行数を変えない範囲で幅を詰め、各行の長さをそろえる** ── 最後の行に1文字だけ残ると読みにくい。
#[must_use]
pub fn wrap(s: &str, size: f64, latin: f64, max: f64) -> Vec<String> {
    let parts = units(s);
    let lines = greedy(&parts, size, latin, max);
    if lines.len() < 2 {
        return lines;
    }
    let n = lines.len();
    let total = width_with(s, size, latin);
    let (mut lo, mut hi) = (total / n as f64, max);
    // 文字1つぶんの幅まで詰めれば十分である
    while hi - lo > size {
        let mid = (lo + hi) / 2.0;
        if greedy(&parts, size, latin, mid).len() == n {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    greedy(&parts, size, latin, hi)
}
