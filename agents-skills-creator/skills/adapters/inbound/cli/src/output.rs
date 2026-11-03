//! 結果を JSON 1つにする（published-language/v1 の create-result ・ validate-result ・ write-skill-md-result）。
//! SAS の結果は書き換えずに入れる。

use serde_json::{Value, json};
use skills_application::use_cases::Refused;
use skills_application::use_cases::approval::Approval;
use skills_application::use_cases::create::Created;
use skills_application::use_cases::distribute::Distributed;
use skills_application::use_cases::update::Updated;
use skills_application::use_cases::validate::{Checked, Finding, SkillChecked};
use skills_application::use_cases::withdraw::Withdrawn;
use skills_application::use_cases::write_skill_md::Built;

/// create の結果の形。
pub const CREATE: &str = "urn:agents-skills-creator:skills:create-result:v1";
/// withdraw の結果の形。
pub const WITHDRAW: &str = "urn:agents-skills-creator:skills:withdraw-result:v1";
/// distribute の結果の形。
pub const DISTRIBUTE: &str = "urn:agents-skills-creator:skills:distribute-result:v1";
/// update の結果の形。
pub const UPDATE: &str = "urn:agents-skills-creator:skills:update-result:v1";
/// validate の結果の形（知らないコマンドの誤りもこの形で返す）。
pub const VALIDATE: &str = "urn:agents-skills-creator:skills:validate-result:v1";
/// write-skill-md の結果の形。
pub const WRITE_SKILL_MD: &str = "urn:agents-skills-creator:skills:write-skill-md-result:v1";

fn finding(f: &Finding) -> Value {
    let mut v = json!({"check": f.check, "result": f.outcome.as_str()});
    if let Some(p) = &f.path {
        v["path"] = json!(p);
    }
    v["reason"] = json!(f.reason);
    v
}

/// 確かめた結果を出す。
pub fn print_validated(c: &Checked) {
    let skills: Vec<Value> = c.skills.iter().map(skill).collect();
    println!(
        "{}",
        json!({"$schema": VALIDATE, "kind": "validate", "skills": skills, "detected": c.detected()})
    );
}

/// Skill 1つの確かめた結果。
fn skill(s: &SkillChecked) -> Value {
    let mut v = json!({
        "folder": s.folder,
        "sas": s.sas.output,
        "sas_exit": s.sas.exit,
        "creator": s.creator.iter().map(finding).collect::<Vec<_>>(),
    });
    if let Some(a) = &s.approval {
        v["approval"] = approval(a);
        if let Some(changes) = &s.changes {
            v["approval"]["changes"] = changes.clone();
        }
    }
    v
}

/// 配布を取り下げた結果を出す。
pub fn print_withdrawn(w: &Withdrawn) {
    let mut out = json!({
        "$schema": WITHDRAW,
        "kind": "withdraw",
        "folder": w.folder,
        "reason": w.reason,
        "places": w.places.iter().map(|p| json!({"path": p.path, "tool": p.tool, "scope": p.scope, "state": p.state})).collect::<Vec<_>>(),
        "removed": w.removed,
        "withdrawn": w.withdrawn,
    });
    if let Some(next) = &w.next {
        out["next"] = json!(next);
    }
    out["after"] = w.after.iter().map(finding).collect();
    println!("{out}");
}

/// 配布した結果を出す。
pub fn print_distributed(d: &Distributed, tool: &str, scope: &str) {
    let mut out = json!({
        "$schema": DISTRIBUTE,
        "kind": "distribute",
        "folder": d.before.folder,
        "tool": tool,
        "scope": scope,
        "before": skill(&d.before),
    });
    if let Some(p) = &d.placed {
        out["placed"] = json!({"path": p.path, "target": p.target, "result": p.result});
    }
    out["after"] = d.after.iter().map(finding).collect();
    out["detected"] = json!(d.detected());
    println!("{out}");
}

/// SAS に直させた結果を出す。
pub fn print_updated(u: &Updated) {
    let mut out = json!({"$schema": UPDATE, "kind": "update"});
    if let Some(flow) = &u.flow {
        out["flow"] = flow.clone();
    }
    out["sas"] = u.sas.output.clone();
    out["sas_exit"] = json!(u.sas.exit);
    println!("{out}");
}

/// SAS に書かせた結果を出す。
pub fn print_created(c: &Created) {
    let mut out = json!({"$schema": CREATE, "kind": "create"});
    if let Some(flow) = &c.flow {
        out["flow"] = flow.clone();
    }
    out["sas"] = c.sas.output.clone();
    out["sas_exit"] = json!(c.sas.exit);
    println!("{out}");
}

/// 始められなかった理由を、schema の形で出す。
pub fn print_refused(schema: &str, r: &Refused) {
    println!(
        "{}",
        json!({"$schema": schema, "kind": "error", "code": r.code, "message": r.message})
    );
}

/// SKILL.md を書いた結果を出す。
pub fn print_written(b: &Built) {
    println!(
        "{}",
        json!({"$schema": WRITE_SKILL_MD, "kind": "write-skill-md", "folder": b.folder, "path": b.path, "changed": b.changed})
    );
}

/// 承認の材料（確認の1画面の材料）を JSON にする。
fn approval(c: &Approval) -> Value {
    let structure: serde_json::Map<String, Value> = c
        .structure
        .iter()
        .map(|(k, v)| ((*k).to_string(), v.clone()))
        .collect();
    let delegate: Vec<Value> = c
        .delegate
        .iter()
        .map(|d| {
            let mut v = json!({"id": d.id, "what": d.what, "how": d.how});
            if let Some(cmd) = &d.command {
                v["command"] = json!(cmd);
            }
            if !d.examples.is_empty() {
                v["examples"] = d
                    .examples
                    .iter()
                    .map(|[id, given, input, expected]| {
                        json!({"id": id, "given": given, "input": input, "expected": expected})
                    })
                    .collect();
            }
            v
        })
        .collect();
    let mut out = json!({
        "name": c.name,
        "description": c.description,
        "images": c.images.iter().map(|(id, e)| json!({"example": id, "expected": e})).collect::<Vec<_>>(),
        "behaviors": c.behaviors.iter().map(|(id, [who, does, then])| json!({"example": id, "who": who, "does": does, "then": then})).collect::<Vec<_>>(),
        "structure": structure,
        "structure_image": c.structure_image,
        "guesses": c.guesses.iter().map(|g| json!({"field": g.field, "content": g.content, "from": g.from, "unknown": g.unknown})).collect::<Vec<_>>(),
        "approvable": c.not_approvable.is_empty(),
        "delegate": delegate,
    });
    if !c.not_approvable.is_empty() {
        out["not_approvable"] = json!(c.not_approvable);
    }
    out
}
