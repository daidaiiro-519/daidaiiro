// SPDX-License-Identifier: MIT
//! 論点と、その中身の形。
//!
//! **この Skill は図を描かない。** 図は SVG の文字列として受け取るだけで、どう描いたかを
//! 認知しない ── 描くのは design-svg であり、成果物は具体の側の `figures/` に在る。

/// 案の記号。
pub const LETTERS: [&str; 8] = ["A", "B", "C", "D", "E", "F", "G", "H"];

/// 出どころの種類。
///
/// **機械が分岐する値は ASCII で、画面へ出す語はこの表が保持する** ── 1つの語が識別子と
/// 表示を兼ねると、表示を直した瞬間に分岐が壊れる。
pub const KINDS: [(&str, &str, &str); 5] = [
    ("measured", "k-fact", "実測"),
    ("primary", "k-src", "原典"),
    ("rule", "k-rule", "決まり"),
    ("assumption", "k-given", "前提"),
    ("unverified", "k-open", "未確認"),
];

/// 論点の状態。**値は ASCII、表示の語はここが保持する。**
pub const STATUS: [(&str, &str); 3] = [("waiting", "未"), ("open", "新規"), ("settled", "決着")];

/// 出どころの種類から、札の形と画面の語を引く。
#[must_use]
pub fn kind_of(key: &str) -> Option<(&'static str, &'static str)> {
    KINDS
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, cls, label)| (*cls, *label))
}

/// 状態から画面の語を引く。
#[must_use]
pub fn status_label(key: &str) -> Option<&'static str> {
    STATUS.iter().find(|(k, _)| *k == key).map(|(_, l)| *l)
}

/// 反証を通過した案。**代償を必ず添える** ── 代償が無いと選べない。
///
/// `name` ・ `gist` ・ `cost` は、表の1行に収まる長さで書く。説明が1行に収まらないなら、
/// それは図か、別の表になるものである。
#[derive(Debug, Clone, Default)]
pub struct Option_ {
    /// 案の名前。
    pub name: String,
    /// 何をする案か。
    pub gist: String,
    /// 採ると何を負担するか。
    pub cost: String,
    /// 対話の途中で変わったときの、変更前。
    pub before: String,
    /// なぜ変えたか。
    pub why: String,
}

impl Option_ {
    /// 案を1つ組む。
    #[must_use]
    pub fn new(name: String, gist: String, cost: String) -> Self {
        Self {
            name,
            gist,
            cost,
            before: String::new(),
            why: String::new(),
        }
    }

    /// 書き手が付けた印を持つか。
    #[must_use]
    pub fn marked(&self) -> bool {
        !self.before.is_empty() && !self.why.is_empty()
    }
}

/// 案ごとの帰結を並べる表。**列は呼ぶ側が決める。**
#[derive(Debug, Clone, Default)]
pub struct Table {
    /// 表の名前。
    pub caption: String,
    /// 列の名前。
    pub columns: Vec<String>,
    /// 行。`(行の見出し, 各列の値)`
    pub rows: Vec<(String, Vec<String>)>,
    /// 表の前に置く1行。
    pub lead: String,
    /// 行の見出しが案の記号でないとき（層の名前など）は真。
    pub plain: bool,
}

/// 根拠1件。`(結論のどこを支えるか, もとにしたこと, 出どころの種類, その出どころ)`
pub type Ground = (String, String, String, String);

/// 扱わない範囲の1件。`(事項, 扱い)` ── 扱いが空なら、扱いが未記載である。
pub type Weakness = (String, String);

/// 1つの論点。並べると、タブ1枚になる。
///
/// 決着した論点も同じ1枚に置く ── 別々のページに散らすと、後の論点が前の決着を前提に
/// していることが見えなくなる。
#[derive(Debug, Clone, Default)]
pub struct Topic {
    /// 番号。
    pub no: usize,
    /// タブに出る短い名前。
    pub label: String,
    /// 何を決めるか。
    pub question: String,
    /// `waiting` ／ `open` ／ `settled`。
    pub status: String,
    /// いまの答え。
    pub answer: String,
    /// この論点が何を縛るか。
    pub note: String,
    /// 図。`(SVG, その図が何を示すか)`
    pub figures: Vec<(String, String)>,
    /// 反証を通過した案。
    pub kept: Vec<Option_>,
    /// 案ごとの帰結を並べる表。
    pub tables: Vec<Table>,
    /// 除外した案。`(案, 何が壊れるか)`
    pub dropped: Vec<(String, String)>,
    /// 反証で分かったこと。
    pub found: Vec<String>,
    /// 決定。`(記号, 決めたこと)`
    pub pick: Option<(String, String)>,
    /// その結論に至った道筋。
    pub path: Vec<String>,
    /// 根拠。
    pub grounds: Vec<Ground>,
    /// この答えが要求する事項。
    pub costs: Vec<String>,
    /// 完成イメージのうち、図で表せないもの。**畳まない。**
    pub example: String,
    /// 扱わない範囲。**欠陥の一覧ではない。**
    pub weaknesses: Vec<Weakness>,
    /// 未修正の誤り。`(誤り, 現状)`。**扱わない範囲と分離する。**
    pub defects: Vec<(String, String)>,
    /// 決着した論点の決定。`(見出し, 中身)`
    pub decision: Vec<(String, String)>,
    /// 論点に属さない補足。`(見出し, 中身)`
    pub extras: Vec<(String, String)>,
}

impl Topic {
    /// 答えを持つか（決定か、決着の記載か）。
    #[must_use]
    pub fn decided(&self) -> bool {
        self.pick.is_some() || !self.decision.is_empty()
    }

    /// 決着しているか。
    #[must_use]
    pub fn settled(&self) -> bool {
        self.status == "settled"
    }
}
