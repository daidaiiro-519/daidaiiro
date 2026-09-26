// SPDX-License-Identifier: MIT
//! no-more-spaghetti の道具の宣言。**能力の正本はここである。**
//!
//! 入口（CLI ・ MCP）はこの宣言から組む ── 能力を2回書くと、片方だけが古くなる。
//! 許可辺は `Cargo.toml` が宣言する ── この crate は部品だけを参照する。

pub mod contract;

use std::path::{Path, PathBuf};

use nms_parts::inward::judge::{judge, Layer as Edge_, Order};
use nms_parts::inward::syntax::Tree;
use nms_parts::inward::Extractor as _;
use nms_parts::label::{authority_label, kind_label, Verdict};
use nms_parts::{init, run, validate};
use serde_json::json;

pub use contract::{Arg, Given, Outcome, Tool};

/// 規則ファイルは、リポジトリに1つである ── **管理する対象を1つにする。**
pub const RULES_PATH: &str = ".coding-rules/rules.json";

/// この Skill の場所。**実行ファイルの位置から辿らない** ── build の置き場所に依存する。
fn skill_root(given: &Given) -> PathBuf {
    PathBuf::from(given.one("skill_root", "."))
}

fn contracts(given: &Given) -> validate::Contracts {
    let references = skill_root(given).join("references");
    validate::Contracts::new(
        references.join("rules.schema.json"),
        references.join("concepts.schema.json"),
        references.join("schema-meta.schema.json"),
    )
}

fn rules_path(given: &Given) -> PathBuf {
    let root = PathBuf::from(given.one("root", "."));
    let named = given.one("rules", "");
    if named.is_empty() {
        return root.join(RULES_PATH);
    }
    let direct = PathBuf::from(named);
    if direct.is_file() {
        direct
    } else {
        root.join(named)
    }
}

fn run_check(given: &Given) -> Outcome {
    let root = PathBuf::from(given.one("root", "."));
    let timeout: u64 = given
        .one("timeout", "")
        .parse()
        .unwrap_or(run::DEFAULT_TIMEOUT);
    let file = rules_path(given);
    match run::check(&root, &file, timeout) {
        Ok(report) => {
            let rules: Vec<_> = report
                .rules
                .iter()
                .map(|x| {
                    json!({
                        "name": x.name,
                        "verdict": x.verdict.key(),
                        "reason": x.reason,
                        "exit": x.code,
                        "output": x.output,
                        "output_size": x.output_size,
                        "saved": x.saved,
                    })
                })
                .collect();
            Outcome::found(
                report.findings.clone(),
                json!({
                    "rules": rules,
                    "pass": report.count(Verdict::Pass),
                    "fail": report.count(Verdict::Fail),
                    "skip": report.count(Verdict::Skip),
                    "run": report.run,
                    "rules_file": file.display().to_string(),
                    "root": root.display().to_string(),
                }),
            )
        }
        Err(e) => Outcome::misuse(format!("規則ファイルを読めない ── {e}")),
    }
}

fn human_check(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    let empty = vec![];
    let rules = out
        .data
        .get("rules")
        .and_then(|x| x.as_array())
        .unwrap_or(&empty);
    let mark = |key: &str| match key {
        "pass" => "合格　",
        "fail" => "不合格",
        _ => "実行せず",
    };
    let mut lines: Vec<String> = rules
        .iter()
        .map(|x| {
            let name = x.get("name").and_then(|v| v.as_str()).unwrap_or_default();
            let verdict = x.get("verdict").and_then(|v| v.as_str()).unwrap_or("skip");
            let reason = x.get("reason").and_then(|v| v.as_str()).unwrap_or_default();
            if reason.is_empty() {
                format!("  {}　{name}", mark(verdict))
            } else {
                format!("  {}　{name}　（{reason}）", mark(verdict))
            }
        })
        .collect();
    for x in rules {
        let verdict = x
            .get("verdict")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let output = x.get("output").and_then(|v| v.as_str()).unwrap_or_default();
        if verdict == "fail" && !output.is_empty() {
            let name = x.get("name").and_then(|v| v.as_str()).unwrap_or_default();
            lines.push(String::new());
            lines.push(format!("── {name} の出力（道具のまま）"));
            lines.push(output.to_owned());
        }
    }
    if rules.is_empty() {
        lines.push("  規則が0件である ── 1件も検査していない".to_owned());
    }
    let n = |key: &str| {
        out.data
            .get(key)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
    };
    lines.push(format!(
        "\n合格 {} ／ 不合格 {} ／ 実行せず {}",
        n("pass"),
        n("fail"),
        n("skip")
    ));
    lines.join("\n")
}

fn run_plan(given: &Given) -> Outcome {
    let root = PathBuf::from(given.one("root", "."));
    let file = rules_path(given);
    match nms_parts::rules::load(&file) {
        Ok(rules) => {
            let plan: Vec<_> = rules
                .iter()
                .map(|r| {
                    json!({
                        "name": r.name,
                        "tool": r.tool,
                        "target": r.target,
                        "authority": r.authority,
                    })
                })
                .collect();
            // **場所は解決して出す** ── 相対のままだと、呼んだ場所によって別を指す
            let where_ = root.canonicalize().unwrap_or(root);
            Outcome::found(
                Vec::new(),
                json!({
                    "root": where_.display().to_string(),
                    "rules_file": file.display().to_string(),
                    "plan": plan,
                    "count": rules.len(),
                }),
            )
        }
        Err(e) => Outcome::misuse(format!("規則ファイルを読めない ── {e}")),
    }
}

fn human_plan(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    let root = out
        .data
        .get("root")
        .and_then(|x| x.as_str())
        .unwrap_or_default();
    let empty = vec![];
    let plan = out
        .data
        .get("plan")
        .and_then(|x| x.as_array())
        .unwrap_or(&empty);
    let mut lines = vec![format!("  場所     {root}")];
    for x in plan {
        let name = x.get("name").and_then(|v| v.as_str()).unwrap_or_default();
        let target = x.get("target").and_then(|v| v.as_str()).unwrap_or_default();
        let place = if target.is_empty() {
            "（成果物の場所）".to_owned()
        } else {
            format!("（{target}）")
        };
        // **原典の立場を、実行の前に見せる** ── 推奨と第三者は、外してよいかの判断が要る
        let mark = x
            .get("authority")
            .and_then(|v| v.as_str())
            .and_then(authority_label)
            .map_or_else(String::new, |label| format!("［{label}］"));
        let tool: Vec<&str> = x
            .get("tool")
            .and_then(|v| v.as_array())
            .map(|list| list.iter().filter_map(serde_json::Value::as_str).collect())
            .unwrap_or_default();
        lines.push(format!("  {name}{mark}\n      {}　{place}", tool.join(" ")));
    }
    lines.push(format!("\n{} 件を実行する。実行はしていない。", plan.len()));
    lines.join("\n")
}

fn run_validate(given: &Given) -> Outcome {
    let file = rules_path(given);
    // **読めないものを渡すのは誤用である。** 検出（1）と同じ番号で返すと、
    // 呼ぶ側は「違反が在った」と解釈する
    if let Some(why) = validate::unreadable(&file) {
        return Outcome::misuse(why);
    }
    if let Some(why) = validate::unresolved_schema(&file) {
        return Outcome::misuse(why);
    }
    let contracts = contracts(given);
    let kind = validate::kind_of(&file).unwrap_or(validate::Kind::Rules);
    let got = match kind {
        validate::Kind::Schema => validate::check_schema(&file, &contracts),
        validate::Kind::Concepts => validate::check_concepts(&file, &contracts),
        _ => validate::check_rules(&file, &contracts).and_then(|mut found| {
            if given.has("root") {
                found.extend(validate::check_layers(&file)?);
            }
            Ok(found)
        }),
    };
    match got {
        Ok(findings) => Outcome::found(
            findings,
            json!({ "file": file.display().to_string(), "kind": kind.key() }),
        ),
        Err(e) => Outcome::misuse(format!("検査できない ── {e}")),
    }
}

fn human_validate(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    let kind = out
        .data
        .get("kind")
        .and_then(|x| x.as_str())
        .unwrap_or("rules");
    let name = format!("{}ファイル", kind_label(kind));
    if out.findings.is_empty() {
        format!("{name}の検査　通った")
    } else {
        format!("{name}の検査　通っていない（{} 件）", out.findings.len())
    }
}

fn run_init(given: &Given) -> Outcome {
    let root = PathBuf::from(given.one("root", "."));
    let target = given.one("target", "").to_owned();
    let layers = match init::parse_layers(given.all("layer")) {
        Ok(layers) => layers,
        Err(why) => return Outcome::misuse(why),
    };
    let contract = skill_root(given).join("references/rules.schema.json");
    match init::create(&root, &target, &layers, &contract) {
        Ok(path) => Outcome::found(
            vec![
                "道具が空である ── 検証方法を書く（x-prompt.write が案内する）".to_owned(),
                "出典が空である ── 内を指す規則なので、モデルの識別子を record に書く".to_owned(),
            ],
            json!({
                "path": path.display().to_string(),
                "layers": layers.iter().map(|l| json!({ "name": l.name, "id": l.id })).collect::<Vec<_>>(),
            }),
        ),
        Err(e) => Outcome::misuse(e.to_string()),
    }
}

fn human_init(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    let path = out
        .data
        .get("path")
        .and_then(|x| x.as_str())
        .unwrap_or_default();
    let empty = vec![];
    let layers = out
        .data
        .get("layers")
        .and_then(|x| x.as_array())
        .unwrap_or(&empty);
    let listed: Vec<String> = layers
        .iter()
        .map(|l| {
            format!(
                "{}={}",
                l.get("name").and_then(|v| v.as_str()).unwrap_or_default(),
                l.get("id").and_then(|v| v.as_str()).unwrap_or_default()
            )
        })
        .collect();
    format!(
        "置いた: {path}\n  層 {}\n\n次に書くもの\n  {}",
        listed.join(" ・ "),
        out.findings.join("\n  ")
    )
}

fn run_output(given: &Given) -> Outcome {
    let root = PathBuf::from(given.one("root", "."));
    let offset: u64 = given.one("offset", "0").parse().unwrap_or(0);
    let length: usize = given
        .one("length", "")
        .parse()
        .unwrap_or(run::OUTPUT_HEAD + run::OUTPUT_TAIL);
    match run::read_output(
        &root,
        given.one("run", ""),
        given.one("name", ""),
        offset,
        length,
    ) {
        Ok(slice) => Outcome::found(
            Vec::new(),
            json!({
                "output": slice.output,
                "offset": slice.offset,
                "output_size": slice.output_size,
                "next_offset": slice.next_offset,
            }),
        ),
        Err(e) => Outcome::misuse(e.to_string()),
    }
}

fn human_output(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    let body = out
        .data
        .get("output")
        .and_then(|x| x.as_str())
        .unwrap_or_default();
    let offset = out
        .data
        .get("offset")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let tail = out
        .data
        .get("next_offset")
        .and_then(serde_json::Value::as_u64)
        .map_or_else(
            || "ここで終わりである".to_owned(),
            |n| format!("続きは offset={n}"),
        );
    format!(
        "{body}\n\n── {offset} から {} 文字。{tail}",
        body.chars().count()
    )
}

/// 層1つを読む。**印は識別子のあとに `:` で足す** ── `independent` ／ `closed` ／
/// `composes` の3つで、それ以外は誤用である。**識別子は `|` で複数を並べられる。**
fn parse_layer(raw: &str) -> Result<Edge_, String> {
    let Some((name, tail)) = raw.split_once('=') else {
        return Err(format!("層の形が違う ── {raw}（名前=識別子 で渡す）"));
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
        return Err(format!("層の形が違う ── {raw}（名前と識別子の両方が要る）"));
    }
    let mut layer = Edge_::of(name.to_owned(), ids);
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

fn run_inward(given: &Given) -> Outcome {
    let language = given.one("language", "");
    let root = PathBuf::from(given.one("root", "."));
    let listed = given.all("layer");
    if listed.is_empty() {
        return Outcome::misuse("層を1つも渡していない".to_owned());
    }
    let mut layers = Vec::new();
    for raw in listed {
        match parse_layer(raw) {
            Ok(layer) => layers.push(layer),
            Err(why) => return Outcome::misuse(why),
        }
    }
    let Some(tree) = Tree::of(language) else {
        return Outcome::misuse(format!(
            "その言語の抽出器が無い ── {language}（在るのは {}）",
            Tree::languages().join(" ・ ")
        ));
    };
    if !root.is_dir() {
        return Outcome::misuse(format!("根が無い ── {}", root.display()));
    }
    let got = match tree.extract(&root) {
        Ok(got) => got,
        Err(e) => return Outcome::misuse(format!("抽出できない ── {e}")),
    };
    let order = Order::inner_to_outer(layers);
    let bad = judge(&order, &got.edges);
    let mut findings: Vec<String> = bad
        .iter()
        .map(|v| format!("{} ── {} → {}（{}）", v.at, v.from, v.to, v.because.label()))
        .collect();
    // **合成する層の抜け道は、食い違いにしない** ── 向きの規則が唯一成立しない場所を、
    // 最も外側の1か所へ集約してある
    findings.extend(
        got.escapes
            .iter()
            .filter(|e| {
                !nms_parts::inward::judge::layer_for(&order, &e.in_point)
                    .is_some_and(|l| l.composes)
            })
            .map(|e| format!("{} ── {}", e.at, e.how)),
    );
    findings.extend(
        got.undecided
            .iter()
            .map(|u| format!("判定できていない ── {u}")),
    );
    Outcome::found(
        findings,
        json!({
            "language": tree.language(),
            "root": root.display().to_string(),
            "edges": got.edges.len(),
            "limits": got.limits,
        }),
    )
}

fn human_inward(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join(" ／ ");
    }
    let edges = out
        .data
        .get("edges")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let mut lines = if out.findings.is_empty() {
        vec![format!("向きは内向きである ── 辺 {edges} 件を見た")]
    } else {
        let mut v = vec![format!(
            "食い違い　{} 件 ／ 見た辺 {edges} 件",
            out.findings.len()
        )];
        v.extend(out.findings.iter().map(|x| format!("  ・{x}")));
        v
    };
    // **測り方の限界は毎回出す。** 黙らせると、取りこぼしの範囲が読めない
    if let Some(limits) = out.data.get("limits").and_then(|x| x.as_array()) {
        lines.extend(
            limits
                .iter()
                .filter_map(|x| x.as_str())
                .map(|x| format!("  （測り方）{x}")),
        );
    }
    lines.join("\n")
}

/// この Skill が持つ道具の一覧。**能力の正本である。**
#[must_use]
pub fn tools() -> Vec<Tool> {
    let skill_root = Arg::opt("skill_root", "この Skill の場所（契約を読む先）", Some("."));
    vec![
        Tool {
            name: "check",
            summary: "規則を全件実行し、終了コードで判定する",
            args: vec![
                Arg::need("root", "成果物の場所"),
                Arg::opt(
                    "rules",
                    "規則ファイルのパス（既定 .coding-rules/rules.json）",
                    None,
                ),
                Arg::opt("timeout", "1件あたりの制限時間（秒）", None),
            ],
            run: run_check,
            human: human_check,
        },
        Tool {
            name: "init",
            summary: "リポジトリの .coding-rules/ へ、成果物の規則ファイルを置く",
            args: vec![
                Arg::need("root", "リポジトリの場所"),
                Arg::need("target", "成果物の場所（リポジトリ自身なら .）"),
                Arg::many("layer", "層を 名前=識別子 で渡す（複数可）"),
                skill_root.clone(),
            ],
            run: run_init,
            human: human_init,
        },
        Tool {
            name: "plan",
            summary: "何を、どこで実行するかを返す（実行はしない）",
            args: vec![
                Arg::need("root", "成果物の場所"),
                Arg::opt("rules", "規則ファイルのパス", None),
            ],
            run: run_plan,
            human: human_plan,
        },
        Tool {
            name: "output",
            summary: "保持した出力の続きを読む（実行はしない）",
            args: vec![
                Arg::need("root", "成果物の場所"),
                Arg::need("run", "実行の識別子"),
                Arg::need("name", "規則の名前"),
                Arg::opt("offset", "読み始める位置（バイト）", Some("0")),
                Arg::opt("length", "読む量（バイト）", None),
            ],
            run: run_output,
            human: human_output,
        },
        Tool {
            name: "inward",
            summary: "依存の向きが内向きかを、原文の構文木から検査する",
            args: vec![
                Arg::need("language", "言語の名前"),
                Arg::need("root", "見る場所"),
                Arg::many("layer", "層を 名前=識別子[:印] で渡す（複数可。印は independent ／ closed ／ composes）"),
            ],
            run: run_inward,
            human: human_inward,
        },
        Tool {
            name: "validate",
            summary: "規則ファイルの形を検査する",
            args: vec![
                Arg::opt(
                    "rules",
                    "規則ファイルのパス（既定 .coding-rules/rules.json）",
                    None,
                ),
                Arg::opt("root", "成果物の場所（層の宣言も見る）", None),
                skill_root,
            ],
            run: run_validate,
            human: human_validate,
        },
    ]
}

/// 契約の置き場所を、呼ぶ側へ見せる ── 入口が `--skill_root` の既定を決めるために使う。
#[must_use]
pub fn default_skill_root() -> &'static Path {
    Path::new(".")
}
