//! port の Releases の実装。gh で Release から取り、gh attestation で証明を照合し、tar で展開する。
//! gh が無いか、attestation を持たない古い版なら止める。照合を飛ばす経路は持たない。

use skills_creator_core::domain::release::{REPO, WORKFLOW};
use skills_creator_core::ports::Releases;
use std::path::Path;
use std::process::{Command, Output};

pub struct GhReleases;

/// gh を起動する。cache を渡せば、gh のキャッシュ（証明の信頼の起点など）をそこに置く。
/// AI ツールの sandbox では、ホームのキャッシュが読み取り専用のことがあるため。
fn gh_in(args: &[&str], cache: Option<&Path>) -> Result<Output, String> {
    let mut command = Command::new("gh");
    command.args(args);
    if let Some(cache) = cache {
        command.env("XDG_CACHE_HOME", cache);
    }
    command.output().map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => {
            "gh が見つからない。基盤のエンジンの証明を照合するには、gh（attestation が使える版）が必要".to_owned()
        }
        _ => format!("gh を起動できない：{error}"),
    })
}

fn failed(what: &str, output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    match stderr.contains("unknown command \"attestation\"") {
        true => "gh が古く、attestation を持たない。gh を新しい版に上げる".to_owned(),
        false => format!("{what}が失敗した：{}", stderr.trim()),
    }
}

impl Releases for GhReleases {
    fn download(&self, tag: &str, names: &[String], into: &Path) -> Result<(), String> {
        let into = into.to_string_lossy();
        let mut args = vec![
            "release",
            "download",
            tag,
            "--repo",
            REPO,
            "--dir",
            &into,
            "--clobber",
        ];
        names
            .iter()
            .for_each(|name| args.extend(["--pattern", name.as_str()]));
        let output = gh_in(&args, None)?;
        match output.status.success() {
            true => Ok(()),
            false => Err(failed("gh release download ", &output)),
        }
    }

    fn attest(&self, file: &Path, bundle: &Path, tag: &str) -> Result<(), String> {
        let signer = format!("{REPO}/{WORKFLOW}");
        let source = format!("refs/tags/{tag}");
        // 証明の信頼の起点のキャッシュは、作業用のフォルダ（bundle の隣）に置く
        let cache = bundle.parent().unwrap_or(Path::new(".")).join("gh-cache");
        let file = file.to_string_lossy();
        let bundle = bundle.to_string_lossy();
        let output = gh_in(
            &[
                "attestation",
                "verify",
                &file,
                "--repo",
                REPO,
                "--bundle",
                &bundle,
                "--signer-workflow",
                &signer,
                "--source-ref",
                &source,
            ],
            Some(&cache),
        )?;
        match output.status.success() {
            true => Ok(()),
            false => Err(failed("gh attestation verify ", &output)),
        }
    }

    fn extract(&self, archive: &Path, into: &Path) -> Result<(), String> {
        let output = Command::new("tar")
            .arg("-xf")
            .arg(archive)
            .arg("-C")
            .arg(into)
            .output()
            .map_err(|error| format!("tar を起動できない：{error}"))?;
        match output.status.success() {
            true => Ok(()),
            false => Err(format!(
                "tar の展開が失敗した：{}",
                String::from_utf8_lossy(&output.stderr).trim()
            )),
        }
    }
}
