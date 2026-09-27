// SPDX-License-Identifier: MIT
//! 並びを突き合わせる。**Ratcliff と Obershelp の方式である。**
//!
//! **位置だけで比べない** ── 1行足しただけで以降が全部「変わった」と出る（実測 ──
//! 実際に出た）。突き合わせてから、行の中の欄を比べる。
//!
//! **移す前と同じ手順を写している** ── 手順が違うと、同じ入力から別の対応が出て、
//! 印の付く場所が変わる。

use std::collections::HashMap;

/// 差分の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// 一致している。
    Equal,
    /// 置き換わった。
    Replace,
    /// 消えた。
    Delete,
    /// 増えた。
    Insert,
}

/// 一致する塊を探す。
fn longest_match(
    a: &[&str],
    b: &[&str],
    b2j: &HashMap<&str, Vec<usize>>,
    alo: usize,
    ahi: usize,
    blo: usize,
    bhi: usize,
) -> (usize, usize, usize) {
    let (mut besti, mut bestj, mut bestsize) = (alo, blo, 0);
    let mut j2len: HashMap<usize, usize> = HashMap::new();
    for (i, line) in a.iter().enumerate().take(ahi).skip(alo) {
        let mut next: HashMap<usize, usize> = HashMap::new();
        if let Some(places) = b2j.get(*line) {
            for j in places {
                if *j < blo {
                    continue;
                }
                if *j >= bhi {
                    break;
                }
                let k = j.checked_sub(1).and_then(|p| j2len.get(&p)).unwrap_or(&0) + 1;
                next.insert(*j, k);
                if k > bestsize {
                    (besti, bestj, bestsize) = (i + 1 - k, j + 1 - k, k);
                }
            }
        }
        j2len = next;
    }
    while besti > alo && bestj > blo && a[besti - 1] == b[bestj - 1] {
        besti -= 1;
        bestj -= 1;
        bestsize += 1;
    }
    while besti + bestsize < ahi
        && bestj + bestsize < bhi
        && a[besti + bestsize] == b[bestj + bestsize]
    {
        bestsize += 1;
    }
    (besti, bestj, bestsize)
}

/// 一致する塊を、前から順に並べる。
fn matching_blocks(a: &[&str], b: &[&str]) -> Vec<(usize, usize, usize)> {
    let mut b2j: HashMap<&str, Vec<usize>> = HashMap::new();
    for (j, line) in b.iter().enumerate() {
        b2j.entry(line).or_default().push(j);
    }
    let mut queue = vec![(0, a.len(), 0, b.len())];
    let mut found = Vec::new();
    while let Some((alo, ahi, blo, bhi)) = queue.pop() {
        let (i, j, k) = longest_match(a, b, &b2j, alo, ahi, blo, bhi);
        if k == 0 {
            continue;
        }
        found.push((i, j, k));
        if alo < i && blo < j {
            queue.push((alo, i, blo, j));
        }
        if i + k < ahi && j + k < bhi {
            queue.push((i + k, ahi, j + k, bhi));
        }
    }
    found.sort_unstable();
    let mut joined: Vec<(usize, usize, usize)> = Vec::new();
    let (mut i1, mut j1, mut k1) = (0, 0, 0);
    for (i2, j2, k2) in found {
        if i1 + k1 == i2 && j1 + k1 == j2 {
            k1 += k2;
        } else {
            if k1 > 0 {
                joined.push((i1, j1, k1));
            }
            (i1, j1, k1) = (i2, j2, k2);
        }
    }
    if k1 > 0 {
        joined.push((i1, j1, k1));
    }
    joined.push((a.len(), b.len(), 0));
    joined
}

/// 差分の手順を並べる。
#[must_use]
pub fn opcodes(a: &[&str], b: &[&str]) -> Vec<(Op, usize, usize, usize, usize)> {
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    for (ai, bj, size) in matching_blocks(a, b) {
        let kind = if i < ai && j < bj {
            Some(Op::Replace)
        } else if i < ai {
            Some(Op::Delete)
        } else if j < bj {
            Some(Op::Insert)
        } else {
            None
        };
        if let Some(kind) = kind {
            out.push((kind, i, ai, j, bj));
        }
        i = ai + size;
        j = bj + size;
        if size > 0 {
            out.push((Op::Equal, ai, i, bj, j));
        }
    }
    out
}
