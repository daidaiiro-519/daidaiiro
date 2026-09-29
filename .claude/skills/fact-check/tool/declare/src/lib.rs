// SPDX-License-Identifier: MIT
//! fact-check の道具の宣言。**能力の正本はここである。**
//!
//! 入口（CLI ・ MCP）はこの宣言から組む ── 能力を2回書くと、片方だけが古くなる。
//! 許可辺は `Cargo.toml` が宣言する ── この crate は部品だけを参照する。

pub mod contract;

use std::path::{Path, PathBuf};

use fc_parts::find::How;
use fc_parts::source::{self, Scan};
use serde_json::json;

pub use contract::{Arg, Given, Outcome, Tool};

/// 数を3桁ごとに区切る。**読み手が桁を数えずに済む。**
/// この Skill の部品が呼ぶ外部の道具。**OS によって無いコマンド（date ・ timeout など）を
/// 書かない** ── 日付の計算と時間の制限は Rust の中で行う。名前を実行時に決める道具は
/// `"*"`（利用者が指定する道具）と書く。skills-creator の check が、部品の呼び出しと照合する。
/// curl ── 原文を取得する
pub const REQUIRES: &[&str] = &["curl"];

fn grouped(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// 置き場所の既定。**呼ぶ側が決める** ── この Skill は置き場所を持たない。
const DEFAULT_DIR: &str = "sources";

fn outdir(given: &Given) -> PathBuf {
    PathBuf::from(given.one("dir", DEFAULT_DIR))
}

fn run_fetch(given: &Given) -> Outcome {
    let url = given.one("url", "");
    if url.is_empty() {
        return Outcome::misuse("出どころを渡していない".to_owned());
    }
    let dir = outdir(given);
    match source::fetch(url, &dir) {
        Ok(got) => match source::write_meta(&got) {
            Ok(meta) => Outcome::found(
                Vec::new(),
                json!({
                    "path": got.path.display().to_string(),
                    "meta": meta.display().to_string(),
                    "url": got.url, "bytes": got.bytes, "lines": got.lines,
                    "sha256": got.sha256, "content_type": got.content_type,
                }),
            ),
            Err(e) => Outcome::misuse(format!("記録を書けない ── {e}")),
        },
        Err(e) => Outcome::misuse(e.to_string()),
    }
}

fn human_fetch(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join("\n");
    }
    let get = |k: &str| {
        out.data
            .get(k)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_owned()
    };
    let n = |k: &str| {
        out.data
            .get(k)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
    };
    let sha: String = get("sha256").chars().take(16).collect();
    format!(
        "取得した: {}\n  {} バイト ・ {} 行 ・ {}\n  sha256 {sha}…",
        get("path"),
        grouped(n("bytes")),
        grouped(n("lines")),
        get("content_type")
    )
}

fn run_list(given: &Given) -> Outcome {
    let dir = outdir(given);
    if !dir.is_dir() {
        return Outcome::found(
            vec![format!("まだ何も取得していない: {}", dir.display())],
            json!({ "dir": dir.display().to_string(), "rows": [] }),
        );
    }
    let rows: Vec<serde_json::Value> = source::listing(&dir)
        .into_iter()
        .map(|(date, lines, url)| json!({ "date": date, "lines": lines, "url": url }))
        .collect();
    Outcome::found(
        Vec::new(),
        json!({ "dir": dir.display().to_string(), "rows": rows }),
    )
}

fn human_list(out: &Outcome) -> String {
    if !out.findings.is_empty() {
        return out.findings.join("\n");
    }
    let empty = vec![];
    let rows = out
        .data
        .get("rows")
        .and_then(|x| x.as_array())
        .unwrap_or(&empty);
    let mut lines = vec![format!("{:12}{:>8}  出どころ", "取得した日", "行")];
    for r in rows {
        lines.push(format!(
            "{:12}{:>8}  {}",
            r.get("date").and_then(|v| v.as_str()).unwrap_or_default(),
            grouped(
                r.get("lines")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0)
            ),
            r.get("url").and_then(|v| v.as_str()).unwrap_or_default()
        ));
    }
    lines.push(format!("── {} 件", rows.len()));
    lines.join("\n")
}

/// 照合するものを集める。**ファイルから読むときは、空の行を捨てる。**
fn needles(given: &Given) -> Vec<String> {
    let from = given.one("from", "");
    if !from.is_empty() {
        return std::fs::read_to_string(from).map_or_else(
            |_| Vec::new(),
            |body| {
                body.lines()
                    .map(str::trim)
                    .filter(|x| !x.is_empty())
                    .map(str::to_owned)
                    .collect()
            },
        );
    }
    given.all("needle").to_vec()
}

fn run_verify(given: &Given) -> Outcome {
    let path = PathBuf::from(given.one("path", ""));
    if path.as_os_str().is_empty() {
        return Outcome::misuse("原文を渡していない".to_owned());
    }
    let asked = given.one("as", "");
    let Some(how) = How::of(asked) else {
        // **種類を渡さなければ止まる** ── 道具の側で「たぶん識別子だろう」と決めない
        return Outcome::misuse(format!(
            "照合の種類を渡していない（--as {}）。受け取ったもの: {asked:?}\n  \
             identifier ── 語として照合する（前後が語の文字なら別の名前）\n  \
             quote      ── 引用として照合する（空白の圧縮規則だけ統一し、語は変えない）\n  \
             text       ── そのまま探す（探索用。照合した証しにはならない）",
            How::all().map(How::key).join(" / ")
        ));
    };
    let wanted = needles(given);
    if wanted.is_empty() {
        return Outcome::misuse("照合するものを渡していない".to_owned());
    }
    let near = given.one("near", "");
    let within: usize = given.one("within", "40").parse().unwrap_or(40);
    let scan = source::scan(
        &path,
        &wanted,
        how,
        if near.is_empty() { None } else { Some(near) },
        within,
        source::MAX_BYTES,
    );
    let findings = report_lines(
        &path,
        &scan,
        how,
        if near.is_empty() { None } else { Some(near) },
        within,
    );
    let ok_all = scan.missing().is_empty() && scan.unreadable.is_empty();
    Outcome::found(
        if ok_all {
            Vec::new()
        } else {
            vec![String::new()]
        }
        .into_iter()
        .filter(|x| !x.is_empty())
        .chain(if ok_all {
            None
        } else {
            Some("照合できていないものが在る".to_owned())
        })
        .collect(),
        json!({
            "path": path.display().to_string(),
            "how": how.key(),
            "lines": findings,
            "checked": scan.results.len(),
            "missing": scan.missing().len(),
            "unreadable": scan.unreadable.len(),
            "can_conclude_absent": scan.can_conclude_absent(),
        }),
    )
}

/// 報告の行。**印字する側と、機械へ返す側が、同じ文字列を使う。**
fn report_lines(
    path: &Path,
    scan: &Scan,
    how: How,
    near: Option<&str>,
    within: usize,
) -> Vec<String> {
    let mut out = Vec::new();
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let meta = path.with_file_name(format!("{name}.meta.json"));
    if let Ok(body) = std::fs::read_to_string(&meta) {
        if let Ok(m) = serde_json::from_str::<serde_json::Value>(&body) {
            let get = |k: &str| {
                m.get(k)
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_owned()
            };
            let n = |k: &str| m.get(k).and_then(serde_json::Value::as_u64).unwrap_or(0);
            out.push(format!("── {}", get("url")));
            out.push(format!(
                "   {} バイト ・ {} 行 ・ 取得した日 {}",
                grouped(n("bytes")),
                grouped(n("lines")),
                get("fetched_at").chars().take(10).collect::<String>()
            ));
        }
    } else {
        out.push(format!(
            "── {}  {} ファイル",
            path.display(),
            scan.docs_read
        ));
    }
    if let Some(a) = near {
        out.push(format!(
            "   アンカー「{a}」の ±{within} 行の内側だけを検査する"
        ));
    }
    let note = if how == How::Text {
        "　※ 探索のための種類。照合した証しにはならない"
    } else {
        ""
    };
    out.push(format!("   照合の種類: {}{note}", how.key()));
    for r in &scan.results {
        if r.hits.is_empty() {
            out.push(format!("  ×    {:34} 読めた範囲には無い", r.needle));
        } else {
            let where_: Vec<String> = r
                .hits
                .iter()
                .take(2)
                .map(|h| format!("{}:{}", h.doc, h.line))
                .collect();
            out.push(format!(
                "  ok   {:34} {} か所　{}",
                r.needle,
                r.hits.len(),
                where_.join(" ・ ")
            ));
        }
    }
    out.push(format!(
        "── 照合できた {} ／ 読めた範囲に無い {} ／ 読めなかった {}",
        scan.results.len(),
        scan.missing().len(),
        scan.unreadable.len()
    ));
    if !scan.unreadable.is_empty() {
        out.push(String::new());
        out.push(
            "**読めなかった範囲が在る。0件を「原文に無い」と結論してはいけない。**".to_owned(),
        );
        for (name, why) in &scan.unreadable {
            out.push(format!("  読めず  {name}　{why}"));
        }
    }
    if !scan.missing().is_empty() {
        out.push(String::new());
        out.push("**読めた範囲に無いものを、原典の名前として書いてはいけない。**".to_owned());
        out.push(
            "見つからない原因は4つ──①名前を言い換えた ②別のページに在る \
             ③読めなかった範囲に在る ④本当に無い。"
                .to_owned(),
        );
        out.push(
            "①なら直す。②なら該当ページを取得して照合し直す。③なら読めるようにする。\
             ④なら「無い」と書く。**推測で補完しない。**"
                .to_owned(),
        );
    }
    out
}

fn human_verify(out: &Outcome) -> String {
    if !out.ok {
        return out.findings.join("\n");
    }
    let empty = vec![];
    out.data
        .get("lines")
        .and_then(|x| x.as_array())
        .unwrap_or(&empty)
        .iter()
        .filter_map(|x| x.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// この Skill が持つ道具の一覧。**能力の正本である。**
#[must_use]
pub fn tools() -> Vec<Tool> {
    let dir = Arg::opt("dir", "置き場所", Some(DEFAULT_DIR));
    vec![
        Tool {
            name: "fetch",
            summary: "原文を取得して保存する",
            args: vec![Arg::need("url", "取得する出どころ"), dir.clone()],
            run: run_fetch,
            human: human_fetch,
        },
        Tool {
            name: "list",
            summary: "取得したものを並べる",
            args: vec![dir.clone()],
            run: run_list,
            human: human_list,
        },
        Tool {
            name: "verify",
            summary: "原文の文字列で照合する",
            args: vec![
                Arg::need("path", "原文"),
                Arg::some("needle", "照合するもの（複数可）。--from で渡すなら省ける"),
                Arg::opt("as", "照合の種類（identifier ／ quote ／ text）", None),
                Arg::opt("near", "この語の近くだけを検査する", None),
                Arg::opt("within", "アンカーから何行の内側か", Some("40")),
                Arg::opt("from", "照合するものを、この一覧から読む", None),
            ],
            run: run_verify,
            human: human_verify,
        },
    ]
}
