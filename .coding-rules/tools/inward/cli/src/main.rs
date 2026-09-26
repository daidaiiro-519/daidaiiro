// SPDX-License-Identifier: MIT
//! 依存の向きを検査する唯一の入口。
//!
//!     inward <言語> <根> <層の名前=識別子>…  [--json]
//!
//! **核と抽出を結ぶ唯一の場所である。** 核は言語を認知せず、抽出はその言語の道具を
//! 呼ぶだけで、両方を知るのはここだけである。
//!
//! 終了コードは 0 正常 ／ 1 検出あり ／ 2 誤用。**判定できなかった範囲が在れば、
//! 0件を結論にしない** ── 検出ありとして返す。

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use inward_core::{judge, layer_for, Layer, Order};
use inward_extract::syntax::Tree;
use inward_extract::{Extracted, Extractor};

/// この道具が扱える言語を並べる。
fn languages() -> Vec<&'static str> {
    Tree::languages()
}

/// 言語から抽出器を決める。**取り方は1つである** ── 原文の構文木だけを見る。
fn extractor(language: &str) -> Option<Box<dyn Extractor>> {
    Tree::of(language).map(|t| -> Box<dyn Extractor> { Box::new(t) })
}

fn parse_layer(x: &str) -> Result<Layer, String> {
    let Some((name, tail)) = x.split_once('=') else {
        return Err(format!("層の形が違う ── {x}（名前=識別子 で渡す）"));
    };
    let mut parts = tail.split(':');
    let ids: Vec<String> = parts
        .next()
        .unwrap_or_default()
        .split('|')
        .filter(|x| !x.is_empty())
        .map(str::to_owned)
        .collect();
    if name.is_empty() || ids.is_empty() {
        return Err(format!("層の形が違う ── {x}（名前と識別子の両方が要る）"));
    }
    let mut layer = Layer::of(name.to_owned(), ids);
    for mark in parts {
        layer = match mark {
            "independent" => layer.independent(),
            "closed" => layer.closed(),
            "composes" => layer.composes(),
            other => {
                return Err(format!(
                    "その印は無い ── {other}（independent ／ closed ／ composes）"
                ))
            }
        };
    }
    Ok(layer)
}

fn parse_layers(rest: &[String]) -> Result<Order, String> {
    let mut layers = Vec::new();
    for x in rest {
        layers.push(parse_layer(x)?);
    }
    if layers.is_empty() {
        return Err("層を1つも渡していない".to_owned());
    }
    Ok(Order::inner_to_outer(layers))
}

fn report(order: &Order, got: &Extracted) -> (Vec<String>, usize) {
    let bad = judge(order, &got.edges);
    let mut lines: Vec<String> = bad
        .iter()
        .map(|v| format!("{} ── {} → {}（{}）", v.at, v.from, v.to, v.because.label()))
        .collect();
    // **合成する層の抜け道は、食い違いにしない。** 依存の向きの規則が唯一成立しない
    // 場所を、最も外側の1か所に集約してある ── そこで読み込むのは、その層の仕事である
    lines.extend(
        got.escapes
            .iter()
            .filter(|e| !layer_for(order, &e.in_point).is_some_and(|l| l.composes))
            .map(|e| format!("{} ── {}", e.at, e.how)),
    );
    lines.extend(
        got.undecided
            .iter()
            .map(|u| format!("判定できていない ── {u}")),
    );
    (lines, got.edges.len())
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let as_json = argv.iter().any(|a| a == "--json");
    let mut positional: Vec<String> = Vec::new();
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--json" => {}
            other => positional.push(other.to_owned()),
        }
        i += 1;
    }
    if positional.len() < 3 {
        eprintln!(
            "使い方: inward <言語> <根> <層の名前=識別子[:印]>… [--json]\n\
             識別子は | で複数を並べられる。印は independent ／ closed ／ composes の3つである\n\
             原文の構文木で測る ── 外の道具を1つも要しない"
        );
        return ExitCode::from(2);
    }
    let (language, root) = (&positional[0], PathBuf::from(&positional[1]));
    let order = match parse_layers(&positional[2..]) {
        Ok(order) => order,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    let Some(extractor) = extractor(language) else {
        eprintln!(
            "その言語の抽出器が無い ── {language}（在るのは {}）",
            languages().join(" ・ ")
        );
        return ExitCode::from(2);
    };
    if !root.is_dir() {
        eprintln!("根が無い ── {}", root.display());
        return ExitCode::from(2);
    }
    let got = match extractor.extract(&root) {
        Ok(got) => got,
        Err(e) => {
            eprintln!("抽出できない ── {e}");
            return ExitCode::from(2);
        }
    };
    let (lines, edges) = report(&order, &got);
    if as_json {
        let body = serde_json::json!({
            "language": extractor.language(),
            "root": root.display().to_string(),
            "edges": edges,
            "findings": lines,
            "limits": got.limits,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&body).unwrap_or_default()
        );
    } else {
        if lines.is_empty() {
            println!("向きは内向きである ── 辺 {edges} 件を見た");
        } else {
            println!("食い違い　{} 件 ／ 見た辺 {edges} 件", lines.len());
            for x in &lines {
                println!("  ・{x}");
            }
        }
        // **測り方の限界は毎回出す。** 黙らせると、取りこぼしの範囲が読めない
        for x in &got.limits {
            println!("  （測り方）{x}");
        }
    }
    if lines.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
