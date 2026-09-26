// SPDX-License-Identifier: MIT
//! 依存の向きの判定。**言語を1つも認知しない。**
//!
//! 受け取るのは層の宣言と、抽出した辺だけである ── どの言語から辺を取ったかを
//! 認知しない。**辺の取り方を参照しない**ので、言語が増えてもこの側は変わらない。
//!
//! 禁じる辺の導出は3つの規則で尽きる。出典は Robert C. Martin の
//! `Nothing in an inner circle can know anything at all about something in an
//! outer circle.` である。

use std::collections::{BTreeMap, BTreeSet};

/// 層1つ。**識別子は文字列である** ── 経路とは限らない。
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Layer {
    /// 層の名前。
    pub name: String,
    /// その言語で層を識別する文字列。**1つの層が複数を持てる** ── 同じ役割の
    /// 点が複数ある層が実在する（実測 ── 入口が4つの点を持つ）。
    pub ids: Vec<String>,
    /// 同じ層の中の兄弟どうしを禁じるか。
    pub independent: bool,
    /// この層を越えて、上の層が下の層へ到達することを禁じるか。
    pub closed: bool,
    /// この層が合成する側か。**合成する層だけが、図に現れない読み込みを持ってよい。**
    pub composes: bool,
}

impl Layer {
    /// 層を組む。**既定では兄弟どうしを禁じず、越えることも禁じない。**
    #[must_use]
    pub fn new(name: String, id: String) -> Self {
        Self::of(name, vec![id])
    }

    /// 識別子を複数持つ層を組む。
    #[must_use]
    pub const fn of(name: String, ids: Vec<String>) -> Self {
        Self {
            name,
            ids,
            independent: false,
            closed: false,
            composes: false,
        }
    }

    /// 兄弟どうしを禁じる層にする。
    #[must_use]
    pub const fn independent(mut self) -> Self {
        self.independent = true;
        self
    }

    /// 越えることを禁じる層にする。
    #[must_use]
    pub const fn closed(mut self) -> Self {
        self.closed = true;
        self
    }

    /// 合成する層にする。**ここだけが、図に現れない読み込みを持ってよい** ── 依存の
    /// 向きの規則が唯一成立しない場所を、最も外側の1か所に集約する。
    #[must_use]
    pub const fn composes(mut self) -> Self {
        self.composes = true;
        self
    }
}

/// 層の宣言。**並びは内から外である** ── 先頭が最も内側である。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Order {
    layers: Vec<Layer>,
}

impl Order {
    /// 内から外の並びで組む。
    #[must_use]
    pub const fn inner_to_outer(layers: Vec<Layer>) -> Self {
        Self { layers }
    }

    /// 並びをそのまま返す。
    #[must_use]
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// 名前から層を引く。
    #[must_use]
    pub fn by_name(&self, name: &str) -> Option<&Layer> {
        self.layers.iter().find(|l| l.name == name)
    }
}

/// 参照1つ。**どの言語から取ったかを保持しない。**
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Edge {
    /// 参照する側の識別子。
    pub from: String,
    /// 参照される側の識別子。
    pub to: String,
    /// どこに書かれているか（人が開ける粒度）。
    pub at: String,
}

impl Edge {
    /// 参照を組む。
    #[must_use]
    pub const fn new(from: String, to: String, at: String) -> Self {
        Self { from, to, at }
    }
}

/// 許してはならない層の対。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Forbidden {
    /// 参照する側の層の名前。
    pub from: String,
    /// 参照される側の層の名前。
    pub to: String,
    /// なぜ禁じるか。
    pub because: Because,
}

/// 禁じる理由。**3つで尽きる。**
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Because {
    /// 内側が外側を参照している。
    InnerReachesOuter,
    /// 兄弟どうしが参照している（その層が独立のとき）。
    SiblingsAreIndependent,
    /// 閉じた層を越えている。
    CrossesAClosedLayer,
}

impl Because {
    /// 画面へ出す語。**機械が分岐する値と、表示を兼ねさせない。**
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::InnerReachesOuter => "内側が外側を参照している",
            Self::SiblingsAreIndependent => "同じ層の兄弟どうしが参照している",
            Self::CrossesAClosedLayer => "閉じた層を越えている",
        }
    }
}

/// 層の並びから、禁じる辺をすべて導く。**並びだけから決まる。**
#[must_use]
pub fn forbidden(order: &Order) -> BTreeSet<Forbidden> {
    let layers = order.layers();
    let mut out = BTreeSet::new();
    for (i, inner) in layers.iter().enumerate() {
        // 内側は外側を参照しない
        for outer in &layers[i + 1..] {
            out.insert(Forbidden {
                from: inner.name.clone(),
                to: outer.name.clone(),
                because: Because::InnerReachesOuter,
            });
        }
        // 兄弟どうしは、その層が独立なら参照しない
        if inner.independent {
            out.insert(Forbidden {
                from: inner.name.clone(),
                to: inner.name.clone(),
                because: Because::SiblingsAreIndependent,
            });
        }
        // 閉じた層を越えて、外側が内側へ到達しない
        let mut crossed = false;
        for outer in &layers[i + 1..] {
            if crossed {
                out.insert(Forbidden {
                    from: outer.name.clone(),
                    to: inner.name.clone(),
                    because: Because::CrossesAClosedLayer,
                });
            }
            crossed |= outer.closed;
        }
    }
    out
}

/// 違反1件。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Violation {
    /// 参照する側の層。
    pub from: String,
    /// 参照される側の層。
    pub to: String,
    /// なぜ禁じるか。
    pub because: Because,
    /// どこに書かれているか。
    pub at: String,
    /// 実際の参照。
    pub edge: Edge,
}

/// 識別子が、どの層に属するかを決める。**呼ぶ側から使える。**
#[must_use]
pub fn layer_for<'a>(order: &'a Order, id: &str) -> Option<&'a Layer> {
    layer_of(order, id)
}

/// 識別子が、どの層に属するかを決める。**最も長く一致する層を採る** ── 層が
/// 入れ子のとき、短いほうへ吸われると違反が消える。
#[must_use]
fn layer_of<'a>(order: &'a Order, id: &str) -> Option<&'a Layer> {
    /// その識別子のうち、いちばん長く一致したものの長さ。
    fn reach(layer: &Layer, id: &str) -> Option<usize> {
        layer
            .ids
            .iter()
            .filter(|x| {
                id == x.as_str()
                    || id.starts_with(&format!("{x}."))
                    || id.starts_with(&format!("{x}/"))
                    || id.starts_with(&format!("{x}::"))
                    || id.starts_with(&format!("{x}\\"))
            })
            .map(String::len)
            .max()
    }
    order
        .layers()
        .iter()
        .filter_map(|l| reach(l, id).map(|n| (n, l)))
        .max_by_key(|(n, _)| *n)
        .map(|(_, l)| l)
}

/// 抽出した辺を、禁じる辺と突き合わせる。**見つけるが、直さない。**
#[must_use]
pub fn judge(order: &Order, edges: &[Edge]) -> Vec<Violation> {
    let rules: BTreeMap<(String, String), Because> = forbidden(order)
        .into_iter()
        .map(|f| ((f.from, f.to), f.because))
        .collect();
    let mut out = Vec::new();
    for e in edges {
        let (Some(from), Some(to)) = (layer_of(order, &e.from), layer_of(order, &e.to)) else {
            continue; // 層の外は対象外である
        };
        if from.name == to.name && !from.independent {
            continue; // 同じ層の中は、独立でなければ自由である
        }
        if let Some(&because) = rules.get(&(from.name.clone(), to.name.clone())) {
            out.push(Violation {
                from: from.name.clone(),
                to: to.name.clone(),
                because,
                at: e.at.clone(),
                edge: e.clone(),
            });
        }
    }
    out.sort();
    out
}
