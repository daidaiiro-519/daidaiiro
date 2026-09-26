// SPDX-License-Identifier: MIT
//! 節の構成が、対応する雛形を満たすかを見る。
//!
//! **節の名前は、雛形が持つ。** この側に一覧を書かない ── 書くと、雛形を直した瞬間に
//! 食い違う。
//!
//! **並び順は問わない。** 見るのは有無だけである ── 順序まで固定すると、節を1つ足した
//! Skill が全部不合格になる。

/// `## ` の節の名前を、並びのまま返す。
#[must_use]
pub fn headings(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.strip_prefix("## "))
        .map(|name| name.trim_end().to_owned())
        .filter(|name| !name.is_empty())
        .collect()
}

/// 差し込む場所を持つ名前かを返す ── 雛形の `{{…}}` は名前ではない。
#[must_use]
fn is_placeholder(name: &str) -> bool {
    match name.find("{{") {
        Some(i) => name[i + 2..].contains("}}"),
        None => false,
    }
}

/// 雛形に在って、文書に無い節を返す。
///
/// **差し込む場所を持つ節は要求しない。**
#[must_use]
pub fn missing(document: &str, template: &str) -> Vec<String> {
    let present = headings(document);
    headings(template)
        .into_iter()
        .filter(|name| !is_placeholder(name))
        .filter(|name| !present.contains(name))
        .collect()
}
