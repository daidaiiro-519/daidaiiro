//! 基盤（abstract schema）── 宣言を読み、宣言された検査を名前で呼び、射影する。
//!
//! **この crate は、方法論の語彙を1つも持たない。**
//! 方法論は concrete schema として注入される。
//! 判定は `禁じた語の一覧` を外に置いて当てる ── **判定文を判定対象に書かない**
//! （書いたら、注記そのものが引っかかった）。
//!
//! 引き受けるのは5つ。
//!   1. 宣言の読み方
//!   2. 宣言された検査を、名前で呼ぶこと
//!   3. 構造化データの検証
//!   4. 人間可読への射影を、全ての型が負う契約にすること
//!   5. 宣言の節の ID と、外の識別子を結ぶ対応表
//!
//! **出すのは、宣言の射影と、つなぎ目と、対応表だけである。**
//! 外の成果物は出さない ── 禁じた語は `forbidden.txt` が持つ。

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

// ── ① 宣言の読み方 ────────────────────────────────────────────
//
// `deny_unknown_fields` と、種類を表す `tag` を明示する。
// **未知の種類は、既定値へ落ちずに誤りとして返る** ── 決まり11「静かに壊さない」を、
// 境界の挙動として固定する。

/// 述語1件。人が読む文（`predicate`）と、機械が呼ぶ名前（`check`）を両方持つ。
///
/// **この2つが食い違っても、機械には分からない。**いまのところ未解決の穴である。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub name: String,
    pub predicate: String,
    pub check: String,
}

/// 操作1件。事前・事後・書き換える先を持つ。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Op {
    pub name: String,
    #[serde(default)]
    pub pre: Vec<String>,
    #[serde(default)]
    pub post: Vec<String>,
    #[serde(default)]
    pub writes: Vec<String>,
}

/// 宣言1件。
///
/// `kind` は文字列のまま持つ ── **基盤が種類を列挙してはならない。**
/// 列挙した瞬間、方法論の語彙が基盤へ入る。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decl {
    pub kind: String,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub ops: Vec<Op>,
    /// 人間可読への射影の名前。**空にできない** ── 契約だからである（④）。
    pub render: String,
    /// 注入する側だけが読む中身。**基盤は形を見ない。**
    ///
    /// 節にしないもの（語の一覧など）は、ここへ入る ── 節は、外の識別子と結ぶ単位だからである。
    #[serde(default)]
    pub body: serde_json::Value,
    /// ID を持つ節。**中身は宣言が持ち、ID を発行する。**
    ///
    /// 基盤は、節が何を表すかを知らない ── そう呼ぶのは注入する側である。
    #[serde(default)]
    pub nodes: Vec<Node>,
}

/// 宣言の集合を読む。未知の欄・未知の形は、ここで誤りになる。
pub fn read(src: &str) -> Result<Vec<Decl>, serde_json::Error> {
    serde_json::from_str(src)
}

// ── ② 検査を、名前で呼ぶ ──────────────────────────────────────
//
// 実体は宣言の外に在り、名前で結ぶ。**表は注入する側が作る** ── 基盤は空の表しか持たない。

pub type CheckFn = fn(&Decl) -> Result<(), String>;

#[derive(Default)]
pub struct Registry {
    checks: BTreeMap<String, CheckFn>,
    ops: BTreeSet<String>,
    renders: BTreeMap<String, fn(&Decl) -> String>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }
    /// 検査の実体を、名前で登録する。
    pub fn check(&mut self, name: &str, f: CheckFn) -> &mut Self {
        self.checks.insert(name.to_string(), f);
        self
    }
    /// 呼び出し口の関門へ、操作を登録する。**登録されていない操作は呼べない。**
    pub fn op(&mut self, qualified: &str) -> &mut Self {
        self.ops.insert(qualified.to_string());
        self
    }
    /// 人間可読への射影を、名前で登録する。
    pub fn render(&mut self, name: &str, f: fn(&Decl) -> String) -> &mut Self {
        self.renders.insert(name.to_string(), f);
        self
    }
    pub fn has_check(&self, n: &str) -> bool {
        self.checks.contains_key(n)
    }
    pub fn check_names(&self) -> BTreeSet<String> {
        self.checks.keys().cloned().collect()
    }
    pub fn op_names(&self) -> BTreeSet<String> {
        self.ops.clone()
    }
}

// ── ③ 3つの穴を、集合の演算で検出する ────────────────────────
//
// 仕様にあって実装にない／実装にあるのに仕様にない／宣言どおりでない。
// **どれも集合の相等を見るだけで、推論は1つも要らない。**

#[derive(Debug, PartialEq, Eq)]
pub struct Holes {
    /// 宣言にあって、実体が無い（＝仕様にあって実装にない）
    pub declared_only: BTreeSet<String>,
    /// 実体があって、宣言に無い（＝実装にあるのに仕様にない）
    pub registered_only: BTreeSet<String>,
}

impl Holes {
    pub fn is_empty(&self) -> bool {
        self.declared_only.is_empty() && self.registered_only.is_empty()
    }
}

/// 宣言が名指した操作と、関門に登録された操作を突き合わせる。
pub fn operation_holes(decls: &[Decl], reg: &Registry) -> Holes {
    let declared: BTreeSet<String> = decls
        .iter()
        .flat_map(|d| d.ops.iter().map(move |o| format!("{}::{}", d.name, o.name)))
        .collect();
    let registered = reg.op_names();
    Holes {
        declared_only: declared.difference(&registered).cloned().collect(),
        registered_only: registered.difference(&declared).cloned().collect(),
    }
}

/// 宣言が名指した検査の名前に、実体が在るかを確かめる。
pub fn missing_checks(decls: &[Decl], reg: &Registry) -> BTreeSet<String> {
    decls
        .iter()
        .flat_map(|d| d.rules.iter().map(|r| r.check.clone()))
        .filter(|n| !reg.has_check(n))
        .collect()
}

/// 射影を持たない宣言を挙げる。**持たない型は、承認を通れない。**
pub fn without_render(decls: &[Decl], reg: &Registry) -> BTreeSet<String> {
    decls
        .iter()
        .filter(|d| d.render.is_empty() || !reg.renders.contains_key(&d.render))
        .map(|d| d.id.clone())
        .collect()
}

// ── ④ 書き込み口の関門 ────────────────────────────────────────
//
// 操作が実際に触った先を記録し、宣言の `writes` と突き合わせる。
// **呼び出し口だけでは、宣言された操作の内側で宣言していない書き込みができてしまう。**

#[derive(Default)]
pub struct Tx {
    touched: BTreeSet<String>,
}

impl Tx {
    pub fn new() -> Self {
        Self::default()
    }
    /// 書き込みは、必ずここを通す。
    pub fn write(&mut self, target: &str) {
        self.touched.insert(target.to_string());
    }
    /// 宣言した `writes` と、実際に触った先を突き合わせる。
    pub fn diff(&self, declared: &[String]) -> (BTreeSet<String>, BTreeSet<String>) {
        let d: BTreeSet<String> = declared.iter().cloned().collect();
        (
            self.touched.difference(&d).cloned().collect(), // 宣言に無い書き込み
            d.difference(&self.touched).cloned().collect(),  // 宣言したのに書いていない
        )
    }
}

// ── ⑥ 射影 ────────────────────────────────────────────────────

pub fn render(d: &Decl, reg: &Registry) -> Option<String> {
    render_as(d, reg, &d.render)
}

/// 射影の名前を指定して出す。
///
/// **人が読む形も、つなぎ目も、ここを通る** ── 出す先が違うだけである。
pub fn render_as(d: &Decl, reg: &Registry, name: &str) -> Option<String> {
    reg.renders.get(name).map(|f| f(d))
}

// ── ⑦ 見つかったこと ──────────────────────────────────────────
//
// **enum にするのは、基盤自身の語彙だけに限る**（試験の条件2）。
// 方法論の語を enum にした瞬間、注入は効かなくなる ── 禁じた語は `forbidden.txt` が持つ。
//
// 基盤が持ってよい語彙は「どんな壊れ方が在るか」である。
// ここへ種類を1つ足すと、**扱っていない場所が全部コンパイル時に出る** ──
// 決まり11「静かに壊さない」を、言語の機能で担保する。

/// 検査で見つかったこと。基盤の語彙であって、方法論の語彙ではない。
#[derive(Debug, PartialEq, Eq)]
pub enum Finding {
    /// 宣言にあって、実体が無い
    DeclaredOnly(String),
    /// 実体があって、宣言に無い
    RegisteredOnly(String),
    /// 宣言された検査の名前に、実体が無い
    MissingCheck(String),
    /// 射影を持たない
    NoRender(String),
    /// 宣言していない先へ書き込んだ
    UndeclaredWrite { op: String, target: String },
    /// 節にあって、対応が無い
    UnboundNode(String),
    /// 外にあって、対応が無い
    UnboundExternal(String),
    /// 対応表にあるが、外に無い
    Dangling(String),
    /// 根に在るべき宣言が、1件も無い
    RootMissing(String),
    /// 根に在るべき宣言が、2件以上ある
    RootDuplicated(String),
    /// 根に在るべき宣言が、根の外に在る
    NotAtRoot(String),
}

impl Finding {
    /// 承認の境目でゼロを要求するか、常時ゼロを要求するか。
    ///
    /// **match に `_ =>` を書かない。**書くと、種類を足したときに静かに素通りする。
    pub fn always_zero(&self) -> bool {
        match self {
            Finding::DeclaredOnly(_) => false, // 開発中の正常な状態でもある
            Finding::RegisteredOnly(_) => true,
            Finding::MissingCheck(_) => true,
            Finding::NoRender(_) => true,
            Finding::UndeclaredWrite { .. } => true,
            Finding::UnboundNode(_) => false, // 実装までの間は、正当な状態である
            Finding::UnboundExternal(_) => true,
            Finding::Dangling(_) => false, // 名前を変えた直後は出る
            Finding::RootMissing(_) => true,
            Finding::RootDuplicated(_) => true,
            Finding::NotAtRoot(_) => true,
        }
    }

    /// 人が読む1行。
    pub fn line(&self) -> String {
        match self {
            Finding::DeclaredOnly(x) => format!("仕様にあって実装にない： {x}"),
            Finding::RegisteredOnly(x) => format!("実装にあるのに仕様にない： {x}"),
            Finding::MissingCheck(x) => format!("実体の無い検査： {x}"),
            Finding::NoRender(x) => format!("射影を持たない： {x} ── 承認を通れない"),
            Finding::UndeclaredWrite { op, target } => {
                format!("宣言に無い書き込み： {op} → {target}")
            }
            Finding::UnboundNode(x) => format!("対応の無い節： {x}"),
            Finding::UnboundExternal(x) => format!("結ばれていない識別子： {x}"),
            Finding::Dangling(x) => format!("切れた対応： {x} ── 外に無い"),
            Finding::RootMissing(x) => format!("根に {x} の宣言が無い ── 全体の枠が決まっていない"),
            Finding::RootDuplicated(x) => format!("根に2件以上ある： {x}"),
            Finding::NotAtRoot(x) => format!("根の外に在る： {x}"),
        }
    }
}

/// 全部の検査を当てて、見つかったことを並べる。
pub fn audit(decls: &[Decl], reg: &Registry) -> Vec<Finding> {
    let mut out = Vec::new();
    let h = operation_holes(decls, reg);
    out.extend(h.declared_only.into_iter().map(Finding::DeclaredOnly));
    out.extend(h.registered_only.into_iter().map(Finding::RegisteredOnly));
    out.extend(missing_checks(decls, reg).into_iter().map(Finding::MissingCheck));
    out.extend(without_render(decls, reg).into_iter().map(Finding::NoRender));
    out
}

// ── ⑧′ 置き場所の決まり ──────────────────────────────────────
//
// **基盤が規定するのは「ある種類の宣言が、ちょうど1件、根に在ること」だけである。**
// どの種類かは呼ぶ側が渡す ── 基盤は種類の名前を1つも持たない。

/// 仕様のファイル1つ。`path` は**根から見た相対の位置**である。
pub struct SpecFile {
    pub path: String,
    pub decls: Vec<Decl>,
}

impl SpecFile {
    /// 根に在るか ── 区切りを1つも含まない位置を根とする。
    pub fn at_root(&self) -> bool {
        !self.path.contains('/')
    }
}

/// 根に在るべき種類が、ちょうど1件、根に在ることを確かめる。
///
/// **種類の名前は引数で受け取る。**基盤が知ってよいのは「根」「1件」だけである。
pub fn root_findings(files: &[SpecFile], root_kind: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut at_root = Vec::new();
    for f in files {
        for d in f.decls.iter().filter(|d| d.kind == root_kind) {
            if f.at_root() {
                at_root.push(d.id.clone());
            } else {
                out.push(Finding::NotAtRoot(format!("{} は {} に在る", d.id, f.path)));
            }
        }
    }
    match at_root.len() {
        0 => out.push(Finding::RootMissing(root_kind.to_string())),
        1 => {}
        _ => out.push(Finding::RootDuplicated(at_root.join(" ・ "))),
    }
    out
}

// ── ⑧ 節と、対応表 ──────────────────────────────────────────
//
// **基盤が知るのは、ID を持つ節と、外の識別子だけである。**
// その節を何と呼び、識別子が何を指すかは、注入する側が決める。

/// ID を持つ節。基盤は `id` と `name` しか読まない。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub name: String,
    /// 注入する側だけが読む中身。基盤は形を見ない。
    #[serde(default)]
    pub body: serde_json::Value,
}

/// 対応表 ── 節の ID と、外の識別子を結ぶ。
///
/// **結ぶのは、外から申告された文字列である。**基盤は外を走査しない。
#[derive(Debug, Default)]
pub struct Bindings {
    map: BTreeMap<String, BTreeSet<String>>,
}

impl Bindings {
    pub fn new() -> Self {
        Self::default()
    }
    /// 1行入れる。同じ節へ複数の識別子を結べる。
    pub fn bind(&mut self, node_id: &str, external: &str) -> &mut Self {
        self.map
            .entry(node_id.to_string())
            .or_default()
            .insert(external.to_string());
        self
    }
    /// 表の中身を、そのまま並べる ── 書き出す側が形式を決める。
    ///
    /// **基盤は、どの形式で保存するかを知らない。**
    pub fn pairs(&self) -> Vec<(String, String)> {
        self.map
            .iter()
            .flat_map(|(k, vs)| vs.iter().map(move |v| (k.clone(), v.clone())))
            .collect()
    }
    pub fn node_ids(&self) -> BTreeSet<String> {
        self.map.keys().cloned().collect()
    }
    pub fn externals(&self) -> BTreeSet<String> {
        self.map.values().flatten().cloned().collect()
    }
}

/// 宣言の中の、ID を持つ節を全部集める。
pub fn node_ids(decls: &[Decl]) -> BTreeSet<String> {
    decls
        .iter()
        .flat_map(|d| d.nodes.iter().map(|x| x.id.clone()))
        .collect()
}

/// 3つの引き算。**推論は1つも要らない。**
#[derive(Debug, PartialEq, Eq)]
pub struct Drift {
    /// 宣言にあって、対応が無い
    pub unbound_nodes: BTreeSet<String>,
    /// 外にあって、対応が無い
    pub unbound_externals: BTreeSet<String>,
    /// 対応表にあるが、外に無い（消えたか、名前が変わった）
    pub dangling: BTreeSet<String>,
}

impl Drift {
    pub fn is_empty(&self) -> bool {
        self.unbound_nodes.is_empty()
            && self.unbound_externals.is_empty()
            && self.dangling.is_empty()
    }
}

/// 節の ID・対応表・外から受け取った識別子の列を、引き算する。
pub fn drift(nodes: &BTreeSet<String>, b: &Bindings, externals: &BTreeSet<String>) -> Drift {
    let bound_nodes = b.node_ids();
    let bound_ext = b.externals();
    Drift {
        unbound_nodes: nodes.difference(&bound_nodes).cloned().collect(),
        unbound_externals: externals.difference(&bound_ext).cloned().collect(),
        dangling: bound_ext.difference(externals).cloned().collect(),
    }
}

/// 引き算の結果を、見つかったことへ移す。
pub fn audit_bindings(
    nodes: &BTreeSet<String>,
    b: &Bindings,
    externals: &BTreeSet<String>,
) -> Vec<Finding> {
    let d = drift(nodes, b, externals);
    let mut out = Vec::new();
    out.extend(d.unbound_nodes.into_iter().map(Finding::UnboundNode));
    out.extend(d.unbound_externals.into_iter().map(Finding::UnboundExternal));
    out.extend(d.dangling.into_iter().map(Finding::Dangling));
    out
}
