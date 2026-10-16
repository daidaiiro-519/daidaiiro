//! skills-creator の Skill のフォルダを作り（skill）、確かめる（verify）ツール。製品の実行ファイルには入らない。
//!
//! ```text
//! cargo xtask skill --out <Skill のフォルダ> --engine <基盤の版> [--target <target>]
//! cargo xtask verify --dir <Skill のフォルダ> [--tag skills-creator-v<版>]
//! ```
//!
//! skill は、skills-creator をビルドして scripts/bin/ に置き、基盤のエンジンはビルドした skills-creator の engine で入れる。
//! SKILL.md ・ LICENSE ・ assets/ は git で追跡しているものをそのまま使い、触らない。
//! verify は、版 ・ 自分のフォルダの check ・ 練習用の Skill の scaffold と check を確かめる。公開された基盤を取るので、ネットワークを使う。

use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// macOS 向けに、2つの CPU の実行ファイルを1つにまとめた target の名前。
pub const UNIVERSAL: &str = "universal2-apple-darwin";
const MAC_SIDES: [&str; 2] = ["aarch64-apple-darwin", "x86_64-apple-darwin"];
/// tag の名前の頭。
pub const TAG_PREFIX: &str = "skills-creator-v";
/// Cargo.toml の [workspace.package] の version。skills-creator の版の正本である。
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// 練習用の Skill を scaffold した直後に、不合格になるはずの項目（記入欄が残る SKILL.md とスキーマ）。
pub const FRESH_FAILS: [&str; 2] = ["skill.placeholders", "schemas.check"];

/// xtask の作業。
#[derive(Debug, PartialEq)]
pub enum Task {
    Skill {
        out: PathBuf,
        engine: String,
        target: Option<String>,
    },
    Verify {
        dir: PathBuf,
        tag: Option<String>,
    },
}

/// 引数を作業に読む。
pub fn parse(args: &[String]) -> Result<Task, String> {
    let (command, rest) = args
        .split_first()
        .ok_or("作業の名前が無い（skill か verify）")?;
    let mut flags = BTreeMap::new();
    let mut words = rest.iter();
    while let Some(key) = words.next() {
        let name = key
            .strip_prefix("--")
            .ok_or(format!("知らない引数: {key}"))?;
        let value = words.next().ok_or(format!("--{name} に値が無い"))?;
        flags.insert(name.to_owned(), value.clone());
    }
    let allowed: &[&str] = match command.as_str() {
        "skill" => &["out", "engine", "target"],
        "verify" => &["dir", "tag"],
        other => return Err(format!("知らない作業: {other}（skill か verify）")),
    };
    if let Some(unknown) = flags.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(format!("{command} は --{unknown} を受け付けない"));
    }
    let required = |key: &str| {
        flags
            .get(key)
            .cloned()
            .ok_or(format!("{command} には --{key} が必要"))
    };
    Ok(match command.as_str() {
        "skill" => Task::Skill {
            out: PathBuf::from(required("out")?),
            engine: required("engine")?,
            target: flags.get("target").cloned(),
        },
        _ => Task::Verify {
            dir: PathBuf::from(required("dir")?),
            tag: flags.get("tag").cloned(),
        },
    })
}

/// 引数を読んで作業を行い、終了コード（0 正常 ／ 1 所見あり ／ 2 誤用）と出す JSON を返す。
pub fn perform(args: &[String]) -> (i32, Value) {
    match parse(args) {
        Err(detail) => (
            2,
            json!({"ok": false, "reason": "使い方が違う", "detail": detail}),
        ),
        Ok(Task::Skill {
            out,
            engine,
            target,
        }) => match skill(&out, &engine, target.as_deref()) {
            Ok(data) => (0, json!({"ok": true, "findings": [], "data": data})),
            Err(error) => (1, json!({"ok": false, "findings": [error]})),
        },
        Ok(Task::Verify { dir, tag }) => match verify(&dir, tag.as_deref()) {
            Ok(findings) => (
                i32::from(!findings.is_empty()),
                json!({"ok": findings.is_empty(), "findings": findings}),
            ),
            Err(error) => (1, json!({"ok": false, "findings": [error]})),
        },
    }
}

/// 基盤の配布物の target。skills-creator の target と同じ OS の配布物を選ぶ。
pub fn engine_target(target: &str) -> &'static str {
    if target.contains("windows") {
        "x86_64-pc-windows-msvc"
    } else if target.contains("apple") {
        UNIVERSAL
    } else {
        "x86_64-unknown-linux-musl"
    }
}

fn exe_suffix(target: &str) -> &'static str {
    if target.contains("windows") {
        ".exe"
    } else {
        ""
    }
}

fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask はワークスペースの中にある")
}

fn run(command: &mut Command) -> Result<String, String> {
    let output = command
        .output()
        .map_err(|error| format!("{command:?} を起動できない：{error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{command:?} が失敗した（{}）：{}{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// 起動して、終了コードと出力の JSON を返す（失敗しても返す）。
fn call(program: &Path, args: &[&str]) -> Result<(i32, Value), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("{} を起動できない：{error}", program.display()))?;
    let value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    Ok((output.status.code().unwrap_or(-1), value))
}

fn cargo() -> Command {
    let mut command = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command.current_dir(workspace());
    command
}

fn host() -> Result<String, String> {
    let info = run(Command::new("rustc").arg("-vV"))?;
    info.lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::to_owned)
        .ok_or("rustc -vV に host が無い".to_owned())
}

fn target_dir() -> Result<PathBuf, String> {
    let metadata: Value = serde_json::from_str(&run(cargo().args([
        "metadata",
        "--no-deps",
        "--format-version",
        "1",
    ]))?)
    .map_err(|error| format!("cargo metadata の出力が JSON でない：{error}"))?;
    metadata["target_directory"]
        .as_str()
        .map(PathBuf::from)
        .ok_or("cargo metadata に target_directory が無い".to_owned())
}

/// skills-creator を target 向けにビルドし、実行ファイルの場所を返す。universal2 は lipo でまとめる。
fn build(target: &str) -> Result<PathBuf, String> {
    let output = target_dir()?;
    let release = |target: &str| output.join(target).join("release");
    let sides: Vec<&str> = if target == UNIVERSAL {
        MAC_SIDES.to_vec()
    } else {
        vec![target]
    };
    for side in &sides {
        run(cargo().args([
            "build",
            "--release",
            "--locked",
            "--package",
            "skills-creator-adapters",
            "--target",
            side,
        ]))?;
    }
    if target != UNIVERSAL {
        return Ok(release(target).join(format!("skills-creator{}", exe_suffix(target))));
    }
    let out = release(UNIVERSAL);
    fs::create_dir_all(&out).map_err(|error| format!("{} を作れない：{error}", out.display()))?;
    run(Command::new("lipo")
        .arg("-create")
        .arg("-output")
        .arg(out.join("skills-creator"))
        .args(MAC_SIDES.map(|side| release(side).join("skills-creator"))))?;
    Ok(out.join("skills-creator"))
}

/// skills-creator の Skill のフォルダを作る。ビルド → scripts/bin/ に置く → skills-creator engine で基盤のエンジンを入れる。
pub fn skill(out: &Path, engine: &str, target: Option<&str>) -> Result<Value, String> {
    let target = match target {
        Some(target) => target.to_owned(),
        None => host()?,
    };
    let built = build(&target)?;
    let bin = out.join("scripts").join("bin");
    fs::create_dir_all(&bin).map_err(|error| format!("{} を作れない：{error}", bin.display()))?;
    let placed = bin.join(format!("skills-creator{}", exe_suffix(&target)));
    fs::copy(&built, &placed)
        .map_err(|error| format!("{} を置けない：{error}", placed.display()))?;
    let out_text = out.to_string_lossy();
    let (code, installed) = call(
        &placed,
        &[
            "engine",
            &out_text,
            "--version",
            engine,
            "--target",
            engine_target(&target),
        ],
    )?;
    if code != 0 {
        return Err(format!("基盤のエンジンを入れられない：{installed}"));
    }
    Ok(json!({"skills-creator": VERSION, "target": target, "engine": installed["data"]}))
}

/// check の結果から、不合格か判定不能の項目の id を並べる。
pub fn failing(report: &Value) -> Vec<String> {
    report["items"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|item| matches!(item["result"].as_str(), Some("fail") | Some("unknown")))
        .filter_map(|item| item["id"].as_str().map(str::to_owned))
        .collect()
}

/// 作った Skill のフォルダを確かめ、所見の文を返す。0件なら合格である。
pub fn verify(dir: &Path, tag: Option<&str>) -> Result<Vec<String>, String> {
    let mut findings = Vec::new();
    let suffix = if cfg!(windows) { ".exe" } else { "" };
    let creator = dir
        .join("scripts")
        .join("bin")
        .join(format!("skills-creator{suffix}"));

    let (_, answered) = call(&creator, &["--version"])?;
    let version = answered["data"]["version"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    if version != VERSION {
        findings.push(format!(
            "--version の版が {version} で、Cargo.toml の版 {VERSION} と違う"
        ));
    }
    if let Some(tag) = tag {
        let from_tag = tag.strip_prefix(TAG_PREFIX).unwrap_or(tag);
        if from_tag != VERSION {
            findings.push(format!(
                "tag の版が {from_tag} で、Cargo.toml の版 {VERSION} と違う"
            ));
        }
    }

    let dir_text = dir.to_string_lossy();
    let (_, own) = call(&creator, &["check", &dir_text])?;
    let own_failing = failing(&own);
    if !own_failing.is_empty() {
        findings.push(format!(
            "skills-creator 自身の check で合格しない項目：{}（{own}）",
            own_failing.join(" ・ ")
        ));
    }

    let work = std::env::temp_dir().join(format!("skills-creator-verify-{}", std::process::id()));
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).map_err(|error| format!("{} を作れない：{error}", work.display()))?;
    let practice = work.join("practice");
    let practice_text = practice.to_string_lossy();
    let (code, made) = call(&creator, &["scaffold", &practice_text])?;
    if code != 0 {
        findings.push(format!("練習用の Skill を scaffold できない：{made}"));
    } else {
        let (_, fresh) = call(&creator, &["check", &practice_text])?;
        let fresh_failing = failing(&fresh);
        if fresh_failing != FRESH_FAILS {
            findings.push(format!(
                "scaffold の直後に合格しない項目が {:?} で、{FRESH_FAILS:?} だけのはず（{fresh}）",
                fresh_failing
            ));
        }
    }
    let _ = fs::remove_dir_all(&work);
    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn parse_reads_the_two_tasks_and_rejects_the_rest() {
        let cases = [
            (
                vec!["skill", "--out", "o", "--engine", "0.1.0"],
                Ok(Task::Skill {
                    out: "o".into(),
                    engine: "0.1.0".into(),
                    target: None,
                }),
            ),
            (
                vec!["verify", "--dir", "d", "--tag", "skills-creator-v0.1.0"],
                Ok(Task::Verify {
                    dir: "d".into(),
                    tag: Some("skills-creator-v0.1.0".into()),
                }),
            ),
            (
                vec!["skill", "--out", "o"],
                Err("skill には --engine が必要".to_owned()),
            ),
            (
                vec!["verify", "--out", "o"],
                Err("verify は --out を受け付けない".to_owned()),
            ),
            (
                vec!["build"],
                Err("知らない作業: build（skill か verify）".to_owned()),
            ),
        ];
        let got: Vec<_> = cases.iter().map(|(args, _)| parse(&words(args))).collect();
        let want: Vec<_> = cases.into_iter().map(|(_, want)| want).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn the_engine_target_follows_the_os_of_the_build() {
        let got: Vec<&str> = [
            "x86_64-unknown-linux-gnu",
            "x86_64-unknown-linux-musl",
            "x86_64-pc-windows-msvc",
            UNIVERSAL,
        ]
        .iter()
        .map(|target| engine_target(target))
        .collect();
        assert_eq!(
            got,
            vec![
                "x86_64-unknown-linux-musl",
                "x86_64-unknown-linux-musl",
                "x86_64-pc-windows-msvc",
                UNIVERSAL
            ]
        );
    }

    #[test]
    fn failing_lists_fail_and_unknown_but_not_skip() {
        let report = json!({"items": [
            {"id": "a", "result": "pass"}, {"id": "b", "result": "fail"},
            {"id": "c", "result": "skip"}, {"id": "d", "result": "unknown"}]});
        assert_eq!(failing(&report), vec!["b", "d"]);
    }
}
