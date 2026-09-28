// SPDX-License-Identifier: MIT
//! 依存の向きを検査する。**この Skill の道具である** ── 根幹3つのうち1つの実装で、
//! 他に使い手が居ない。リポジトリ側へ置くと、この Skill を別のリポジトリへ持って
//! いったとき、向きを検査できない。
//!
//! **解析器を自作しない。** 文法は tree-sitter が持つ ── 言語ごとに文法を追う保守は
//! 成立しない（実測 ── 5言語ぶんを1つの走査で試作したところ、相対の参照と文字列に
//! よる読み込みを取りこぼした）。
//!
//! **外の道具を呼ばない。** 文法は binary へ焼き込むので、検査する側にその言語の
//! 道具が入っていなくても測れる。
//!
//! **抽出器は3つを返す** ── 依存 ／ 静的に追跡できない読み込み ／ 判定できなかった範囲。
//! **判定できなかったことを、合格に寄せない。**

use std::path::Path;

use self::judge::Edge;

pub mod judge;
pub mod names;
pub mod syntax;

/// 図に現れない依存の経路。**そこを通れば検査を素通りできる。**
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Escape {
    /// どのモジュールに在るか（層を決めるために使う）。
    pub in_point: String,
    /// どこに書かれているか。
    pub at: String,
    /// 何を使って外へ出ているか。
    pub how: String,
}

impl Escape {
    /// 静的に追跡できない読み込みを組む。
    #[must_use]
    pub const fn new(in_point: String, at: String, how: String) -> Self {
        Self { in_point, at, how }
    }
}

/// 抽出の結果。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Extracted {
    /// 取れた依存。
    pub edges: Vec<Edge>,
    /// 図に現れない経路。
    pub escapes: Vec<Escape>,
    /// 判定できなかった範囲。**0件でなければ、0件を結論にしない。**
    pub undecided: Vec<String>,
    /// 作業領域の中のモジュール（ファイルごとの、参照する側の名前）。**参照が作業領域の中を
    /// 指すかを判定するために使う。**
    pub points: Vec<String>,
    /// 読めなかった設定。**（その設定が効くディレクトリ, 文面）** ── 層に属すモジュールがその下に在るときだけ、
    /// 判定できなかった範囲になる。
    pub unreadable: Vec<(String, String)>,
    /// 測り方の限界。**申告であって、判定できなかった事実ではない** ── 採用範囲へ
    /// 書くものなので、検出には数えない。
    pub limits: Vec<String>,
}

/// その言語で依存を取る抽出器。**1言語につき1つ。**
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

    /// 依存を取る。
    ///
    /// # Errors
    ///
    /// 道具の起動が「見つからない」以外の理由で失敗したときに返す。
    fn extract(&self, root: &Path) -> std::io::Result<Extracted>;
}
