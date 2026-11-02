//! 結果を JSON 1つにする（published-language/v1/check-result.schema.json）。SAS の結果は書き換えずに入れる。

use serde_json::{Value, json};
use skills_application::use_cases::check::{Checked, Finding, Refused};

const SCHEMA: &str = "urn:agents-skills-creator:skills:check-result:v1";

fn finding(f: &Finding) -> Value {
    let mut v = json!({"check": f.check, "result": f.outcome.as_str()});
    if let Some(p) = &f.path {
        v["path"] = json!(p);
    }
    v["reason"] = json!(f.reason);
    v
}

/// 確かめた結果を出す。
pub fn print_checked(c: &Checked) {
    let skills: Vec<Value> = c
        .skills
        .iter()
        .map(|s| {
            json!({
                "folder": s.folder,
                "sas": s.sas.output,
                "sas_exit": s.sas.exit,
                "creator": s.creator.iter().map(finding).collect::<Vec<_>>(),
            })
        })
        .collect();
    println!(
        "{}",
        json!({"$schema": SCHEMA, "kind": "check", "skills": skills, "detected": c.detected()})
    );
}

/// 始められなかった理由を出す。
pub fn print_refused(r: &Refused) {
    println!(
        "{}",
        json!({"$schema": SCHEMA, "kind": "error", "code": r.code, "message": r.message})
    );
}
