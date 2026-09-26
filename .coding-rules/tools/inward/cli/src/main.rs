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

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use inward_core::{judge, layer_for, Layer, Order};
use inward_extract::manifest::{Kind, Manifest};
use inward_extract::syntax::Tree;
use inward_extract::{cpp::Cpp, jvm::Jvm, scripted::Scripted, Extracted, Extractor};

/// 探査の脚本の置き場所。**実行ファイルの位置から辿らない** ── build の置き場所に
/// 依存する。引数で受け取り、渡されなければこの道具のフォルダを使う。
fn probes(given: Option<&str>) -> PathBuf {
    given.map_or_else(
        || PathBuf::from(".coding-rules/tools/inward/extract/probes"),
        PathBuf::from,
    )
}

/// 辺の取り方。**既定は原文の構文木である** ── 外の道具を1つも要しない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Via {
    /// 原文の構文木から取る（tree-sitter）。
    Syntax,
    /// その言語の道具に出させる。**経路の別名と探索路まで解決する。**
    Tool,
}

/// 道具に出させる言語のうち、脚本が要るもの。**その言語の実行時からしか解析器へ
/// 触れない場合だけである。**
const SCRIPTED: [(&str, &str, &str, &[&str]); 4] = [
    ("python", "python3", "python.py", &["py"]),
    ("typescript", "node", "typescript.js", &[]),
    ("ruby", "ruby", "ruby.rb", &[]),
    ("php", "php", "php.php", &[]),
];

/// この道具が扱える言語を並べる。
fn languages() -> Vec<&'static str> {
    Tree::languages()
}

fn by_tool(language: &str, probes: &Path) -> Option<Box<dyn Extractor>> {
    if let Some((name, program, script, extensions)) =
        SCRIPTED.iter().find(|(name, _, _, _)| *name == language)
    {
        return Some(Box::new(Scripted::new(name, program, probes.join(script), extensions)));
    }
    match language {
        "java" => Some(Box::new(Jvm::new("java"))),
        "kotlin" => Some(Box::new(Jvm::new("kotlin"))),
        "cpp" => Some(Box::new(Cpp::new())),
        "rust" => Some(Box::new(Manifest::new(Kind::Rust))),
        "go" => Some(Box::new(Manifest::new(Kind::Go))),
        "csharp" => Some(Box::new(Manifest::new(Kind::CSharp))),
        _ => None,
    }
}

fn extractor(language: &str, probes: &Path, via: Via) -> Option<Box<dyn Extractor>> {
    match via {
        Via::Syntax => Tree::of(language).map(|t| -> Box<dyn Extractor> { Box::new(t) }),
        Via::Tool => by_tool(language, probes),
    }
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
    let mut probes_dir = None;
    let mut via = Via::Syntax;
    let mut positional: Vec<String> = Vec::new();
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--json" => {}
            "--probes" => {
                i += 1;
                probes_dir = argv.get(i).cloned();
            }
            "--via" => {
                i += 1;
                match argv.get(i).map(String::as_str) {
                    Some("syntax") => via = Via::Syntax,
                    Some("tool") => via = Via::Tool,
                    other => {
                        eprintln!(
                            "その取り方は無い ── {}（syntax ／ tool）",
                            other.unwrap_or("（無し）")
                        );
                        return ExitCode::from(2);
                    }
                }
            }
            other => positional.push(other.to_owned()),
        }
        i += 1;
    }
    if positional.len() < 3 {
        eprintln!(
            "使い方: inward <言語> <根> <層の名前=識別子[:印]>… \
             [--via syntax|tool] [--probes <場所>] [--json]\n\
             識別子は | で複数を並べられる。印は independent ／ closed ／ composes の3つである\n\
             --via syntax（既定）は原文の構文木で測り、外の道具を要しない。\
             tool はその言語の道具に出させる"
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
    let dir = probes(probes_dir.as_deref());
    let Some(extractor) = extractor(language, &dir, via) else {
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
            "via": if via == Via::Syntax { "syntax" } else { "tool" },
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
