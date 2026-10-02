// SPDX-License-Identifier: MIT
//! references の道具の出力を、言語の組をまたいで突き合わせる（ACDR 0070）。**見つけるが、直さない。**
//!
//! 同じ references が、言語によって違って見えてはならない（ACDR 0069）。基準の Skill（ふつうは
//! Rust の組で生んだもの）と比べる Skill の CLI を、それぞれの `tool.json` から起動し、実物の
//! references に get ・ view ・ import を当てて、出力を比べる。**起動は登録に任せる** ── どの言語の
//! 組でも、同じ道具で比べられる。
//!
//! validate は合否と種類の一覧だけを比べる ── 検出の文言と件数は、JSON Schema の検査の道具ごとに違い、
//! 契約はそれを規定しない。

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

use crate::behavior;
use crate::data_access::files;
use crate::data_access::process;

/// 1回の起動の制限時間。**描画の大きい種類でも収まる長さにする。**
const LIMIT: Duration = Duration::from_secs(120);

/// 突き合わせの結果。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Report {
    /// 比べた事例の数。
    pub cases: usize,
    /// 違った事例。**空なら、すべて一致した。**
    pub mismatches: Vec<String>,
}

/// 起動できる CLI 1つ。
struct Cli {
    command: PathBuf,
    args: Vec<String>,
}

impl Cli {
    fn of(root: &Path) -> Result<Self, String> {
        let (command, args) = behavior::cli_of(root)?;
        Ok(Self { command, args })
    }

    /// 起動して、終了コードと `--json` の出力を返す。**別の作業場所から起動する。**
    fn call(&self, args: &[String]) -> (i32, Value) {
        let mut with = self.args.clone();
        with.extend(args.iter().cloned());
        with.push("--json".to_owned());
        match process::run(&self.command, &with, files::temp_dir(), LIMIT) {
            Ok(ran) => (
                ran.code,
                serde_json::from_str(&ran.stdout).unwrap_or(Value::String(ran.stdout)),
            ),
            Err(why) => (-1, Value::String(format!("{why:?}"))),
        }
    }
}

fn strs(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

/// 最初に違う位置の前後を、短く示す。
fn first_difference(a: &str, b: &str) -> String {
    let at = a
        .chars()
        .zip(b.chars())
        .position(|(x, y)| x != y)
        .unwrap_or_else(|| a.chars().count().min(b.chars().count()));
    let near = |s: &str| -> String { s.chars().skip(at.saturating_sub(20)).take(60).collect() };
    format!("{at}文字目 ── 基準「{}」／比べた側「{}」", near(a), near(b))
}

fn text_of(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// 種類の JSON に在る項目の id を並べる。
fn ids(data: &Path) -> Vec<String> {
    let Ok(body) = files::read_to_string(data) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&body) else {
        return Vec::new();
    };
    v.get("items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|x| x.get("id").and_then(Value::as_str).map(str::to_owned))
        .collect()
}

/// 回答の例を探す。**組み立ての産物のフォルダには入らない。**
fn answers(dir: &Path, out: &mut Vec<PathBuf>) {
    for p in files::list(dir).unwrap_or_default() {
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if files::is_dir(&p) {
            let generated = [
                "target",
                "node_modules",
                ".venv",
                "bin",
                "obj",
                "__pycache__",
            ];
            if !generated.contains(&name.as_str()) {
                answers(&p, out);
            }
        } else if name.starts_with("answer") && name.ends_with(".json") {
            out.push(p);
        }
    }
}

/// 1件を比べ、違えば報告へ積む。**key を渡すと、data のその欄だけを比べる。**
fn compare(
    report: &mut Report,
    label: String,
    x: (i32, Value),
    y: (i32, Value),
    key: Option<&str>,
) {
    report.cases += 1;
    let pick = |v: &Value| match key {
        Some(k) => v
            .pointer(&format!("/data/{k}"))
            .cloned()
            .unwrap_or(Value::Null),
        None => v.get("data").cloned().unwrap_or_else(|| v.clone()),
    };
    // **基準が失敗した事例は、一致と数えない** ── 両方が同じ誤りを返すと、比べていないのに一致に見える
    if x.1.get("ok") != Some(&Value::Bool(true)) {
        report.mismatches.push(format!(
            "{label}: 基準が道具を実行できない（終了コード {}）── {}",
            x.0,
            text_of(&x.1).chars().take(160).collect::<String>()
        ));
        return;
    }
    let (px, py) = (pick(&x.1), pick(&y.1));
    if x.0 != y.0 || px != py || x.1.get("ok") != y.1.get("ok") {
        report.mismatches.push(format!(
            "{label}: 基準と違う（終了コード {} ／ {}）── {}",
            x.0,
            y.0,
            first_difference(&text_of(&px), &text_of(&py))
        ));
    }
}

/// 2つの Skill の references の道具の出力を、`corpus` の下の Skill の references で突き合わせる。
/// `document_schema` は、取り込み（import）の事例で空の references に置くスキーマである。
///
/// # Errors
///
/// どちらかの CLI を起動できないとき、`corpus` を読めないときに返す。
pub fn conform(
    base: &Path,
    other: &Path,
    corpus: &Path,
    document_schema: &Path,
) -> Result<Report, String> {
    let a = Cli::of(base)?;
    let b = Cli::of(other)?;
    let mut report = Report::default();
    // **絶対の経路にする** ── CLI は別の作業場所から起動するので、相対の経路では references に届かない
    let corpus = files::canonicalize(corpus)
        .map_err(|e| format!("{} を読めない ── {e}", corpus.display()))?;
    let mut skills =
        files::list(&corpus).map_err(|e| format!("{} を読めない ── {e}", corpus.display()))?;
    skills.sort();
    for skill in skills
        .iter()
        .filter(|p| files::is_dir(p.join("references")))
    {
        let refs = skill.join("references");
        let name = skill
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let root = strs(&["--skill_root", &skill.display().to_string()]);
        let mut kinds: Vec<PathBuf> = files::list(&refs)
            .unwrap_or_default()
            .into_iter()
            .filter(|p| p.to_string_lossy().ends_with(".schema.json"))
            // **見た目の正本の写し（view.*）は種類ではない** ── references の道具の kinds と同じ扱いにする
            .filter(|p| {
                !p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("view."))
            })
            .collect();
        kinds.sort();
        for schema in kinds {
            let file = schema
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let kind = file.trim_end_matches(".schema.json").to_owned();
            let data = refs.join(format!("{kind}.json"));
            if !files::is_file(&data) {
                continue;
            }
            for (verb, key) in [("get", None), ("view", Some("html"))] {
                let args = [strs(&[verb, "--kind", &kind]), root.clone()].concat();
                compare(
                    &mut report,
                    format!("{name} {verb} {kind}"),
                    a.call(&args),
                    b.call(&args),
                    key,
                );
            }
            for id in ids(&data) {
                for (verb, key) in [("get", None), ("view", Some("html"))] {
                    let args = [strs(&[verb, "--kind", &kind, "--id", &id]), root.clone()].concat();
                    compare(
                        &mut report,
                        format!("{name} {verb} {kind} {id}"),
                        a.call(&args),
                        b.call(&args),
                        key,
                    );
                }
                // **除く欄を指定した取り出し**（ACDR 0090）
                let args = [
                    strs(&["get", "--kind", &kind, "--id", &id, "--omit", "source"]),
                    root.clone(),
                ]
                .concat();
                compare(
                    &mut report,
                    format!("{name} get {kind} {id} --omit source"),
                    a.call(&args),
                    b.call(&args),
                    None,
                );
            }
            // **旗を動詞の前に置いても、後ろに置いたときと同じに読む**（ACDR 0090）
            let usual = [strs(&["get", "--kind", &kind]), root.clone()].concat();
            let first = [root.clone(), strs(&["get", "--kind", &kind])].concat();
            let x = a.call(&usual);
            for (side, cli) in [("基準", &a), ("比べる側", &b)] {
                report.cases += 1;
                let y = cli.call(&first);
                if y != x {
                    report.mismatches.push(format!(
                        "{name} get {kind}: {side}の道具が、動詞の前の旗を後ろの旗と同じに読まない（終了コード {}）",
                        y.0
                    ));
                }
            }
        }
        if files::is_file(refs.join("answer.schema.json")) {
            let mut found = Vec::new();
            answers(&skill.join("tool"), &mut found);
            answers(&refs, &mut found);
            found.sort();
            for fx in found {
                let args = [
                    strs(&[
                        "view",
                        "--kind",
                        "answer",
                        "--file",
                        &fx.display().to_string(),
                    ]),
                    root.clone(),
                ]
                .concat();
                let label = format!(
                    "{name} view answer {}",
                    fx.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()
                );
                compare(
                    &mut report,
                    label.clone(),
                    a.call(&args),
                    b.call(&args),
                    Some("html"),
                );
                // **回答の検査の合否を比べる** ── 指す先の照合（x-refers ・ x-quotes）を含む（ACDR 0090）
                let args = [
                    strs(&[
                        "validate",
                        "--kind",
                        "answer",
                        "--file",
                        &fx.display().to_string(),
                    ]),
                    root.clone(),
                ]
                .concat();
                let (x, y) = (a.call(&args), b.call(&args));
                report.cases += 1;
                if x.0 != y.0 {
                    report.mismatches.push(format!(
                        "{} の validate: 合否が基準と違う（基準 {} ／ 比べる側 {}）",
                        label.replace(" view answer ", " "),
                        x.0,
                        y.0
                    ));
                }
            }
        }
        // **validate は合否と種類の一覧だけを比べる** ── 検出の文言と件数は、検査の道具ごとに違う
        let args = [strs(&["validate"]), root.clone()].concat();
        let (x, y) = (a.call(&args), b.call(&args));
        let passed = |v: &Value| {
            v.get("findings")
                .and_then(Value::as_array)
                .is_some_and(Vec::is_empty)
        };
        report.cases += 1;
        if x.1.pointer("/data/kinds").is_none() {
            report.mismatches.push(format!(
                "{name} validate: 基準が道具を実行できない（終了コード {}）",
                x.0
            ));
        } else if passed(&x.1) != passed(&y.1)
            || x.1.pointer("/data/kinds") != y.1.pointer("/data/kinds")
        {
            report
                .mismatches
                .push(format!("{name} validate: 合否か種類の一覧が基準と違う"));
        }
    }
    // **取り込み（import）は、書き出した document.json のバイト列を比べる**
    let schema = files::read_to_string(document_schema)
        .map_err(|e| format!("{} を読めない ── {e}", document_schema.display()))?;
    for skill in &skills {
        let md = skill.join("SKILL.md");
        if !files::is_file(&md) {
            continue;
        }
        let name = skill
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut outs = Vec::new();
        for (side, cli) in [("a", &a), ("b", &b)] {
            let dir = files::temp_dir().join(format!("sc-conform-{name}-{side}"));
            let refs = dir.join("references");
            let _ = files::remove_file(refs.join("document.json"));
            files::create_dir_all(&refs).map_err(|e| e.to_string())?;
            files::write(refs.join("document.schema.json"), &schema).map_err(|e| e.to_string())?;
            let args = strs(&[
                "import",
                "--file",
                &md.display().to_string(),
                "--id",
                &name,
                "--source",
                "SKILL.md",
                "--fetched",
                "-",
                "--skill_root",
                &dir.display().to_string(),
            ]);
            let (code, _) = cli.call(&args);
            outs.push((
                code,
                files::read_to_string(refs.join("document.json")).unwrap_or_default(),
            ));
        }
        report.cases += 1;
        if outs[0].1.is_empty() {
            report.mismatches.push(format!(
                "{name} import SKILL.md: 基準が取り込めない（終了コード {}）",
                outs[0].0
            ));
        } else if outs[0] != outs[1] {
            report.mismatches.push(format!(
                "{name} import SKILL.md: 基準と違う（終了コード {} ／ {}）── {}",
                outs[0].0,
                outs[1].0,
                first_difference(&outs[0].1, &outs[1].1)
            ));
        }
    }
    Ok(report)
}
