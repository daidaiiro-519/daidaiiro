//! 業務イベント駆動（concrete schema）── **注入が効いているかを確かめる試験である。**
//!
//! 方法論の語彙は、全部こちらに在る ── 業務イベント ・ ポリシー ・ 受入条件。
//! ユースケース駆動とは語彙が1つも重ならない。
//! **それでも基盤は1行も変わらない** ── 基盤が知るのは「ID を持つ節」と「外の識別子」だけである。

use base::{Bindings, Decl, Registry};
use std::collections::BTreeSet;

pub fn decls() -> Vec<Decl> {
    base::read(include_str!("../spec/order-placed.json")).expect("宣言が読めない")
}

/// 配線の根。**ユースケース駆動の側と、同じ5つの口しか使っていない。**
pub fn wire() -> Registry {
    let mut r = Registry::new();
    r.check("check.immutable", |_| Ok(()));

    r.op("出荷が指示された::発行する");
    r.op("出荷が指示されたら、在庫を引き落とす::反応する");

    r.render("render.event", |d| {
        let mut s = format!("<h1>{}（業務イベント）</h1>\n<h2>受入条件</h2>\n<ul>\n", d.name);
        for x in &d.nodes {
            s.push_str(&format!("<li><code>{}</code>　{}</li>\n", x.id, x.name));
        }
        s.push_str("</ul>\n");
        s
    });
    r.render("render.policy", |d| {
        let mut s = format!("<h1>{}（ポリシー）</h1>\n<ul>\n", d.name);
        for o in &d.ops {
            s.push_str(&format!("<li><b>{}</b> ── 事前 {:?}</li>\n", o.name, o.pre));
        }
        s.push_str("</ul>\n");
        s
    });
    r
}

/// 対応表 ── 受入条件の ID と、テストの識別子を結ぶ。
pub fn bindings() -> Bindings {
    let mut b = Bindings::new();
    b.bind("AC-01J8A2P", "event::tests::引当が済んでいなければ発行されない");
    b.bind("AC-01J8A4T", "event::tests::二度受けても在庫は一度しか減らない");
    b
}

/// テストの一覧 ── その言語のテスト実行系が列挙したもの。
pub fn test_list() -> BTreeSet<String> {
    ["event::tests::引当が済んでいなければ発行されない",
     "event::tests::二度受けても在庫は一度しか減らない"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}
