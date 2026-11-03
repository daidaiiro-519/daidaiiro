//! UC2 Agent を作るの validate（UC4 確かめるの Agent の側）：渡した Agent のフォルダごとに、SAS の検証の結果（書き換えない）と、
//! creator の検査の結果と、承認の材料を返す。確かめても Agent のファイルは変えない。
//!
//! creator の検査：
//! - name-is-folder：agent.json の name がフォルダ名と同じ
//! - skills-linked：skills の各名前が、配下の skills/ に同じ名前のリンクとしてあり、その先が Skill として skills validate を
//!   通る（文脈 skills の公開された言葉で確かめる）。skills/ にあって skills に無いものも violated
//! - calls-are-linked：ステップが呼ぶ Skill が、skills にある
//! - passes-are-outputs：ステップが渡すものの名前が、そのステップで呼んだ Skill の outputs の name にある（outputs は
//!   skills validate の結果の承認の材料から読む。Skill の項目を直接読まない）
//! - agent-md-built：AGENT.md と codex.toml が、agent.json から write-agent-md で作るものと同じ
//!
//! 承認の材料は、SAS の検証で検出があるか、意味の rule の not-judged が1件でも残れば、承認を受け付けない（approvable が
//! false。終了コードは変えない）。そのまま打てる承認のコマンド（この validate の操作 ID と、いまの中身のハッシュ入り）を
//! approve に出す。

use crate::ports::{AgentFiles, AgentValidator, Run, SkillsCli};
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

/// skills validate の結果の要点：Skill として通ったか、理由、outputs の名前。
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
    let sas_exit = one.get("sas_exit").and_then(Value::as_i64).unwrap_or(-1);
    let ok = run.exit == 0 && failed.is_empty() && sas_exit == 0;
    let why = if ok {
        String::new()
    } else if one.is_null() {
        format!("skills validate の結果を読めない：{}", run.output)
    } else {
        format!(
            "skills validate で検出（creator：{}、sas の終了コード {sas_exit}）",
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
                        "{name} が、agent.json から write-agent-md で作るものと違う（agents write-agent-md {folder} で作り直す）"
                    ),
                );
            }
            Err(_) => {
                return finding(
                    CHECK,
                    Outcome::Violated,
                    Some(path),
                    format!("{name} が無い（agents write-agent-md {folder} で作る）"),
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

/// Agent のフォルダごとに確かめる。
///
/// # Errors
/// フォルダが1つも無い（usage）・ sas を呼べない（sas_unavailable）とき。
pub fn validate(
    folders: &[String],
    sas: &dyn AgentValidator,
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
            }
            Err(r) => creator.push(finding(
                "name-is-folder",
                Outcome::Error,
                None,
                r.message.clone(),
            )),
        }
        let run = sas
            .validate(&format!("{folder}/{AGENT_JSON}"))
            .map_err(|message| Refused {
                code: "sas_unavailable",
                message,
            })?;
        let approval = agent.ok().map(|a| {
            let mut ap = approval(folder, &a);
            let mut not = Vec::new();
            if run.exit != 0 {
                not.push(format!("SAS の検証で検出がある（sas validate の終了コード {}）", run.exit));
            }
            let not_judged = count_not_judged(&run.output);
            if not_judged > 0 {
                not.push(format!("意味の判定が済んでいない（not-judged が {not_judged} 件。approval の delegate の a で判定して sas validate --answer で渡す）"));
            }
            ap["approvable"] = Value::Bool(not.is_empty());
            if !not.is_empty() {
                ap["not_approvable"] = serde_json::json!(not);
            }
            if let (Some(op), Some(hash)) = (
                run.output.pointer("/operations/0").and_then(Value::as_str),
                run.output.pointer("/targets/0/hash").and_then(Value::as_str),
            ) {
                let verb = if a.get("approved").is_some() { "replace" } else { "add" };
                ap["approve"] = serde_json::json!({
                    "command": format!("sas update {folder}/{AGENT_JSON} --patch '[{{\"op\":\"{verb}\",\"path\":\"/approved\",\"value\":\"{op}\"}}]' --if-match {hash}"),
                    "how": "作る人が承認の材料を見て承認したら、人が自分の端末でこのコマンドを打つ（AI のセッションから打つと、SAS が human_field で断る）",
                });
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
