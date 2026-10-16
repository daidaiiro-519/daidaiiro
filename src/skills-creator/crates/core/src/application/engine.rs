//! 基盤のエンジンを、Skill のフォルダの scripts/bin/ に入れる（入れ替える）。
//! 版を指して Release から取り、SHA256SUMS と証明を照合してから置く。記録のファイルは置かない。
//! 途中で失敗したら、Skill のフォルダは元のまま残し、どの段で止まったかを返す。

use super::Env;
use crate::domain::release::{
    asset, exe_suffix, expected_sha256, host_target, sha256_hex, tag, BUNDLE, ENGINE_BINARIES, SUMS,
};
use crate::ports::{Files, Programs, Releases};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// 基盤のエンジンを skill の scripts/bin/ に入れる。入れた版 ・ target ・ ファイルを返す。
pub fn install<F: Files, P: Programs, R: Releases>(
    env: &Env<F, P, R>,
    skill: &Path,
    version: &str,
    target: &str,
) -> Result<Value, String> {
    let work = env.files.temp_dir()?;
    let result = fetch_and_place(env, &work, skill, version, target);
    let _ = env.files.remove_dir(&work);
    result
}

fn fetch_and_place<F: Files, P: Programs, R: Releases>(
    env: &Env<F, P, R>,
    work: &Path,
    skill: &Path,
    version: &str,
    target: &str,
) -> Result<Value, String> {
    let tag = tag(version);
    let asset = asset(version, target);
    env.releases
        .download(
            &tag,
            &[asset.clone(), SUMS.to_owned(), BUNDLE.to_owned()],
            work,
        )
        .map_err(|error| format!("取得で止まった：{error}"))?;

    let sums = env.files.read(&work.join(SUMS))?;
    let expected = expected_sha256(&sums, &asset)
        .ok_or(format!("照合で止まった：{SUMS} に {asset} の行が無い"))?;
    let actual = sha256_hex(&env.files.read_bytes(&work.join(&asset))?);
    if actual != expected {
        return Err(format!(
            "照合で止まった：{asset} の SHA-256 が {SUMS} と合わない"
        ));
    }
    env.releases
        .attest(&work.join(&asset), &work.join(BUNDLE), &tag)
        .map_err(|error| format!("照合で止まった：証明が合わない：{error}"))?;

    let unpacked = work.join("unpacked");
    env.files.create_dir(&unpacked)?;
    env.releases
        .extract(&work.join(&asset), &unpacked)
        .map_err(|error| format!("展開で止まった：{error}"))?;
    let from = unpacked.join("schema-driven").join("scripts").join("bin");
    let suffix = exe_suffix(target);
    let names: Vec<String> = ENGINE_BINARIES
        .iter()
        .map(|binary| format!("{binary}{suffix}"))
        .collect();
    if let Some(missing) = names
        .iter()
        .find(|name| !env.files.exists(&from.join(name)))
    {
        return Err(format!("展開で止まった：配布物に {missing} が無い"));
    }
    if target == host_target() {
        confirm_version(env, &from.join(&names[0]), version)?;
    }

    let bin = skill.join("scripts").join("bin");
    env.files.create_dir(&bin)?;
    let staged: Vec<(PathBuf, PathBuf)> = names
        .iter()
        .map(|name| (bin.join(format!(".{name}.new")), bin.join(name)))
        .collect();
    let copied = names.iter().zip(&staged).try_for_each(|(name, (new, _))| {
        env.files.copy(&from.join(name), new)?;
        env.files.set_executable(new)
    });
    if let Err(error) = copied {
        staged.iter().for_each(|(new, _)| {
            let _ = env.files.remove_file(new);
        });
        return Err(format!("置く段で止まった（フォルダは元のまま）：{error}"));
    }
    staged
        .iter()
        .try_for_each(|(new, to)| env.files.rename(new, to))?;

    let placed: Vec<String> = names
        .iter()
        .map(|name| format!("scripts/bin/{name}"))
        .collect();
    Ok(json!({"name": "schema-driven", "version": version, "target": target, "files": placed}))
}

/// 展開したエンジンに版を答えさせ、指した版と合うかを確かめる。
fn confirm_version<F: Files, P: Programs, R: Releases>(
    env: &Env<F, P, R>,
    engine: &Path,
    version: &str,
) -> Result<(), String> {
    let run = env.programs.run(engine, &["--version".to_owned()])?;
    let answered: Value = serde_json::from_str(&run.stdout).unwrap_or(Value::Null);
    match answered["data"]["version"].as_str() {
        Some(found) if found == version => Ok(()),
        found => Err(format!(
            "照合で止まった：展開したエンジンの版が {} で、指した版 {version} と違う",
            found.unwrap_or("（答えない）")
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{FakePrograms, FakeReleases, MemFiles};

    const ENGINE: &[u8] = b"engine-binary";
    const ASSET: &str = "schema-driven-0.1.0-x86_64-unknown-linux-musl.tar.gz";

    /// 0.1.0 の Linux 向けの配布物を持つ Release と、空のファイル。
    fn env(sha_of_asset: &str) -> Env<MemFiles, FakePrograms, FakeReleases> {
        let files = MemFiles::default();
        let releases = FakeReleases {
            files: files.clone(),
            ..FakeReleases::default()
        };
        releases.assets.borrow_mut().extend([
            (ASSET.to_owned(), b"archive".to_vec()),
            (
                SUMS.to_owned(),
                format!("{sha_of_asset}  {ASSET}\n").into_bytes(),
            ),
            (BUNDLE.to_owned(), b"{}".to_vec()),
        ]);
        releases.unpacked.borrow_mut().extend([
            (
                "schema-driven/scripts/bin/schema-driven".to_owned(),
                ENGINE.to_vec(),
            ),
            (
                "schema-driven/scripts/bin/schema-driven-mcp".to_owned(),
                ENGINE.to_vec(),
            ),
        ]);
        let programs = FakePrograms::default();
        programs.answer(
            "/tmp/work/unpacked/schema-driven/scripts/bin/schema-driven",
            "--version",
            0,
            r#"{"ok":true,"data":{"name":"schema-driven","version":"0.1.0"}}"#,
        );
        Env {
            files,
            programs,
            releases,
        }
    }

    /// "archive" の SHA-256（前もって計算した値）
    const ARCHIVE_SHA: &str = "0eb3e36bfb24dcd9bb1d1bece1531216b59539a8fde17ee80224af0653c92aa3";
    /// どの配布物にも合わない SHA-256
    const OTHER_SHA: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn a_verified_engine_is_placed_in_scripts_bin() {
        let env = env(ARCHIVE_SHA);
        let placed = install(
            &env,
            Path::new("/s/practice"),
            "0.1.0",
            "x86_64-unknown-linux-musl",
        );
        assert_eq!(
            placed,
            Ok(
                json!({"name": "schema-driven", "version": "0.1.0", "target": "x86_64-unknown-linux-musl",
                      "files": ["scripts/bin/schema-driven", "scripts/bin/schema-driven-mcp"]})
            )
        );
        assert_eq!(
            env.files.under("/s/practice"),
            vec![
                "/s/practice/scripts/bin/schema-driven",
                "/s/practice/scripts/bin/schema-driven-mcp"
            ]
        );
        assert_eq!(env.files.under("/tmp/work"), Vec::<String>::new());
    }

    #[test]
    fn a_wrong_sha256_stops_before_anything_is_placed() {
        let env = env(OTHER_SHA);
        let result = install(
            &env,
            Path::new("/s/practice"),
            "0.1.0",
            "x86_64-unknown-linux-musl",
        );
        assert_eq!(
            result,
            Err(format!(
                "照合で止まった：{ASSET} の SHA-256 が SHA256SUMS と合わない"
            ))
        );
        assert_eq!(env.files.under("/s/practice"), Vec::<String>::new());
    }

    #[test]
    fn a_failed_attestation_stops_before_anything_is_placed() {
        let env = env(ARCHIVE_SHA);
        *env.releases.attest_fails.borrow_mut() = true;
        let result = install(
            &env,
            Path::new("/s/practice"),
            "0.1.0",
            "x86_64-unknown-linux-musl",
        );
        assert_eq!(
            result,
            Err("照合で止まった：証明が合わない：証明が一致しない（試験）".to_owned())
        );
        assert_eq!(env.files.under("/s/practice"), Vec::<String>::new());
    }

    #[test]
    fn a_version_that_is_not_published_stops_at_the_download() {
        let env = env(ARCHIVE_SHA);
        let result = install(
            &env,
            Path::new("/s/practice"),
            "9.9.9",
            "x86_64-unknown-linux-musl",
        );
        assert_eq!(
            result,
            Err("取得で止まった：schema-driven-v9.9.9 に schema-driven-9.9.9-x86_64-unknown-linux-musl.tar.gz が無い".to_owned())
        );
    }

    #[test]
    fn an_engine_that_answers_another_version_is_not_placed() {
        let env = env(ARCHIVE_SHA);
        env.programs.answer(
            "/tmp/work/unpacked/schema-driven/scripts/bin/schema-driven",
            "--version",
            0,
            r#"{"ok":true,"data":{"name":"schema-driven","version":"0.0.9"}}"#,
        );
        let result = install(
            &env,
            Path::new("/s/practice"),
            "0.1.0",
            "x86_64-unknown-linux-musl",
        );
        assert_eq!(
            result,
            Err(
                "照合で止まった：展開したエンジンの版が 0.0.9 で、指した版 0.1.0 と違う".to_owned()
            )
        );
        assert_eq!(env.files.under("/s/practice"), Vec::<String>::new());
    }

    #[test]
    fn a_failed_copy_leaves_the_existing_engine_as_it_was() {
        let env = env(ARCHIVE_SHA);
        env.files
            .put("/s/practice/scripts/bin/schema-driven", "old");
        *env.files.fail_on.borrow_mut() = Some(PathBuf::from(
            "/s/practice/scripts/bin/.schema-driven-mcp.new",
        ));
        let result = install(
            &env,
            Path::new("/s/practice"),
            "0.1.0",
            "x86_64-unknown-linux-musl",
        );
        assert_eq!(
            result,
            Err("置く段で止まった（フォルダは元のまま）：/s/practice/scripts/bin/.schema-driven-mcp.new に書けない（試験）".to_owned())
        );
        assert_eq!(
            env.files.under("/s/practice"),
            vec!["/s/practice/scripts/bin/schema-driven"]
        );
        assert_eq!(
            env.files
                .text("/s/practice/scripts/bin/schema-driven")
                .as_deref(),
            Some("old")
        );
    }
}
