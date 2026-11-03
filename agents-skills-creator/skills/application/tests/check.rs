//! UC4 確かめる（Skill だけ）の手続きを、ポートの代役で確かめる。代役は使う application の tests/ に置く（SAS と同じ）。

#![allow(clippy::unwrap_used, clippy::panic)]

use serde_json::json;
use skills_application::ports::{
    JsonSchemaChecker, SasRun, SkillFiles, SkillHistory, SkillValidator,
};
use skills_application::use_cases::validate::{Outcome, validate};
use skills_domain::services::skill_md;
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
    /// skill.json と、そこから write-skill-md で作る SKILL.md を置く。
    fn skill(mut self, folder: &str, name: &str) -> Self {
        let value = json!({"$schema": "urn:agents-skills-creator:skill:v1", "name": name});
        self.files
            .insert(format!("{folder}/skill.json"), value.to_string());
        self.files
            .insert(format!("{folder}/SKILL.md"), skill_md::render(&value));
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

/// 根の required の欄が値にあるかだけを見る、JSON Schema の検査器の代役。
struct Schemas;

impl JsonSchemaChecker for Schemas {
    fn check(
        &self,
        schema: &serde_json::Value,
        value: &serde_json::Value,
    ) -> Result<Vec<(String, String)>, String> {
        Ok(schema
            .get("required")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_str)
            .filter(|k| value.get(k).is_none())
            .map(|k| (String::new(), format!("{k} が無い")))
            .collect())
    }
}

impl SkillHistory for Sas {
    fn log(&self, _: &str) -> Result<SasRun, String> {
        Ok(SasRun {
            exit: 0,
            output: json!([]),
        })
    }
    fn operation(&self, _: &str) -> Result<SasRun, String> {
        Ok(SasRun {
            exit: 0,
            output: json!({}),
        })
    }
}

impl SkillValidator for Sas {
    fn knows_concrete(&self, urn: &str) -> Result<bool, String> {
        Ok(urn == "urn:example:known:v1")
    }
    fn validate(&self, skill_json: &str) -> Result<SasRun, String> {
        self.asked.borrow_mut().push(skill_json.to_string());
        Ok(SasRun {
            exit: self.exit,
            output: json!({"kind": "validation", "path": skill_json}),
        })
    }
}

fn outcomes(c: &skills_application::use_cases::validate::SkillChecked) -> Vec<(&str, Outcome)> {
    c.creator.iter().map(|f| (f.check, f.outcome)).collect()
}

#[test]
fn a_usable_skill_passes_every_check_and_keeps_the_sas_output() {
    let files = Files::default().skill("skills/fact-check", "fact-check");
    let sas = Sas::ok();
    let c = validate(
        &["skills/fact-check/".to_string()],
        &sas,
        &sas,
        &files,
        &Schemas,
    )
    .unwrap();
    assert!(!c.detected());
    assert_eq!(
        outcomes(&c.skills[0]),
        [
            ("name-is-folder", Outcome::Verified),
            ("name-unique", Outcome::Verified),
            ("links-resolve", Outcome::Verified),
            ("skill-md-built", Outcome::Verified),
            ("references-exist", Outcome::Verified),
            ("json-schema-conforms", Outcome::Verified),
            ("grounds-are-outputs", Outcome::Verified),
            ("tools-exist", Outcome::Verified),
            ("assets-exist", Outcome::Verified),
            ("tools-in-role", Outcome::Verified),
            ("tool-io-declared", Outcome::Verified)
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
    let c = validate(
        &["skills/old".to_string()],
        &Sas::ok(),
        &Sas::ok(),
        &files,
        &Schemas,
    )
    .unwrap();
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
    let c = validate(
        &["skills/fact-check".to_string()],
        &Sas::ok(),
        &Sas::ok(),
        &files,
        &Schemas,
    )
    .unwrap();
    assert_eq!(c.skills[0].creator[1].outcome, Outcome::Verified);
}

#[test]
fn dangling_links_and_a_missing_folder_are_reported() {
    let mut files = Files::default().skill("skills/fact-check", "fact-check");
    files
        .dangling
        .push("skills/fact-check/references".to_string());
    let c = validate(
        &["skills/fact-check".to_string()],
        &Sas::ok(),
        &Sas::ok(),
        &files,
        &Schemas,
    )
    .unwrap();
    let link = &c.skills[0].creator[2];
    assert_eq!(
        (link.check, link.outcome, link.path.as_deref()),
        (
            "links-resolve",
            Outcome::Violated,
            Some("skills/fact-check/references")
        )
    );

    let c = validate(
        &["skills/gone".to_string()],
        &Sas::ok(),
        &Sas::ok(),
        &Files::default(),
        &Schemas,
    )
    .unwrap();
    assert_eq!(
        outcomes(&c.skills[0]),
        [
            ("name-is-folder", Outcome::Error),
            ("name-unique", Outcome::Error),
            ("links-resolve", Outcome::Violated),
            ("skill-md-built", Outcome::Error),
            ("references-exist", Outcome::Error),
            ("json-schema-conforms", Outcome::Error),
            ("grounds-are-outputs", Outcome::Error),
            ("tools-exist", Outcome::Error),
            ("assets-exist", Outcome::Error),
            ("tools-in-role", Outcome::Error),
            ("tool-io-declared", Outcome::Error)
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
    let c = validate(
        &["skills/fact-check".to_string()],
        &sas,
        &sas,
        &files,
        &Schemas,
    )
    .unwrap();
    assert!(c.creator_all_verified());
    assert!(c.detected());
}

#[test]
fn nothing_to_check_is_a_usage_error() {
    let r = validate(&[], &Sas::ok(), &Sas::ok(), &Files::default(), &Schemas).unwrap_err();
    assert_eq!(r.code, "usage");
}

trait AllVerified {
    fn creator_all_verified(&self) -> bool;
}

impl AllVerified for skills_application::use_cases::validate::Checked {
    fn creator_all_verified(&self) -> bool {
        self.skills
            .iter()
            .flat_map(|s| &s.creator)
            .all(|f| f.outcome == Outcome::Verified)
    }
}

#[test]
fn a_stale_or_missing_skill_md_is_reported() {
    let mut files = Files::default().skill("skills/fact-check", "fact-check");
    files
        .files
        .insert("skills/fact-check/SKILL.md".to_string(), "古い".to_string());
    let c = validate(
        &["skills/fact-check".to_string()],
        &Sas::ok(),
        &Sas::ok(),
        &files,
        &Schemas,
    )
    .unwrap();
    let md = &c.skills[0].creator[3];
    assert_eq!(
        (md.check, md.outcome),
        ("skill-md-built", Outcome::Violated)
    );
    files.files.remove("skills/fact-check/SKILL.md");
    let c = validate(
        &["skills/fact-check".to_string()],
        &Sas::ok(),
        &Sas::ok(),
        &files,
        &Schemas,
    )
    .unwrap();
    assert!(c.skills[0].creator[3].reason.contains("write-skill-md"));
}

#[test]
fn references_must_be_inside_the_folder_and_exist() {
    let mut files = Files::default();
    let value = json!({"$schema": "urn:agents-skills-creator:skill:v1", "name": "s", "references": [
        {"name": "ある", "path": "references/a.json"},
        {"name": "無い", "path": "references/b.json"},
        {"name": "外", "path": "../shared/c.json"}]});
    files
        .files
        .insert("skills/s/skill.json".to_string(), value.to_string());
    files
        .files
        .insert("skills/s/SKILL.md".to_string(), skill_md::render(&value));
    files
        .files
        .insert("skills/s/references/a.json".to_string(), "[]".to_string());
    files
        .files
        .insert("skills/shared/c.json".to_string(), "[]".to_string());
    let c = validate(
        &["skills/s".to_string()],
        &Sas::ok(),
        &Sas::ok(),
        &files,
        &Schemas,
    )
    .unwrap();
    let refs: Vec<(Outcome, Option<&str>)> = c.skills[0].creator[4..7]
        .iter()
        .map(|f| (f.outcome, f.path.as_deref()))
        .collect();
    assert_eq!(
        refs,
        [
            (Outcome::Verified, None),
            (Outcome::Violated, Some("skills/s/references/b.json")),
            (Outcome::Violated, Some("skills/s/../shared/c.json"))
        ]
    );
}

#[test]
fn json_resources_must_follow_their_schema_and_output_schemas_must_exist() {
    let mut files = Files::default();
    let value = json!({"$schema": "urn:agents-skills-creator:skill:v1", "name": "s",
        "references": [
            {"name": "従う", "path": "references/a.json", "form": "JSON", "schema": "references/a.schema.json"},
            {"name": "従わない", "path": "references/b.json", "form": "JSON", "schema": "references/a.schema.json"},
            {"name": "SAS", "path": "references/c.json", "form": "JSON", "schema": "urn:example:known:v1"}],
        "outputs": [
            {"name": "引けない", "form": "JSON", "schema": "urn:example:unknown:v1"},
            {"name": "無い", "form": "JSON", "schema": "references/out.schema.json"}]});
    for (p, t) in [
        ("skills/s/skill.json", value.to_string()),
        ("skills/s/SKILL.md", skill_md::render(&value)),
        (
            "skills/s/references/a.schema.json",
            json!({"required": ["id"]}).to_string(),
        ),
        ("skills/s/references/a.json", json!({"id": "x"}).to_string()),
        ("skills/s/references/b.json", json!({}).to_string()),
        (
            "skills/s/references/c.json",
            json!({"$schema": "urn:example:known:v1"}).to_string(),
        ),
    ] {
        files.files.insert(p.to_string(), t);
    }
    let c = validate(
        &["skills/s".to_string()],
        &Sas::ok(),
        &Sas::ok(),
        &files,
        &Schemas,
    )
    .unwrap();
    let got: Vec<(Outcome, Option<&str>)> = c.skills[0]
        .creator
        .iter()
        .filter(|f| f.check == "json-schema-conforms")
        .map(|f| (f.outcome, f.path.as_deref()))
        .collect();
    assert_eq!(
        got,
        [
            (Outcome::Verified, None),
            (Outcome::Violated, Some("skills/s/references/b.json")),
            (Outcome::Verified, None),
            (Outcome::Violated, Some("skills/s/skill.json")),
            (
                Outcome::Violated,
                Some("skills/s/references/out.schema.json")
            )
        ]
    );
}
