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
