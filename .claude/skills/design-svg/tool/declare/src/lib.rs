// SPDX-License-Identifier: MIT
//! design-svg の道具の宣言。**能力の正本はここである。**
//!
//! 計算は部品（`ds_parts`）が保持し、この宣言は**呼び方だけ**を固定する。
//! 入口（CLI ・ MCP）はこの宣言から組む ── 能力を2回書くと、片方だけが古くなる。
//! 許可辺は `Cargo.toml` が宣言する ── この crate は部品だけを参照する。
//!
//! **この Skill は、呼ぶ側の語彙を知らない。** 受けるのは節点・辺・囲みという
//! 一般名詞と、部品・トークン・配置戦略の名前だけである ── 固有の語彙から
//! この宣言へ直す変換は、呼ぶ側が持つ。

pub mod contract;

use std::cell::RefCell;
use std::path::PathBuf;

use ds_parts::grid::{self, Grid, Key};
use ds_parts::layout_contract::{LayoutResult, Sizes, Strategy, Unsupported};
use ds_parts::{canvas, catalog, compose, lint, radial, theme, tree, verify};
use serde::Serialize as _;
use serde_json::{json, Map, Value};

pub use contract::{catalog, Arg, Given, Outcome, Tool};

/// 配置戦略 ── 名前から実体へ。**呼ぶ側に関数を渡させない。**
/// 既定（層状）は `None` で表す ── 組み立て側が既定を持つ。
const LAYOUTS: [(&str, Option<Strategy>); 4] = [
    ("graph", None),
    ("radial", Some(radial::layout_radial)),
    ("tree", Some(tree::layout_tree)),
    ("grid", Some(declared_grid)),
];

thread_local! {
    /// 格子の座標。**配置戦略は関数であって値を束ねられない**ので、宣言から読んだ座標をここに
    /// 置き、戦略がそれを参照する。組み立ての直前に置き、直後に空へ戻す。
    static GRID: RefCell<Grid> = RefCell::new(Grid::default());
}

/// 宣言が持つ座標のとおりに置く。**座標は [`GRID`] から受け取る。**
fn declared_grid(
    sizes: &Sizes,
    edges: &[(String, String)],
    gap_rank: f64,
    gap_order: f64,
    _direction: &str,
) -> Result<LayoutResult, Unsupported> {
    GRID.with(|g| grid::layout_grid(sizes, edges, gap_rank, gap_order, &g.borrow()))
}

/// 宣言の `grid` を読む ── `{"at": {節点: [列, 行]}, "cols": [...], "rows": [...],
/// "elbow": [[始点, 終点, "vertical" | "horizontal"], ...]}`。**`cols` ・ `rows` ・ `elbow` は省ける。**
fn grid_of(d: &Value) -> Result<Grid, String> {
    let g = d
        .get("grid")
        .and_then(Value::as_object)
        .ok_or("格子の配置には grid が要る ── {\"at\": {節点: [列, 行]}}")?;
    let at = g
        .get("at")
        .and_then(Value::as_object)
        .ok_or("grid.at が無い ── 節点ごとの [列, 行] を渡す")?;
    let mut out = Grid::default();
    for (id, cell) in at {
        let pair = cell
            .as_array()
            .filter(|a| a.len() == 2)
            .ok_or_else(|| format!("grid.at.{id} は [列, 行] でなければならない"))?;
        out.at
            .push((id.clone(), (Key::of(&pair[0]), Key::of(&pair[1]))));
    }
    let keys = |k: &str| {
        g.get(k)
            .and_then(Value::as_array)
            .map(|a| a.iter().map(Key::of).collect())
            .unwrap_or_default()
    };
    out.cols = keys("cols");
    out.rows = keys("rows");
    for e in g
        .get("elbow")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
    {
        let parts: Vec<&str> = e
            .as_array()
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        let [from, to, corner] = parts[..] else {
            return Err(format!(
                "grid.elbow の各行は [始点, 終点, 置き方] である: {e}"
            ));
        };
        out.elbow
            .push(((from.to_owned(), to.to_owned()), corner.to_owned()));
    }
    Ok(out)
}

/// 欄の文字列。**無ければ空** ── 空の名前は、どの辺とも一致しない。
fn str_of(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_owned()
}

/// 部品の置き場所 ── `lint` が既定で探す場所である。
const PARTS_SRC: &str = "tool/parts/src";

fn read_json(path: &str) -> Result<Value, String> {
    let body = std::fs::read_to_string(path).map_err(|e| format!("読めない: {path} ── {e}"))?;
    serde_json::from_str(&body).map_err(|e| format!("読めない: {path} ── {e}"))
}

/// 生成物を置く。**置き場所を渡されなければ、そのまま返す。**
fn write(svg: &str, out: &str, data: &mut Map<String, Value>) -> Result<(), String> {
    if !out.is_empty() {
        std::fs::write(out, svg).map_err(|e| format!("書けない: {out} ── {e}"))?;
    }
    data.insert("path".to_owned(), Value::from(out));
    data.insert(
        "svg".to_owned(),
        Value::from(if out.is_empty() { svg } else { "" }),
    );
    // **字数で数える** ── バイトで数えると、日本語を含む図で読み手の数え方と食い違う
    data.insert("bytes".to_owned(), Value::from(svg.chars().count()));
    Ok(())
}

/// 描いた SVG を検査して、結果を組む。**幾何の検査を3種とも当てる。**
fn drawn(svg: &str, out: &str, mut data: Map<String, Value>) -> Outcome {
    if let Err(why) = write(svg, out, &mut data) {
        return Outcome::misuse(why);
    }
    Outcome::found(verify::all(svg), Value::Object(data))
}

fn run_catalog(given: &Given) -> Outcome {
    let d = match catalog::catalog() {
        Ok(d) => d,
        Err(why) => return Outcome::misuse(why),
    };
    // **移す前と同じ形で書く** ── 1字下げ ・ 非 ASCII をそのまま
    let mut buf = Vec::new();
    let mut w = serde_json::Serializer::with_formatter(
        &mut buf,
        serde_json::ser::PrettyFormatter::with_indent(b" "),
    );
    if let Err(e) = d.serialize(&mut w) {
        return Outcome::misuse(format!("目録を組めない ── {e}"));
    }
    let body = String::from_utf8_lossy(&buf).into_owned();
    let out = given.one("out", "");
    if !out.is_empty() {
        if let Err(e) = std::fs::write(out, format!("{body}\n")) {
            return Outcome::misuse(format!("書けない: {out} ── {e}"));
        }
    }
    let keys = |k: &str| -> Vec<String> {
        let mut v: Vec<String> = d[k]
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        v.sort();
        v
    };
    Outcome::found(
        Vec::new(),
        json!({
            "path": out,
            "parts": keys("parts"),
            "strategies": keys("strategies"),
            "tokens": d["tokens"].as_object().map_or(0, Map::len),
            "body": if out.is_empty() { body } else { String::new() },
        }),
    )
}

fn human_catalog(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join("\n");
    }
    let d = &out.data;
    let body = d["body"].as_str().unwrap_or_default();
    if !body.is_empty() {
        return body.to_owned();
    }
    let count = |k: &str| d[k].as_array().map_or(0, Vec::len);
    format!(
        "目録を書き出した: {}　／　部品 {} ・ 配置戦略 {} ・ トークン {}",
        d["path"].as_str().unwrap_or_default(),
        count("parts"),
        count("strategies"),
        d["tokens"]
    )
}

/// 宣言の中の `theme` を、テーマの上書きとして受ける。**渡すのは差分だけでよい** ──
/// 既定のテーマへ重ねる。
///
/// **入口がテーマを通さないと、呼ぶ側は台本を書くことになる** ── そしてその台本は
/// 呼ぶ側の作業場に残るだけで、成果物の隣には何も残らない。
fn theme_of(d: &Value, strict: bool) -> Result<Option<Map<String, Value>>, String> {
    let Some(over) = d.get("theme").filter(|v| !is_empty(v)) else {
        return Ok(None);
    };
    let Some(over) = over.as_object() else {
        return Err("theme は名前と値の対でなければならない".to_owned());
    };
    let base = theme::default_theme();
    if strict {
        // **役割の行は、指す先の名前で判定する** ── `role.<役割>.<名前>` の `<名前>` が、既定の
        // トークンか、既存の役割の行が使っている名前なら受け付ける。断ると、新しい役割を宣言から
        // 足せなくなる。**それ以外は綴りの誤りとして断る**
        let role_target = |k: &str| -> Option<String> {
            let (role, token) = k.strip_prefix(theme::ROLE_PREFIX)?.split_once('.')?;
            (!role.is_empty() && !token.is_empty()).then(|| token.to_owned())
        };
        let readable: std::collections::BTreeSet<String> = base
            .keys()
            .filter_map(|k| role_target(k))
            .chain(base.keys().cloned())
            .collect();
        let known =
            |k: &str| base.contains_key(k) || role_target(k).is_some_and(|t| readable.contains(&t));
        let mut unknown: Vec<&String> = over.keys().filter(|k| !known(k)).collect();
        unknown.sort();
        if !unknown.is_empty() {
            let names: Vec<&str> = unknown.iter().map(|s| s.as_str()).collect();
            return Err(format!(
                "知らないトークン: {} ── 目録（catalog）に在る名前だけを使う",
                names.join(" ・ ")
            ));
        }
    }
    let mut merged = base.clone();
    for (k, v) in over {
        merged.insert(k.clone(), v.clone());
    }
    Ok(Some(merged))
}

/// 値が「無い」と同じか ── 空の対応表 ・ 空の並び ・ 空の文字列 ・ null ・ 偽 ・ 0。
fn is_empty(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::Bool(b) => !b,
        Value::Number(n) => n.as_f64() == Some(0.0),
        Value::String(s) => s.is_empty(),
        Value::Array(a) => a.is_empty(),
        Value::Object(o) => o.is_empty(),
    }
}

fn list(d: &Value, key: &str) -> Vec<Value> {
    d.get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn run_figure(given: &Given) -> Outcome {
    let d = match read_json(given.one("declaration", "")) {
        Ok(d) => d,
        Err(why) => return Outcome::misuse(why),
    };
    // **宣言に書いたものを、入口が読む** ── 引数でしか渡せないと、
    // 宣言だけでは同じ図が組み直せない（実測で、direction が無視された）
    let layout = d
        .get("layout")
        .and_then(Value::as_str)
        .unwrap_or_else(|| given.one("layout", "graph"));
    let direction = d
        .get("direction")
        .and_then(Value::as_str)
        .unwrap_or_else(|| given.one("direction", "TB"));
    let Some((_, strategy)) = LAYOUTS.iter().find(|(name, _)| *name == layout) else {
        let names: Vec<&str> = LAYOUTS.iter().map(|(n, _)| *n).collect();
        return Outcome::misuse(format!(
            "知らない配置戦略: {layout}。使えるのは {} である",
            names.join("／")
        ));
    };
    let theme_ = match theme_of(&d, true) {
        Ok(t) => t,
        Err(why) => return Outcome::misuse(why),
    };
    if layout == "grid" {
        match grid_of(&d) {
            Ok(g) => {
                // **鍵線が宣言に無い辺を指すなら断る** ── 配置は段ごとなので、図全体を見られるのはここだけ
                let declared: Vec<(String, String)> = list(&d, "edges")
                    .iter()
                    .map(|e| (str_of(e, "from"), str_of(e, "to")))
                    .collect();
                let absent: Vec<&(String, String)> = g
                    .elbow
                    .iter()
                    .map(|(k, _)| k)
                    .filter(|k| !declared.contains(k))
                    .collect();
                if !absent.is_empty() {
                    return Outcome::misuse(format!("辺に無いものが elbow にあります: {absent:?}"));
                }
                GRID.with(|slot| *slot.borrow_mut() = g);
            }
            Err(why) => return Outcome::misuse(why),
        }
    }
    let placed = compose::render_figure(
        &list(&d, "nodes"),
        &list(&d, "edges"),
        &list(&d, "groups"),
        direction,
        theme_.as_ref(),
        *strategy,
    );
    // **座標を残さない** ── 次の呼び出しが、前の図の格子を読むことになる
    GRID.with(|slot| *slot.borrow_mut() = Grid::default());
    match placed {
        Ok(svg) => drawn(&svg, given.one("out", ""), Map::new()),
        Err(why) => Outcome::misuse(why),
    }
}

fn run_chart(given: &Given) -> Outcome {
    let kind = given.one("kind", "");
    let d = match read_json(given.one("data", "")) {
        Ok(d) => d,
        Err(why) => return Outcome::misuse(why),
    };
    let Some(props) = d.as_object() else {
        return Outcome::misuse("データは名前と値の対でなければならない".to_owned());
    };
    let theme_ = match theme_of(&d, true) {
        Ok(t) => t,
        Err(why) => return Outcome::misuse(why),
    };
    match compose::render_chart(kind, props, theme::PLAIN, None, theme_.as_ref()) {
        Ok(svg) => {
            let mut data = Map::new();
            data.insert("kind".to_owned(), Value::from(kind));
            drawn(&svg, given.one("out", ""), data)
        }
        Err(why) => Outcome::misuse(why),
    }
}

/// 画布の大きさを読む。**引数が宣言に勝つ。**
fn side(given: &Given, d: &Value, key: &str) -> Result<f64, String> {
    let arg = given.one(key, "");
    if !arg.is_empty() {
        return arg
            .trim()
            .parse()
            .map_err(|_| format!("{key} が数でない: {arg}"));
    }
    Ok(d.get(key).and_then(Value::as_f64).unwrap_or(0.0))
}

fn run_canvas(given: &Given) -> Outcome {
    let d = match read_json(given.one("layers", "")) {
        Ok(d) => d,
        Err(why) => return Outcome::misuse(why),
    };
    // **渡す JSON は `{"width", "height", "layers"}` か、層の並びだけ**
    let body = match &d {
        Value::Array(a) => a.clone(),
        Value::Object(_) => match d.get("layers").and_then(Value::as_array) {
            Some(a) => a.clone(),
            None => return Outcome::misuse("layers が無い ── 層の並びを渡す".to_owned()),
        },
        _ => return Outcome::misuse("層は並びか、名前と値の対で渡す".to_owned()),
    };
    let (w, h) = match (side(given, &d, "width"), side(given, &d, "height")) {
        (Ok(w), Ok(h)) => (w, h),
        (Err(why), _) | (_, Err(why)) => return Outcome::misuse(why),
    };
    if w == 0.0 || h == 0.0 {
        return Outcome::misuse("画布の大きさが無い ── width と height を渡す".to_owned());
    }
    // **画布のテーマは名前を検査しない** ── 移す前の振る舞いを保つ
    let theme_ = match theme_of(&d, false) {
        Ok(t) => t,
        Err(why) => return Outcome::misuse(why),
    };
    let background = d
        .get("background")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty());
    match canvas::render_canvas(w, h, &body, theme_.as_ref(), background) {
        Ok(svg) => drawn(&svg, given.one("out", ""), Map::new()),
        Err(why) => Outcome::misuse(why),
    }
}

fn human_svg(out: &Outcome) -> String {
    let mut lines: Vec<String> = out.findings.iter().map(|x| format!("  × {x}")).collect();
    if !out.ok {
        return lines.join("\n");
    }
    // **宣言が通らなかったときも、検出を読める形で出す**
    let path = out.data["path"].as_str().unwrap_or_default();
    if !path.is_empty() {
        lines.push(if out.findings.is_empty() {
            "幾何の検査　通った".to_owned()
        } else {
            format!("幾何の検査　通っていない（{} 件）", out.findings.len())
        });
        lines.push(format!("書き出し: {path}　／　{} 字", out.data["bytes"]));
    } else {
        let svg = out.data["svg"].as_str().unwrap_or_default();
        if !svg.is_empty() {
            lines.push(svg.to_owned());
        }
    }
    lines.join("\n")
}

fn run_verify(given: &Given) -> Outcome {
    let path = given.one("svg", "");
    match std::fs::read_to_string(path) {
        Ok(body) => Outcome::found(verify::all(&body), json!({ "svg": path })),
        Err(e) => Outcome::misuse(format!("読めない: {path} ── {e}")),
    }
}

/// 幾何の検査の結果を並べる。**この検査が見ないもの**が3つある ── 極端な縦横比、
/// 配置戦略の選び違い、詰まり・読みにくさ・配色の良し悪し。目視の代わりにはならない。
fn human_verify(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join("\n");
    }
    let mut lines: Vec<String> = out.findings.iter().map(|x| format!("  × {x}")).collect();
    lines.push(if out.findings.is_empty() {
        "幾何の検査　通った".to_owned()
    } else {
        format!("幾何の検査　通っていない（{} 件）", out.findings.len())
    });
    lines.join("\n")
}

fn run_lint(given: &Given) -> Outcome {
    let path = given.one("path", "");
    let root: PathBuf = if path.is_empty() {
        match given.skill_root() {
            Ok(root) => root.join(PARTS_SRC),
            Err(why) => return Outcome::misuse(why),
        }
    } else {
        PathBuf::from(path)
    };
    match lint::findings(&root) {
        Ok(hits) => Outcome::found(
            hits.into_iter()
                .map(|(name, line, func, text)| format!("{name}:{line} {func} ── {}", text.trim()))
                .collect(),
            json!({ "root": root.display().to_string() }),
        ),
        // **無い場所を検査して「0 箇所」と返さない。** 呼ぶ側は合格と受け取る
        Err(why) => Outcome::misuse(why),
    }
}

fn human_lint(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join("\n");
    }
    let mut lines: Vec<String> = out.findings.iter().map(|x| format!("  × {x}")).collect();
    lines.push(format!("生の数値: {} 箇所", out.findings.len()));
    lines.join("\n")
}

/// この Skill が持つ道具の一覧。**能力の正本である。**
#[must_use]
pub fn tools() -> Vec<Tool> {
    let out = Arg::opt("out", "書き出し先の SVG", None);
    vec![
        Tool {
            name: "catalog",
            summary: "目録を出す（部品・トークン・役割・配置戦略）",
            args: vec![Arg::opt("out", "書き出し先。省くとそのまま出す", None)],
            run: run_catalog,
            human: human_catalog,
        },
        Tool {
            name: "figure",
            summary: "宣言（節点・辺・囲み）から図を組む",
            args: vec![
                Arg::need("declaration", "宣言の JSON"),
                out.clone(),
                Arg::opt(
                    "layout",
                    "配置戦略（graph ／ radial ／ tree）",
                    Some("graph"),
                ),
                Arg::opt("direction", "向き（TB ／ LR）", Some("TB")),
            ],
            run: run_figure,
            human: human_svg,
        },
        Tool {
            name: "chart",
            summary: "量を描く部品を1つ選んで描く",
            args: vec![
                Arg::need("kind", "部品の名前"),
                Arg::need("data", "データの JSON"),
                out.clone(),
            ],
            run: run_chart,
            human: human_svg,
        },
        Tool {
            name: "canvas",
            summary: "画布へ部品を重ねて、意匠そのものを組む",
            args: vec![
                Arg::need("layers", "層の JSON"),
                out,
                Arg::opt("width", "画布の幅", None),
                Arg::opt("height", "画布の高さ", None),
            ],
            run: run_canvas,
            human: human_svg,
        },
        Tool {
            name: "verify",
            summary: "生成物の幾何を検査する",
            args: vec![Arg::need("svg", "検査する SVG")],
            run: run_verify,
            human: human_verify,
        },
        Tool {
            name: "lint",
            summary: "生の数値を探す",
            args: vec![
                Arg::opt("path", "探す場所。省くと部品の crate 全体", None),
                Arg::opt(
                    "skill_root",
                    "この Skill の置き場所（既定は、実行ファイルの1つ上）",
                    None,
                ),
            ],
            run: run_lint,
            human: human_lint,
        },
    ]
}
