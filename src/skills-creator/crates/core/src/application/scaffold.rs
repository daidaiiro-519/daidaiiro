//! 新しい Skill の雛形を置く。まだ無いフォルダにだけ置く。
//! 置くのは SKILL.md ・ 入口（scripts/<Skill名> と scripts/<Skill名>.cmd）・ 照合した基盤のエンジン ・
//! concrete スキーマの雛形（基盤の create で作る）である。
//! 作業用のフォルダでそろえてから移すので、途中で失敗したら何も残らない。

use super::{engine, Env};
use crate::domain::release::exe_suffix;
use crate::domain::release::host_target;
use crate::domain::skill_md::valid_name;
use crate::ports::{Files, Programs, Releases};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// 雛形のファイル（skills-creator の assets/templates/ の名前）と、置く先（Skill のフォルダからの相対パス）。
/// 置く先の `{name}` は Skill の名前に置き換える。
const TEMPLATES: [(&str, &str); 3] = [
    ("SKILL.md.tmpl", "SKILL.md"),
    ("entry.tmpl", "scripts/{name}"),
    ("entry.cmd.tmpl", "scripts/{name}.cmd"),
];

/// 雛形の中で、Skill の名前に置き換える記入欄。ほかの記入欄は、作る人が書き換える。
const NAME_FIELD: &str = "{{name}}";

/// skills-creator 自身のフォルダの、基盤のエンジン。
pub fn own_engine(own: &Path) -> PathBuf {
    own.join("scripts")
        .join("bin")
        .join(format!("schema-driven{}", exe_suffix(host_target())))
}

/// dir に新しい Skill の雛形を置く。version を省けば、skills-creator 自身のエンジンと同じ版を入れる。
pub fn scaffold<F: Files, P: Programs, R: Releases>(
    env: &Env<F, P, R>,
    own: &Path,
    dir: &Path,
    version: Option<&str>,
    target: &str,
) -> Result<Value, String> {
    if env.files.exists(dir) {
        return Err(format!(
            "{} は既に在る。scaffold はまだ無いフォルダにだけ置く。直すときは check の結果から直す",
            dir.display()
        ));
    }
    let name = dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    if !valid_name(&name) {
        return Err(format!(
            "フォルダの名前 {name} が Skill の名前の決まり（64文字までの a-z ・ 0-9 ・ -、- で始めず終えず続けない）に合わない"
        ));
    }
    let version = match version {
        Some(version) => version.to_owned(),
        None => own_version(env, own)?,
    };
    let parent = dir.parent().unwrap_or(Path::new("."));
    let stage = parent.join(format!(".{name}.scaffold"));
    let _ = env.files.remove_dir(&stage);
    env.files.create_dir(&stage)?;
    let result = fill(env, own, &stage, &name, &version, target);
    match result {
        Ok(placed) => {
            env.files.rename(&stage, dir)?;
            Ok(
                json!({"path": dir.to_string_lossy(), "name": name, "engine": placed["engine"], "files": placed["files"],
                      "next": "記入欄 {{…}} を書き換え、assets/ のスキーマの未記入の欄を埋め、check で確かめる"}),
            )
        }
        Err(error) => {
            let _ = env.files.remove_dir(&stage);
            Err(error)
        }
    }
}

fn own_version<F: Files, P: Programs, R: Releases>(
    env: &Env<F, P, R>,
    own: &Path,
) -> Result<String, String> {
    let run = env
        .programs
        .run(&own_engine(own), &["--version".to_owned()])?;
    let answered: Value = serde_json::from_str(&run.stdout).unwrap_or(Value::Null);
    answered["data"]["version"]
        .as_str()
        .map(str::to_owned)
        .ok_or("skills-creator 自身の基盤のエンジンが版を答えない。--engine で版を渡す".to_owned())
}

fn fill<F: Files, P: Programs, R: Releases>(
    env: &Env<F, P, R>,
    own: &Path,
    stage: &Path,
    name: &str,
    version: &str,
    target: &str,
) -> Result<Value, String> {
    let templates = own.join("assets").join("templates");
    env.files.create_dir(&stage.join("scripts"))?;
    let mut files: Vec<String> = TEMPLATES
        .iter()
        .map(|(template, to)| {
            let text = env.files.read(&templates.join(template))?;
            let to = to.replace("{name}", name);
            let path = stage.join(&to);
            env.files.write(&path, &text.replace(NAME_FIELD, name))?;
            if to.starts_with("scripts/") {
                env.files.set_executable(&path)?;
            }
            Ok(to)
        })
        .collect::<Result<_, String>>()?;

    let placed = engine::install(env, stage, version, target)?;
    files.extend(
        placed["files"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned),
    );

    env.files.create_dir(&stage.join("assets"))?;
    let schema = format!("assets/{name}.schema.json");
    let created = env.programs.run(
        &own_engine(own),
        &[
            "create".to_owned(),
            "--schema".to_owned(),
            "schema-driven/abstract@v1".to_owned(),
            "--path".to_owned(),
            stage.join(&schema).to_string_lossy().into_owned(),
        ],
    )?;
    if created.code != 0 {
        return Err(format!(
            "concrete スキーマの雛形を作れない：{}",
            created.stdout
        ));
    }
    files.push(schema);
    Ok(json!({"engine": {"version": version, "target": target}, "files": files}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::release::{BUNDLE, SUMS};
    use crate::tests::{FakePrograms, FakeReleases, MemFiles};

    const ASSET: &str = "schema-driven-0.1.0-x86_64-unknown-linux-musl.tar.gz";
    /// "archive" の SHA-256（前もって計算した値）
    const ARCHIVE_SHA: &str = "0eb3e36bfb24dcd9bb1d1bece1531216b59539a8fde17ee80224af0653c92aa3";
    const OWN: &str = "/own/skills-creator";

    /// skills-creator 自身のフォルダ（雛形とエンジン）と、0.1.0 を持つ Release。
    fn env() -> Env<MemFiles, FakePrograms, FakeReleases> {
        let files = MemFiles::default();
        files.put(
            "/own/skills-creator/assets/templates/SKILL.md.tmpl",
            "---\nname: {{name}}\ndescription: {{何をするか}}\n---\n",
        );
        files.put(
            "/own/skills-creator/assets/templates/entry.tmpl",
            "#!/usr/bin/env python3\n# {{name}} の入口\n",
        );
        files.put(
            "/own/skills-creator/assets/templates/entry.cmd.tmpl",
            "@py -3 \"%~dp0{{name}}\" %*\n",
        );
        let releases = FakeReleases {
            files: files.clone(),
            ..FakeReleases::default()
        };
        releases.assets.borrow_mut().extend([
            (ASSET.to_owned(), b"archive".to_vec()),
            (
                SUMS.to_owned(),
                format!("{ARCHIVE_SHA}  {ASSET}\n").into_bytes(),
            ),
            (BUNDLE.to_owned(), b"{}".to_vec()),
        ]);
        releases.unpacked.borrow_mut().extend([
            (
                "schema-driven/scripts/bin/schema-driven".to_owned(),
                b"e".to_vec(),
            ),
            (
                "schema-driven/scripts/bin/schema-driven-mcp".to_owned(),
                b"e".to_vec(),
            ),
        ]);
        let programs = FakePrograms::default();
        programs.answer(
            "/own/skills-creator/scripts/bin/schema-driven",
            "--version",
            0,
            r#"{"ok":true,"data":{"version":"0.1.0"}}"#,
        );
        programs.answer(
            "/tmp/work/unpacked/schema-driven/scripts/bin/schema-driven",
            "--version",
            0,
            r#"{"ok":true,"data":{"version":"0.1.0"}}"#,
        );
        programs.answer(
            "/own/skills-creator/scripts/bin/schema-driven",
            "create --schema schema-driven/abstract@v1 --path /s/.practice.scaffold/assets/practice.schema.json",
            0,
            r#"{"ok":true}"#,
        );
        Env {
            files,
            programs,
            releases,
        }
    }

    #[test]
    fn a_new_skill_gets_the_templates_the_entry_and_a_verified_engine() {
        let env = env();
        let made = scaffold(
            &env,
            Path::new(OWN),
            Path::new("/s/practice"),
            None,
            "x86_64-unknown-linux-musl",
        )
        .unwrap();
        assert_eq!(
            made["files"],
            json!([
                "SKILL.md",
                "scripts/practice",
                "scripts/practice.cmd",
                "scripts/bin/schema-driven",
                "scripts/bin/schema-driven-mcp",
                "assets/practice.schema.json"
            ])
        );
        assert_eq!(
            made["engine"],
            json!({"version": "0.1.0", "target": "x86_64-unknown-linux-musl"})
        );
        assert_eq!(
            env.files.text("/s/practice/SKILL.md").as_deref(),
            Some("---\nname: practice\ndescription: {{何をするか}}\n---\n")
        );
        assert_eq!(
            env.files
                .text("/s/practice/scripts/practice.cmd")
                .as_deref(),
            Some("@py -3 \"%~dp0practice\" %*\n")
        );
        assert!(env
            .files
            .executable
            .borrow()
            .contains(Path::new("/s/.practice.scaffold/scripts/practice")));
        assert_eq!(
            env.files.under("/s/.practice.scaffold"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn an_existing_folder_is_refused() {
        let env = env();
        env.files.put("/s/practice/SKILL.md", "mine");
        let result = scaffold(
            &env,
            Path::new(OWN),
            Path::new("/s/practice"),
            None,
            "x86_64-unknown-linux-musl",
        );
        assert_eq!(
            result,
            Err("/s/practice は既に在る。scaffold はまだ無いフォルダにだけ置く。直すときは check の結果から直す".to_owned())
        );
        assert_eq!(
            env.files.text("/s/practice/SKILL.md").as_deref(),
            Some("mine")
        );
    }

    #[test]
    fn a_folder_name_that_breaks_the_name_rules_is_refused() {
        let env = env();
        let result = scaffold(
            &env,
            Path::new(OWN),
            Path::new("/s/Practice"),
            None,
            "x86_64-unknown-linux-musl",
        );
        assert_eq!(
            result,
            Err("フォルダの名前 Practice が Skill の名前の決まり（64文字までの a-z ・ 0-9 ・ -、- で始めず終えず続けない）に合わない".to_owned())
        );
    }

    #[test]
    fn a_failed_engine_install_leaves_nothing_behind() {
        let env = env();
        *env.releases.attest_fails.borrow_mut() = true;
        let result = scaffold(
            &env,
            Path::new(OWN),
            Path::new("/s/practice"),
            None,
            "x86_64-unknown-linux-musl",
        );
        assert_eq!(
            result,
            Err("照合で止まった：証明が合わない：証明が一致しない（試験）".to_owned())
        );
        assert_eq!(env.files.under("/s"), Vec::<String>::new());
    }
}
