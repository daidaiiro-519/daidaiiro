// SPDX-License-Identifier: MIT
//! slide-deck のサービス層。道具の一覧を持ち、**能力の正本はここである。**
//!
//! プレゼンテーション層（CLI ・ MCP）はこの一覧から組む ── 能力を2回書くと、片方だけが古くなる。
//! 依存の向きは `Cargo.toml` が宣言する ── この crate は業務ロジック層だけを参照する。
//!
//! **この Skill は図を描かない。** 渡すのは配色だけである ── `theme` が、テーマの
//! キーを**図の中の役割の名前**へ複製して出す。誰に組ませるかは配線表が決める。

pub mod contract;
mod refs;

use std::path::{Path, PathBuf};

use sd_business_logic::{deck as build, review, theme as colors};
use serde_json::{json, Map, Value};

pub use contract::{catalog, Arg, Given, Outcome, Tool};

/// 既定のテーマ。
const DEFAULT_THEME: &str = "warm-paper";
/// 題を書いていないことが、出来上がりから分かる文字列。
const DEFAULT_TITLE: &str = "題を記入する";

/// この Skill の置き場所。**呼ぶ側が決める** ── どこから呼ばれるかを、この側で
/// 推測しない。
/// Skill の置き場所が見つからなければ、誤用として返す。**黙って「.」へ寄せない** ──
/// 実行した場所で結果が変わり、契約を読めずに止まる（ACDR 0019 ・ 0029）。
macro_rules! or_misuse {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(why) => return Outcome::misuse(why),
        }
    };
}

fn skill_root(given: &Given) -> Result<PathBuf, String> {
    given.skill_root()
}

fn references(given: &Given) -> Result<PathBuf, String> {
    Ok(skill_root(given)?.join("references"))
}

/// 平らな対応表を、1字下げの JSON へ組む。
fn flat_json(value: &Value) -> String {
    serde_json::to_string_pretty(value)
        .unwrap_or_default()
        .replace("  ", " ")
}

/// 検出を、人が読む行へ組む。
fn crosses(findings: &[String]) -> String {
    findings
        .iter()
        .map(|x| format!("  × {x}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn run_check(given: &Given) -> Outcome {
    let refs = or_misuse!(references(given));
    let decks = given.all("deck").to_vec();
    let bad = colors::findings(&refs, &decks);
    Outcome::found(
        bad,
        json!({
            "themes": colors::theme_files(&refs).len(),
            "decks": decks.len(),
        }),
    )
}

fn human_check(out: &Outcome) -> String {
    let count = |key: &str| {
        out.data
            .get(key)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
    };
    let verdict = if out.findings.is_empty() {
        "通った".to_owned()
    } else {
        format!("通っていない（{} 件）", out.findings.len())
    };
    let tail = format!(
        "テーマの検査　{verdict}　／　テーマ {} 本、デッキ {} 本",
        count("themes"),
        count("decks")
    );
    if out.findings.is_empty() {
        tail
    } else {
        format!("{}\n{tail}", crosses(&out.findings))
    }
}

fn run_theme(given: &Given) -> Outcome {
    let refs = or_misuse!(references(given));
    let names = colors::theme_names(&refs);
    let name = given.one("name", "");
    if name.is_empty() {
        return Outcome::found(Vec::new(), json!({ "names": names, "tokens": {} }));
    }
    let roles = match colors::as_roles(&refs, name) {
        Ok(roles) => roles,
        Err(why) => {
            return Outcome::found(vec![why], json!({ "names": names, "tokens": {} }));
        }
    };
    let mut tokens = Map::new();
    for (role, value) in roles {
        tokens.insert(role, Value::String(value));
    }
    let tokens = Value::Object(tokens);
    let out = given.one("out", "");
    if !out.is_empty() {
        if let Err(why) = colors::save(Path::new(out), &(flat_json(&tokens) + "\n")) {
            return Outcome::misuse(why);
        }
    }
    Outcome::found(
        Vec::new(),
        json!({ "name": name, "tokens": tokens, "path": out, "names": names }),
    )
}

fn human_theme(out: &Outcome) -> String {
    if !out.findings.is_empty() {
        return crosses(&out.findings);
    }
    let tokens = out.data.get("tokens").unwrap_or(&Value::Null);
    let count = tokens.as_object().map_or(0, Map::len);
    if count == 0 {
        let names: Vec<&str> = out
            .data
            .get("names")
            .and_then(|x| x.as_array())
            .map(|x| x.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        return format!("テーマ: {}", names.join(" ・ "));
    }
    let path = out
        .data
        .get("path")
        .and_then(|x| x.as_str())
        .unwrap_or_default();
    if path.is_empty() {
        flat_json(tokens)
    } else {
        format!("書き出し: {path}　／　{count} 件")
    }
}

fn run_new(given: &Given) -> Outcome {
    let out = given.one("out", "");
    if out.is_empty() {
        return Outcome::misuse("書き出し先を渡していない".to_owned());
    }
    let path = Path::new(out);
    if build::exists(path) {
        return Outcome::found(
            vec![format!("既に在る: {out} ── 作り直さない")],
            json!({ "out": out }),
        );
    }
    let example = or_misuse!(references(given)).join("deck-example.json");
    let mut deck = match build::load(&example) {
        Ok(value) => value,
        Err(why) => return Outcome::misuse(why),
    };
    let theme = given.one("theme", DEFAULT_THEME);
    let title = given.one("title", DEFAULT_TITLE);
    if let Some(map) = deck.as_object_mut() {
        // **`$schema` は references の中でスキーマを指す印である** ── 起こしたデッキへ写さない
        map.shift_remove("$schema");
        map.insert("title".to_owned(), Value::String(title.to_owned()));
        map.insert("theme".to_owned(), Value::String(theme.to_owned()));
    }
    if let Err(why) = build::save(path, &(flat_json(&deck) + "\n")) {
        return Outcome::misuse(why);
    }
    Outcome::found(Vec::new(), json!({ "out": out, "theme": theme }))
}

fn human_new(out: &Outcome) -> String {
    if !out.findings.is_empty() {
        return crosses(&out.findings);
    }
    let get = |key: &str| {
        out.data
            .get(key)
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_owned()
    };
    let (place, theme) = (get("out"), get("theme"));
    [
        format!("作った: {place}　／　テーマ {theme}"),
        "次にすること ── 枚を書き、図は組ませて、返った SVG を figure へ置く。".to_owned(),
        format!("組む: `slide-deck render {place} <出力.html>`"),
        format!("配色は `slide-deck theme {theme}` が出す役割ごとの色を、描く側へ渡す。"),
    ]
    .join("\n")
}

/// 組めなかった理由を、行ごとの検出へ割る。**先頭の1行は見出しである。**
fn split_why(why: &str) -> Vec<String> {
    let mut lines = why.lines();
    let head = lines.next().unwrap_or_default();
    if head.ends_with(':') {
        // **字下げを落とさない** ── 落とすと、印字する側が付け直すことになり、
        // 2つの入口で見え方が相違する
        return lines.map(str::to_owned).collect();
    }
    vec![why.to_owned()]
}

fn run_render(given: &Given) -> Outcome {
    let deck = given.one("deck", "");
    if deck.is_empty() {
        return Outcome::misuse("デッキの入力を渡していない".to_owned());
    }
    let source = PathBuf::from(deck);
    let asked = given.one("out", "");
    let dest = if asked.is_empty() {
        source.with_extension("html")
    } else {
        PathBuf::from(asked)
    };
    let check_only = !given.one("check", "").is_empty();
    match build::build_deck(&or_misuse!(references(given)), &source, &dest, check_only) {
        Ok(built) => Outcome::found(
            if built.same {
                Vec::new()
            } else {
                vec![format!("{}: {}", dest.display(), built.note)]
            },
            json!({
                "deck": deck, "out": dest.display().to_string(), "note": built.note,
            }),
        ),
        Err(why) => Outcome::found(split_why(&why), json!({ "deck": deck })),
    }
}

fn human_render(out: &Outcome) -> String {
    if out.findings.is_empty() {
        let get = |key: &str| {
            out.data
                .get(key)
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_owned()
        };
        return format!("組んだ: {}　／　{}", get("out"), get("note"));
    }
    let mut lines: Vec<String> = out
        .findings
        .iter()
        .map(|x| {
            if x.trim_start().starts_with('×') {
                x.clone()
            } else {
                format!("  × {x}")
            }
        })
        .collect();
    lines.push(format!("組めていない（{} 件）", out.findings.len()));
    lines.join("\n")
}

fn run_review(given: &Given) -> Outcome {
    let input = given.one("input", "");
    let out = given.one("out", "");
    if input.is_empty() || out.is_empty() {
        return Outcome::misuse("照合の入力と、比較ページの書き出し先を渡していない".to_owned());
    }
    match review::build_compare(
        &or_misuse!(references(given)),
        Path::new(input),
        Path::new(out),
    ) {
        Ok(n) => Outcome::found(Vec::new(), json!({ "out": out, "slides": n })),
        Err(why) => Outcome::found(split_why(&why), json!({ "input": input })),
    }
}

fn human_review(out: &Outcome) -> String {
    if !out.findings.is_empty() {
        return format!(
            "{}\n組めていない（{} 件）",
            crosses(&out.findings),
            out.findings.len()
        );
    }
    format!(
        "組んだ: {}　／　{} 枚",
        out.data
            .get("out")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        out.data.get("slides").and_then(Value::as_u64).unwrap_or(0)
    )
}

fn run_export(given: &Given) -> Outcome {
    let input = given.one("input", "");
    let out = given.one("out", "");
    if input.is_empty() || out.is_empty() {
        return Outcome::misuse("照合の入力と、書き出し先のフォルダを渡していない".to_owned());
    }
    let dir = Path::new(out);
    let htmls = match review::build_exports(&or_misuse!(references(given)), Path::new(input), dir) {
        Ok(list) => list,
        Err(why) => return Outcome::found(split_why(&why), json!({ "input": input })),
    };
    let browser = given.one("browser", "");
    let mut findings = Vec::new();
    let mut pdfs = Vec::new();
    if browser.is_empty() {
        // **PDF を出さなかったことを、黙らない** ── 出したと読めてしまう
        findings.push(
            "PDF は出していない ── browser にブラウザの場所を渡すと、HTML を描画して PDF を出す"
                .to_owned(),
        );
    } else {
        for html in &htmls {
            let pdf = html.with_extension("pdf");
            match review::print_pdf(Path::new(browser), html, &pdf) {
                Ok(()) => pdfs.push(pdf.display().to_string()),
                Err(why) => findings.push(why),
            }
        }
    }
    let htmls: Vec<String> = htmls.iter().map(|p| p.display().to_string()).collect();
    Outcome::found(findings, json!({ "html": htmls, "pdf": pdfs }))
}

fn human_export(out: &Outcome) -> String {
    let list = |key: &str| -> Vec<String> {
        out.data
            .get(key)
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut lines: Vec<String> = list("html")
        .into_iter()
        .chain(list("pdf"))
        .map(|p| format!("書き出し: {p}"))
        .collect();
    if !out.findings.is_empty() {
        lines.push(crosses(&out.findings));
    }
    lines.join("\n")
}

/// この Skill が持つ道具の一覧。**能力の正本である。**
#[must_use]
pub fn tools() -> Vec<Tool> {
    let root = Arg::opt(
        "skill_root",
        "この Skill の置き場所（既定は、実行ファイルの1つ上）",
        None,
    );
    let mut all = vec![
        Tool {
            name: "new",
            summary: "デッキの入力（JSON）を起こす",
            args: vec![
                Arg::need("out", "書き出し先の JSON"),
                Arg::opt("theme", "テーマの名前", Some(DEFAULT_THEME)),
                Arg::opt("title", "題", Some(DEFAULT_TITLE)),
                root.clone(),
            ],
            run: run_new,
            human: human_new,
        },
        Tool {
            name: "render",
            summary: "入力（JSON）から1枚の HTML を組む",
            args: vec![
                Arg::need("deck", "デッキの入力（JSON）"),
                Arg::opt("out", "書き出し先の HTML。省くと入力と同じ名前", None),
                Arg::opt("check", "組み直さず、差が無いかだけを検査する", None),
                root.clone(),
            ],
            run: run_render,
            human: human_render,
        },
        Tool {
            name: "review",
            summary: "照合の入力から、変更前と変更後を並べた比較ページを組む",
            args: vec![
                Arg::need("input", "照合の入力（review.schema.json の形の JSON）"),
                Arg::need("out", "比較ページの書き出し先。画像は隣の img/ へ写す"),
                root.clone(),
            ],
            run: run_review,
            human: human_review,
        },
        Tool {
            name: "export",
            summary: "照合の入力から、台本 ・ 確認記録 ・ 観点ごとの結果を HTML と PDF で出す",
            args: vec![
                Arg::need("input", "照合の入力（review.schema.json の形の JSON）"),
                Arg::need("out", "書き出し先のフォルダ"),
                Arg::opt(
                    "browser",
                    "PDF を出すときのブラウザの場所。省くと HTML だけを出す",
                    None,
                ),
                root.clone(),
            ],
            run: run_export,
            human: human_export,
        },
        Tool {
            name: "check",
            summary: "テーマの形を検査する",
            args: vec![Arg::some("deck", "デッキの HTML（複数可）"), root.clone()],
            run: run_check,
            human: human_check,
        },
        Tool {
            name: "theme",
            summary: "テーマを、図の中の役割ごとの色へ複製して出す",
            args: vec![
                Arg::opt("name", "テーマの名前。省くと一覧", None),
                Arg::opt("out", "書き出し先の JSON", None),
                root,
            ],
            run: run_theme,
            human: human_theme,
        },
    ];
    all.extend(refs::tools());
    all
}
