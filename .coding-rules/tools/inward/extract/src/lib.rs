// SPDX-License-Identifier: MIT
//! 辺の抽出。**言語ごとに1つで、その言語自身の道具に出させる。**
//!
//! **解析器を自作しない。** 言語ごとに文法を追う保守は成立しない ── 実測では、
//! 5言語ぶんを1つの走査で試作したところ、相対の参照と文字列による読み込みを
//! 取りこぼした。読んだ OSS（grimp）も、1言語だけで走査に1ファイルを使っている。
//!
//! **抽出器は3つを返す** ── 辺 ／ 抜け道 ／ 判定できなかった範囲。
//! **判定できなかったことを、合格に寄せない。**

#![forbid(unsafe_code)]

use std::path::Path;

use inward_core::Edge;

pub mod cpp;
pub mod jvm;
pub mod manifest;
pub mod probe;
pub mod scripted;
pub mod syntax;
pub mod tool;

/// 図に現れない依存の経路。**そこを通れば検査を素通りできる。**
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Escape {
    /// どの点に在るか（層を決めるために要る）。
    pub in_point: String,
    /// どこに書かれているか。
    pub at: String,
    /// 何を使って外へ出ているか。
    pub how: String,
}

impl Escape {
    /// 抜け道を組む。
    #[must_use]
    pub const fn new(in_point: String, at: String, how: String) -> Self {
        Self { in_point, at, how }
    }
}

/// 抽出の結果。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Extracted {
    /// 取れた辺。
    pub edges: Vec<Edge>,
    /// 図に現れない経路。
    pub escapes: Vec<Escape>,
    /// 判定できなかった範囲。**0件でなければ、0件を結論にしない。**
    pub undecided: Vec<String>,
    /// 測り方の限界。**申告であって、判定できなかった事実ではない** ── 採用範囲へ
    /// 書くものなので、検出には数えない。
    pub limits: Vec<String>,
}

/// その言語で辺を取る手。**1言語につき1つ。**
///
/// 実装は次の3つを守る。
///
/// - **その言語の一級の道具に出させる。** 自分で構文を解析しない
/// - **道具が無ければ `undecided` へ入れる。** 空の `edges` を返して合格にしない
/// - **図に現れない経路を `escapes` へ入れる。** 静的には行き先が分からないので、
///   存在だけを報告する
pub trait Extractor {
    /// この抽出器が扱う言語の名前。**機械が分岐する値なので ASCII である。**
    fn language(&self) -> &'static str;

    /// その言語の道具が使えるかを返す。
    fn available(&self) -> bool;

    /// 辺を取る。
    ///
    /// # Errors
    ///
    /// 道具の起動が「見つからない」以外の理由で失敗したときに返す。
    fn extract(&self, root: &Path) -> std::io::Result<Extracted>;
}
