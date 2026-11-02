//! 結果を JSON 1つにする（published-language/v1 の check-result ・ build-result ・ confirm ・ new-procedure）。
//! SAS の結果は書き換えずに入れる。

use serde_json::{Value, json};
use skills_application::use_cases::Refused;
use skills_application::use_cases::build::Built;
use skills_application::use_cases::check::{Checked, Finding};
use skills_application::use_cases::confirm::Confirmation;

/// check の結果の形（使い方の誤りもこの形で返す）。
pub const CHECK: &str = "urn:agents-skills-creator:skills:check-result:v1";
/// build の結果の形。
pub const BUILD: &str = "urn:agents-skills-creator:skills:build-result:v1";
/// confirm の結果の形。
pub const CONFIRM: &str = "urn:agents-skills-creator:skills:confirm:v1";

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
        json!({"$schema": CHECK, "kind": "check", "skills": skills, "detected": c.detected()})
    );
}

/// 始められなかった理由を、schema の形で出す。
pub fn print_refused(schema: &str, r: &Refused) {
    println!(
        "{}",
        json!({"$schema": schema, "kind": "error", "code": r.code, "message": r.message})
    );
}

/// SKILL.md を作った結果を出す。
pub fn print_built(b: &Built) {
    println!(
        "{}",
        json!({"$schema": BUILD, "kind": "build", "folder": b.folder, "path": b.path, "changed": b.changed})
    );
}

/// 手順を出す。
pub fn print_procedure(p: &Value) {
    println!("{p}");
}

/// 確認の1画面の材料を出す。
pub fn print_confirmation(c: &Confirmation) {
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
        "$schema": CONFIRM,
        "kind": "confirm",
        "folder": c.folder,
        "name": c.name,
        "description": c.description,
        "images": c.images.iter().map(|(id, e)| json!({"example": id, "expected": e})).collect::<Vec<_>>(),
        "behaviors": c.behaviors.iter().map(|(id, [who, does, then])| json!({"example": id, "who": who, "does": does, "then": then})).collect::<Vec<_>>(),
        "structure": structure,
        "guesses": c.guesses.iter().map(|g| json!({"field": g.field, "content": g.content, "from": g.from, "unknown": g.unknown})).collect::<Vec<_>>(),
        "approvable": c.not_approvable.is_empty(),
        "delegate": delegate,
    });
    if !c.not_approvable.is_empty() {
        out["not_approvable"] = json!(c.not_approvable);
    }
    println!("{out}");
}
