// SPDX-License-Identifier: MIT
//! 層状配置の計測の道具 ── 自前（`sugiyama`）と本家（Graphviz の `dot`）を同じ宣言で測る。
//!
//!     cargo run -q -p ds_business_logic --example bench_layout
//!
//! 測るのは4つ。どれも「良し悪し」ではなく「観測値」で、判断はこの外で行う。
//!
//! 1. 交差の数 ── 辺どうしが何度すれ違うか。層の中の並びの出来を映す
//! 2. 辺の長さ ── 通る点列の総延長。層の配分の出来を映す（網状単体法が最小化するもの）
//! 3. 場所の大きさ ── 面積と縦横比。帯に伸びていないか
//! 4. 揺れ ── 同じグラフを、入力の並び順だけ変えて解き直したとき、節点がどれだけ動くか
//!
//! **並び順の入れ替えは、この道具が持つ擬似乱数で行う。** Python 版の `random.Random(0)` とは
//! 別の並びになるので、揺れの数は Python 版の記録と直接は比べられない。

use std::collections::BTreeSet;
use std::process::{Command, Stdio};

use ds_business_logic::layout_contract::LayoutResult;
use ds_business_logic::sugiyama::layout_graph;

const NODE: (f64, f64) = (90.0, 40.0);
const GAP_RANK: f64 = 60.0;
const GAP_ORDER: f64 = 30.0;
/// インチ → px。
const PT: f64 = 72.0;

type P = (f64, f64);

fn crosses(p: P, q: P, r: P, s: P) -> bool {
    let pts = [p, q, r, s];
    let same = |a: P, b: P| a == b;
    if same(p, r) || same(p, s) || same(q, r) || same(q, s) {
        return false;
    }
    let distinct = (0..4).all(|i| (i + 1..4).all(|j| pts[i] != pts[j]));
    if !distinct {
        return false;
    }
    let side = |a: P, b: P, c: P| {
        let v = (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0);
        i32::from(v > 1e-9) - i32::from(v < -1e-9)
    };
    side(p, q, r) * side(p, q, s) < 0 && side(r, s, p) * side(r, s, q) < 0
}

fn crossings(paths: &[Vec<P>]) -> usize {
    let segs: Vec<(P, P)> = paths
        .iter()
        .flat_map(|pt| pt.windows(2).map(|w| (w[0], w[1])))
        .collect();
    let mut n = 0;
    for i in 0..segs.len() {
        for j in i + 1..segs.len() {
            if crosses(segs[i].0, segs[i].1, segs[j].0, segs[j].1) {
                n += 1;
            }
        }
    }
    n
}

fn edge_length(paths: &[Vec<P>]) -> f64 {
    paths
        .iter()
        .flat_map(|pt| {
            pt.windows(2)
                .map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1))
        })
        .sum()
}

fn measure(paths: &[Vec<P>], width: f64, height: f64) -> (usize, f64, f64, f64) {
    let (w, h) = (width.max(1e-9), height.max(1e-9));
    (
        crossings(paths),
        edge_length(paths).round(),
        (w * h / 1000.0).round(),
        (w.max(h) / w.min(h) * 10.0).round() / 10.0,
    )
}

fn paths_of(r: &LayoutResult) -> Vec<Vec<P>> {
    r.edge_paths.iter().map(|(_, p)| p.clone()).collect()
}

fn solve(nodes: &[String], edges: &[(String, String)]) -> LayoutResult {
    let sizes: Vec<(String, P)> = nodes.iter().map(|n| (n.clone(), NODE)).collect();
    layout_graph(&sizes, edges, GAP_RANK, GAP_ORDER, "TB").expect("解ける")
}

fn moved(a: &LayoutResult, b: &LayoutResult) -> Vec<f64> {
    a.positions
        .iter()
        .map(|(k, p)| {
            let q = b.position(k).expect("在る");
            (q.0 - p.0).abs() + (q.1 - p.1).abs()
        })
        .collect()
}

/// 擬似乱数 ── xorshift64。**再現できる並びを作るためだけに使う。**
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = (self.next() % (i as u64 + 1)) as usize;
            v.swap(i, j);
        }
    }
}

/// 入力の並び順だけを変えて解き直し、節点がどれだけ動くかを測る。**読み手から見て同じ図なので、
/// 動かないのが正しい。**
fn wobble(nodes: &[String], edges: &[(String, String)], rounds: usize) -> (f64, usize, usize) {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let base = solve(nodes, edges);
    let mut moves = Vec::new();
    let mut shapes = BTreeSet::new();
    for _ in 0..rounds {
        let mut ns = nodes.to_vec();
        rng.shuffle(&mut ns);
        let mut es = edges.to_vec();
        rng.shuffle(&mut es);
        let r = solve(&ns, &es);
        moves.push(moved(&base, &r).into_iter().fold(0.0, f64::max));
        shapes.insert((r.width.round() as i64, r.height.round() as i64));
    }
    (
        moves.iter().copied().fold(0.0, f64::max).round(),
        moves.iter().filter(|m| **m < 0.5).count(),
        shapes.len(),
    )
}

/// 節点を1つ足して、元から居た節点がどれだけ動くかを測る。**足した1つの周りだけが動くのが望ましい。**
fn nudge(nodes: &[String], edges: &[(String, String)]) -> (f64, String, String) {
    let base = solve(nodes, edges);
    let target = edges[edges.len() / 2].0.clone();
    let mut grown = nodes.to_vec();
    grown.push("新".to_owned());
    let mut es = edges.to_vec();
    es.push((target, "新".to_owned()));
    let after = solve(&grown, &es);
    let m = moved(&base, &after);
    (
        m.iter().copied().fold(0.0, f64::max).round(),
        format!("{}/{}", m.iter().filter(|v| **v > 0.5).count(), m.len()),
        format!(
            "{}x{}→{}x{}",
            base.width.round(),
            base.height.round(),
            after.width.round(),
            after.height.round()
        ),
    )
}

fn to_dot(nodes: &[String], edges: &[(String, String)]) -> String {
    let mut lines = vec![
        "digraph g {".to_owned(),
        "  graph [rankdir=TB];".to_owned(),
        format!(
            "  node [shape=box fixedsize=true width={:.4} height={:.4}];",
            NODE.0 / PT,
            NODE.1 / PT
        ),
        format!(
            "  graph [nodesep={:.4} ranksep={:.4}];",
            GAP_ORDER / PT,
            GAP_RANK / PT
        ),
    ];
    lines.extend(nodes.iter().map(|n| format!("  \"{n}\";")));
    lines.extend(edges.iter().map(|(a, b)| format!("  \"{a}\" -> \"{b}\";")));
    lines.push("}".to_owned());
    lines.join("\n")
}

/// `dot -Tplain` に解かせて、同じ基準で測れる形へ読み替える。**本家は節点の縁から縁へ、自前は
/// 中心から中心へ辺を返すので、両端を節点の中心へ置き換えて同じ土俵に載せる。**
fn run_dot(src: &str) -> Option<(Vec<Vec<P>>, f64, f64)> {
    use std::io::Write as _;
    let mut child = Command::new("dot")
        .arg("-Tplain")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(src.as_bytes()).ok()?;
    let out = child.wait_with_output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let (mut w, mut h) = (0.0, 0.0);
    let mut positions: Vec<(String, P)> = Vec::new();
    let mut paths: Vec<(String, String, Vec<P>)> = Vec::new();
    let num = |s: &str| s.parse::<f64>().unwrap_or(0.0) * PT;
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        match f.first() {
            Some(&"graph") => (w, h) = (num(f[2]), num(f[3])),
            Some(&"node") => {
                positions.push((f[1].trim_matches('"').to_owned(), (num(f[2]), num(f[3]))))
            }
            Some(&"edge") => {
                let n: usize = f[3].parse().unwrap_or(0);
                let pts = (0..n)
                    .map(|i| (num(f[4 + 2 * i]), num(f[5 + 2 * i])))
                    .collect();
                paths.push((
                    f[1].trim_matches('"').to_owned(),
                    f[2].trim_matches('"').to_owned(),
                    pts,
                ));
            }
            _ => {}
        }
    }
    let at = |k: &str| positions.iter().find(|(n, _)| n == k).map(|(_, p)| *p);
    let fixed = paths
        .into_iter()
        .map(|(t, hd, mut pts)| {
            if let (Some(a), Some(b)) = (at(&t), at(&hd)) {
                let last = pts.len() - 1;
                pts[0] = a;
                pts[last] = b;
            }
            pts
        })
        .collect();
    Some((fixed, w, h))
}

fn s(x: &str) -> String {
    x.to_owned()
}

fn tree(width: usize, depth: usize) -> (Vec<String>, Vec<(String, String)>) {
    let mut nodes = vec![s("根")];
    let mut edges = Vec::new();
    for i in 0..width {
        let c = format!("枝{i}");
        nodes.push(c.clone());
        edges.push((s("根"), c.clone()));
        for j in 0..depth {
            let g = format!("葉{i}_{j}");
            nodes.push(g.clone());
            edges.push((c.clone(), g));
        }
    }
    (nodes, edges)
}

fn skipping(n: usize) -> (Vec<String>, Vec<(String, String)>) {
    let nodes: Vec<String> = (0..n).map(|i| format!("層{i}")).collect();
    let mut edges: Vec<(String, String)> = (0..n - 1)
        .map(|i| (nodes[i].clone(), nodes[i + 1].clone()))
        .collect();
    for (a, b) in [(0, n - 1), (1, 5), (2, 7)] {
        edges.push((nodes[a].clone(), nodes[b].clone()));
    }
    (nodes, edges)
}

fn tangled(a: usize, b: usize) -> (Vec<String>, Vec<(String, String)>) {
    let left: Vec<String> = (0..a).map(|i| format!("左{i}")).collect();
    let right: Vec<String> = (0..b).map(|j| format!("右{j}")).collect();
    let mut edges = Vec::new();
    for (i, l) in left.iter().enumerate() {
        for (j, r) in right.iter().enumerate() {
            if (i + j) % 2 == 0 {
                edges.push((l.clone(), r.clone()));
            }
        }
    }
    (left.into_iter().chain(right).collect(), edges)
}

fn cyclic() -> (Vec<String>, Vec<(String, String)>) {
    let mut nodes: Vec<String> = (0..6).map(|i| format!("環{i}")).collect();
    nodes.extend([s("入口"), s("出口")]);
    let mut edges: Vec<(String, String)> = (0..6)
        .map(|i| (format!("環{i}"), format!("環{}", (i + 1) % 6)))
        .collect();
    edges.extend([
        (s("入口"), s("環0")),
        (s("環3"), s("出口")),
        (s("入口"), s("環4")),
    ]);
    (nodes, edges)
}

/// 層が構造から決まらない図 ── 長さの違う道が1点へ合流する。
fn uneven() -> (Vec<String>, Vec<(String, String)>) {
    let pairs = [
        ("長1", "長2"),
        ("長2", "長3"),
        ("長3", "長4"),
        ("長4", "合流"),
        ("短1", "短2"),
        ("短2", "合流"),
        ("中1", "中2"),
        ("中2", "中3"),
        ("中3", "合流"),
        ("合流", "出口"),
    ];
    let nodes = [
        "長1", "長2", "長3", "長4", "短1", "短2", "中1", "中2", "中3", "合流", "出口",
    ];
    (
        nodes.iter().map(|x| s(x)).collect(),
        pairs.iter().map(|(a, b)| (s(a), s(b))).collect(),
    )
}

fn main() {
    let cases = [
        ("枝の多い木", tree(12, 2)),
        ("多層をまたぐ辺", skipping(8)),
        ("交差の多いグラフ", tangled(5, 5)),
        ("サイクルを含む", cyclic()),
        ("層が決まらない図", uneven()),
    ];
    println!(
        "{:<16}{:<8}{:>6}{:>10}{:>8}{:>8}",
        "案件", "手", "交差", "辺の長さ", "面積", "縦横比"
    );
    println!("{}", "-".repeat(56));
    for (name, (nodes, edges)) in &cases {
        let r = solve(nodes, edges);
        let m = measure(&paths_of(&r), r.width, r.height);
        println!(
            "{name:<16}{:<8}{:>6}{:>10}{:>8}{:>8}",
            "自前", m.0, m.1, m.2, m.3
        );
        match run_dot(&to_dot(nodes, edges)) {
            Some((paths, w, h)) => {
                let t = measure(&paths, w, h);
                println!(
                    "{:<16}{:<8}{:>6}{:>10}{:>8}{:>8}",
                    "", "本家", t.0, t.1, t.2, t.3
                );
            }
            None => println!("{:<16}{:<8}{:>6}", "", "本家", "未導入"),
        }
    }
    println!("\n揺れ ── 宣言は同じで、入力の並び順だけを変えて8回解き直す");
    println!(
        "{:<16}{:>16}{:>16}{:>12}",
        "案件", "最も動いた節点", "全く動かない回", "外形の種類"
    );
    println!("{}", "-".repeat(60));
    for (name, (nodes, edges)) in &cases {
        let (m, still, shapes) = wobble(nodes, edges, 8);
        println!("{name:<16}{m:>16}{:>16}{shapes:>12}", format!("{still}/8"));
    }
    println!("\n継ぎ足し ── 節点を1つ足したとき、元から居た節点がどれだけ動くか");
    println!(
        "{:<16}{:>18}{:>16}{:>22}",
        "案件", "最も動いた既存節点", "動いた既存節点", "外形の変化"
    );
    println!("{}", "-".repeat(72));
    for (name, (nodes, edges)) in &cases {
        let (m, count, shape) = nudge(nodes, edges);
        println!("{name:<16}{m:>18}{count:>16}{shape:>22}");
    }
}
