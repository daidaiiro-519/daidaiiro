//! UC4 確かめる（Skill だけ）の手続きを、ポートの代役で確かめる。代役は使う application の tests/ に置く（SAS と同じ）。

#![allow(clippy::unwrap_used, clippy::panic)]

use serde_json::json;
use skills_application::ports::{SasRun, SkillFiles, SkillValidator};
use skills_application::use_cases::check::{Outcome, check};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// 経路 → 中身のファイルと、先の無いリンクを持つ代役。
#[derive(Default)]
struct Files {
    files: BTreeMap<String, String>,
    /// 先の無いリンク（経路）
    dangling: Vec<String>,
}

impl Files {
    fn skill(mut self, folder: &str, name: &str) -> Self {
        self.files.insert(
            format!("{folder}/skill.json"),
            json!({"$schema": "urn:agents-skills-creator:skill:v1", "name": name}).to_string(),
        );
        self
    }
}

impl SkillFiles for Files {
    fn exists(&self, path: &str) -> bool {
        !self.dangling.iter().any(|d| d == path)
            && self
                .files
                .keys()
                .any(|k| k == path || k.starts_with(&format!("{path}/")))
    }
    fn read(&self, path: &str) -> Result<String, String> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| format!("{path} が無い"))
    }
    fn siblings(&self, folder: &str) -> Result<Vec<String>, String> {
        let (parent, _) = folder.rsplit_once('/').unwrap_or(("", folder));
        let mut out: Vec<String> = self
            .files
            .keys()
            .filter_map(|k| {
                let rest = k.strip_prefix(&format!("{parent}/"))?;
                let first = rest.split('/').next()?;
                (rest.contains('/')).then(|| format!("{parent}/{first}"))
            })
            .filter(|f| f != folder)
            .collect();
        out.sort();
        out.dedup();
        Ok(out)
    }
    fn dangling_links(&self, folder: &str) -> Result<Vec<String>, String> {
        Ok(self
            .dangling
            .iter()
            .filter(|d| d.starts_with(&format!("{folder}/")))
            .cloned()
            .collect())
    }
}

/// 渡された経路を覚え、決まった結果を返す SAS の代役。
struct Sas {
    exit: i32,
    asked: RefCell<Vec<String>>,
}

impl Sas {
    fn ok() -> Self {
        Self {
            exit: 0,
            asked: RefCell::new(Vec::new()),
        }
    }
}

impl SkillValidator for Sas {
    fn validate(&self, skill_json: &str) -> Result<SasRun, String> {
        self.asked.borrow_mut().push(skill_json.to_string());
        Ok(SasRun {
            exit: self.exit,
            output: json!({"kind": "validation", "path": skill_json}),
        })
    }
}

fn outcomes(c: &skills_application::use_cases::check::SkillChecked) -> Vec<(&str, Outcome)> {
    c.creator.iter().map(|f| (f.check, f.outcome)).collect()
}

#[test]
fn a_usable_skill_passes_every_check_and_keeps_the_sas_output() {
    let files = Files::default().skill("skills/fact-check", "fact-check");
    let sas = Sas::ok();
    let c = check(&["skills/fact-check/".to_string()], &sas, &files).unwrap();
    assert!(!c.detected());
    assert_eq!(
        outcomes(&c.skills[0]),
        [
            ("name-is-folder", Outcome::Verified),
            ("name-unique", Outcome::Verified),
            ("links-resolve", Outcome::Verified)
        ]
    );
    // SAS には <フォルダ>/skill.json を渡し、出力は書き換えずに持つ
    assert_eq!(
        sas.asked.borrow().as_slice(),
        ["skills/fact-check/skill.json"]
    );
    assert_eq!(
        c.skills[0].sas.output,
        json!({"kind": "validation", "path": "skills/fact-check/skill.json"})
    );
    // 渡されたフォルダはそのまま返す
    assert_eq!(c.skills[0].folder, "skills/fact-check/");
}

#[test]
fn name_must_be_the_folder_name_and_must_not_overlap() {
    let files = Files::default()
        .skill("skills/fact-check", "fact-check")
        .skill("skills/old", "fact-check");
    let c = check(&["skills/old".to_string()], &Sas::ok(), &files).unwrap();
    assert!(c.detected());
    let f = &c.skills[0].creator;
    assert_eq!(f[0].outcome, Outcome::Violated);
    assert_eq!(f[1].outcome, Outcome::Violated);
    assert_eq!(f[1].path.as_deref(), Some("skills/fact-check"));
}

#[test]
fn folders_that_are_not_skills_do_not_overlap() {
    // 同じ親フォルダにあっても、skill.json の無いフォルダは Skill として数えない
    let mut files = Files::default().skill("skills/fact-check", "fact-check");
    files
        .files
        .insert("skills/notes/readme.md".to_string(), "メモ".to_string());
    let c = check(&["skills/fact-check".to_string()], &Sas::ok(), &files).unwrap();
    assert_eq!(c.skills[0].creator[1].outcome, Outcome::Verified);
}

#[test]
fn dangling_links_and_a_missing_folder_are_reported() {
    let mut files = Files::default().skill("skills/fact-check", "fact-check");
    files
        .dangling
        .push("skills/fact-check/references".to_string());
    let c = check(&["skills/fact-check".to_string()], &Sas::ok(), &files).unwrap();
    let link = &c.skills[0].creator[2];
    assert_eq!(
        (link.check, link.outcome, link.path.as_deref()),
        (
            "links-resolve",
            Outcome::Violated,
            Some("skills/fact-check/references")
        )
    );

    let c = check(&["skills/gone".to_string()], &Sas::ok(), &Files::default()).unwrap();
    assert_eq!(
        outcomes(&c.skills[0]),
        [
            ("name-is-folder", Outcome::Error),
            ("name-unique", Outcome::Error),
            ("links-resolve", Outcome::Violated)
        ]
    );
}

#[test]
fn a_sas_detection_alone_is_a_detection() {
    let files = Files::default().skill("skills/fact-check", "fact-check");
    let sas = Sas {
        exit: 1,
        asked: RefCell::new(Vec::new()),
    };
    let c = check(&["skills/fact-check".to_string()], &sas, &files).unwrap();
    assert!(c.creator_all_verified());
    assert!(c.detected());
}

#[test]
fn nothing_to_check_is_a_usage_error() {
    let r = check(&[], &Sas::ok(), &Files::default()).unwrap_err();
    assert_eq!(r.code, "usage");
}

trait AllVerified {
    fn creator_all_verified(&self) -> bool;
}

impl AllVerified for skills_application::use_cases::check::Checked {
    fn creator_all_verified(&self) -> bool {
        self.skills
            .iter()
            .flat_map(|s| &s.creator)
            .all(|f| f.outcome == Outcome::Verified)
    }
}
