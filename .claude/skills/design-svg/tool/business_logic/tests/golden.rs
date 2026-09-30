// SPDX-License-Identifier: MIT
//! 固定した事例 ── **Python から移したときの出力を、1バイトも変えずに保つ。**
//!
//! `golden/` の各事例は、入力の宣言（`.json`）と、移す前の実装が描いた SVG（`.svg`）の対である。
//! 部品の種類 ・ 配置戦略 ・ 向き ・ 囲み ・ 入れ子 ・ テーマの上書き ・ 背景を1回ずつ覆う。
//!
//! **期待値を書き換えて通さない。** 出力を意図して変えたときだけ、変えた理由と一緒に差し替える。

use std::path::PathBuf;

use ds_business_logic::layout_contract::Strategy;
use serde_json::{Map, Value};

fn render(kind: &str, d: &Value) -> Result<String, String> {
    let mut theme: Map<String, Value> = ds_business_logic::theme::default_theme().clone();
    if let Some(over) = d.get("theme").and_then(Value::as_object) {
        for (k, v) in over {
            theme.insert(k.clone(), v.clone());
        }
    }
    let list = |k: &str| {
        d.get(k)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    match kind {
        "canvas" => ds_business_logic::canvas::render_canvas(
            d["width"].as_f64().ok_or("width")?,
            d["height"].as_f64().ok_or("height")?,
            &list("layers"),
            Some(&theme),
            d.get("background").and_then(Value::as_str),
        ),
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
                Some(&theme),
                layout,
            )
        }
        "chart" => ds_business_logic::compose::render_chart(
            d["kind"].as_str().ok_or("kind")?,
            d["data"].as_object().ok_or("data")?,
            "plain",
            None,
            Some(&theme),
        ),
        other => Err(format!("知らない事例の種類: {other}")),
    }
}

#[test]
fn every_case_renders_byte_for_byte() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let mut cases: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("事例が在る")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    cases.sort();
    assert!(cases.len() >= 50, "事例が減っている: {}", cases.len());
    let mut bad = Vec::new();
    for p in &cases {
        let stem = p
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let kind = stem.split('-').next().unwrap_or_default();
        let d: Value =
            serde_json::from_str(&std::fs::read_to_string(p).expect("読める")).expect("JSON");
        let want = std::fs::read_to_string(p.with_extension("svg")).expect("読める");
        match render(kind, &d) {
            Ok(got) if got == want => {}
            Ok(got) => {
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
        "{} 件が移す前と違う:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

#[test]
fn the_cases_themselves_are_sound() {
    // 固定した出力が幾何の検査を通ることも縛る ── **壊れた出力を固定していないことの確認**
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let mut faulty = Vec::new();
    for e in std::fs::read_dir(&dir).expect("事例が在る").flatten() {
        let p = e.path();
        if p.extension().is_some_and(|x| x == "svg") {
            let f = ds_business_logic::verify::all(&std::fs::read_to_string(&p).expect("読める"));
            if !f.is_empty() {
                faulty.push(format!(
                    "{}: {}",
                    p.file_name().unwrap_or_default().to_string_lossy(),
                    f.len()
                ));
            }
        }
    }
    faulty.sort();
    assert!(faulty.is_empty(), "幾何の検査で検出が在る事例: {faulty:?}");
}
