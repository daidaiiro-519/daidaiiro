//! check：Skill のフォルダを、項目ごとに5つの結果（pass ・ fail ・ warn ・ unknown ・ skip）で返す。
//! 型の判定は、読み取ったインスタンスを基盤のエンジンに型のスキーマで検査させて得る。

use crate::engine;
use crate::read::{self, Read};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// 項目 id と、何を見るか。この順に結果を返す
pub const ITEMS: [(&str, &str); 19] = [
    ("skill.frontmatter", "先頭の欄が在り、YAML として読める"),
    ("skill.fields", "先頭の欄は Agent Skills の仕様の欄だけ"),
    ("skill.name", "name の規則とフォルダの名前"),
    ("skill.description", "description が在り、1,024文字まで"),
    ("skill.compatibility", "compatibility は500文字まで"),
    ("skill.lines", "SKILL.md は500行未満"),
    ("skill.sections", "6つの節が欠けず、重ならず、順どおり"),
    ("skill.placeholders", "記入欄 {{…}} が残っていない"),
    (
        "skill.body.paths",
        "本文に Skill の置き場所 ・ ツール専用の変数 ・ 絶対パスが無い",
    ),
    (
        "skill.body.ai-names",
        "本文の AI ・ ツールの名前（一覧と照らす。警告だけ）",
    ),
    ("skill.links", "本文のリンクの指す先が在る"),
    (
        "tool.schema",
        "tool.json が型 skills-creator/tool@v1 に合う",
    ),
    (
        "tool.abstract",
        "tool.json の abstract の版をエンジンが扱える",
    ),
    (
        "tool.command",
        "tool.json の command が Skill のフォルダの中の相対パス",
    ),
    (
        "assets.schemas",
        "assets/ のスキーマ（基盤の check-schemas）",
    ),
    (
        "tools.declared",
        "tools.json の宣言（基盤の --skill --json）",
    ),
    (
        "dist.command",
        "tool.json の command の実行ファイルが入っている",
    ),
    ("dist.sha256", "宣言した実行ファイルの SHA-256 が合う"),
    ("dist.tools-listed", "宣言したツールが一覧に出る"),
];

pub struct Options {
    pub engine: PathBuf,
    /// 型のスキーマを assets/ に持つ skills-creator の Skill のフォルダ
    pub own_skill: PathBuf,
    pub dist: bool,
}

#[derive(Default)]
struct Results(Vec<(String, &'static str, String, Option<&'static str>)>);

impl Results {
    fn set(&mut self, id: &str, status: &'static str, detail: impl Into<String>) {
        self.set_from(id, status, detail, None);
    }
    fn set_from(
        &mut self,
        id: &str,
        status: &'static str,
        detail: impl Into<String>,
        from: Option<&'static str>,
    ) {
        let detail = detail.into();
        match self.0.iter_mut().find(|r| r.0 == id) {
            // 1つの項目に2つ以上の検出があれば、悪いほうを残し、中身をつなぐ
            Some(r) => {
                if rank(status) > rank(r.1) {
                    r.1 = status;
                }
                if !detail.is_empty() {
                    r.2 = if r.2.is_empty() {
                        detail
                    } else {
                        format!("{} ／ {detail}", r.2)
                    };
                }
            }
            None => self.0.push((id.to_owned(), status, detail, from)),
        }
    }
    fn has(&self, id: &str) -> bool {
        self.0.iter().any(|r| r.0 == id)
    }
    fn failed(&self) -> bool {
        self.0.iter().any(|r| r.1 == "fail" || r.1 == "unknown")
    }
}

fn rank(status: &str) -> u8 {
    match status {
        "fail" => 4,
        "unknown" => 3,
        "warn" => 2,
        "pass" => 1,
        _ => 0,
    }
}

pub fn check(dir: &Path, options: &Options) -> Value {
    let mut results = Results::default();
    let folder = dir
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_default();

    // SKILL.md
    let skill_text = fs::read_to_string(dir.join("SKILL.md"));
    let ai_names = ai_names(&options.own_skill);
    let read = skill_text.as_ref().ok().map(|text| {
        read::read(
            text,
            &folder,
            &|target| dir.join(target).exists(),
            &ai_names,
        )
    });
    match (&skill_text, &read) {
        (Err(_), _) => {
            results.set("skill.frontmatter", "fail", "SKILL.md が無い");
            for id in SKILL_IDS {
                if !results.has(id) {
                    results.set(id, "skip", "前提の項目 skill.frontmatter が不合格");
                }
            }
        }
        (
            _,
            Some(Read {
                frontmatter_error: Some(error),
                ..
            }),
        ) => {
            results.set("skill.frontmatter", "fail", error.clone());
            for id in [
                "skill.fields",
                "skill.name",
                "skill.description",
                "skill.compatibility",
            ] {
                results.set(id, "skip", "前提の項目 skill.frontmatter が不合格");
            }
        }
        _ => results.set("skill.frontmatter", "pass", ""),
    }

    // tool.json
    let tool = fs::read_to_string(dir.join("tool.json"));
    let tool: Option<Value> = match tool {
        Err(_) => {
            results.set("tool.schema", "fail", "tool.json が無い");
            None
        }
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Err(error) => {
                results.set(
                    "tool.schema",
                    "fail",
                    format!("JSON として読めない：{error}"),
                );
                None
            }
            Ok(value)
                if value.get("$schema").and_then(Value::as_str)
                    != Some("skills-creator/tool@v1") =>
            {
                results.set(
                    "tool.schema",
                    "fail",
                    "$schema が型 skills-creator/tool@v1 を指していない",
                );
                None
            }
            Ok(value) => Some(value),
        },
    };
    if tool.is_none() {
        for id in ["tool.abstract", "tool.command"] {
            results.set(id, "skip", "前提の項目 tool.schema が不合格");
        }
    }

    // 型の判定を、基盤のエンジンに任せる
    type_check(&mut results, read.as_ref(), tool.as_ref(), options);

    // 警告：AI ・ ツールの名前
    if let Some(read) = &read {
        if read.ai_hits.is_empty() {
            results.set("skill.body.ai-names", "pass", "");
        } else {
            let hits: Vec<String> = read
                .ai_hits
                .iter()
                .map(|(l, n)| format!("{n}（SKILL.md:{l}）"))
                .collect();
            results.set(
                "skill.body.ai-names",
                "warn",
                format!(
                    "一覧に当たった：{}。宛先になっているかは、レビューで見る",
                    hits.join("、")
                ),
            );
        }
    } else {
        results.set("skill.body.ai-names", "skip", "SKILL.md が無い");
    }

    schemas(&mut results, dir, options);
    declared(&mut results, dir, options);
    dist(&mut results, dir, tool.as_ref(), options);

    // 型の判定で検出が無かった項目は合格
    for (id, _) in ITEMS {
        if !results.has(id) {
            results.set(id, "pass", "");
        }
    }
    output(results, dir, options)
}

fn ai_names(own_skill: &Path) -> Vec<String> {
    fs::read_to_string(own_skill.join("assets/skill.schema.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|s| s.pointer("/$defs/ai-names/enum").cloned())
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

const SKILL_IDS: [&str; 10] = [
    "skill.fields",
    "skill.name",
    "skill.description",
    "skill.compatibility",
    "skill.lines",
    "skill.sections",
    "skill.placeholders",
    "skill.body.paths",
    "skill.links",
    "skill.frontmatter",
];

fn type_check(results: &mut Results, read: Option<&Read>, tool: Option<&Value>, options: &Options) {
    let scratch = std::env::temp_dir().join(format!("skills-creator-check-{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    if fs::create_dir_all(&scratch).is_err() {
        unknown_all(
            results,
            read.is_some(),
            tool.is_some(),
            "読み取った結果を置く一時フォルダを作れない",
        );
        return;
    }
    if let Some(read) = read {
        let _ = fs::write(scratch.join("skill.json"), read.instance.to_string());
    }
    if let Some(tool) = tool {
        let _ = fs::write(scratch.join("tool.json"), tool.to_string());
    }
    if read.is_none() && tool.is_none() {
        let _ = fs::remove_dir_all(&scratch);
        return;
    }
    let own = options.own_skill.to_string_lossy().into_owned();
    let dir = scratch.to_string_lossy().into_owned();
    let answer = engine::call(
        &options.engine,
        &["--skill", &own, "check", "--dir", &dir, "--json"],
    );
    let _ = fs::remove_dir_all(&scratch);
    let answer = match answer {
        Ok(value) if value["ok"] == true => value,
        Ok(value) => {
            let why = format!(
                "基盤の check が答えなかった：{} {}",
                value["reason"], value["detail"]
            );
            unknown_all(results, read.is_some(), tool.is_some(), &why);
            return;
        }
        Err(why) => {
            unknown_all(results, read.is_some(), tool.is_some(), &why);
            return;
        }
    };
    let skipped_frontmatter = read.map(|r| r.frontmatter_error.is_some()).unwrap_or(false);
    for instance in answer["instances"].as_array().into_iter().flatten() {
        let is_tool = instance["path"]
            .as_str()
            .unwrap_or("")
            .ends_with("tool.json");
        for error in instance["errors"].as_array().into_iter().flatten() {
            let property = error["property"].as_str().unwrap_or("");
            let reason = error["reason"].as_str().unwrap_or("");
            let id = if is_tool {
                tool_item(property, reason)
            } else {
                skill_item(property, reason)
            };
            if skipped_frontmatter
                && (property.starts_with("/frontmatter") || property == "/name_is_folder")
            {
                continue;
            }
            let detail = describe(property, reason, read.map(|r| &r.instance));
            results.set(id, "fail", detail);
        }
        // 必須の欄が無いものは、基盤が未記入として返す
        for unfilled in instance["unfilled"].as_array().into_iter().flatten() {
            let property = unfilled["property"].as_str().unwrap_or("");
            if skipped_frontmatter && property.starts_with("/frontmatter") {
                continue;
            }
            let id = if is_tool {
                tool_item(property, "")
            } else {
                skill_item(property, "")
            };
            results.set(id, "fail", format!("{property}：必ず書く欄が無い"));
        }
    }
}

fn unknown_all(results: &mut Results, skill: bool, tool: bool, why: &str) {
    if skill {
        for id in SKILL_IDS {
            if !results.has(id) {
                results.set(id, "unknown", why);
            }
        }
    }
    if tool {
        for id in ["tool.schema", "tool.abstract", "tool.command"] {
            results.set(id, "unknown", why);
        }
    }
}

fn skill_item(property: &str, reason: &str) -> &'static str {
    match property {
        "/frontmatter/name" | "/name_is_folder" => "skill.name",
        "/frontmatter/description" => "skill.description",
        "/frontmatter/compatibility" => "skill.compatibility",
        "/frontmatter" if reason.contains("additionalProperties") => "skill.fields",
        "/frontmatter" if reason.contains("description") => "skill.description",
        "/frontmatter" if reason.contains("name") => "skill.name",
        "/lines" => "skill.lines",
        "/sections" => "skill.sections",
        "/placeholders" => "skill.placeholders",
        "/tool_paths" => "skill.body.paths",
        "/broken_links" => "skill.links",
        _ => "skill.fields",
    }
}

fn tool_item(property: &str, reason: &str) -> &'static str {
    match property {
        "/abstract" => "tool.abstract",
        "/command" => "tool.command",
        _ if reason.contains("abstract") && !reason.contains("additionalProperties") => {
            "tool.abstract"
        }
        _ if reason.contains("command") && !reason.contains("additionalProperties") => {
            "tool.command"
        }
        _ => "tool.schema",
    }
}

/// 基盤の理由に、見つけた行を添える
fn describe(property: &str, reason: &str, instance: Option<&Value>) -> String {
    let hits = match property {
        "/placeholders" | "/tool_paths" | "/broken_links" => instance
            .and_then(|i| i.get(&property[1..]))
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|h| {
                        format!(
                            "SKILL.md:{} {}",
                            h["line"],
                            h["text"].as_str().unwrap_or("")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("、")
            }),
        "/name_is_folder" => instance.map(|i| {
            format!(
                "name {} がフォルダの名前 {} と違う",
                i["frontmatter"]["name"], i["folder"]
            )
        }),
        "/lines" => instance.map(|i| format!("{}行ある（500行未満にする）", i["lines"])),
        _ => None,
    };
    match hits {
        Some(hits) => hits,
        None => format!("{property}：{reason}"),
    }
}

fn has_schemas(dir: &Path) -> bool {
    fs::read_dir(dir).into_iter().flatten().flatten().any(|e| {
        let path = e.path();
        if path.is_dir() {
            has_schemas(&path)
        } else {
            path.to_string_lossy().ends_with(".schema.json")
        }
    })
}

fn schemas(results: &mut Results, dir: &Path, options: &Options) {
    let assets = dir.join("assets");
    if !has_schemas(&assets) {
        results.set("assets.schemas", "skip", "assets/ にスキーマが無い");
        return;
    }
    let from = Some("schema-driven check-schemas");
    match engine::call(
        &options.engine,
        &[
            "check-schemas",
            "--dir",
            &assets.to_string_lossy(),
            "--json",
        ],
    ) {
        Ok(value) if value["ok"] == true => {
            let findings = value["findings"].as_array().cloned().unwrap_or_default();
            if findings.is_empty() {
                results.set_from(
                    "assets.schemas",
                    "pass",
                    "基盤の check-schemas が通った",
                    from,
                );
            } else {
                let list: Vec<String> = findings
                    .iter()
                    .take(5)
                    .map(|f| {
                        format!(
                            "{} {}",
                            f["schema"].as_str().unwrap_or(""),
                            f["message"].as_str().unwrap_or("")
                        )
                    })
                    .collect();
                results.set_from(
                    "assets.schemas",
                    "fail",
                    format!("基盤の検出 {}件：{}", findings.len(), list.join("、")),
                    from,
                );
            }
        }
        Ok(value) => results.set_from(
            "assets.schemas",
            "unknown",
            format!("基盤が答えなかった：{}", value["detail"]),
            from,
        ),
        Err(why) => results.set_from("assets.schemas", "unknown", why, from),
    }
}

fn declared_names(dir: &Path) -> Option<Vec<String>> {
    let text = fs::read_to_string(dir.join("tools.json")).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    Some(
        value["tools"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|t| t["name"].as_str().map(str::to_owned))
            .collect(),
    )
}

fn declared(results: &mut Results, dir: &Path, options: &Options) {
    if !dir.join("tools.json").exists() {
        results.set("tools.declared", "skip", "tools.json が無い");
        return;
    }
    let from = Some("schema-driven --skill --json");
    match engine::call(
        &options.engine,
        &["--skill", &dir.to_string_lossy(), "--json"],
    ) {
        Ok(value) if value["ok"] == true => {
            results.set_from("tools.declared", "pass", "基盤が宣言を読めた", from)
        }
        Ok(value) => results.set_from(
            "tools.declared",
            "fail",
            format!(
                "{} {}",
                value["reason"].as_str().unwrap_or(""),
                value["detail"].as_str().unwrap_or("")
            ),
            from,
        ),
        Err(why) => results.set_from("tools.declared", "unknown", why, from),
    }
}

fn dist(results: &mut Results, dir: &Path, tool: Option<&Value>, options: &Options) {
    let ids = ["dist.command", "dist.sha256", "dist.tools-listed"];
    if !options.dist {
        for id in ids {
            results.set(id, "skip", "配布物の段で見る（--level dist）");
        }
        return;
    }
    if results.failed() {
        for id in ids {
            results.set(id, "skip", "書いたその場の段で合格していない");
        }
        return;
    }
    let command = tool
        .and_then(|t| t["command"].as_str())
        .map(|c| dir.join(c));
    let Some(command) = command.filter(|c| c.is_file()) else {
        for id in ids {
            results.set(
                id,
                "skip",
                "実行ファイルが入っていない（配布物を作る段で入る）",
            );
        }
        return;
    };
    results.set("dist.command", "pass", "");
    let Some(names) = declared_names(dir) else {
        results.set("dist.sha256", "skip", "tools.json が無い");
        results.set("dist.tools-listed", "skip", "tools.json が無い");
        return;
    };
    let text = fs::read_to_string(dir.join("tools.json")).unwrap_or_default();
    let value: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    let mut wrong = Vec::new();
    for t in value["tools"].as_array().into_iter().flatten() {
        let file = t["command"][0].as_str().unwrap_or("");
        let want = t["sha256"].as_str().unwrap_or("");
        match fs::read(dir.join(file)) {
            Ok(bytes) => {
                let got = format!("{:x}", Sha256::digest(&bytes));
                if got != want {
                    wrong.push(format!(
                        "{}：{file} の SHA-256 が宣言と違う",
                        t["name"].as_str().unwrap_or("")
                    ));
                }
            }
            Err(_) => wrong.push(format!(
                "{}：{file} が無い",
                t["name"].as_str().unwrap_or("")
            )),
        }
    }
    if wrong.is_empty() {
        results.set("dist.sha256", "pass", "");
    } else {
        results.set("dist.sha256", "fail", wrong.join("、"));
    }
    match engine::call(&command, &["--skill", &dir.to_string_lossy(), "--json"]) {
        Ok(listing) => {
            let listed: Vec<&str> = listing["data"]["tools"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|t| t["name"].as_str())
                .collect();
            let missing: Vec<&String> = names
                .iter()
                .filter(|n| !listed.contains(&n.as_str()))
                .collect();
            if missing.is_empty() {
                results.set("dist.tools-listed", "pass", "");
            } else {
                results.set(
                    "dist.tools-listed",
                    "fail",
                    format!("一覧に無い：{missing:?}"),
                );
            }
        }
        Err(why) => results.set("dist.tools-listed", "unknown", why),
    }
}

fn output(results: Results, dir: &Path, options: &Options) -> Value {
    let mut ordered = Vec::new();
    for (id, label) in ITEMS {
        if let Some((_, status, detail, from)) = results.0.iter().find(|r| r.0 == id) {
            let mut item = json!({"id": id, "label": label, "status": status});
            if !detail.is_empty() {
                item["detail"] = Value::from(detail.clone());
            }
            if let Some(from) = from {
                item["from"] = Value::from(*from);
            }
            ordered.push(item);
        }
    }
    let count = |s: &str| results.0.iter().filter(|r| r.1 == s).count();
    let passed = !results.failed();
    json!({
        "ok": passed,
        "level": if options.dist { "dist" } else { "source" },
        "skill": dir.to_string_lossy(),
        "results": ordered,
        "counts": {"pass": count("pass"), "fail": count("fail"), "warn": count("warn"),
                   "unknown": count("unknown"), "skip": count("skip")},
    })
}
