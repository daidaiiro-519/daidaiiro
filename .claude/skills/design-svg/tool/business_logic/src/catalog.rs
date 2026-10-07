// SPDX-License-Identifier: MIT
//! 目録 ── **このエンジンが受け取れるものを、外へ公開する。**
//!
//! 利用側（変換器を書く人）は、ここだけを参照すれば済む ── どの部品があり、それぞれがどんな値を
//! 読み、どんなトークンで見た目が決まり、どの配置戦略が選べるか。
//!
//! **部品が読む値の表は、部品の関数の本文と突き合わせて縛る** ── 手書きの目録は、実物とずれても
//! 誰も気づけない（実測 ── 契約が3か所に書かれたまま11ファイルで破れていた）。表と本文が食い
//! 違えば、事例が失敗する。置き方 ・ 名前を自分で描くか ・ 見本の大きさは、**見本を実際に描いて
//! 確かめた結果**を載せる。
//!
//! **ここに、呼ぶ側の言い分の名前を置かない。** この目録が答えるのは「何を受け取れるか」だけで、
//! 「何を表せるか」ではない。

use serde_json::{json, Map, Value};

use crate::boolean::circle_polygon;
use crate::registry::{self, Origin};
use crate::style;
use crate::theme::{self, PLAIN, ROLE_PREFIX};

/// 読む値1つ。`(名前, 必須か, 既定値の書き方)`
pub type Prop = (&'static str, bool, Option<&'static str>);

/// 部品ごとの、**自分で読む**値と、受け取った値をそのまま渡す先。
///
/// 渡し先が値で決まるなら `props:<キー>` と書く。
pub const PARTS: &[(&str, &[Prop], Option<&str>)] = &[
    (
        "bars",
        &[
            ("axis_label", false, None),
            ("bars", true, None),
            ("baseline", false, Some("0")),
            ("item_axis_label", false, None),
        ],
        None,
    ),
    (
        "boolean",
        &[("op", true, None), ("shapes", true, None)],
        None,
    ),
    (
        "box",
        &[
            ("label", false, Some("''")),
            ("role", false, Some("'plain'")),
        ],
        None,
    ),
    (
        "donut",
        &[("centre", false, None), ("slices", true, None)],
        None,
    ),
    (
        "edge",
        &[
            ("arrow", false, Some("'head'")),
            ("arrowhead", false, Some("'solid'")),
            ("dashed", false, None),
            ("label", false, None),
            ("label_at", false, None),
            ("points", true, None),
        ],
        None,
    ),
    (
        "exchange",
        &[
            ("groups", false, Some("[]")),
            ("participants", true, None),
            ("steps", true, None),
        ],
        None,
    ),
    ("flow", &[("links", true, None)], None),
    (
        "frame",
        &[
            ("height", true, None),
            ("width", true, None),
            ("x", true, None),
            ("y", true, None),
        ],
        None,
    ),
    (
        "frame_label",
        &[("label", true, None), ("x", true, None), ("y", true, None)],
        None,
    ),
    ("hex", &[("label", false, Some("''"))], None),
    (
        "lanes",
        &[
            ("axis_label", false, None),
            ("rows", true, None),
            ("span", false, None),
        ],
        None,
    ),
    (
        "path",
        &[("d", true, None), ("filled", false, Some("True"))],
        None,
    ),
    (
        "pie",
        &[("legend", false, Some("True")), ("slices", true, None)],
        Some("donut"),
    ),
    ("ranking", &[("items", true, None)], None),
    (
        "scatter",
        &[
            ("points", true, None),
            ("x_label", false, None),
            ("y_label", false, None),
        ],
        None,
    ),
    (
        "spatial",
        &[
            ("axis_label", false, None),
            ("cols", false, Some("1")),
            ("ground", false, None),
            ("items", true, None),
        ],
        None,
    ),
    (
        "table",
        &[
            ("axes", false, None),
            ("headers", true, None),
            ("rows", true, None),
        ],
        None,
    ),
    (
        "titled",
        &[("label", false, Some("''")), ("of", true, None)],
        Some("props:of"),
    ),
];

/// 図の宣言そのものが受け取るキー。
pub const DECLARATION: &[(&str, &[Prop])] = &[
    (
        "nodes",
        &[
            ("figure", false, None),
            ("id", true, None),
            ("label", false, None),
            ("role", false, Some("'plain'")),
            ("style", false, None),
        ],
    ),
    (
        "edges",
        &[
            ("arrow", false, Some("'head'")),
            ("dashed", false, Some("False")),
            ("from", true, None),
            ("label", false, None),
            ("to", true, None),
        ],
    ),
    ("groups", &[("label", false, None), ("members", true, None)]),
];

/// 選べる配置戦略と、**それが何を根拠に位置を決めるか**。
pub const STRATEGIES: [(&str, &str); 4] = [
    ("layout_graph", "辺の向きから層を決める（層状）"),
    ("layout_radial", "順が巡って戻る（環状）"),
    ("layout_tree", "中心から枝分かれする（放射の木）"),
    ("layout_grid", "宣言が持つ座標のとおりに置く（格子）"),
];

fn spec(kind: &str) -> Option<(&'static [Prop], Option<&'static str>)> {
    PARTS
        .iter()
        .find(|(k, ..)| *k == kind)
        .map(|(_, p, f)| (*p, *f))
}

fn props_json(list: &[Prop]) -> Map<String, Value> {
    let mut out = Map::new();
    let mut sorted: Vec<&Prop> = list.iter().collect();
    sorted.sort_by_key(|(k, ..)| *k);
    for (name, required, default) in sorted {
        out.insert(
            (*name).to_owned(),
            json!({"required": required, "default": default}),
        );
    }
    out
}

/// 見本の中身。**データであって、処理の数ではない** ── 名前に束ねて1か所に置く。
/// 値の `"SLICES"` は [`SLICES`] に置き換え、`"BOOLEAN"` は円を多角形へ直して組む。
const EXAMPLES: &str = r#"{
        "box": {}, "hex": {},
        "donut": {"slices": "SLICES", "centre": "18"},
        "pie": {"slices": "SLICES", "centre": "18"},
        "boolean": "BOOLEAN",
        "path": {"d": "M0,40 Q30,0 60,40 Q90,80 120,40"},
        "titled": {"of": "donut", "slices": "SLICES", "centre": "18"},
        "bars": {"bars": [{"name": "文書", "value": 13}, {"name": "図", "value": 5}]},
        "ranking": {"items": [{"name": "文書", "value": 13}, {"name": "図", "value": 5}]},
        "lanes": {"rows": [{"name": "設計", "bars": [{"from": 0, "to": 3}]}]},
        "scatter": {"points": [{"name": "a", "x": 1, "y": 2}, {"name": "b", "x": 3, "y": 4}]},
        "flow": {"links": [{"from": "A", "to": "B", "value": 5}]},
        "spatial": {"items": [{"name": "領域", "depth": 2}], "cols": 1},
        "table": {"headers": ["部品", "数"], "rows": [["形", "7"]]},
        "exchange": {"participants": ["甲", "乙"], "steps": [{"from": "甲", "to": "乙", "label": "渡す"}]},
        "frame": {"x": 0, "y": 0, "width": 80, "height": 40},
        "frame_label": {"x": 0, "y": 20, "label": "ラベル"},
        "edge": {"points": [[0, 0], [60, 40]]}
    }"#;

/// 内訳を描く部品の見本が共有する並び。
const SLICES: &str = r#"[{"name": "文書", "value": 13}, {"name": "図", "value": 5}]"#;

/// 形の演算の見本 ── 2つの円の中心の x と、共通の中心の y ・ 半径。
const BOOLEAN_CX: [f64; 2] = [40.0, 66.0];
const BOOLEAN_CY: f64 = 40.0;
const BOOLEAN_R: f64 = 34.0;

/// 部品ごとの、最小の動く入力。
///
/// **利用側が「とりあえず1つ描いてみる」ための足がかり**であり、同時に目録が実物と繋がっている
/// 証拠でもある。部品を足したらここへ1行足す ── 足し忘れは事例で検出する。
#[must_use]
pub fn examples() -> Map<String, Value> {
    let poly = |cx: f64| -> Value {
        Value::Array(
            circle_polygon(cx, BOOLEAN_CY, BOOLEAN_R, crate::boolean::CIRCLE_FACETS)
                .iter()
                .map(|(x, y)| json!([x, y]))
                .collect(),
        )
    };
    let raw_text = EXAMPLES.replace("\"SLICES\"", SLICES);
    let mut raw: Value = serde_json::from_str(&raw_text).expect("見本は JSON である");
    // 形の演算の見本だけは、円を多角形へ直して組む ── 手で頂点を書かない
    raw["boolean"] =
        json!({"shapes": [poly(BOOLEAN_CX[0]), poly(BOOLEAN_CX[1])], "op": "subtract"});
    // **名前を先に置き、見本の値で上書きする**
    let mut out = Map::new();
    for (kind, v) in raw.as_object().expect("対応表である") {
        let mut one = Map::new();
        one.insert("label".to_owned(), Value::from("名前"));
        for (k, x) in v.as_object().expect("対応表である") {
            one.insert(k.clone(), x.clone());
        }
        out.insert(kind.clone(), Value::Object(one));
    }
    out
}

/// その部品が自分で読む値の一覧。
///
/// # Errors
///
/// 台帳に無い名前のときに返す。
pub fn props_of(kind: &str) -> Result<Map<String, Value>, String> {
    let (list, _) = spec(kind).ok_or_else(|| {
        let known: Vec<String> = registry::known_kinds()
            .iter()
            .map(|k| crate::py::quote(k))
            .collect();
        format!(
            "台帳に無い部品です: {kind}（使えるのは [{}]）",
            known.join(", ")
        )
    })?;
    Ok(props_json(list))
}

/// その部品へ渡せる値の全部。**素通しの先が読むものも含む。**
///
/// 渡し先が値で決まる部品は、先が定まらないので自分の分だけを返す ── 利用側は、指す部品の目録を
/// 併せて見る。
fn accepted(kind: &str, seen: &[&str]) -> Map<String, Value> {
    let Some((list, fwd)) = spec(kind) else {
        return Map::new();
    };
    let mut out = props_json(list);
    if let Some(next) = fwd.filter(|n| !n.starts_with("props:") && !seen.contains(n)) {
        let mut chain = seen.to_vec();
        chain.push(kind);
        for (k, v) in accepted(next, &chain) {
            out.entry(k).or_insert(v);
        }
    }
    let mut sorted: Vec<(String, Value)> = out.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    sorted.into_iter().collect()
}

/// 小数第1位で丸める。**正確な2進の値を丸める** ── 移す前と同じ丸め方である。
fn round1(v: f64) -> f64 {
    format!("{v:.1}").parse().unwrap_or(v)
}

/// 部品ごとの、受け取る値 ・ 置き方 ・ 名前を自分で描くか ・ 見本の大きさ。
///
/// # Errors
///
/// 見本を描けないときに返す。
pub fn parts() -> Result<Map<String, Value>, String> {
    let st = style::resolve(PLAIN, None, None)?;
    let ex = examples();
    let mut out = Map::new();
    for kind in registry::known_kinds() {
        let mut entry = Map::new();
        entry.insert("props".to_owned(), Value::Object(accepted(kind, &[])));
        let own: Vec<String> = props_of(kind)?.keys().cloned().collect();
        entry.insert("reads_itself".to_owned(), json!(own));
        if let Some((_, Some(fwd))) = spec(kind) {
            entry.insert("forwards_to".to_owned(), Value::from(fwd));
        }
        if let Some(example) = ex.get(kind).and_then(Value::as_object) {
            let r = registry::render(kind, example, &st)?;
            entry.insert("example".to_owned(), Value::Object(example.clone()));
            entry.insert(
                "placement".to_owned(),
                Value::from(if r.origin == Origin::Own {
                    "own-origin"
                } else {
                    "absolute"
                }),
            );
            entry.insert("labels_itself".to_owned(), Value::Bool(r.labels_itself));
            let size = |v: f64, int: bool| {
                if int {
                    json!(v as i64)
                } else {
                    json!(round1(v))
                }
            };
            entry.insert(
                "example_size".to_owned(),
                json!([size(r.width, r.int_size.0), size(r.height, r.int_size.1)]),
            );
        }
        out.insert(kind.to_owned(), Value::Object(entry));
    }
    Ok(out)
}

/// 正本の範囲を、書いたとおりの数で引く。
fn range_of(key: &str) -> Value {
    let whole: Value =
        serde_json::from_str(include_str!("../../../references/theme.json")).unwrap_or(Value::Null);
    whole
        .pointer(&format!(
            "/ranges/{}",
            key.replace('~', "~0").replace('/', "~1")
        ))
        .cloned()
        .unwrap_or(Value::Null)
}

/// 見た目を決める値の一覧。**範囲を持つものは、その範囲も添える。**
#[must_use]
pub fn tokens() -> Map<String, Value> {
    let mut keys: Vec<&String> = theme::default_theme()
        .keys()
        .filter(|k| !k.starts_with(ROLE_PREFIX))
        .collect();
    keys.sort();
    keys.into_iter()
        .map(|k| {
            (
                k.clone(),
                json!({"default": theme::default_theme()[k], "range": range_of(k)}),
            )
        })
        .collect()
}

/// 役割ごとの、上書きするトークン。**テーマへ行を足せば増える。**
#[must_use]
pub fn roles() -> Map<String, Value> {
    let mut out: Vec<(String, Map<String, Value>)> = vec![(PLAIN.to_owned(), Map::new())];
    for (k, v) in theme::default_theme() {
        let Some(rest) = k.strip_prefix(ROLE_PREFIX) else {
            continue;
        };
        let (name, token) = rest.split_once('.').unwrap_or((rest, ""));
        match out.iter_mut().find(|(n, _)| n == name) {
            Some(slot) => {
                slot.1.insert(token.to_owned(), v.clone());
            }
            None => {
                let mut m = Map::new();
                m.insert(token.to_owned(), v.clone());
                out.push((name.to_owned(), m));
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out.into_iter()
        .map(|(k, m)| (k, Value::Object(m)))
        .collect()
}

/// 図の宣言そのものが受け取るキー。
#[must_use]
pub fn declaration() -> Map<String, Value> {
    DECLARATION
        .iter()
        .map(|(k, list)| ((*k).to_owned(), Value::Object(props_json(list))))
        .collect()
}

/// 選べる配置戦略。
#[must_use]
pub fn strategies() -> Map<String, Value> {
    STRATEGIES
        .iter()
        .map(|(k, v)| ((*k).to_owned(), Value::from(*v)))
        .collect()
}

/// 目録の全体。**これ1つで、利用側は変換器を書ける。**
///
/// # Errors
///
/// 見本を描けないときに返す。
pub fn catalog() -> Result<Value, String> {
    Ok(json!({
        "declaration": declaration(),
        "parts": parts()?,
        "tokens": tokens(),
        "roles": roles(),
        "strategies": strategies(),
    }))
}
