// SPDX-License-Identifier: MIT
//! references の道具を、道具の一覧に載せる。**どの Skill も同じ4つを持つ** ── 取り出す ・ 検査する ・ 描画する ・
//! 取り込む（ACDR 0043）。実体は業務ロジック層の `refs` が持つ。

use std::path::{Path, PathBuf};

use serde_json::json;

// **この Skill の層は、外部の crate と別の組に置く** ── 接頭辞で並びが変わらないようにする
use qa_business_logic::refs;

use crate::contract::{Arg, Given, Outcome, Tool};

fn refs_dir(given: &Given) -> Result<PathBuf, String> {
    Ok(given.skill_root()?.join("references"))
}

macro_rules! or_misuse {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(why) => return Outcome::misuse(why),
        }
    };
}

fn opt<'a>(given: &'a Given, name: &str) -> Option<&'a str> {
    Some(given.one(name, "")).filter(|s| !s.is_empty())
}

fn run_get(given: &Given) -> Outcome {
    let dir = or_misuse!(refs_dir(given));
    let got = or_misuse!(refs::get(&dir, given.one("kind", ""), opt(given, "id")));
    // **除く欄を渡すと、入れ子のすべてから除いて返す**（例：omit=source）
    let keys: Vec<&str> = opt(given, "omit")
        .map(|o| {
            o.split(',')
                .map(str::trim)
                .filter(|k| !k.is_empty())
                .collect()
        })
        .unwrap_or_default();
    Outcome::found(Vec::new(), refs::omit(&got, &keys))
}

fn run_validate(given: &Given) -> Outcome {
    let dir = or_misuse!(refs_dir(given));
    let found = match opt(given, "file") {
        Some(file) => or_misuse!(refs::validate_file(
            &dir,
            given.one("kind", ""),
            Path::new(file)
        )),
        None => or_misuse!(refs::validate(&dir)),
    };
    let kinds: Vec<String> = or_misuse!(refs::kinds(&dir))
        .into_iter()
        .map(|k| k.name)
        .collect();
    // **何を検査したかを返す** ── 種類の一覧だけを返すと、合格したのか、検査が実行されなかったのかを
    // 読み手が区別できない（実測 2026-10-01、試しの相談3件とも）
    let checked = opt(given, "file").unwrap_or("references");
    Outcome::found(found, json!({ "kinds": kinds, "checked": checked }))
}

fn human_validate(out: &Outcome) -> String {
    if !out.ok || !out.findings.is_empty() {
        return human(out);
    }
    let checked = out
        .data
        .get("checked")
        .and_then(|x| x.as_str())
        .unwrap_or("references");
    format!("合格 ── {checked} に検出は無い")
}

fn run_view(given: &Given) -> Outcome {
    let dir = or_misuse!(refs_dir(given));
    let html = or_misuse!(refs::view(
        &dir,
        given.one("kind", ""),
        opt(given, "id"),
        opt(given, "file").map(Path::new)
    ));
    match opt(given, "out") {
        Some(out) => match refs::save(Path::new(out), &html) {
            Ok(()) => Outcome::found(Vec::new(), json!({ "out": out, "bytes": html.len() })),
            Err(e) => Outcome::misuse(e),
        },
        None => Outcome::found(Vec::new(), json!({ "html": html })),
    }
}

fn run_import(given: &Given) -> Outcome {
    let dir = or_misuse!(refs_dir(given));
    let file = given.one("file", "");
    let text = or_misuse!(refs::read_text(Path::new(file)));
    let doc = refs::import_markdown(
        given.one("id", ""),
        given.one("source", file),
        given.one("fetched", ""),
        &text,
    );
    let path = or_misuse!(refs::put_document(&dir, doc));
    let found = or_misuse!(refs::validate(&dir));
    Outcome::found(found, json!({ "path": path.display().to_string() }))
}

fn human(out: &Outcome) -> String {
    if !out.ok || !out.findings.is_empty() {
        return out
            .findings
            .iter()
            .map(|x| format!("  ×  {x}"))
            .collect::<Vec<_>>()
            .join("\n");
    }
    if let Some(html) = out.data.get("html").and_then(|x| x.as_str()) {
        return html.to_owned();
    }
    serde_json::to_string_pretty(&out.data).unwrap_or_default()
}

/// references の4つの道具。**サービス層の `tools()` がこれを足す。**
#[must_use]
pub fn tools() -> Vec<Tool> {
    let root = || {
        Arg::opt(
            "skill_root",
            "この Skill の置き場所（既定は、実行ファイルの1つ上）",
            None,
        )
    };
    vec![
        Tool {
            name: "get",
            summary: "references の種類の JSON を取り出す。id を渡すと、その1件だけを返す",
            args: vec![Arg::need("kind", "種類の名前"), Arg::opt("id", "項目の id", None), Arg::opt("omit", "除く欄の名前（, で区切る。例：source）", None), root()],
            run: run_get,
            human,
        },
        Tool {
            name: "validate",
            summary: "references の JSON を、指しているスキーマで検査する。file を渡すと、その JSON を種類のスキーマで検査する",
            args: vec![Arg::opt("kind", "種類の名前（file と一緒に渡す）", None), Arg::opt("file", "検査する JSON", None), root()],
            run: run_validate,
            human: human_validate,
        },
        Tool {
            name: "view",
            summary: "references の種類の JSON を、スキーマの title と x-view に従って HTML に描画する",
            args: vec![Arg::need("kind", "種類の名前"), Arg::opt("id", "項目の id", None), Arg::opt("file", "描画する JSON（回答など）", None), Arg::opt("out", "HTML の置き場所", None), root()],
            run: run_view,
            human,
        },
        Tool {
            name: "import",
            summary: "Markdown の文書を、見出しを節の入れ子に分けて document へ取り込む",
            args: vec![Arg::need("file", "取り込む Markdown"), Arg::need("id", "document の id"), Arg::opt("source", "元の場所（既定は file）", None), Arg::opt("fetched", "取り込んだ日", None), root()],
            run: run_import,
            human,
        },
    ]
}
