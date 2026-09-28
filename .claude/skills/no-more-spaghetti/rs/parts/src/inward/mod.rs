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

use self::judge::{judge, layer_for, unresolved, Edge, Order, Unresolved};
use self::syntax::Tree;

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

/// 測った結果。**検出と、測り方の限界を分けて持つ** ── 混ぜると、解決しなかった参照が
/// 違反として読まれる。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Measured {
    /// 言語の名前。
    pub language: &'static str,
    /// 食い違い。**0件のときだけ合格である。**
    pub findings: Vec<String>,
    /// 取れた依存。
    pub edges: Vec<Edge>,
    /// 参照する側と参照される側の両方が層に属す依存の数。
    pub layered_edges: usize,
    /// 測り方の限界。
    pub limits: Vec<String>,
}

/// 1つの成果物の依存の向きを測る。**CLI と、規則の実行の両方がここを呼ぶ** ──
/// 2か所で組むと、報告する条件が片方だけ変わる。
///
/// # Errors
///
/// その言語の抽出器が無いとき、根が無いとき、層が空のとき、抽出できないときに返す。
pub fn measure(language: &str, root: &Path, layers: Vec<judge::Layer>) -> Result<Measured, String> {
    if layers.is_empty() {
        return Err("層を1つも渡していない".to_owned());
    }
    let Some(tree) = Tree::of(language) else {
        return Err(format!(
            "その言語の抽出器が無い ── {language}（在るのは {}）",
            Tree::languages().join(" ・ ")
        ));
    };
    if !root.is_dir() {
        return Err(format!("根が無い ── {}", root.display()));
    }
    let got = tree
        .extract(root)
        .map_err(|e| format!("抽出できない ── {e}"))?;
    let order = Order::inner_to_outer(layers);
    let mut findings: Vec<String> = judge(&order, &got.edges)
        .iter()
        .map(|v| format!("{} ── {} → {}（{}）", v.at, v.from, v.to, v.because.label()))
        .collect();
    // **静的に追跡できない読み込みは、参照元が層に属すときだけ出す** ── 層の外のファイルは
    // 向きの規則を課されない（実測 ── 違反を意図して含む試験データを検出した）。
    // **合成する層のものも、食い違いにしない** ── 向きの規則が唯一成立しない場所を、
    // 最も外側の1か所へ集約してある
    findings.extend(
        got.escapes
            .iter()
            .filter(|e| layer_for(&order, &e.in_point).is_some_and(|l| !l.composes))
            .map(|e| format!("{} ── {}", e.at, e.how)),
    );
    findings.extend(
        got.undecided
            .iter()
            .map(|u| format!("判定できていない ── {u}")),
    );
    // **読めなかった設定は、層に属すモジュールがその下に在るときだけ出す** ── 層の外の設定は、
    // 照合に使われない（実測 ── 例のディレクトリの生成物を指す tsconfig）
    findings.extend(
        got.unreadable
            .iter()
            .filter(|(dir, _)| {
                got.points.iter().any(|p| {
                    (dir.is_empty()
                        || p.starts_with(&format!("{dir}/"))
                        || p.starts_with(&format!("{}.", dir.replace('/', "."))))
                        && layer_for(&order, p).is_some()
                })
            })
            .map(|(_, u)| format!("判定できていない ── 設定を読めない ── {u}")),
    );
    // **判定できなかった参照を、合格にしない**（向きの判定が決める）
    findings.extend(
        unresolved(&order, &got.edges, &got.points, &tree.names())
            .into_iter()
            .map(|u| match u {
                Unresolved::Unlayered { at, to } => {
                    format!("判定できていない ── {at} ── {to} はどの層にも属さない")
                }
                Unresolved::Missing { at, to } => format!(
                    "判定できていない ── {at} ── {to} の参照先が実在しない（生成物か、名前の誤り）"
                ),
            }),
    );
    let layered_edges = got
        .edges
        .iter()
        .filter(|e| layer_for(&order, &e.from).is_some() && layer_for(&order, &e.to).is_some())
        .count();
    Ok(Measured {
        language: tree.language(),
        findings,
        edges: got.edges,
        layered_edges,
        limits: got.limits,
    })
}
