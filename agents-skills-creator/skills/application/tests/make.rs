//! UC1 作る（create ・ write-skill-md ・ 承認の材料）の手続きを、ポートの代役で確かめる。

#![allow(clippy::unwrap_used, clippy::panic)]

use serde_json::{Value, json};
use skills_application::ports::{GeneratedFiles, SasRun, SkillFiles, SkillWriter};
use skills_application::use_cases::approval::{Approval, approval};
use skills_application::use_cases::create::{Mode, create};
use skills_application::use_cases::write_skill_md::write_skill_md;
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
            {"what": "書き換える", "from": "作る人の答え"},
            {"what": "要約する", "from": "AI が足した"}]},
        "description": "照合する",
        "steps": [{"id": 1, "do": "取る", "completion": "ある"}],
        "rules": [{"rule": "言い換えない", "why": "別のキーになる", "bad": "x", "examples": ["E1"]}],
        "minimal_guarantees": ["外は変えない"],
        "completion_criteria": [{"id": "C1", "criterion": "どれにも印", "check": "件数", "from": "目的の得るものから"}],
        "examples": [{"id": "E1", "caller": "人が直接", "given": "一覧がある", "input": "照合して", "expected": "印が付く",
                      "behavior": {"who": "書く人", "does": "照合する", "then": "書ける"}}]
    })
}

#[test]
fn build_writes_once_and_then_finds_nothing_to_change() {
    let files = Files::with("skills/s/skill.json", &skill());
    let first = write_skill_md(Some("skills/s"), &files, &files).unwrap();
    assert!(first.changed);
    assert!(
        files
            .read("skills/s/SKILL.md")
            .unwrap()
            .starts_with("---\nname: s\n")
    );
    let second = write_skill_md(Some("skills/s"), &files, &files).unwrap();
    assert!(!second.changed);
    assert_eq!(
        write_skill_md(None, &files, &files).unwrap_err().code,
        "usage"
    );
    assert_eq!(
        write_skill_md(Some("skills/none"), &files, &files)
            .unwrap_err()
            .code,
        "not_found"
    );
}

#[test]
fn guesses_are_everything_but_the_answers_in_the_order_of_loss() {
    let files = Files::with("skills/s/skill.json", &skill());
    let c = approve(&files);
    let fields: Vec<&str> = c.guesses.iter().map(|g| g.field.as_str()).collect();
    // 受け持たないことは、作る人の答えの件を除く。そのあと 最低保証 → rules → 完了条件 → 手順 → そのほか
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
    assert_eq!(from[3], "目的の得るものから");
    assert!(c.not_approvable.is_empty());
    assert_eq!(c.behaviors[0].1[0], "書く人");
}

#[test]
fn an_unknown_purpose_comes_first_and_cannot_be_approved() {
    let mut v = skill();
    v["purpose"]["gets"] = json!("分からない");
    let files = Files::with("skills/s/skill.json", &v);
    let c = approve(&files);
    assert_eq!(c.guesses[0].field, "/purpose/gets");
    assert!(c.guesses[0].unknown);
    assert_eq!(c.not_approvable.len(), 1);
}

#[test]
fn answered_references_are_not_guesses_and_the_reference_question_is_asked_only_when_needed() {
    let mut v = skill();
    v["references"] = json!([
        {"name": "判断基準", "path": "references/c.json", "holds": "h", "form": "JSON", "read_by": [1], "from": "作る人の答え"},
        {"name": "用語集", "path": "references/t.json", "holds": "h", "form": "JSON", "read_by": [1], "from": "AI が足した"}]);
    let files = Files::with("skills/s/skill.json", &v);
    let c = approve(&files);
    let fields: Vec<&str> = c.guesses.iter().map(|g| g.field.as_str()).collect();
    assert!(fields.contains(&"/references/1"));
    assert!(!fields.contains(&"/references/0"));
    // 完了条件は from が作る人の答えでなければ推測のまま
    assert!(fields.contains(&"/completion_criteria/0"));
}

fn approve(files: &Files) -> Approval {
    let v: Value = serde_json::from_str(&files.read("skills/s/skill.json").unwrap()).unwrap();
    approval("skills/s", &v)
}

/// sas create を打たずに、渡された引数を返す代役。
struct Writer;

impl SkillWriter for Writer {
    fn create_semantic(&self, path: &str, concrete: &str) -> Result<SasRun, String> {
        Ok(SasRun {
            exit: 0,
            output: json!({"kind": "procedure", "path": path, "schema": concrete}),
        })
    }
    fn create_value(&self, path: &str, concrete: &str, value: &str) -> Result<SasRun, String> {
        Ok(SasRun {
            exit: 0,
            output: json!({"kind": "change", "path": path, "schema": concrete, "value": value}),
        })
    }
}

#[test]
fn create_adds_the_flow_only_for_the_skill_concrete_and_passes_sas_through() {
    let skill = "urn:agents-skills-creator:skill:v1";
    let kind = "urn:agents-skills-creator:kind:advisor:v1";
    let c = create(Some("skills/"), Some(skill), Some(Mode::Semantic), &Writer).unwrap();
    assert_eq!(c.sas.output["path"], json!("skills/<name>/skill.json"));
    let f = c.flow.unwrap();
    assert_eq!(f["first_reply"].as_array().unwrap().len(), 4);
    assert_eq!(f["ask_only_when"].as_array().unwrap().len(), 2);
    let k = create(Some("skills"), Some(kind), Some(Mode::Semantic), &Writer).unwrap();
    assert!(k.flow.is_none());
    assert_eq!(k.sas.output["schema"], json!(kind));
    let v = create(
        Some("skills/s"),
        Some(kind),
        Some(Mode::Value("{}".to_string())),
        &Writer,
    )
    .unwrap();
    assert_eq!(v.sas.output["path"], json!("skills/s/skill.json"));
    assert!(v.flow.is_none());
    for (t, c, m) in [
        (None, Some(skill), Some(Mode::Semantic)),
        (Some("skills"), None, Some(Mode::Semantic)),
        (Some("skills"), Some(skill), None),
    ] {
        assert_eq!(create(t, c, m, &Writer).unwrap_err().code, "usage");
    }
}

#[test]
fn the_structure_image_is_made_from_the_skill() {
    let mut v = skill();
    v["inputs"] = json!([{"name": "一覧", "form": "テキスト"}]);
    v["outputs"] = json!([{"name": "照合の結果", "form": "JSON", "where": "標準出力"}]);
    v["steps"] = json!([
        {"id": 1, "do": "取る", "completion": "ある"},
        {"id": 2, "do": "照合する", "completion": "照合の結果が出ている"}]);
    v["edge_cases"] = json!([{"id": "2a", "when": "読めない", "then": "読む", "end": "ステップ2へ戻る", "examples": ["E1"]}]);
    v["references"] = json!([{"name": "決まり", "path": "references/r.json", "holds": "h", "form": "JSON", "read_by": [2], "from": "作る人の答え", "grounds_for": ["照合の結果"]},
        {"name": "用語集", "path": "references/t.json", "holds": "h", "form": "JSON", "read_by": [1], "from": "作る人の答え"}]);
    let files = Files::with("skills/s/skill.json", &v);
    let i = approve(&files).structure_image;
    assert_eq!(i["scope"]["inputs"], json!(["一覧"]));
    assert_eq!(i["scope"]["outside"], json!(["書き換える", "要約する"]));
    assert_eq!(
        i["materials_to_outputs"][0]["read_in"][0]["do"],
        json!("照合する")
    );
    assert_eq!(
        i["materials_to_outputs"][0]["grounds"],
        json!(["照合の結果"])
    );
    assert_eq!(i["steps_and_branches"]["branches"][0]["at_step"], json!(2));
    assert_eq!(i["common_and_variable"]["common"], json!([]));
}
