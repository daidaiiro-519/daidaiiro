//! UC1 作る（new ・ build ・ confirm）の手続きを、ポートの代役で確かめる。

#![allow(clippy::unwrap_used, clippy::panic)]

use serde_json::{Value, json};
use skills_application::ports::{GeneratedFiles, SkillFiles};
use skills_application::use_cases::build::build;
use skills_application::use_cases::confirm::confirm;
use skills_application::use_cases::new::new_procedure;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// 経路 → 中身のファイルを持ち、書いたものも同じ所に置く代役。
#[derive(Default)]
struct Files(RefCell<BTreeMap<String, String>>);

impl Files {
    fn with(path: &str, v: &Value) -> Self {
        let f = Self::default();
        f.0.borrow_mut().insert(path.to_string(), v.to_string());
        f
    }
}

impl SkillFiles for Files {
    fn exists(&self, path: &str) -> bool {
        self.0.borrow().contains_key(path)
    }
    fn read(&self, path: &str) -> Result<String, String> {
        self.0
            .borrow()
            .get(path)
            .cloned()
            .ok_or_else(|| format!("{path} が無い"))
    }
    fn siblings(&self, _: &str) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
    fn dangling_links(&self, _: &str) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
}

impl GeneratedFiles for Files {
    fn write(&self, path: &str, text: &str) -> Result<(), String> {
        self.0
            .borrow_mut()
            .insert(path.to_string(), text.to_string());
        Ok(())
    }
}

fn skill() -> Value {
    json!({
        "$schema": "urn:agents-skills-creator:skill:v1",
        "name": "s",
        "purpose": {"who": "人", "when": "書くとき", "gets": "分かった状態"},
        "role": {"does": ["照合する"], "does_not": [
            {"what": "書き換える", "from": "問3 の答え"},
            {"what": "要約する", "from": "AI が足した"}]},
        "description": "照合する",
        "steps": [{"id": 1, "do": "取る", "completion": "ある"}],
        "rules": [{"rule": "言い換えない", "why": "別のキーになる", "bad": "x", "examples": ["E1"]}],
        "minimal_guarantees": ["外は変えない"],
        "completion_criteria": [{"id": "C1", "criterion": "どれにも印", "check": "件数", "from": "問2 の答え"}],
        "examples": [{"id": "E1", "caller": "人が直接", "given": "一覧がある", "input": "照合して", "expected": "印が付く",
                      "behavior": {"who": "書く人", "does": "照合する", "then": "書ける"}}]
    })
}

#[test]
fn build_writes_once_and_then_finds_nothing_to_change() {
    let files = Files::with("skills/s/skill.json", &skill());
    let first = build(Some("skills/s"), &files, &files).unwrap();
    assert!(first.changed);
    assert!(
        files
            .read("skills/s/SKILL.md")
            .unwrap()
            .starts_with("---\nname: s\n")
    );
    let second = build(Some("skills/s"), &files, &files).unwrap();
    assert!(!second.changed);
    assert_eq!(build(None, &files, &files).unwrap_err().code, "usage");
    assert_eq!(
        build(Some("skills/none"), &files, &files).unwrap_err().code,
        "not_found"
    );
}

#[test]
fn guesses_are_everything_but_the_answers_in_the_order_of_loss() {
    let files = Files::with("skills/s/skill.json", &skill());
    let c = confirm(Some("skills/s"), &files).unwrap();
    let fields: Vec<&str> = c.guesses.iter().map(|g| g.field.as_str()).collect();
    // 受け持たないことは、答え（問3）から取った件を除く。そのあと 最低保証 → rules → 完了条件 → 手順 → そのほか
    assert_eq!(
        &fields[..6],
        [
            "/role/does_not/1",
            "/minimal_guarantees/0",
            "/rules/0",
            "/completion_criteria/0",
            "/steps/0",
            "/name"
        ]
    );
    assert!(!fields.iter().any(|f| f.starts_with("/purpose")));
    let from: Vec<&str> = c.guesses.iter().map(|g| g.from.as_str()).collect();
    assert_eq!(from[1], "outputs の置き場所から");
    assert_eq!(from[2], "例 E1 から");
    assert_eq!(from[3], "問2 の答え");
    assert!(c.not_approvable.is_empty());
    assert_eq!(c.behaviors[0].1[0], "書く人");
}

#[test]
fn an_unknown_purpose_comes_first_and_cannot_be_approved() {
    let mut v = skill();
    v["purpose"]["gets"] = json!("分からない");
    let files = Files::with("skills/s/skill.json", &v);
    let c = confirm(Some("skills/s"), &files).unwrap();
    assert_eq!(c.guesses[0].field, "/purpose/gets");
    assert!(c.guesses[0].unknown);
    assert_eq!(c.not_approvable.len(), 1);
}

#[test]
fn the_procedure_copies_the_rules_of_the_concrete_and_answers_only_purpose_and_role() {
    let p = new_procedure(Some("skills/"));
    let compose = p["compose"].as_array().unwrap();
    let answered: Vec<&str> = compose
        .iter()
        .filter(|f| f["source"] == "answer")
        .map(|f| f["field"].as_str().unwrap())
        .collect();
    assert_eq!(answered, ["/purpose", "/role"]);
    let purpose = compose.iter().find(|f| f["field"] == "/purpose").unwrap();
    assert_eq!(purpose["rules"][0]["id"], json!("purpose-standalone"));
    assert_eq!(
        p["then"][0],
        json!("agents-skills-creator build skills/<name>")
    );
    assert_eq!(
        new_procedure(None)["write"]["command"],
        json!(
            "sas create <Skill の置き場所>/<name>/skill.json --concrete urn:agents-skills-creator:skill:v1 --value <JSON>"
        )
    );
}
