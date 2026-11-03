//! UC2 Agent を作るの validate（UC4 確かめるの Agent の側）：渡した Agent のフォルダごとに、SAS の検証の結果（書き換えない）と、
//! creator の検査の結果と、承認の材料を返す。確かめても Agent のファイルは変えない。
//!
//! creator の検査：
//! - name-is-folder：agent.json の name がフォルダ名と同じ
//! - skills-linked：skills の各名前が、配下の skills/ に同じ名前のリンクとしてあり、その先が Skill として agents-skills-creator skills validate を
//!   通る（文脈 skills の公開された言葉で確かめる）。skills/ にあって skills に無いものも violated
//! - calls-are-linked：ステップが呼ぶ Skill が、skills にある
//! - passes-are-outputs：ステップが渡すものの名前が、そのステップで呼んだ Skill の outputs の name にある（outputs は
//!   agents-skills-creator skills validate の結果の承認の材料から読む。Skill の項目を直接読まない）
//! - agent-md-built：AGENT.md と codex.toml が、agent.json から write-agent-md で作るものと同じ
//! - examples-distinct：given と input が同じ例が2つない（Skill と同じ見方）
//!
//! 承認の材料は、SAS の検証で検出があるか、意味の判定まで済んだ確かめ（最後に書いたあとの、not-judged ・ violated ・
//! error が0の validate の操作）が記録に無ければ、承認を受け付けない（approvable が false。終了コードは変えない）。
//! 承認できるときだけ、そのまま打てる承認のコマンドを approve に出す。
//!
//! いまの中身のハッシュで最後に渡された判定（--answer の操作）を SAS の記録から読んで approval.judgments に重ね、判定が
//! そろっていれば delegate を出さない。approved が判定まで済んだ確かめを指し、そのあと人が承認を書いたほかに書き換えて
//! いなければ approval.approved_by にその操作 ID を出す（Skill と同じ見方）。

use crate::ports::{AgentFiles, AgentHistory, AgentValidator, Run, SkillsCli};
use crate::use_cases::approval::approval;
use crate::use_cases::{
    AGENT_JSON, Finding, Outcome, Refused, finding, read_agent, strings, trimmed,
};
use agents_domain::services::agent_md;
use serde_json::Value;
use std::collections::BTreeMap;

/// Agent のフォルダ1つの結果。
#[derive(Debug, Clone, PartialEq)]
pub struct AgentChecked {
    /// 渡されたフォルダ（渡されたまま）
    pub folder: String,
    /// SAS の検証の結果（書き換えない）
    pub sas: Run,
    /// creator の検査の結果（検査の順）
    pub creator: Vec<Finding>,
    /// 承認の材料（agent.json を読めたときだけ）
    pub approval: Option<Value>,
}

impl AgentChecked {
    /// SAS か creator のどちらかで検出があったか。
    #[must_use]
    pub fn detected(&self) -> bool {
        self.sas.exit != 0 || self.creator.iter().any(|f| f.outcome != Outcome::Verified)
    }
}

/// 確かめた結果（渡した順）。
#[derive(Debug, Clone, PartialEq)]
pub struct Checked {
    /// Agent ごとの結果
    pub agents: Vec<AgentChecked>,
}

impl Checked {
    /// どれかで検出があったか。
    #[must_use]
    pub fn detected(&self) -> bool {
        self.agents.iter().any(AgentChecked::detected)
    }
}

/// agents-skills-creator skills validate の結果の要点：Skill として通ったか、理由、outputs の名前。
fn skill_result(run: &Run) -> (bool, String, Vec<String>) {
    let one = run.output.pointer("/skills/0").cloned().unwrap_or_default();
    let outputs: Vec<String> = one
        .pointer("/approval/structure_image/scope/outputs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect();
    let failed: Vec<String> = one
        .get("creator")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|f| f.get("result").and_then(Value::as_str) != Some("verified"))
        .filter_map(|f| {
            f.get("check")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .collect();
    let sas_exit = one
        .get("semantic_agent_schema_exit")
        .and_then(Value::as_i64)
        .unwrap_or(-1);
    let ok = run.exit == 0 && failed.is_empty() && sas_exit == 0;
    let why = if ok {
        String::new()
    } else if one.is_null() {
        format!(
            "agents-skills-creator skills validate の結果を読めない：{}",
            run.output
        )
    } else {
        format!(
            "agents-skills-creator skills validate で検出（creator：{}、semantic-agent-schema の終了コード {sas_exit}）",
            failed.join(" ・ ")
        )
    };
    (ok, why, outputs)
}

/// 配下の Skill を確かめ、Skill の名前ごとの outputs を返す（Skill として通らなかったものは入れない）。
fn skills_linked(
    files: &dyn AgentFiles,
    cli: &dyn SkillsCli,
    folder: &str,
    agent: &Value,
    out: &mut Vec<Finding>,
) -> BTreeMap<String, Vec<String>> {
    const CHECK: &str = "skills-linked";
    let names = strings(agent, "skills");
    let dir = format!("{folder}/skills");
    let mut outputs = BTreeMap::new();
    let before = out.len();
    for name in &names {
        let at = format!("{dir}/{name}");
        let Some(target) = files.resolve(&at) else {
            out.push(finding(
                CHECK,
                Outcome::Violated,
                Some(at.clone()),
                format!("配下の skills/ に {name} のリンクが無いか、リンクの先が無い"),
            ));
            continue;
        };
        match cli.validate(&target) {
            Ok(run) => {
                let (ok, why, outs) = skill_result(&run);
                if ok {
                    outputs.insert(name.clone(), outs);
                } else {
                    out.push(finding(
                        CHECK,
                        Outcome::Violated,
                        Some(at.clone()),
                        format!("{name}（{target}）が Skill として通らない：{why}"),
                    ));
                }
            }
            Err(e) => out.push(finding(CHECK, Outcome::Error, Some(at.clone()), e)),
        }
    }
    for extra in files
        .entries(&dir)
        .into_iter()
        .filter(|e| !names.contains(e))
    {
        out.push(finding(
            CHECK,
            Outcome::Violated,
            Some(format!("{dir}/{extra}")),
            format!("配下の skills/ の {extra} が、agent.json の skills に無い"),
        ));
    }
    if out.len() == before {
        out.push(finding(
            CHECK,
            Outcome::Verified,
            None,
            format!(
                "配下の Skill {} 個が、どれもリンクの先にあり、Skill として通る",
                names.len()
            ),
        ));
    }
    outputs
}

/// ステップが呼ぶ Skill が skills にあるか、渡すものの名前が呼んだ Skill の outputs にあるか。
fn steps_checks(agent: &Value, outputs: &BTreeMap<String, Vec<String>>, out: &mut Vec<Finding>) {
    let names = strings(agent, "skills");
    let steps: Vec<&Value> = agent
        .get("steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .collect();
    let mut calls_bad = Vec::new();
    let mut passes = Vec::new();
    for st in &steps {
        let id = st.get("id").map(Value::to_string).unwrap_or_default();
        let calls = strings(st, "calls");
        for c in calls.iter().filter(|c| !names.contains(c)) {
            calls_bad.push(finding(
                "calls-are-linked",
                Outcome::Violated,
                Some(format!("steps/{id}")),
                format!("ステップ {id} が呼ぶ {c} が、skills に無い"),
            ));
        }
        for p in strings(st, "passes") {
            let known: Vec<&String> = calls.iter().filter(|c| outputs.contains_key(*c)).collect();
            if known.len() < calls.len() {
                passes.push(finding("passes-are-outputs", Outcome::Error, Some(format!("steps/{id}")), format!("ステップ {id} が呼ぶ Skill のうち、Skill として通らないものがあり、{p} を確かめられない")));
            } else if !calls
                .iter()
                .any(|c| outputs.get(c).is_some_and(|o| o.contains(&p)))
            {
                passes.push(finding(
                    "passes-are-outputs",
                    Outcome::Violated,
                    Some(format!("steps/{id}")),
                    format!(
                        "ステップ {id} が渡す {p} が、呼んだ Skill（{}）の outputs に無い",
                        calls.join(" ・ ")
                    ),
                ));
            }
        }
    }
    if calls_bad.is_empty() {
        out.push(finding(
            "calls-are-linked",
            Outcome::Verified,
            None,
            "ステップが呼ぶ Skill は、どれも skills にある".to_string(),
        ));
    }
    out.extend(calls_bad);
    if passes.is_empty() {
        out.push(finding(
            "passes-are-outputs",
            Outcome::Verified,
            None,
            "ステップが渡すものは、どれも呼んだ Skill の outputs にある".to_string(),
        ));
    }
    out.extend(passes);
}

fn agent_md_built(files: &dyn AgentFiles, folder: &str, agent: &Value) -> Finding {
    const CHECK: &str = "agent-md-built";
    for (name, text) in [
        (agent_md::FILE, agent_md::render(agent, folder)),
        (agent_md::CODEX_FILE, agent_md::render_codex(agent, folder)),
    ] {
        let path = format!("{folder}/{name}");
        match files.read(&path) {
            Ok(t) if t == text => {}
            Ok(_) => {
                return finding(
                    CHECK,
                    Outcome::Violated,
                    Some(path),
                    format!(
                        "{name} が、agent.json から write-agent-md で作るものと違う（agents-skills-creator agents write-agent-md {folder} で作り直す）"
                    ),
                );
            }
            Err(_) => {
                return finding(
                    CHECK,
                    Outcome::Violated,
                    Some(path),
                    format!(
                        "{name} が無い（agents-skills-creator agents write-agent-md {folder} で作る）"
                    ),
                );
            }
        }
    }
    finding(
        CHECK,
        Outcome::Verified,
        None,
        "AGENT.md と codex.toml が、agent.json から write-agent-md で作るものと同じ".to_string(),
    )
}

/// 意味の判定まで済んだ確かめ：SAS の記録で、最後に書いたあとの validate の操作のうち、not-judged ・ violated ・ error が
/// どれも 0 のもの（semantic-agent-schema validate --answer で判定を渡した確かめ）の ID。
pub(crate) fn judged_validation(history: &dyn AgentHistory, path: &str) -> Option<String> {
    let log = history.log(path).ok().filter(|r| r.exit == 0)?;
    let rows: Vec<Value> = match log.output {
        Value::Array(rows) => rows,
        other @ Value::Object(_) => vec![other],
        _ => Vec::new(),
    };
    for row in &rows {
        match row.get("op").and_then(Value::as_str) {
            Some("create" | "update" | "delete") => return None,
            Some("validate") => {
                let n = |k: &str| row.pointer(&format!("/counts/{k}")).and_then(Value::as_u64);
                if n("not-judged") == Some(0) && n("violated") == Some(0) && n("error") == Some(0) {
                    return row
                        .get("operation")
                        .and_then(Value::as_str)
                        .map(ToString::to_string);
                }
            }
            _ => {}
        }
    }
    None
}

/// SAS の検証の結果のうち、判定を渡していない（not-judged）件数。
fn count_not_judged(output: &Value) -> usize {
    let one = |v: &Value| -> usize {
        v.get("targets")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .flat_map(|t| {
                t.get("results")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
            })
            .filter(|r| r.get("result").and_then(Value::as_str) == Some("not-judged"))
            .count()
    };
    match output {
        Value::Array(rows) => rows.iter().map(one).sum(),
        other => one(other),
    }
}

/// given と input が同じ例が2つないか（後に出た例の所と、重なった相手の id を出す）。
fn examples_distinct(agent: &Value) -> Finding {
    const CHECK: &str = "examples-distinct";
    let examples = agent
        .get("examples")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    let text = |e: &Value, k: &str| {
        e.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    for (i, e) in examples.iter().enumerate() {
        if let Some(same) = examples[..i]
            .iter()
            .find(|o| text(o, "given") == text(e, "given") && text(o, "input") == text(e, "input"))
        {
            return finding(
                CHECK,
                Outcome::Violated,
                Some(format!("/examples/{i}")),
                format!(
                    "例 {} の given と input が {} と同じ。足さずに {} の expected ・ covers を直し、結び先を変える",
                    text(e, "id"),
                    text(same, "id"),
                    text(same, "id")
                ),
            );
        }
    }
    finding(
        CHECK,
        Outcome::Verified,
        None,
        format!(
            "例 {} 個の given と input が、どれも重ならない",
            examples.len()
        ),
    )
}

/// SAS の記録の行（semantic-agent-schema log。新しい順）。読めなければ None。
fn log_rows(history: &dyn AgentHistory, path: &str) -> Option<Vec<Value>> {
    let log = history.log(path).ok().filter(|r| r.exit == 0)?;
    Some(match log.output {
        Value::Array(rows) => rows,
        other @ Value::Object(_) => vec![other],
        _ => Vec::new(),
    })
}

/// 承認より新しい行（新しい順）のうち、人が /approved だけを書いた update でない、最初の書き換え。
pub(crate) fn write_after_approval<'a>(
    history: &dyn AgentHistory,
    newer: &'a [Value],
) -> Option<&'a Value> {
    newer.iter().find(|row| {
        let op = row.get("op").and_then(Value::as_str).unwrap_or_default();
        if !matches!(op, "create" | "update" | "delete") {
            return false;
        }
        let id = row
            .get("operation")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let only_approval = op == "update"
            && row.get("session").is_none_or(Value::is_null)
            && history.operation(id).ok().is_some_and(|d| {
                d.output
                    .pointer("/diff/changes")
                    .and_then(Value::as_array)
                    .is_some_and(|c| {
                        !c.is_empty()
                            && c.iter().all(|x| {
                                x.get("pointer").and_then(Value::as_str) == Some("/approved")
                            })
                    })
            });
        !only_approval
    })
}

/// 承認済みなら、承認した確かめの操作 ID（Skill と同じ見方）。
fn approved_by(history: &dyn AgentHistory, path: &str, agent: Option<&Value>) -> Option<String> {
    let approved = agent?.get("approved").and_then(Value::as_str)?;
    let rows = log_rows(history, path)?;
    let at = rows.iter().position(|r| {
        r.get("operation").and_then(Value::as_str) == Some(approved)
            && r.get("op").and_then(Value::as_str) == Some("validate")
    })?;
    let n = |k: &str| {
        rows[at]
            .pointer(&format!("/counts/{k}"))
            .and_then(Value::as_u64)
    };
    let judged = n("not-judged") == Some(0) && n("violated") == Some(0) && n("error") == Some(0);
    (judged && write_after_approval(history, &rows[..at]).is_none()).then(|| approved.to_string())
}

/// いまの中身のハッシュで最後に渡された意味の判定（Skill と同じ見方）。この validate を記録する前に読む。
fn latest_judgments(history: &dyn AgentHistory, path: &str, hash: &str) -> Option<Value> {
    if hash.is_empty() {
        return None;
    }
    for row in log_rows(history, path)? {
        match row.get("op").and_then(Value::as_str) {
            Some("create" | "update" | "delete") => return None,
            Some("validate") => {}
            _ => continue,
        }
        let id = row
            .get("operation")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let Ok(detail) = history.operation(id) else {
            continue;
        };
        let Some(target) = detail.output.pointer("/targets/0") else {
            continue;
        };
        if target.get("hash").and_then(Value::as_str) != Some(hash) {
            continue;
        }
        let results: Vec<Value> = target
            .get("results")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|r| r.get("by").and_then(Value::as_str) == Some("ai"))
            .map(|r| {
                serde_json::json!({
                    "rule": r.get("rule").cloned().unwrap_or_default(),
                    "pointer": r.get("pointer").cloned().unwrap_or_default(),
                    "result": r.get("result").cloned().unwrap_or_default(),
                    "reason": r.get("reason").cloned().unwrap_or_default(),
                })
            })
            .collect();
        if results
            .iter()
            .any(|r| r.get("result").and_then(Value::as_str) != Some("not-judged"))
        {
            return Some(serde_json::json!({"operation": id, "results": results}));
        }
    }
    None
}

/// 判定の結果のうち、result が kind の件数。
fn count_judged(judgments: &Value, kind: &str) -> usize {
    judgments
        .get("results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|r| r.get("result").and_then(Value::as_str) == Some(kind))
        .count()
}

/// Agent のフォルダごとに確かめる。
///
/// # Errors
/// フォルダが1つも無い（usage）・ semantic-agent-schema を呼べない（semantic_agent_schema_unavailable）とき。
pub fn validate(
    folders: &[String],
    sas: &dyn AgentValidator,
    history: &dyn AgentHistory,
    cli: &dyn SkillsCli,
    files: &dyn AgentFiles,
) -> Result<Checked, Refused> {
    if folders.is_empty() {
        return Err(Refused {
            code: "usage",
            message: "確かめる Agent のフォルダを1つ以上渡してください".to_string(),
        });
    }
    let mut agents = Vec::new();
    for given in folders {
        let folder = trimmed(given);
        let mut creator = Vec::new();
        let agent = read_agent(files, folder);
        match &agent {
            Ok(agent) => {
                let name = agent
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let dir = folder.rsplit('/').next().unwrap_or(folder);
                creator.push(if name == dir {
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
                });
                let outputs = skills_linked(files, cli, folder, agent, &mut creator);
                steps_checks(agent, &outputs, &mut creator);
                creator.push(agent_md_built(files, folder, agent));
                creator.push(examples_distinct(agent));
            }
            Err(r) => creator.push(finding(
                "name-is-folder",
                Outcome::Error,
                None,
                r.message.clone(),
            )),
        }
        let path = format!("{folder}/{AGENT_JSON}");
        // 意味の判定まで済んだ確かめと、最後に渡された判定は、この確かめを記録する前に読む
        let judged = judged_validation(history, &path);
        let run = sas.validate(&path).map_err(|message| Refused {
            code: "semantic_agent_schema_unavailable",
            message,
        })?;
        let hash = run
            .output
            .pointer("/targets/0/hash")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let judgments = latest_judgments(history, &path, &hash);
        let approved = approved_by(history, &path, agent.as_ref().ok());
        let approval = agent.ok().map(|a| {
            let mut ap = approval(folder, &a);
            let mut not = Vec::new();
            if run.exit != 0 {
                not.push(format!("SAS の検証で検出がある（semantic-agent-schema validate の終了コード {}）", run.exit));
            }
            if judged.is_none() && approved.is_none() {
                not.push(match &judgments {
                    Some(j) if count_judged(j, "not-judged") == 0 => format!(
                        "意味の判定で violated が {} 件 ・ error が {} 件ある（approval.judgments の {}）。直して（update）判定をし直し、semantic-agent-schema validate --answer で渡す",
                        count_judged(j, "violated"),
                        count_judged(j, "error"),
                        j.get("operation").and_then(Value::as_str).unwrap_or_default()
                    ),
                    _ => format!("意味の判定が済んでいない（not-judged が {} 件。approval の delegate の a ・ b で判定して semantic-agent-schema validate --answer で渡す）", count_not_judged(&run.output)),
                });
            }
            ap["approvable"] = Value::Bool(not.is_empty());
            // 判定がそろったか承認済みなら、別の会話に任せる手順は出さない
            let complete = judgments
                .as_ref()
                .is_some_and(|j| count_judged(j, "not-judged") == 0);
            if (complete || approved.is_some())
                && let Some(o) = ap.as_object_mut()
            {
                o.remove("delegate");
            }
            // 承認のコマンドは、承認できて、まだ承認していないときだけ出す
            if not.is_empty() && judged.is_some() && approved.is_none() {
                ap["approve"] = serde_json::json!({
                    "command": format!("agents-skills-creator approve {folder}"),
                    "how": "作る人が承認の材料を見て承認したら、人が自分の端末でこのコマンドを打つ。判定まで済んだ確かめの操作 ID とハッシュは approve が引く（AI のセッションから打つと、SAS が human_field で断る）",
                });
            }
            if !not.is_empty() {
                ap["not_approvable"] = serde_json::json!(not);
            }
            if let Some(j) = &judgments {
                ap["judgments"] = j.clone();
            }
            if let Some(op) = &approved {
                ap["approved_by"] = serde_json::json!(op);
            }
            ap
        });
        agents.push(AgentChecked {
            folder: given.clone(),
            sas: run,
            creator,
            approval,
        });
    }
    Ok(Checked { agents })
}
