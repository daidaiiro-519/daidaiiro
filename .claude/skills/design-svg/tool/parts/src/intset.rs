// SPDX-License-Identifier: MIT
//! 小さな整数の集合 ── **辿る順まで、移す前の集合と同じにする。**
//!
//! 移す前の配置は、整数の集合を辿り、最初に条件を満たした要素を採る ── 辿る順が違うと、別の
//! 要素を採り、別の配置が出る。辿る順は集合の内部の表の並びで決まるので、表の組み方ごと写す。
//!
//! 写すのは、配置が使う4つの操作だけである ── 足す ・ 1つを除いた複製 ・ 1つを加えた複製 ・
//! 辿る。

/// 表の最小の大きさ。
const MIN_SIZE: usize = 8;
/// 近くを順に見る数。
const LINEAR_PROBES: usize = 9;
/// 攪拌の刻み。
const PERTURB_SHIFT: u32 = 5;
/// 次に見る位置を決める掛け数。
const PROBE_MUL: usize = 5;
/// 表を組み直す詰まり具合（埋まった数 × 分母 ≥ 大きさ × 分子）。
const LOAD_NUM: usize = 3;
const LOAD_DEN: usize = 5;
/// 組み直すときの大きさ ── 小さな集合は使っている数の4倍、大きな集合は2倍。
const GROW_SMALL: usize = 4;
const GROW_LARGE: usize = 2;
const GROW_SWITCH: usize = 50_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    Empty,
    Dummy,
    Key(usize),
}

/// 小さな整数の集合。
#[derive(Debug, Clone)]
pub struct IntSet {
    table: Vec<Slot>,
    fill: usize,
    used: usize,
}

impl Default for IntSet {
    fn default() -> Self {
        Self::new()
    }
}

impl IntSet {
    /// 空の集合。
    #[must_use]
    pub fn new() -> Self {
        Self {
            table: vec![Slot::Empty; MIN_SIZE],
            fill: 0,
            used: 0,
        }
    }

    fn mask(&self) -> usize {
        self.table.len() - 1
    }

    /// 要素の数。
    #[must_use]
    pub const fn len(&self) -> usize {
        self.used
    }

    /// 空か。
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.used == 0
    }

    /// 含むか。
    #[must_use]
    pub fn contains(&self, key: usize) -> bool {
        let mask = self.mask();
        let mut i = key & mask;
        let mut perturb = key;
        loop {
            let probes = if i + LINEAR_PROBES <= mask {
                LINEAR_PROBES
            } else {
                0
            };
            for j in 0..=probes {
                match self.table[i + j] {
                    Slot::Empty => return false,
                    Slot::Key(k) if k == key => return true,
                    _ => {}
                }
            }
            perturb >>= PERTURB_SHIFT;
            i = (i
                .wrapping_mul(PROBE_MUL)
                .wrapping_add(1)
                .wrapping_add(perturb))
                & mask;
        }
    }

    /// 表を組み直す。**古い表の並びの順に入れ直す。**
    fn resize(&mut self, minused: usize) {
        let mut newsize = MIN_SIZE;
        while newsize <= minused {
            newsize <<= 1;
        }
        let old = std::mem::replace(&mut self.table, vec![Slot::Empty; newsize]);
        for s in old {
            if let Slot::Key(k) = s {
                Self::insert_clean(&mut self.table, k);
            }
        }
        self.fill = self.used;
    }

    /// 重なりも印も無い表へ入れる。
    fn insert_clean(table: &mut [Slot], key: usize) {
        let mask = table.len() - 1;
        let mut i = key & mask;
        let mut perturb = key;
        loop {
            if table[i] == Slot::Empty {
                table[i] = Slot::Key(key);
                return;
            }
            if i + LINEAR_PROBES <= mask {
                for j in 1..=LINEAR_PROBES {
                    if table[i + j] == Slot::Empty {
                        table[i + j] = Slot::Key(key);
                        return;
                    }
                }
            }
            perturb >>= PERTURB_SHIFT;
            i = (i
                .wrapping_mul(PROBE_MUL)
                .wrapping_add(1)
                .wrapping_add(perturb))
                & mask;
        }
    }

    /// 足す。
    pub fn add(&mut self, key: usize) {
        let mask = self.mask();
        let mut i = key & mask;
        let mut perturb = key;
        let mut freeslot: Option<usize> = None;
        loop {
            let probes = if i + LINEAR_PROBES <= mask {
                LINEAR_PROBES
            } else {
                0
            };
            for j in 0..=probes {
                let at = i + j;
                match self.table[at] {
                    Slot::Empty => {
                        if let Some(free) = freeslot {
                            self.table[free] = Slot::Key(key);
                            self.used += 1;
                            return;
                        }
                        self.table[at] = Slot::Key(key);
                        self.fill += 1;
                        self.used += 1;
                        if self.fill * LOAD_DEN >= mask * LOAD_NUM {
                            let target = if self.used > GROW_SWITCH {
                                self.used * GROW_LARGE
                            } else {
                                self.used * GROW_SMALL
                            };
                            self.resize(target);
                        }
                        return;
                    }
                    Slot::Key(k) if k == key => return,
                    Slot::Dummy => {
                        if freeslot.is_none() {
                            freeslot = Some(at);
                        }
                    }
                    Slot::Key(_) => {}
                }
            }
            perturb >>= PERTURB_SHIFT;
            i = (i
                .wrapping_mul(PROBE_MUL)
                .wrapping_add(1)
                .wrapping_add(perturb))
                & mask;
        }
    }

    /// 除く。**跡には印を残す。**
    pub fn discard(&mut self, key: usize) {
        let mask = self.mask();
        let mut i = key & mask;
        let mut perturb = key;
        loop {
            let probes = if i + LINEAR_PROBES <= mask {
                LINEAR_PROBES
            } else {
                0
            };
            for j in 0..=probes {
                match self.table[i + j] {
                    Slot::Empty => return,
                    Slot::Key(k) if k == key => {
                        self.table[i + j] = Slot::Dummy;
                        self.used -= 1;
                        return;
                    }
                    _ => {}
                }
            }
            perturb >>= PERTURB_SHIFT;
            i = (i
                .wrapping_mul(PROBE_MUL)
                .wrapping_add(1)
                .wrapping_add(perturb))
                & mask;
        }
    }

    /// 辿る。**表の並びの順である。**
    #[must_use]
    pub fn iter(&self) -> Vec<usize> {
        self.table
            .iter()
            .filter_map(|s| match s {
                Slot::Key(k) => Some(*k),
                _ => None,
            })
            .collect()
    }

    /// 複製する。**複製の組み方も写す** ── 同じ大きさで印が無ければ、表をそのまま写す。
    #[must_use]
    pub fn copy(&self) -> Self {
        let mut out = Self::new();
        out.merge(self);
        out
    }

    /// 別の集合をまとめて足す。
    fn merge(&mut self, other: &Self) {
        if other.used == 0 {
            return;
        }
        if (self.fill + other.used) * LOAD_DEN >= self.mask() * LOAD_NUM {
            self.resize((self.used + other.used) * GROW_LARGE);
        }
        if self.fill == 0 && self.mask() == other.mask() && other.fill == other.used {
            self.table.clone_from(&other.table);
            self.fill = other.fill;
            self.used = other.used;
            return;
        }
        if self.fill == 0 {
            for s in &other.table {
                if let Slot::Key(k) = s {
                    Self::insert_clean(&mut self.table, *k);
                }
            }
            self.fill = other.used;
            self.used = other.used;
            return;
        }
        for s in &other.table {
            if let Slot::Key(k) = s {
                self.add(*k);
            }
        }
    }

    /// 1つを除いた複製。`集合 - {key}`
    #[must_use]
    pub fn without(&self, key: usize) -> Self {
        // 大きい集合は、複製してから除く
        if (self.used >> 2) > 1 {
            let mut out = self.copy();
            out.discard(key);
            return out;
        }
        let mut out = Self::new();
        for k in self.iter() {
            if k != key {
                out.add(k);
            }
        }
        out
    }

    /// 1つを加えた複製。`集合 | {key}`
    #[must_use]
    pub fn with(&self, key: usize) -> Self {
        let mut out = self.copy();
        let mut single = Self::new();
        single.add(key);
        out.merge(&single);
        out
    }
}
