// SPDX-License-Identifier: MIT
//! 配置戦略が遵守する契約 ── **4つの戦略が共有する形を、中立な場所に置く。**
//!
//! 共有される契約を、実装の1つが所有してはいけない ── 以前は層状配置が戻り値の形を持ち、
//! 環状 ・ 放射の木 ・ 格子 ・ 合成がそこから借りていた（実測5か所）。契約は「呼ぶ側が何を
//! 必要としているか」の視点で定義する。

use crate::geometry::Point;

/// 配置戦略が返すもの。**どの戦略もこの形を返す。**
#[derive(Debug, Clone, Default)]
pub struct LayoutResult {
    /// 実節点 → 左上の座標。**宣言の順に持つ。**
    pub positions: Vec<(String, Point)>,
    /// 辺の通し番号 → 通る点の並び。**番号の順に持つ。**
    pub edge_paths: Vec<(usize, Vec<Point>)>,
    /// 幅。
    pub width: f64,
    /// 高さ。
    pub height: f64,
}

impl LayoutResult {
    /// 節点の位置を引く。
    #[must_use]
    pub fn position(&self, id: &str) -> Option<Point> {
        self.positions
            .iter()
            .find(|(k, _)| k == id)
            .map(|(_, p)| *p)
    }

    /// 辺の経路を引く。
    #[must_use]
    pub fn path(&self, index: usize) -> Option<&Vec<Point>> {
        self.edge_paths
            .iter()
            .find(|(k, _)| *k == index)
            .map(|(_, p)| p)
    }
}

/// 節点の大きさ。**宣言の順に持つ。**
pub type Sizes = [(String, (f64, f64))];

/// 配置戦略。`(大きさ, 辺, 層の間隔, 層の中の間隔, 向き)` を受け、**この戦略では描けないときは、
/// その旨を返す。**
pub type Strategy =
    fn(&Sizes, &[(String, String)], f64, f64, &str) -> Result<LayoutResult, Unsupported>;

/// 誤り。**入力の誤りと、戦略の能力の外を分ける。**
#[derive(Debug, Clone)]
pub enum Unsupported {
    /// この配置のやり方では描けない ── 別の描き方なら描ける。
    ByStrategy(String),
    /// 入力が誤っている。
    Invalid(String),
}

impl std::fmt::Display for Unsupported {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByStrategy(m) | Self::Invalid(m) => f.write_str(m),
        }
    }
}
