//! UC4 確かめる（Skill だけ）：渡した Skill のフォルダごとに、SAS の検証の結果（書き換えない）と、SAS が文書1つでは
//! 判定できない creator の検査の結果と、承認の材料（approval）を返す。確かめても Skill のファイルは変えない（SAS の記録 .git/sas/sas.db には
//! 検証の操作が1件足される）。
//!
//! creator の検査：
//! - name-is-folder：skill.json の name がフォルダ名と同じ（Agent Skills の仕様。SAS は文書1つしか見ない）
//! - name-unique：同じ親フォルダにある Skill のあいだで name が重ならない（複数の Skill をまたぐ）
//! - links-resolve：フォルダそのものと、フォルダの中のシンボリックリンクの先がある（ファイルの外の事実）
//! - skill-md-built：SKILL.md が、skill.json から write-skill-md で作るものと同じ（AI ツールが読むのは SKILL.md）
//! - references-exist：参照する資料（references）の各 path が Skill のフォルダの中にあり、ファイルがある（ファイルの外の事実）
//! - json-schema-conforms：参照する資料と出力のうち schema を持つ件ごとに、schema があり（URN なら SAS が引ける）、
//!   JSON の資料は中身がその schema に従う（URN なら sas validate、置き場所なら JSON Schema で確かめる）。出力は実行する
//!   まで中身が無いので、schema があるかだけを見る
//! - grounds-are-outputs：参照する資料の grounds_for（資料が根拠になる出力）の各名前が、outputs の name にある
//!   （名前どうしの突き合わせで機械で決まる。references と grounds_for が無ければ verified が1件）

use crate::ports::{JsonSchemaChecker, SasRun, SkillFiles, SkillHistory, SkillValidator};
pub use crate::use_cases::Refused;
use crate::use_cases::approval::{Approval, approval};
use crate::use_cases::{read_skill, trimmed};
use serde_json::Value;
use skills_domain::services::skill_md;

/// creator の検査の結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// 満たす
    Verified,
    /// 満たさない
    Violated,
    /// 確かめる材料を読めなかった（skill.json が無い ・ JSON でない ・ name が無い）
    Error,
}

impl Outcome {
    /// 出力に書く名前（SAS の判定の区分と同じ語）。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Violated => "violated",
            Self::Error => "error",
        }
    }
}

/// creator の検査の結果1件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// 検査の名前（name-is-folder ・ name-unique ・ links-resolve）
    pub check: &'static str,
    /// 結果
    pub outcome: Outcome,
    /// 違反に当たる所（重なった相手のフォルダ ・ 先の無いリンク）
    pub path: Option<String>,
    /// 理由
    pub reason: String,
}

/// Skill のフォルダ1つの結果。
#[derive(Debug, Clone, PartialEq)]
pub struct SkillChecked {
    /// 渡されたフォルダの経路（渡されたまま）
    pub folder: String,
    /// SAS の検証の結果（書き換えない）
    pub sas: SasRun,
    /// creator の検査の結果（検査の順）
    pub creator: Vec<Finding>,
    /// 承認の材料（skill.json を読めたときだけ）
    pub approval: Option<Approval>,
    /// 前の確かめから変わった所（SAS の記録から。読めなければ None）
    pub changes: Option<Value>,
}

impl SkillChecked {
    /// SAS か creator のどちらかで検出があったか。
    #[must_use]
    pub fn detected(&self) -> bool {
        self.sas.exit != 0 || self.creator.iter().any(|f| f.outcome != Outcome::Verified)
    }
}

/// 確かめた結果（渡した順）。
#[derive(Debug, Clone, PartialEq)]
pub struct Checked {
    /// Skill ごとの結果
    pub skills: Vec<SkillChecked>,
}

impl Checked {
    /// どれかの Skill で検出があったか。
    #[must_use]
    pub fn detected(&self) -> bool {
        self.skills.iter().any(SkillChecked::detected)
    }
}

/// フォルダ名（経路の最後の部分）。
fn folder_name(folder: &str) -> &str {
    trimmed(folder).rsplit('/').next().unwrap_or_default()
}

/// フォルダの skill.json の name。読めなければ理由。
fn skill_name(files: &dyn SkillFiles, folder: &str) -> Result<String, String> {
    let path = format!("{}/skill.json", trimmed(folder));
    let text = files.read(&path)?;
    let value: Value =
        serde_json::from_str(&text).map_err(|e| format!("{path} が JSON として読めない：{e}"))?;
    value
        .get("name")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("{path} に name が無い"))
}

fn finding(check: &'static str, outcome: Outcome, path: Option<String>, reason: String) -> Finding {
    Finding {
        check,
        outcome,
        path,
        reason,
    }
}

/// フォルダそのものと、フォルダの中のシンボリックリンクの先があるか。
fn links_resolve(files: &dyn SkillFiles, folder: &str) -> Vec<Finding> {
    const CHECK: &str = "links-resolve";
    let folder = trimmed(folder);
    if !files.exists(folder) {
        return vec![finding(
            CHECK,
            Outcome::Violated,
            Some(folder.to_string()),
            format!("フォルダ {folder} が無い（先の無いリンクか、無い経路）"),
        )];
    }
    match files.dangling_links(folder) {
        Ok(broken) if broken.is_empty() => vec![finding(
            CHECK,
            Outcome::Verified,
            None,
            "フォルダと、フォルダの中のリンクの先がある".to_string(),
        )],
        Ok(broken) => broken
            .into_iter()
            .map(|l| {
                let reason = format!("リンク {l} の先が無い");
                finding(CHECK, Outcome::Violated, Some(l), reason)
            })
            .collect(),
        Err(e) => vec![finding(CHECK, Outcome::Error, None, e)],
    }
}

/// name がフォルダ名と同じか、同じ親フォルダのほかの Skill と name が重ならないか。
fn name_checks(files: &dyn SkillFiles, folder: &str) -> Vec<Finding> {
    let name = match skill_name(files, folder) {
        Ok(n) => n,
        Err(e) => {
            return vec![
                finding("name-is-folder", Outcome::Error, None, e.clone()),
                finding("name-unique", Outcome::Error, None, e),
            ];
        }
    };
    let dir = folder_name(folder);
    let is_folder = if name == dir {
        finding(
            "name-is-folder",
            Outcome::Verified,
            None,
            format!("name {name} がフォルダ名と同じ"),
        )
    } else {
        finding(
            "name-is-folder",
            Outcome::Violated,
            None,
            format!("name {name} がフォルダ名 {dir} と違う"),
        )
    };
    // 同じ親フォルダのほかのフォルダのうち、skill.json の name を読めるものが Skill
    let unique = match files.siblings(trimmed(folder)) {
        Ok(others) => {
            let same: Vec<Finding> = others
                .into_iter()
                .filter(|o| skill_name(files, o).is_ok_and(|n| n == name))
                .map(|o| {
                    let reason = format!("name {name} が、同じ親フォルダの {o} と重なる");
                    finding("name-unique", Outcome::Violated, Some(o), reason)
                })
                .collect();
            if same.is_empty() {
                vec![finding(
                    "name-unique",
                    Outcome::Verified,
                    None,
                    format!("name {name} は、同じ親フォルダのほかの Skill と重ならない"),
                )]
            } else {
                same
            }
        }
        Err(e) => vec![finding("name-unique", Outcome::Error, None, e)],
    };
    let mut out = vec![is_folder];
    out.extend(unique);
    out
}

/// SKILL.md が、skill.json から write-skill-md で作るものと同じか。
fn skill_md_built(files: &dyn SkillFiles, folder: &str) -> Finding {
    const CHECK: &str = "skill-md-built";
    let path = format!("{}/{}", trimmed(folder), skill_md::FILE);
    let skill = match read_skill(files, folder) {
        Ok(v) => v,
        Err(r) => return finding(CHECK, Outcome::Error, None, r.message),
    };
    match files.read(&path) {
        Err(_) => finding(
            CHECK,
            Outcome::Violated,
            Some(path),
            format!(
                "SKILL.md が無い（skills write-skill-md {} で作る）",
                trimmed(folder)
            ),
        ),
        Ok(text) if text == skill_md::render(&skill) => finding(
            CHECK,
            Outcome::Verified,
            None,
            "SKILL.md が、skill.json から write-skill-md で作るものと同じ".to_string(),
        ),
        Ok(_) => finding(
            CHECK,
            Outcome::Violated,
            Some(path),
            format!(
                "SKILL.md が、skill.json から write-skill-md で作るものと違う（skills write-skill-md {} で作り直す）",
                trimmed(folder)
            ),
        ),
    }
}

/// 参照する資料の各 path が、Skill のフォルダの中にあり、ファイルがあるか。資料が無ければ verified。
fn references_exist(files: &dyn SkillFiles, folder: &str) -> Vec<Finding> {
    const CHECK: &str = "references-exist";
    let skill = match read_skill(files, folder) {
        Ok(v) => v,
        Err(r) => return vec![finding(CHECK, Outcome::Error, None, r.message)],
    };
    let references = skill
        .get("references")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if references.is_empty() {
        return vec![finding(
            CHECK,
            Outcome::Verified,
            None,
            "references が無い（参照する資料を持たない Skill）".to_string(),
        )];
    }
    references
        .iter()
        .map(|r| {
            let name = r.get("name").and_then(Value::as_str).unwrap_or_default();
            let rel = r.get("path").and_then(Value::as_str).unwrap_or_default();
            let path = format!("{}/{rel}", trimmed(folder));
            // フォルダの外を指す経路（/ で始まる ・ .. を含む）は、配った先に一緒に届かない
            if outside(rel) {
                return finding(
                    CHECK,
                    Outcome::Violated,
                    Some(path),
                    format!("資料 {name} の置き場所 {rel} が、Skill のフォルダの外を指す"),
                );
            }
            if files.exists(&path) {
                finding(
                    CHECK,
                    Outcome::Verified,
                    None,
                    format!("資料 {name} が {path} にある"),
                )
            } else {
                finding(
                    CHECK,
                    Outcome::Violated,
                    Some(path.clone()),
                    format!("資料 {name} が {path} に無い"),
                )
            }
        })
        .collect()
}

/// 参照する資料の grounds_for の各名前が、outputs の name にあるか（無い名前ごとに violated を1件）。
fn grounds_are_outputs(files: &dyn SkillFiles, folder: &str) -> Vec<Finding> {
    const CHECK: &str = "grounds-are-outputs";
    let skill = match read_skill(files, folder) {
        Ok(v) => v,
        Err(r) => return vec![finding(CHECK, Outcome::Error, None, r.message)],
    };
    let names = |v: Option<&Value>| -> Vec<String> {
        v.and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(ToString::to_string)
            .collect()
    };
    let outputs: Vec<String> = skill
        .get("outputs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|o| o.get("name").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect();
    let mut out = Vec::new();
    let mut seen = false;
    for (i, r) in skill
        .get("references")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let material = r.get("name").and_then(Value::as_str).unwrap_or_default();
        for g in names(r.get("grounds_for")) {
            seen = true;
            if !outputs.contains(&g) {
                out.push(finding(
                    CHECK,
                    Outcome::Violated,
                    Some(format!(
                        "{}/skill.json#/references/{i}/grounds_for",
                        trimmed(folder)
                    )),
                    format!("資料 {material} の grounds_for の {g} が、outputs の name に無い"),
                ));
            }
        }
    }
    if out.is_empty() {
        out.push(finding(
            CHECK,
            Outcome::Verified,
            None,
            if seen {
                "grounds_for の名前は、どれも outputs の name にある".to_string()
            } else {
                "grounds_for を持つ資料が無い".to_string()
            },
        ));
    }
    out
}

/// Skill のフォルダからの経路が、フォルダの外を指すか（/ で始まる ・ .. を含む ・ 空）。
fn outside(rel: &str) -> bool {
    rel.is_empty() || rel.starts_with('/') || rel.split('/').any(|p| p == "..")
}

/// 参照する資料と出力のうち schema を持つ件ごとに、schema があるか、JSON の資料がそれに従うか。
fn json_schema_conforms(
    files: &dyn SkillFiles,
    sas: &dyn SkillValidator,
    schemas: &dyn JsonSchemaChecker,
    folder: &str,
) -> Vec<Finding> {
    const CHECK: &str = "json-schema-conforms";
    let skill = match read_skill(files, folder) {
        Ok(v) => v,
        Err(r) => return vec![finding(CHECK, Outcome::Error, None, r.message)],
    };
    let dir = trimmed(folder);
    let mut out = Vec::new();
    for (field, label) in [("references", "資料"), ("outputs", "出力")] {
        for item in skill
            .get(field)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(schema) = item.get("schema").and_then(Value::as_str) else {
                continue;
            };
            let name = item.get("name").and_then(Value::as_str).unwrap_or_default();
            let is_json = item
                .get("form")
                .and_then(Value::as_str)
                .is_some_and(|f| f.starts_with("JSON"));
            let content = (field == "references" && is_json)
                .then(|| item.get("path").and_then(Value::as_str))
                .flatten()
                .filter(|p| !outside(p))
                .map(|p| format!("{dir}/{p}"));
            out.push(conforms(
                files, sas, schemas, dir, label, name, schema, content,
            ));
        }
    }
    if out.is_empty() {
        out.push(finding(
            CHECK,
            Outcome::Verified,
            None,
            "schema を持つ資料と出力が無い".to_string(),
        ));
    }
    out
}

/// 1件の schema と、（JSON の資料なら）その中身を確かめる。content は資料の経路（出力 ・ JSON でない資料は None）。
#[allow(clippy::too_many_arguments)]
fn conforms(
    files: &dyn SkillFiles,
    sas: &dyn SkillValidator,
    schemas: &dyn JsonSchemaChecker,
    dir: &str,
    label: &str,
    name: &str,
    schema: &str,
    content: Option<String>,
) -> Finding {
    const CHECK: &str = "json-schema-conforms";
    let bad = |path: String, reason: String| finding(CHECK, Outcome::Violated, Some(path), reason);
    let error = |reason: String| finding(CHECK, Outcome::Error, None, reason);
    if schema.starts_with("urn:") {
        // SAS の Concrete：SAS が引けるか。資料なら、資料の $schema がその URN で、sas validate が通るか
        match sas.knows_concrete(schema) {
            Ok(true) => {}
            Ok(false) => {
                return bad(
                    format!("{dir}/skill.json"),
                    format!(
                        "{label} {name} の schema {schema} を、SAS が打った場所の中の Concrete として引けない"
                    ),
                );
            }
            Err(e) => return error(e),
        }
        let Some(path) = content else {
            return finding(
                CHECK,
                Outcome::Verified,
                None,
                format!("{label} {name} の schema {schema} を SAS が引ける"),
            );
        };
        let declared = files
            .read(&path)
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .and_then(|v| {
                v.get("$schema")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            });
        if declared.as_deref() != Some(schema) {
            return bad(
                path,
                format!(
                    "資料 {name} の $schema（{}）が、schema {schema} と違う",
                    declared.unwrap_or_default()
                ),
            );
        }
        return match sas.validate(&path) {
            Ok(r) if r.exit == 0 => finding(
                CHECK,
                Outcome::Verified,
                None,
                format!("資料 {name} が {schema} に従う（sas validate）"),
            ),
            Ok(r) => bad(
                path,
                format!(
                    "資料 {name} が {schema} に従わない（sas validate の終了コード {}：{}）",
                    r.exit,
                    summary(&r.output)
                ),
            ),
            Err(e) => error(e),
        };
    }
    // Skill のフォルダに置いた、ふつうの JSON Schema
    let schema_path = format!("{dir}/{schema}");
    if outside(schema) {
        return bad(
            schema_path,
            format!("{label} {name} の schema {schema} が、Skill のフォルダの外を指す"),
        );
    }
    let Ok(text) = files.read(&schema_path) else {
        return bad(
            schema_path.clone(),
            format!("{label} {name} の schema が {schema_path} に無い"),
        );
    };
    let Some(path) = content else {
        return finding(
            CHECK,
            Outcome::Verified,
            None,
            format!("{label} {name} の schema が {schema_path} にある"),
        );
    };
    let schema_value: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            return bad(
                schema_path,
                format!("schema {schema} が JSON として読めない：{e}"),
            );
        }
    };
    let value: Value = match files.read(&path).map(|t| serde_json::from_str(&t)) {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => return bad(path, format!("資料 {name} が JSON として読めない：{e}")),
        Err(e) => return bad(path, format!("資料 {name} を読めない：{e}")),
    };
    match schemas.check(&schema_value, &value) {
        Ok(broken) if broken.is_empty() => finding(
            CHECK,
            Outcome::Verified,
            None,
            format!("資料 {name} が {schema} に従う"),
        ),
        Ok(broken) => {
            let first: Vec<String> = broken
                .iter()
                .take(3)
                .map(|(at, why)| format!("{}：{why}", if at.is_empty() { "/" } else { at }))
                .collect();
            bad(
                path,
                format!(
                    "資料 {name} が {schema} に従わない（{} か所。{}）",
                    broken.len(),
                    first.join(" ／ ")
                ),
            )
        }
        Err(e) => bad(schema_path, e),
    }
}

/// sas の出力の要点（断ったなら code と message、検証なら件数）。
fn summary(output: &Value) -> String {
    match output.get("kind").and_then(Value::as_str) {
        Some("error") => format!(
            "{} {}",
            output
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or_default(),
            output
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default()
        ),
        _ => output
            .get("counts")
            .map(Value::to_string)
            .unwrap_or_default(),
    }
}

/// 前の確かめ（SAS の記録の validate の操作）から、その文書に書いた操作と、操作ごとの理由と所ごとの前と後。
/// 差分は SAS が記録したもの（sas log --operation の diff.changes）をそのまま使う。記録を読めなければ None。
fn changes_since_last_validation(history: &dyn SkillHistory, path: &str) -> Option<Value> {
    let log = history.log(path).ok()?;
    if log.exit != 0 {
        return None;
    }
    let rows: Vec<Value> = match log.output {
        Value::Array(rows) => rows,
        Value::Object(_) => vec![log.output],
        _ => Vec::new(),
    };
    let mut since = Value::Null;
    let mut operations = Vec::new();
    // sas log は新しい順。最初に出会う validate が前の確かめ
    for row in &rows {
        let op = row.get("op").and_then(Value::as_str).unwrap_or_default();
        let id = row
            .get("operation")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if op == "validate" {
            since = Value::String(id.to_string());
            break;
        }
        let detail = history
            .operation(id)
            .ok()
            .map(|r| r.output)
            .unwrap_or_default();
        let mut entry = serde_json::json!({
            "operation": id,
            "op": op,
            "at": row.get("at").cloned().unwrap_or_default(),
            "changes": detail.pointer("/diff/changes").cloned().unwrap_or_else(|| Value::Array(Vec::new())),
        });
        if let Some(reason) = row.get("reason") {
            entry["reason"] = reason.clone();
        }
        operations.push(entry);
    }
    Some(serde_json::json!({"since": since, "operations": operations}))
}

/// UC4 確かめる（Skill だけ）：フォルダごとに creator の検査7つと SAS の検証をして、承認の材料と一緒に渡した順に返す。
/// Skill のファイルは変えない。
///
/// # Errors
/// フォルダが1つも無い（usage）・ sas を呼べない（sas_unavailable）とき。
pub fn validate(
    folders: &[String],
    sas: &dyn SkillValidator,
    history: &dyn SkillHistory,
    files: &dyn SkillFiles,
    schemas: &dyn JsonSchemaChecker,
) -> Result<Checked, Refused> {
    if folders.is_empty() {
        return Err(Refused {
            code: "usage",
            message: "確かめる Skill のフォルダを1つ以上渡してください".to_string(),
        });
    }
    let mut skills = Vec::new();
    for folder in folders {
        let mut creator = name_checks(files, folder);
        creator.extend(links_resolve(files, folder));
        creator.push(skill_md_built(files, folder));
        creator.extend(references_exist(files, folder));
        creator.extend(json_schema_conforms(files, sas, schemas, folder));
        creator.extend(grounds_are_outputs(files, folder));
        // 前の確かめから変わった所は、この確かめを記録する前に読む
        let changes =
            changes_since_last_validation(history, &format!("{}/skill.json", trimmed(folder)));
        let run = sas
            .validate(&format!("{}/skill.json", trimmed(folder)))
            .map_err(|message| Refused {
                code: "sas_unavailable",
                message,
            })?;
        let skill = read_skill(files, folder).ok();
        skills.push(SkillChecked {
            folder: folder.clone(),
            sas: run,
            creator,
            approval: skill.map(|v| approval(folder, &v)),
            changes,
        });
    }
    Ok(Checked { skills })
}
