// SPDX-License-Identifier: MIT
//! 規約 ── 宣言した規律に、実物が適合しているか。
//!
//! 規律を文章にだけ書くと、破れても何も起きない。だから宣言した規約はすべてここで照合する
//! ── 破った瞬間に失敗する。**「何を禁じるか」を書けるものだけを置く。**
//!
//!     cargo test -p ds_business_logic

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use ds_business_logic::catalog::examples;
use ds_business_logic::registry::{known_kinds, render, render_node, Origin};
use ds_business_logic::style::resolve;

/// 責務の層。数字が小さいほど土台に近い。
///
/// 依存の深さ（`use` の連鎖）とは別物である ── 深さは「呼ぶ順序」と「型をどこから借りたか」を
/// 映すだけなので、責務の側をここに明示し、揃い続けることを機械で検証する。
const LAYER: [(&str, u8); 35] = [
    // 0 語彙 ・ 台帳 ── 誰の都合も知らない。名前 ・ 数 ・ 形 ・ 登録簿
    ("theme", 0),
    ("text", 0),
    ("geometry", 0),
    ("boolean", 0),
    ("registry", 0),
    ("lint", 0),
    ("layout_contract", 0),
    ("ids", 0),
    ("py", 0),
    ("intset", 0),
    ("xml", 0),
    ("props", 0),
    // 図の表記法 ── class の一覧を theme.json から読むだけ
    ("classes", 0),
    // サービス層が読み書きするファイル ── データアクセス層へ渡すだけで、判定を持たない
    ("files", 0),
    // 1 方針 ・ 配置 ── 値の解決と、座標の解き方
    ("style", 1),
    ("sugiyama", 1),
    ("radial", 1),
    ("tree", 1),
    ("grid", 1),
    ("nesting", 1),
    ("labels", 1),
    // class を値へ解決する
    ("resolve", 1),
    // 2 部品 ── 1つの形を描く
    ("shapes", 2),
    ("shapes_freeform", 2),
    ("shapes_hex", 2),
    ("shapes_interaction", 2),
    ("shapes_quantity", 2),
    ("shapes_table", 2),
    ("shapes_titled", 2),
    // 3 合成 ── 全部を知ってよい唯一の場所
    ("compose", 3),
    ("catalog", 3),
    // 返す前の道 ── 解決と検査1〜4を、作成者の SVG と figure ・ chart の出力に同じ順で通す
    ("publish", 3),
    // 層の数直線に載らないもの ── 生成物を外から検査する直交した軸
    ("verify", u8::MAX),
    ("checks", u8::MAX),
    // references の実装 ── skills-creator の雛形の複製であり、描画エンジンの外に在る。
    // 同じ crate のモジュールを参照しない（参照すれば、雛形との一致が崩れる）
    ("refs", u8::MAX),
];

/// skills-creator の雛形を中身のまま複製したモジュール。**描画エンジンの語彙の検査の対象外である** ──
/// 中身を書き換えると `skills-creator check` が雛形との不一致を検出する。
const TEMPLATE_COPIES: [&str; 1] = ["refs"];

fn layer(name: &str) -> Option<u8> {
    LAYER
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, l)| *l)
        .filter(|l| *l != u8::MAX)
}

fn src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// モジュール → 本文。**crate の根（`lib.rs`）は層に載らない** ── 全部を束ねる場所である。
fn modules() -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for e in std::fs::read_dir(src()).expect("src が在る").flatten() {
        let p = e.path();
        let stem = p
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if p.extension().is_some_and(|x| x == "rs") && stem != "lib" {
            out.insert(stem, std::fs::read_to_string(&p).expect("読める"));
        }
    }
    out
}

/// 本文が参照する同じ crate のモジュール。**注釈の中の言及は数えない。**
fn imports(body: &str, known: &BTreeSet<String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in body.lines().filter(|l| !l.trim_start().starts_with("//")) {
        let mut rest = line;
        while let Some(at) = rest.find("crate::") {
            rest = &rest[at + "crate::".len()..];
            // `use crate::{a, b::C}` の形も読む
            let head: String = if let Some(inner) = rest.strip_prefix('{') {
                for part in inner.split('}').next().unwrap_or_default().split(',') {
                    let name: String = part
                        .trim()
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if known.contains(&name) {
                        out.insert(name);
                    }
                }
                continue;
            } else {
                rest.chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect()
            };
            if known.contains(&head) {
                out.insert(head);
            }
        }
    }
    out
}

fn graph() -> BTreeMap<String, BTreeSet<String>> {
    let all = modules();
    let known: BTreeSet<String> = all.keys().cloned().collect();
    all.iter()
        .map(|(n, b)| {
            let mut deps = imports(b, &known);
            deps.remove(n);
            (n.clone(), deps)
        })
        .collect()
}

#[test]
fn every_module_is_assigned_to_a_layer() {
    // 配分の漏れがあると、そのモジュールだけ検査の外に出る
    let missing: Vec<String> = modules()
        .into_keys()
        .filter(|n| !LAYER.iter().any(|(k, _)| k == n))
        .collect();
    assert!(missing.is_empty(), "層の配分が無い: {missing:?}");
}

#[test]
fn template_copies_call_no_engine_module() {
    // 雛形の複製が描画エンジンを参照すると、層の数直線の外に置いた理由が消える
    let g = graph();
    for name in TEMPLATE_COPIES {
        let deps = g.get(name).expect("雛形の複製が在る");
        assert!(deps.is_empty(), "{name} がエンジンを参照している: {deps:?}");
    }
}

#[test]
fn lower_layers_do_not_call_upper_ones() {
    let mut bad = Vec::new();
    for (name, deps) in graph() {
        let Some(me) = layer(&name) else { continue };
        for d in deps {
            if let Some(them) = layer(&d) {
                if them > me {
                    bad.push(format!("{name}(層{me}) → {d}(層{them})"));
                }
            }
        }
    }
    assert!(bad.is_empty(), "下から上への呼び出し: {}", bad.join(" / "));
}

#[test]
fn the_base_calls_nobody() {
    // 層0が層1以上を呼ぶと、土台が方針を知ることになる
    for (name, deps) in graph() {
        if layer(&name) != Some(0) {
            continue;
        }
        let up: Vec<&String> = deps.iter().filter(|d| layer(d).unwrap_or(0) > 0).collect();
        assert!(up.is_empty(), "{name}(層0) が上を呼んでいる: {up:?}");
    }
}

#[test]
fn no_dependency_cycles() {
    let dep = graph();
    let mut settled: BTreeSet<String> = BTreeSet::new();
    for _ in 0..=dep.len() {
        for (n, d) in &dep {
            if d.iter().all(|x| settled.contains(x)) {
                settled.insert(n.clone());
            }
        }
    }
    let left: Vec<&String> = dep.keys().filter(|n| !settled.contains(*n)).collect();
    assert!(left.is_empty(), "循環に含まれる: {left:?}");
}

#[test]
fn implementations_do_not_own_the_shared_layout_contract() {
    // 実装の1つが所有すると、その実装を差し替える判断が借り手を巻き込む
    let owners: Vec<String> = modules()
        .into_iter()
        .filter(|(_, b)| {
            b.contains("pub struct LayoutResult") || b.contains("pub enum Unsupported")
        })
        .map(|(n, _)| n)
        .collect();
    assert_eq!(owners, vec!["layout_contract".to_owned()]);
}

#[test]
fn strategies_do_not_know_each_other() {
    let strategies = ["sugiyama", "radial", "tree", "grid"];
    let g = graph();
    for s in strategies {
        let others: Vec<&String> = g[s]
            .iter()
            .filter(|d| strategies.contains(&d.as_str()) && *d != s)
            .collect();
        assert!(others.is_empty(), "{s} が他の戦略を呼んでいる: {others:?}");
    }
}

#[test]
fn the_engine_holds_no_caller_vocabulary() {
    // 呼ぶ側を指す語。一般の日本語として使う語は入れない ── 誤検出する検査は無いより悪い
    const FORBIDDEN: [&str; 5] = ["Waffle", "waffle", "主張", "asserts", "ユビキタス"];
    let mut bad = Vec::new();
    for (name, body) in modules() {
        let hit: Vec<&str> = FORBIDDEN
            .iter()
            .copied()
            .filter(|w| body.contains(w))
            .collect();
        if !hit.is_empty() && name != "lint" && !TEMPLATE_COPIES.contains(&name.as_str()) {
            bad.push(format!("{name}: {hit:?}"));
        }
    }
    assert!(
        bad.is_empty(),
        "呼ぶ側の語彙が残っている: {}",
        bad.join(" / ")
    );
}

#[test]
fn no_guessed_numbers_remain() {
    // 書いてあるだけの規約は守られない ── 検査は実装されていても、呼ばれなければ無いのと同じ
    let found = ds_business_logic::lint::findings(&src()).expect("検査できる");
    let shown: Vec<String> = found
        .iter()
        .take(8)
        .map(|(f, l, _, t)| format!("{f}:{l} {t}"))
        .collect();
    assert!(
        found.is_empty(),
        "値の出どころが不明な数値: {}",
        shown.join(" / ")
    );
}

#[test]
fn absolute_component_cannot_be_a_node() {
    let ex = examples();
    let props = ex["edge"].as_object().expect("見本は対応表である");
    let err = render_node(
        "edge",
        props,
        &resolve("plain", None, None).expect("解決できる"),
    )
    .expect_err("断る");
    assert!(err.contains("edge"), "{err}");
}

#[test]
fn own_origin_component_can_be_a_node() {
    let ex = examples();
    let props = ex["box"].as_object().expect("見本は対応表である");
    let r = render_node(
        "box",
        props,
        &resolve("plain", None, None).expect("解決できる"),
    )
    .expect("置ける");
    assert_eq!(r.origin, Origin::Own);
}

#[test]
fn component_returns_no_svg_root() {
    // ルートを持つと、他の部品と合成したとき二重の svg / viewBox が生まれる
    let st = resolve("plain", None, None).expect("解決できる");
    let ex = examples();
    for kind in known_kinds() {
        let r = render(kind, ex[kind].as_object().expect("対応表"), &st).expect("描ける");
        assert!(!r.svg.contains("<svg"), "{kind} がルートタグを返している");
    }
}

#[test]
fn component_is_deterministic() {
    // 同じ入力から常に同じ出力。乱数も時刻も使わない
    let st = resolve("plain", None, None).expect("解決できる");
    let ex = examples();
    for kind in known_kinds() {
        let props = ex[kind].as_object().expect("対応表");
        let a = render(kind, props, &st).expect("描ける");
        let b = render(kind, props, &st).expect("描ける");
        assert_eq!(a.svg, b.svg, "{kind}");
        assert_eq!((a.width, a.height), (b.width, b.height), "{kind}");
    }
}

#[test]
fn chosen_strategy_is_used_even_with_groups() {
    use ds_business_logic::layout_contract::{LayoutResult, Sizes, Unsupported};
    use std::sync::atomic::{AtomicUsize, Ordering};
    static CALLED: AtomicUsize = AtomicUsize::new(0);
    fn spy(
        s: &Sizes,
        e: &[(String, String)],
        r: f64,
        o: f64,
        d: &str,
    ) -> Result<LayoutResult, Unsupported> {
        CALLED.fetch_add(1, Ordering::SeqCst);
        ds_business_logic::sugiyama::layout_graph(s, e, r, o, d)
    }
    let v = |x: &str| -> serde_json::Value { serde_json::from_str(x).expect("JSON") };
    let nodes = [
        v(r#"{"id":"a","label":"甲"}"#),
        v(r#"{"id":"b","label":"乙"}"#),
    ];
    let edges = [v(r#"{"from":"a","to":"b"}"#)];
    let groups = [v(r#"{"label":"束","members":["a","b"]}"#)];
    let before = CALLED.load(Ordering::SeqCst);
    ds_business_logic::compose::render_figure(&nodes, &edges, &groups, "TB", None, Some(spy))
        .expect("描ける");
    assert!(
        CALLED.load(Ordering::SeqCst) > before,
        "群を渡すと、選んだ戦略が使われずに捨てられている"
    );
    let before = CALLED.load(Ordering::SeqCst);
    ds_business_logic::compose::render_figure(&nodes[..1], &[], &[], "TB", None, Some(spy))
        .expect("描ける");
    assert!(
        CALLED.load(Ordering::SeqCst) > before,
        "群が無いときも同じ戦略が使われる"
    );
}
