//! 結果を JSON 1つにする（published-language/v1 の create-result ・ validate-result ・ write-agent-md-result ・
//! distribute-result ・ withdraw-result）。Semantic Agent Schema と skills の結果は書き換えずに入れる。

use agents_application::use_cases::approve::Approved;
use agents_application::use_cases::create::Created;
use agents_application::use_cases::distribute::Distributed;
use agents_application::use_cases::update::Updated;
use agents_application::use_cases::validate::{AgentChecked, Checked};
use agents_application::use_cases::withdraw::Withdrawn;
use agents_application::use_cases::write_agent_md::Written;
use agents_application::use_cases::{Finding, Refused};
use serde_json::{Value, json};

/// approve の結果の形。
pub const APPROVE: &str = "urn:agents-skills-creator:agents:approve-result:v1";
/// create の結果の形。
pub const CREATE: &str = "urn:agents-skills-creator:agents:create-result:v1";
/// update の結果の形。
pub const UPDATE: &str = "urn:agents-skills-creator:agents:update-result:v1";
/// validate の結果の形（知らないコマンドの誤りもこの形で返す）。
pub const VALIDATE: &str = "urn:agents-skills-creator:agents:validate-result:v1";
/// write-agent-md の結果の形。
pub const WRITE_AGENT_MD: &str = "urn:agents-skills-creator:agents:write-agent-md-result:v1";
/// distribute の結果の形。
pub const DISTRIBUTE: &str = "urn:agents-skills-creator:agents:distribute-result:v1";
/// withdraw の結果の形。
pub const WITHDRAW: &str = "urn:agents-skills-creator:agents:withdraw-result:v1";

fn finding(f: &Finding) -> Value {
    let mut v = json!({"check": f.check, "result": f.outcome.as_str()});
    if let Some(p) = &f.path {
        v["path"] = json!(p);
    }
    v["reason"] = json!(f.reason);
    v
}

fn agent(a: &AgentChecked) -> Value {
    let mut v = json!({
        "folder": a.folder,
        "semantic_agent_schema": a.sas.output,
        "semantic_agent_schema_exit": a.sas.exit,
        "creator": a.creator.iter().map(finding).collect::<Vec<_>>(),
    });
    if let Some(ap) = &a.approval {
        v["approval"] = ap.clone();
    }
    v
}

/// 始められなかった理由を、schema の形で出す。
pub fn print_refused(schema: &str, r: &Refused) {
    println!(
        "{}",
        json!({"$schema": schema, "kind": "error", "code": r.code, "message": r.message})
    );
}

/// 書かせた結果を出す。
pub fn print_created(c: &Created) {
    let mut out = json!({"$schema": CREATE, "kind": "create"});
    if let Some(f) = &c.flow {
        out["flow"] = f.clone();
    }
    out["semantic_agent_schema"] = c.sas.output.clone();
    out["semantic_agent_schema_exit"] = json!(c.sas.exit);
    println!("{out}");
}

/// 直させた結果を出す。
pub fn print_updated(u: &Updated) {
    let mut out = json!({"$schema": UPDATE, "kind": "update"});
    if let Some(f) = &u.flow {
        out["flow"] = f.clone();
    }
    out["semantic_agent_schema"] = u.sas.output.clone();
    out["semantic_agent_schema_exit"] = json!(u.sas.exit);
    println!("{out}");
}

/// 確かめた結果を出す。
pub fn print_validated(c: &Checked) {
    println!(
        "{}",
        json!({"$schema": VALIDATE, "kind": "validate", "agents": c.agents.iter().map(agent).collect::<Vec<_>>(), "detected": c.detected()})
    );
}

/// 書いた結果を出す。
pub fn print_written(w: &Written) {
    println!(
        "{}",
        json!({"$schema": WRITE_AGENT_MD, "kind": "write-agent-md", "folder": w.folder,
               "files": w.files.iter().map(|(p, c)| json!({"path": p, "changed": c})).collect::<Vec<_>>()})
    );
}

/// 配布した結果を出す。
pub fn print_distributed(d: &Distributed, tool: &str, scope: &str) {
    let mut out = json!({"$schema": DISTRIBUTE, "kind": "distribute", "folder": d.before.folder, "tool": tool, "scope": scope, "before": agent(&d.before)});
    if let Some(p) = &d.placed {
        out["placed"] = json!({"path": p.path, "source": p.source, "result": p.result});
    }
    out["after"] = d.after.iter().map(finding).collect();
    out["detected"] = json!(d.detected());
    println!("{out}");
}

/// 取り下げた結果を出す。
pub fn print_withdrawn(w: &Withdrawn) {
    let mut out = json!({
        "$schema": WITHDRAW, "kind": "withdraw", "folder": w.folder, "reason": w.reason,
        "places": w.places.iter().map(|p| json!({"path": p.path, "tool": p.tool, "scope": p.scope, "state": p.state})).collect::<Vec<_>>(),
        "removed": w.removed, "withdrawn": w.withdrawn,
    });
    if let Some(n) = &w.next {
        out["next"] = json!(n);
    }
    out["after"] = w.after.iter().map(finding).collect();
    println!("{out}");
}

/// 承認を書かせた結果を出す。
pub fn print_approved(a: &Approved) {
    let mut out = json!({"$schema": APPROVE, "kind": "approve", "folder": a.folder, "approved": a.operation, "semantic_agent_schema": a.sas.output, "semantic_agent_schema_exit": a.sas.exit});
    if let Some(n) = &a.next {
        out["next"] = json!(n);
    }
    println!("{out}");
}
