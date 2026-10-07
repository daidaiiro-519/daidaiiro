// SPDX-License-Identifier: MIT
//! 固定した事例 ── **figure と chart の出力を、1バイトも変えずに保つ。**
//!
//! `golden/` の各事例は、入力の宣言（`.json`）と、解決と検査1〜4を通した SVG（`.svg`）の対である。
//! 部品の種類 ・ 配置戦略 ・ 向き ・ 囲み ・ 入れ子 ・ 色のトークンの上書きを1回ずつ覆う。
//!
//! **期待値を書き換えて通さない。** 出力を意図して変えたときだけ、変えた理由と一緒に差し替える
//! ── 直近の差し替えは、class の付与（段A）と値のトークンへの変換（段B）である（design-svg の作り直しの ACDR 1）。

use std::path::PathBuf;

use ds_business_logic::layout_contract::Strategy;
use serde_json::{Map, Value};

/// 事例を描き、解決と検査1〜4を通す。`(SVG, 検出)`
fn render(kind: &str, d: &Value) -> Result<(String, Vec<String>), String> {
    let mut theme: Map<String, Value> = ds_business_logic::theme::default_theme().clone();
    let over = d.get("theme").and_then(Value::as_object);
    for (k, v) in over.into_iter().flatten() {
        theme.insert(k.clone(), v.clone());
    }
    let raw = draw(kind, d, &theme)?;
    // **トークンを上書きした事例は、ダークモードの style を出さない**
    let done = ds_business_logic::publish::generated(&raw, Some(&theme), over.is_none())?;
    Ok((done.svg, done.findings))
}

fn draw(kind: &str, d: &Value, theme: &Map<String, Value>) -> Result<String, String> {
    let list = |k: &str| {
        d.get(k)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    match kind {
        "figure" => {
            let layout: Option<Strategy> =
                match d.get("layout").and_then(Value::as_str).unwrap_or("graph") {
                    "radial" => Some(ds_business_logic::radial::layout_radial),
                    "tree" => Some(ds_business_logic::tree::layout_tree),
                    _ => None,
                };
            let direction = d.get("direction").and_then(Value::as_str).unwrap_or("TB");
            ds_business_logic::compose::render_figure(
                &list("nodes"),
                &list("edges"),
                &list("groups"),
                direction,
                Some(theme),
                layout,
            )
        }
        "chart" => ds_business_logic::compose::render_chart(
            d["kind"].as_str().ok_or("kind")?,
            d["data"].as_object().ok_or("data")?,
            "plain",
            None,
            Some(theme),
        ),
        other => Err(format!("知らない事例の種類: {other}")),
    }
}

fn cases() -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let mut cases: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("事例が在る")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    cases.sort();
    cases
}

fn stem(p: &std::path::Path) -> String {
    p.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

fn input(p: &std::path::Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(p).expect("読める")).expect("JSON")
}

#[test]
fn each_kind_keeps_its_cases() {
    // **種類ごとに下限を置く** ── 合計の下限では、ある種類が全部消えても気付かない
    let count = |kind: &str| cases().iter().filter(|p| stem(p).starts_with(kind)).count();
    assert!(
        count("figure-") >= 9,
        "figure の事例が減っている: {}",
        count("figure-")
    );
    assert!(
        count("chart-") >= 20,
        "chart の事例が減っている: {}",
        count("chart-")
    );
}

#[test]
fn every_case_renders_byte_for_byte() {
    let mut bad = Vec::new();
    for p in &cases() {
        let stem = stem(p);
        let kind = stem.split('-').next().unwrap_or_default();
        let want = std::fs::read_to_string(p.with_extension("svg")).expect("読める");
        match render(kind, &input(p)) {
            Ok((got, _)) if got == want => {}
            Ok((got, _)) => {
                let at = got
                    .chars()
                    .zip(want.chars())
                    .position(|(a, b)| a != b)
                    .unwrap_or(0);
                let near = |s: &str| {
                    s.chars()
                        .skip(at.saturating_sub(40))
                        .take(100)
                        .collect::<String>()
                };
                bad.push(format!(
                    "{stem}\n  期待: {}\n  実際: {}",
                    near(&want),
                    near(&got)
                ));
            }
            Err(e) => bad.push(format!("{stem}: {e}")),
        }
    }
    assert!(
        bad.is_empty(),
        "{} 件が固定した出力と違う:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

#[test]
fn the_cases_themselves_are_sound() {
    // 固定した出力が検査1〜4を通ることも縛る ── **壊れた出力を固定していないことの確認**。
    // 検査1〜3は class が持つ値を外した中間の SVG に、検査4は解決のあとの SVG に適用している
    let mut faulty = Vec::new();
    for p in &cases() {
        let stem = stem(p);
        let kind = stem.split('-').next().unwrap_or_default();
        match render(kind, &input(p)) {
            Ok((_, f)) if f.is_empty() => {}
            Ok((_, f)) => faulty.push(format!("{stem}: {f:?}")),
            Err(e) => faulty.push(format!("{stem}: {e}")),
        }
    }
    assert!(
        faulty.is_empty(),
        "検査1〜4で検出が在る事例:\n{}",
        faulty.join("\n")
    );
}
