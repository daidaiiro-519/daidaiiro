// SPDX-License-Identifier: MIT
//! 格子配置 ── 節点を、与えられた座標の格子へ置く。**縦横の交点が意味を持つときに使う。**
//!
//! 列の幅も行の高さも、そこに実際に居る節点の大きさから決める ── 決め打ちの欄を持たない。中身が
//! 図のときは大きさがまちまちになるため。

use std::collections::HashMap;

use serde_json::Value;

use crate::geometry::{shift_to_origin, Point};
use crate::layout_contract::{LayoutResult, Sizes, Unsupported};
use crate::py::sum;

/// 格子の座標の鍵。**数は数として、そうでないものは文字として並べる。**
#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    /// 数。
    Num(f64),
    /// 文字。
    Text(String),
}

impl Key {
    /// 値から鍵を作る。
    #[must_use]
    pub fn of(v: &Value) -> Self {
        v.as_f64()
            .map_or_else(|| Self::Text(crate::props::text(v)), Self::Num)
    }

    fn sort_key(&self) -> (u8, f64, String) {
        match self {
            Self::Num(n) => (0, *n, String::new()),
            Self::Text(s) => (1, 0.0, s.clone()),
        }
    }
}

/// 格子の指定。
#[derive(Debug, Clone, Default)]
pub struct Grid {
    /// 節点 → `(列の鍵, 行の鍵)`。
    pub at: Vec<(String, (Key, Key))>,
    /// 列の並び。**空なら鍵の昇順。**
    pub cols: Vec<Key>,
    /// 行の並び。**空なら鍵の昇順。**
    pub rows: Vec<Key>,
    /// 鍵線にしたい辺 → 曲がり角の置き方（`vertical` ／ `horizontal`）。
    pub elbow: Vec<((String, String), String)>,
}

fn sorted_keys(keys: Vec<Key>) -> Vec<Key> {
    let mut uniq: Vec<Key> = Vec::new();
    for k in keys {
        if !uniq.contains(&k) {
            uniq.push(k);
        }
    }
    uniq.sort_by(|a, b| {
        let (x, y) = (a.sort_key(), b.sort_key());
        x.0.cmp(&y.0)
            .then(x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal))
            .then(x.2.cmp(&y.2))
    });
    uniq
}

/// 2つの中心を結ぶ経路。**角が端のどちらかと重なるなら、挟まない** ── 長さ0の経路が出て、終端が
/// どのインクにも着かなかった（実測）。
fn path(a: Point, b: Point, bend: Option<&str>) -> Vec<Point> {
    let Some(bend) = bend else { return vec![a, b] };
    let corner = if bend == "vertical" {
        (a.0, b.1)
    } else {
        (b.0, a.1)
    };
    if corner == a || corner == b {
        return vec![a, b];
    }
    vec![a, corner, b]
}

/// 節点を、与えられた座標の格子へ置く。
///
/// # Errors
///
/// 座標の無い節点が在るとき ・ 鍵線が辺に無い辺を指したとき ・ 置き方が決められた値でないときに
/// 返す。
pub fn layout_grid(
    sizes: &Sizes,
    edges: &[(String, String)],
    gap_rank: f64,
    gap_order: f64,
    grid: &Grid,
) -> Result<LayoutResult, Unsupported> {
    let bad: Vec<&(String, String)> = grid
        .elbow
        .iter()
        .filter(|(_, v)| v != "vertical" && v != "horizontal")
        .map(|(k, _)| k)
        .collect();
    if !bad.is_empty() {
        return Err(Unsupported::Invalid(format!(
            "曲がり角の置き方は vertical か horizontal です: {bad:?}"
        )));
    }
    let absent: Vec<&(String, String)> = grid
        .elbow
        .iter()
        .map(|(k, _)| k)
        .filter(|k| !edges.contains(k))
        .collect();
    if !absent.is_empty() {
        return Err(Unsupported::Invalid(format!(
            "辺に無いものが elbow にあります: {absent:?}"
        )));
    }
    let ids: Vec<String> = sizes.iter().map(|(k, _)| k.clone()).collect();
    if ids.is_empty() {
        return Ok(LayoutResult::default());
    }
    let at: HashMap<&str, &(Key, Key)> = grid.at.iter().map(|(k, v)| (k.as_str(), v)).collect();
    let missing: Vec<&String> = ids
        .iter()
        .filter(|i| !at.contains_key(i.as_str()))
        .collect();
    if !missing.is_empty() {
        return Err(Unsupported::Invalid(format!(
            "座標が無い節点があります: {missing:?}"
        )));
    }
    let size_of: HashMap<&str, (f64, f64)> = sizes.iter().map(|(k, s)| (k.as_str(), *s)).collect();
    let cols = if grid.cols.is_empty() {
        sorted_keys(ids.iter().map(|i| at[i.as_str()].0.clone()).collect())
    } else {
        grid.cols.clone()
    };
    let rows = if grid.rows.is_empty() {
        sorted_keys(ids.iter().map(|i| at[i.as_str()].1.clone()).collect())
    } else {
        grid.rows.clone()
    };
    // **列の幅 ・ 行の高さは、そこに居るもののうち最も大きいものに合わせる**
    let col_w: Vec<f64> = cols
        .iter()
        .map(|c| {
            ids.iter()
                .filter(|i| at[i.as_str()].0 == *c)
                .map(|i| size_of[i.as_str()].0)
                .reduce(f64::max)
                .unwrap_or(0.0)
        })
        .collect();
    let row_h: Vec<f64> = rows
        .iter()
        .map(|r| {
            ids.iter()
                .filter(|i| at[i.as_str()].1 == *r)
                .map(|i| size_of[i.as_str()].1)
                .reduce(f64::max)
                .unwrap_or(0.0)
        })
        .collect();
    let col_x: Vec<f64> = (0..cols.len())
        .map(|c| sum(col_w[..c].iter().copied()) + gap_order * c as f64)
        .collect();
    let row_y: Vec<f64> = (0..rows.len())
        .map(|r| sum(row_h[..r].iter().copied()) + gap_rank * r as f64)
        .collect();
    let mut positions = Vec::new();
    let mut centres = Vec::new();
    for i in &ids {
        let (ck, rk) = at[i.as_str()];
        let c = cols
            .iter()
            .position(|k| k == ck)
            .ok_or_else(|| Unsupported::Invalid(format!("{ck:?} is not in list")))?;
        let r = rows
            .iter()
            .position(|k| k == rk)
            .ok_or_else(|| Unsupported::Invalid(format!("{rk:?} is not in list")))?;
        let (w, h) = size_of[i.as_str()];
        // 欄の中で中央へ寄せる ── 左上へ寄せると、縦横の対応が読み取りにくくなる
        let x = col_x[c] + (col_w[c] - w) / 2.0;
        let y = row_y[r] + (row_h[r] - h) / 2.0;
        positions.push((i.clone(), (x, y)));
        centres.push((i.clone(), (x + w / 2.0, y + h / 2.0)));
    }
    let (positions, (min_x, min_y)) = shift_to_origin(&positions);
    let centres: HashMap<String, Point> = centres
        .into_iter()
        .map(|(k, (x, y))| (k, (x - min_x, y - min_y)))
        .collect();
    let width = positions
        .iter()
        .map(|(k, p)| p.0 + size_of[k.as_str()].0)
        .fold(f64::NEG_INFINITY, f64::max);
    let height = positions
        .iter()
        .map(|(k, p)| p.1 + size_of[k.as_str()].1)
        .fold(f64::NEG_INFINITY, f64::max);
    let edge_paths = edges
        .iter()
        .enumerate()
        .filter_map(|(n, (a, b))| {
            let bend = grid
                .elbow
                .iter()
                .find(|(k, _)| k.0 == *a && k.1 == *b)
                .map(|(_, v)| v.as_str());
            Some((n, path(*centres.get(a)?, *centres.get(b)?, bend)))
        })
        .collect();
    Ok(LayoutResult {
        positions,
        edge_paths,
        width,
        height,
    })
}
