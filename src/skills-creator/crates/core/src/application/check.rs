//! Skill のフォルダを、項目ごとに検査する。どの項目も既定は「判定不能」で、確かめた分岐からだけ合格を置く。
//! SKILL.md の型は、読み取ったインスタンスを skills-creator 自身の基盤のエンジンに型のスキーマで検査させる。
//! スキーマ ・ エンジン ・ 入口は、検査する Skill のフォルダの基盤のエンジンで確かめる。

use super::scaffold::own_engine;
use super::Env;
use crate::domain::release::{exe_suffix, host_target};
use crate::domain::report::{Outcome, Report};
use crate::domain::skill_md::read;
use crate::ports::{Files, Programs, Releases};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// 入口とエンジンが同じ結果を返すかを確かめる、基盤の操作。
const DELEGATE_PROBE: [&str; 5] = [
    "prompt",
    "--schema",
    "schema-driven/abstract@v1",
    "--property",
    "/title",
];

/// dir の Skill を検査する。own は skills-creator 自身のフォルダ（型のスキーマとエンジンを持つ）。
pub fn check<F: Files, P: Programs, R: Releases>(
    env: &Env<F, P, R>,
    own: &Path,
    dir: &Path,
) -> Report {
    let mut report = Report::default();
    legacy(env, dir, &mut report);
    skill_md(env, own, dir, &mut report);
    let schemas = schemas(env, dir, &mut report);
    let engine = dir
        .join("scripts")
        .join("bin")
        .join(format!("schema-driven{}", exe_suffix(host_target())));
    match engine_version(env, &engine) {
        Ok(version) => {
            report.set(
                "engine.runs",
                Outcome::Pass,
                format!("schema-driven {version}"),
            );
            schemas_check(env, &engine, dir, &schemas, &mut report);
            abstracts(env, &engine, &schemas, &mut report);
            delegate(env, &engine, dir, &mut report);
        }
        Err(why) => {
            report.set("engine.runs", Outcome::Fail, why);
            ["schemas.check", "engine.abstract", "entry.delegate"]
                .iter()
                .for_each(|id| {
                    report.set(
                        id,
                        Outcome::Unknown,
                        "基盤のエンジンが動かないので確かめられない",
                    )
                });
        }
    }
    report
}

fn legacy<F: Files, P: Programs, R: Releases>(env: &Env<F, P, R>, dir: &Path, report: &mut Report) {
    let found: Vec<&str> = ["tool.json", "tools.json"]
        .into_iter()
        .filter(|name| env.files.exists(&dir.join(name)))
        .collect();
    match found.is_empty() {
        true => report.set("legacy.absent", Outcome::Pass, ""),
        false => report.set(
            "legacy.absent",
            Outcome::Fail,
            format!("古い形のファイルが在る：{}", found.join(" ・ ")),
        ),
    }
}

fn skill_md<F: Files, P: Programs, R: Releases>(
    env: &Env<F, P, R>,
    own: &Path,
    dir: &Path,
    report: &mut Report,
) {
    let Ok(text) = env.files.read(&dir.join("SKILL.md")) else {
        ["skill.frontmatter", "skill.type", "skill.placeholders"]
            .iter()
            .for_each(|id| report.set(id, Outcome::Fail, "SKILL.md が無い"));
        return;
    };
    let folder = dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let exists = |target: &str| env.files.exists(&dir.join(target));
    let read = read(&text, &folder, &exists);
    match &read.frontmatter_error {
        None => report.set("skill.frontmatter", Outcome::Pass, ""),
        Some(error) => report.set("skill.frontmatter", Outcome::Fail, error.clone()),
    }
    let placeholders = read.instance["placeholders"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    match placeholders.is_empty() {
        true => report.set("skill.placeholders", Outcome::Pass, ""),
        false => report.set(
            "skill.placeholders",
            Outcome::Fail,
            format!(
                "{} か所（最初は {} 行目）",
                placeholders.len(),
                placeholders[0]["line"]
            ),
        ),
    }
    let (outcome, detail) = type_check(env, own, &read.instance);
    report.set("skill.type", outcome, detail);
}

/// 読み取ったインスタンスを、skills-creator 自身のエンジンに型のスキーマ（assets/skill.schema.json）で検査させる。
fn type_check<F: Files, P: Programs, R: Releases>(
    env: &Env<F, P, R>,
    own: &Path,
    instance: &Value,
) -> (Outcome, String) {
    let work = match env.files.temp_dir() {
        Ok(work) => work,
        Err(error) => return (Outcome::Unknown, error),
    };
    let result = (|| {
        env.files
            .write(&work.join("skill.json"), &instance.to_string())?;
        let run = env.programs.run(
            &own_engine(own),
            &[
                "--skill".to_owned(),
                own.to_string_lossy().into_owned(),
                "check".to_owned(),
                "--dir".to_owned(),
                work.to_string_lossy().into_owned(),
            ],
        )?;
        serde_json::from_str::<Value>(&run.stdout)
            .map_err(|_| format!("基盤のエンジンの出力が JSON でない：{}", run.stdout))
    })();
    let _ = env.files.remove_dir(&work);
    let answer = match result {
        Ok(answer) => answer,
        Err(error) => return (Outcome::Unknown, error),
    };
    let checked = &answer["instances"][0];
    let problems: Vec<String> = checked["errors"]
        .as_array()
        .into_iter()
        .flatten()
        // 記入欄は skill.placeholders が見る。1つの原因で2つの項目を落とさない
        .filter(|error| {
            !error["property"]
                .as_str()
                .unwrap_or("")
                .starts_with("/placeholders")
        })
        .map(|error| {
            format!(
                "{} {}",
                error["property"].as_str().unwrap_or(""),
                error["reason"].as_str().unwrap_or("")
            )
        })
        .chain(
            checked["unfilled"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|unfilled| format!("{} が無い", unfilled["property"].as_str().unwrap_or(""))),
        )
        .collect();
    match (checked.is_object(), problems.is_empty()) {
        (false, _) => (
            Outcome::Unknown,
            format!("基盤のエンジンが型を判定しなかった：{answer}"),
        ),
        (true, true) => (Outcome::Pass, String::new()),
        (true, false) => (Outcome::Fail, problems.join(" ／ ")),
    }
}

fn schemas<F: Files, P: Programs, R: Releases>(
    env: &Env<F, P, R>,
    dir: &Path,
    report: &mut Report,
) -> Vec<PathBuf> {
    let found: Vec<PathBuf> = env
        .files
        .list(&dir.join("assets"))
        .into_iter()
        .filter(|path| path.to_string_lossy().ends_with(".schema.json"))
        .collect();
    match found.is_empty() {
        true => report.set(
            "schemas.present",
            Outcome::Fail,
            "assets/ に *.schema.json が無い",
        ),
        false => report.set(
            "schemas.present",
            Outcome::Pass,
            format!("{} 件", found.len()),
        ),
    }
    found
}

fn engine_version<F: Files, P: Programs, R: Releases>(
    env: &Env<F, P, R>,
    engine: &Path,
) -> Result<String, String> {
    if !env.files.exists(engine) {
        return Err("scripts/bin/schema-driven が無い。engine で入れる".to_owned());
    }
    let run = env.programs.run(engine, &["--version".to_owned()])?;
    let answered: Value = serde_json::from_str(&run.stdout).unwrap_or(Value::Null);
    answered["data"]["version"]
        .as_str()
        .map(str::to_owned)
        .ok_or(format!("起動したが版を答えない（終了コード {}）", run.code))
}

fn schemas_check<F: Files, P: Programs, R: Releases>(
    env: &Env<F, P, R>,
    engine: &Path,
    dir: &Path,
    schemas: &[PathBuf],
    report: &mut Report,
) {
    if schemas.is_empty() {
        report.set(
            "schemas.check",
            Outcome::Skip,
            "スキーマが無い（schemas.present が見る）",
        );
        return;
    }
    let args = [
        "check-schemas".to_owned(),
        "--dir".to_owned(),
        dir.join("assets").to_string_lossy().into_owned(),
    ];
    let answer = env
        .programs
        .run(engine, &args)
        .ok()
        .and_then(|run| serde_json::from_str::<Value>(&run.stdout).ok());
    let Some(answer) = answer else {
        report.set(
            "schemas.check",
            Outcome::Unknown,
            "check-schemas が答えない",
        );
        return;
    };
    let findings = answer["findings"].as_array().cloned().unwrap_or_default();
    match (answer["ok"] == Value::Bool(true), findings.first()) {
        (true, None) => report.set("schemas.check", Outcome::Pass, ""),
        (_, Some(first)) => report.set(
            "schemas.check",
            Outcome::Fail,
            format!(
                "{} 件（最初は {} の {}：{}）",
                findings.len(),
                first["schema"].as_str().unwrap_or(""),
                first["at"].as_str().unwrap_or(""),
                first["message"].as_str().unwrap_or("")
            ),
        ),
        (false, None) => report.set(
            "schemas.check",
            Outcome::Unknown,
            format!("check-schemas が失敗した：{answer}"),
        ),
    }
}

fn abstracts<F: Files, P: Programs, R: Releases>(
    env: &Env<F, P, R>,
    engine: &Path,
    schemas: &[PathBuf],
    report: &mut Report,
) {
    let names: BTreeSet<String> = schemas
        .iter()
        .filter_map(|path| env.files.read(path).ok())
        .filter_map(|text| serde_json::from_str::<Value>(&text).ok())
        .filter_map(|schema| schema["x-abstract"].as_str().map(str::to_owned))
        .collect();
    if names.is_empty() {
        report.set(
            "engine.abstract",
            Outcome::Skip,
            "x-abstract を書いたスキーマが無い（schemas.check が見る）",
        );
        return;
    }
    let unsupported: Vec<String> = names
        .iter()
        .filter(|name| {
            let args = ["prompt", "--schema", name, "--property", "/title"].map(str::to_owned);
            !matches!(env.programs.run(engine, &args), Ok(run) if run.code == 0)
        })
        .cloned()
        .collect();
    match unsupported.is_empty() {
        true => report.set(
            "engine.abstract",
            Outcome::Pass,
            names.into_iter().collect::<Vec<_>>().join(" ・ "),
        ),
        false => report.set(
            "engine.abstract",
            Outcome::Fail,
            format!("エンジンが扱えない：{}", unsupported.join(" ・ ")),
        ),
    }
}

fn delegate<F: Files, P: Programs, R: Releases>(
    env: &Env<F, P, R>,
    engine: &Path,
    dir: &Path,
    report: &mut Report,
) {
    let name = dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let entry_name = if host_target().contains("windows") {
        format!("{name}.cmd")
    } else {
        name
    };
    let entry = dir.join("scripts").join(entry_name);
    if !env.files.exists(&entry) {
        report.set(
            "entry.delegate",
            Outcome::Skip,
            "入口が無い（SKILL.md がエンジンを直接呼ぶ）",
        );
        return;
    }
    let probe: Vec<String> = DELEGATE_PROBE.iter().map(|arg| (*arg).to_owned()).collect();
    let direct: Vec<String> = ["--skill".to_owned(), dir.to_string_lossy().into_owned()]
        .into_iter()
        .chain(probe.iter().cloned())
        .collect();
    let through_entry = env.programs.run(&entry, &probe);
    let straight = env.programs.run(engine, &direct);
    match (through_entry, straight) {
        (Ok(a), Ok(b)) if a.code == b.code && a.stdout == b.stdout && b.code == 0 => {
            report.set("entry.delegate", Outcome::Pass, "")
        }
        (Ok(a), Ok(b)) => report.set(
            "entry.delegate",
            Outcome::Fail,
            format!(
                "入口とエンジンの結果が違う（終了コード {} と {}）",
                a.code, b.code
            ),
        ),
        (Err(error), _) => report.set(
            "entry.delegate",
            Outcome::Fail,
            format!("入口を起動できない：{error}"),
        ),
        (_, Err(error)) => report.set("entry.delegate", Outcome::Unknown, error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{FakePrograms, FakeReleases, MemFiles};

    const OWN: &str = "/own/skills-creator";
    const ENGINE: &str = "/s/practice/scripts/bin/schema-driven";
    const OWN_ENGINE: &str = "/own/skills-creator/scripts/bin/schema-driven";
    const VALID_MD: &str = "---\nname: practice\ndescription: 練習\n---\n## 目的\n";
    const PROBE: &str = "prompt --schema schema-driven/abstract@v1 --property /title";
    const TITLE_PROMPT: &str = r#"{"ok":true,"prompt":{"read":{"means":"名前"}}}"#;

    /// どの項目も合格する Skill のフォルダ。各試験は、ここから1か所だけ壊す。
    fn good() -> Env<MemFiles, FakePrograms, FakeReleases> {
        let files = MemFiles::default();
        files.put("/s/practice/SKILL.md", VALID_MD);
        files.put(
            "/s/practice/assets/practice.schema.json",
            r#"{"x-abstract":"schema-driven/abstract@v1"}"#,
        );
        files.put(ENGINE, "engine");
        files.put("/s/practice/scripts/practice", "#!/usr/bin/env python3");
        let programs = FakePrograms::default();
        programs.answer(
            OWN_ENGINE,
            "--skill /own/skills-creator check --dir /tmp/work",
            0,
            r#"{"ok":true,"instances":[{"errors":[],"unfilled":[]}],"findings":[]}"#,
        );
        programs.answer(
            ENGINE,
            "--version",
            0,
            r#"{"ok":true,"data":{"version":"0.1.0"}}"#,
        );
        programs.answer(
            ENGINE,
            "check-schemas --dir /s/practice/assets",
            0,
            r#"{"ok":true,"findings":[]}"#,
        );
        programs.answer(ENGINE, PROBE, 0, TITLE_PROMPT);
        programs.answer("/s/practice/scripts/practice", PROBE, 0, TITLE_PROMPT);
        programs.answer(
            ENGINE,
            &format!("--skill /s/practice {PROBE}"),
            0,
            TITLE_PROMPT,
        );
        Env {
            files,
            programs,
            releases: FakeReleases::default(),
        }
    }

    fn outcomes(env: &Env<MemFiles, FakePrograms, FakeReleases>) -> Vec<(&'static str, Outcome)> {
        let report = check(env, Path::new(OWN), Path::new("/s/practice"));
        crate::domain::report::ITEMS
            .iter()
            .map(|(id, _)| (*id, report.outcome(id).unwrap()))
            .collect()
    }

    fn all_pass_but(changed: &[(&'static str, Outcome)]) -> Vec<(&'static str, Outcome)> {
        crate::domain::report::ITEMS
            .iter()
            .map(|(id, _)| {
                let changed = changed
                    .iter()
                    .find(|(known, _)| known == id)
                    .map(|(_, outcome)| *outcome);
                (*id, changed.unwrap_or(Outcome::Pass))
            })
            .collect()
    }

    #[test]
    fn a_good_skill_passes_every_item() {
        let env = good();
        assert_eq!(outcomes(&env), all_pass_but(&[]));
        assert_eq!(
            check(&env, Path::new(OWN), Path::new("/s/practice")).code(),
            0
        );
    }

    #[test]
    fn a_left_placeholder_fails_only_the_placeholder_item() {
        let env = good();
        env.files.put(
            "/s/practice/SKILL.md",
            "---\nname: practice\ndescription: 練習\n---\n## 目的\n{{目的}}\n",
        );
        assert_eq!(
            outcomes(&env),
            all_pass_but(&[("skill.placeholders", Outcome::Fail)])
        );
    }

    #[test]
    fn a_skill_md_the_type_rejects_fails_the_type_item() {
        let env = good();
        env.programs.answer(
            OWN_ENGINE,
            "--skill /own/skills-creator check --dir /tmp/work",
            0,
            r#"{"ok":true,"instances":[{"errors":[{"property":"/sections","reason":"6つ無い"}],"unfilled":[]}],"findings":[]}"#,
        );
        assert_eq!(
            outcomes(&env),
            all_pass_but(&[("skill.type", Outcome::Fail)])
        );
    }

    #[test]
    fn a_placeholder_the_type_also_rejects_fails_only_the_placeholder_item() {
        let env = good();
        env.files.put(
            "/s/practice/SKILL.md",
            "---\nname: practice\ndescription: 練習\n---\n## 目的\n{{目的}}\n",
        );
        env.programs.answer(
            OWN_ENGINE,
            "--skill /own/skills-creator check --dir /tmp/work",
            0,
            r#"{"ok":true,"instances":[{"errors":[{"property":"/placeholders","reason":"要素の数が maxItems の 0 より多い"}],"unfilled":[]}],"findings":[]}"#,
        );
        assert_eq!(
            outcomes(&env),
            all_pass_but(&[("skill.placeholders", Outcome::Fail)])
        );
    }

    #[test]
    fn a_missing_skill_md_fails_the_three_skill_items() {
        let env = good();
        env.files
            .remove_file(Path::new("/s/practice/SKILL.md"))
            .unwrap();
        assert_eq!(
            outcomes(&env),
            all_pass_but(&[
                ("skill.frontmatter", Outcome::Fail),
                ("skill.type", Outcome::Fail),
                ("skill.placeholders", Outcome::Fail)
            ])
        );
    }

    #[test]
    fn no_schema_fails_the_presence_and_skips_what_depends_on_it() {
        let env = good();
        env.files
            .remove_file(Path::new("/s/practice/assets/practice.schema.json"))
            .unwrap();
        assert_eq!(
            outcomes(&env),
            all_pass_but(&[
                ("schemas.present", Outcome::Fail),
                ("schemas.check", Outcome::Skip),
                ("engine.abstract", Outcome::Skip)
            ])
        );
    }

    #[test]
    fn a_schema_with_findings_fails_the_schema_check() {
        let env = good();
        env.programs.answer(
            ENGINE,
            "check-schemas --dir /s/practice/assets",
            0,
            r#"{"ok":true,"findings":[{"schema":"practice.schema.json","at":"/title","message":"/title が無い"}]}"#,
        );
        assert_eq!(
            outcomes(&env),
            all_pass_but(&[("schemas.check", Outcome::Fail)])
        );
    }

    #[test]
    fn an_engine_that_does_not_run_fails_and_leaves_its_dependents_unknown() {
        let env = good();
        env.programs.answer(ENGINE, "--version", 126, "");
        assert_eq!(
            outcomes(&env),
            all_pass_but(&[
                ("engine.runs", Outcome::Fail),
                ("schemas.check", Outcome::Unknown),
                ("engine.abstract", Outcome::Unknown),
                ("entry.delegate", Outcome::Unknown)
            ])
        );
        assert_eq!(
            check(&env, Path::new(OWN), Path::new("/s/practice")).code(),
            1
        );
    }

    #[test]
    fn an_abstract_the_engine_does_not_know_fails() {
        let env = good();
        env.programs.answer(ENGINE, PROBE, 2, r#"{"ok":false}"#);
        assert_eq!(
            outcomes(&env),
            all_pass_but(&[("engine.abstract", Outcome::Fail)])
        );
    }

    #[test]
    fn an_entry_that_answers_differently_fails_the_delegate_item() {
        let env = good();
        env.programs.answer(
            "/s/practice/scripts/practice",
            PROBE,
            0,
            r#"{"ok":true,"prompt":"別のもの"}"#,
        );
        assert_eq!(
            outcomes(&env),
            all_pass_but(&[("entry.delegate", Outcome::Fail)])
        );
    }

    #[test]
    fn a_skill_without_an_entry_skips_the_delegate_item() {
        let env = good();
        env.files
            .remove_file(Path::new("/s/practice/scripts/practice"))
            .unwrap();
        assert_eq!(
            outcomes(&env),
            all_pass_but(&[("entry.delegate", Outcome::Skip)])
        );
    }

    #[test]
    fn a_legacy_tool_json_fails_the_legacy_item() {
        let env = good();
        env.files.put("/s/practice/tool.json", "{}");
        assert_eq!(
            outcomes(&env),
            all_pass_but(&[("legacy.absent", Outcome::Fail)])
        );
    }
}
