//! 基盤の Skill のフォルダを作り（skill）、展開した場所で確かめる（verify）ツール。製品の実行ファイルには入らない。
//!
//! ```text
//! cargo xtask skill --out <Skill のフォルダ> [--target <target>]
//! cargo xtask verify --dir <Skill のフォルダ> [--tag schema-driven-v<版>]
//! ```
//!
//! skill は、エンジンをビルドし、実行ファイルを scripts/bin/ へ、基盤のスキーマの写しを assets/ へ置き、
//! dist.json（版 ・ target ・ abstract スキーマの名前 ・ ファイルごとの SHA-256）を書く。SKILL.md と LICENSE は触らない。
//! verify は、置いたものを SHA-256 で照合し、実行ファイルを起動して、版 ・ check-schemas ・ MCP のツールの一覧を確かめる。

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 基盤の Skill の名前。
pub const NAME: &str = "schema-driven";
/// エンジンの実行ファイル。CLI と MCP の2つ。
pub const BINARIES: [&str; 2] = ["schema-driven", "schema-driven-mcp"];
/// macOS 向けに、2つの CPU の実行ファイルを1つにまとめた target の名前。
pub const UNIVERSAL: &str = "universal2-apple-darwin";
const MAC_SIDES: [&str; 2] = ["aarch64-apple-darwin", "x86_64-apple-darwin"];
/// MCP が並べるはずのツールの名前。
pub const TOOLS: [&str; 9] = [
    "create",
    "get",
    "update",
    "delete",
    "prompt",
    "check",
    "check-schemas",
    "approve",
    "render",
];
/// tag の名前の頭。tag は `schema-driven-v<版>` である。
pub const TAG_PREFIX: &str = "schema-driven-v";
/// Cargo.toml の [workspace.package] の version。基盤の版の正本である。
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// xtask の作業。
#[derive(Debug, PartialEq)]
pub enum Task {
    Skill {
        out: PathBuf,
        target: Option<String>,
    },
    Verify {
        dir: PathBuf,
        tag: Option<String>,
    },
}

/// 引数を作業に読む。合わなければ、使い方の誤りの説明を返す。
pub fn parse(args: &[String]) -> Result<Task, String> {
    let (command, rest) = args
        .split_first()
        .ok_or("作業の名前が無い（skill か verify）")?;
    let mut flags = std::collections::BTreeMap::new();
    let mut words = rest.iter();
    while let Some(key) = words.next() {
        let name = key
            .strip_prefix("--")
            .ok_or(format!("知らない引数: {key}"))?;
        let value = words.next().ok_or(format!("--{name} に値が無い"))?;
        flags.insert(name.to_owned(), value.clone());
    }
    let allowed: &[&str] = match command.as_str() {
        "skill" => &["out", "target"],
        "verify" => &["dir", "tag"],
        other => return Err(format!("知らない作業: {other}（skill か verify）")),
    };
    if let Some(unknown) = flags.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(format!("{command} は --{unknown} を受け付けない"));
    }
    let required = |key: &str| {
        flags
            .get(key)
            .map(PathBuf::from)
            .ok_or(format!("{command} には --{key} が必要"))
    };
    Ok(match command.as_str() {
        "skill" => Task::Skill {
            out: required("out")?,
            target: flags.get("target").cloned(),
        },
        _ => Task::Verify {
            dir: required("dir")?,
            tag: flags.get("tag").cloned(),
        },
    })
}

/// 引数を読んで作業を行い、終了コード（0 正常 ／ 1 所見あり ／ 2 誤用）と、出す JSON を返す。
pub fn perform(args: &[String]) -> (i32, Value) {
    match parse(args) {
        Err(detail) => (
            2,
            json!({"ok": false, "reason": "使い方が違う", "detail": detail}),
        ),
        Ok(Task::Skill { out, target }) => match skill(&out, target.as_deref()) {
            Ok(dist) => (0, json!({"ok": true, "findings": [], "data": dist})),
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

/// バイト列の SHA-256 を、小文字の16進で返す。
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// target の実行ファイルに付く拡張子。
pub fn exe_suffix(target: &str) -> &'static str {
    if target.contains("windows") {
        ".exe"
    } else {
        ""
    }
}

/// tag の名前から版を取り出す。`schema-driven-v` で始まらなければ None。
pub fn tag_version(tag: &str) -> Option<&str> {
    tag.strip_prefix(TAG_PREFIX)
}

/// 出どころごとの版を、Cargo.toml の版と比べ、違うものを所見の文にして返す。
pub fn version_mismatches(expected: &str, sources: &[(&str, &str)]) -> Vec<String> {
    sources
        .iter()
        .filter(|(_, version)| *version != expected)
        .map(|(source, version)| {
            format!("{source} の版が {version} で、Cargo.toml の版 {expected} と違う")
        })
        .collect()
}

/// dist.json の値を作る。files は（Skill のフォルダの根からの相対パス, SHA-256）の並び。
pub fn dist(target: &str, abstracts: &[String], files: &[(String, String)]) -> Value {
    let files: Vec<Value> = files
        .iter()
        .map(|(path, sha256)| json!({"path": path, "sha256": sha256}))
        .collect();
    json!({
        "$schema": "schema-driven/dist@v1",
        "name": NAME,
        "version": VERSION,
        "target": target,
        "abstract": abstracts,
        "files": files,
    })
}

/// dist.json の値を、dist.schema.json で確かめる。合わない箇所を所見の文にして返す。
pub fn dist_errors(value: &Value) -> Result<Vec<String>, String> {
    let schema: Value = read_json(&manifest_dir().join("dist.schema.json"))?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|error| format!("dist.schema.json を読めない：{error}"))?;
    Ok(validator
        .iter_errors(value)
        .map(|error| format!("dist.json {}：{error}", error.instance_path()))
        .collect())
}

fn manifest_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// ワークスペースの根（src/schema-driven/）。
fn workspace() -> &'static Path {
    manifest_dir()
        .parent()
        .expect("xtask はワークスペースの中にある")
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("{} を読めない：{error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("{} が JSON でない：{error}", path.display()))
}

fn run(command: &mut Command) -> Result<String, String> {
    let output = command
        .output()
        .map_err(|error| format!("{command:?} を起動できない：{error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{command:?} が失敗した（{}）：{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn cargo() -> Command {
    let mut command = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command.current_dir(workspace());
    command
}

/// この環境の target（rustc -vV の host）。
fn host() -> Result<String, String> {
    let info = run(Command::new("rustc").arg("-vV"))?;
    info.lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::to_owned)
        .ok_or("rustc -vV に host が無い".to_owned())
}

/// cargo がビルドの出力を置くフォルダ（CARGO_TARGET_DIR や設定で変わるので、cargo metadata に聞く）。
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

/// エンジンを target 向けにビルドし、実行ファイルのあるフォルダを返す。
/// universal2-apple-darwin は、2つの CPU 向けにビルドしてから lipo で1つにまとめる。
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
            "schema-driven-adapters",
            "--target",
            side,
        ]))?;
    }
    if target != UNIVERSAL {
        return Ok(release(target));
    }
    let out = release(UNIVERSAL);
    fs::create_dir_all(&out).map_err(|error| format!("{} を作れない：{error}", out.display()))?;
    for binary in BINARIES {
        run(Command::new("lipo")
            .arg("-create")
            .arg("-output")
            .arg(out.join(binary))
            .args(MAC_SIDES.map(|side| release(side).join(binary))))?;
    }
    Ok(out)
}

/// ファイルを写し、写した先のバイト列の SHA-256 を返す。
fn place(from: &Path, to: &Path) -> Result<String, String> {
    fs::copy(from, to)
        .map_err(|error| format!("{} を {} へ写せない：{error}", from.display(), to.display()))?;
    let bytes = fs::read(to).map_err(|error| format!("{} を読めない：{error}", to.display()))?;
    Ok(sha256_hex(&bytes))
}

fn fresh(dir: &Path) -> Result<(), String> {
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir).map_err(|error| format!("{} を作れない：{error}", dir.display()))
}

/// Skill のフォルダを作る。ビルド → scripts/bin/ と assets/ へ置く → dist.json を書く。書いた dist.json の値を返す。
pub fn skill(out: &Path, target: Option<&str>) -> Result<Value, String> {
    let target = match target {
        Some(target) => target.to_owned(),
        None => host()?,
    };
    let built = build(&target)?;
    let suffix = exe_suffix(&target);
    let mut files = Vec::new();

    let bin = out.join("scripts/bin");
    fresh(&bin)?;
    for binary in BINARIES {
        let name = format!("{binary}{suffix}");
        let sha256 = place(&built.join(&name), &bin.join(&name))?;
        files.push((format!("scripts/bin/{name}"), sha256));
    }

    let assets = out.join("assets");
    fresh(&assets)?;
    let schemas = workspace().join("crates/core/schemas");
    let mut sources: Vec<PathBuf> = fs::read_dir(&schemas)
        .map_err(|error| format!("{} を読めない：{error}", schemas.display()))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    sources.sort();
    let mut abstracts = Vec::new();
    for source in &sources {
        let name = source
            .file_name()
            .expect("ファイルの名前")
            .to_string_lossy();
        let id = read_json(source)?["$id"]
            .as_str()
            .ok_or(format!("{} に $id が無い", source.display()))?
            .to_owned();
        let sha256 = place(source, &assets.join(name.as_ref()))?;
        files.push((format!("assets/{name}"), sha256));
        abstracts.push(id);
    }

    let value = dist(&target, &abstracts, &files);
    let errors = dist_errors(&value)?;
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    let text = serde_json::to_string_pretty(&value).expect("JSON にできる") + "\n";
    let path = out.join("dist.json");
    fs::write(&path, text).map_err(|error| format!("{} を書けない：{error}", path.display()))?;
    Ok(value)
}

/// MCP の実行ファイルを起動し、serverInfo の版と、ツールの名前の一覧を返す。
fn mcp(binary: &Path) -> Result<(String, Vec<String>), String> {
    use rmcp::transport::TokioChildProcess;
    use rmcp::ServiceExt;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("tokio を始められない：{error}"))?;
    runtime.block_on(async {
        let transport = TokioChildProcess::new(tokio::process::Command::new(binary))
            .map_err(|error| format!("{} を起動できない：{error}", binary.display()))?;
        let client = ()
            .serve(transport)
            .await
            .map_err(|error| format!("MCP の初期化が終わらない：{error}"))?;
        let version = client
            .peer_info()
            .and_then(|info| info.server_info.clone())
            .map(|server| server.version)
            .unwrap_or_default();
        let tools = client
            .list_all_tools()
            .await
            .map_err(|error| format!("tools/list が失敗した：{error}"))?
            .iter()
            .map(|tool| tool.name.to_string())
            .collect();
        let _ = client.cancel().await;
        Ok((version, tools))
    })
}

/// --skill の Skill のフォルダのスキーマを名前（$id）で引いて、インスタンスを検査できるかを確かめる。
/// Windows では経路の区切りが `\` なので、ここでスキーマの URI が壊れたことがある。
fn skill_schema_by_name(cli: &Path) -> Result<Vec<String>, String> {
    let work = std::env::temp_dir().join(format!("schema-driven-verify-{}", std::process::id()));
    let _ = fs::remove_dir_all(&work);
    let write = |rel: &str, value: Value| {
        let path = work.join(rel);
        fs::create_dir_all(path.parent().expect("親のフォルダ"))
            .and_then(|_| fs::write(&path, value.to_string()))
            .map_err(|error| format!("{} を書けない：{error}", path.display()))
    };
    write(
        "example/assets/t.schema.json",
        json!({"$schema": "https://json-schema.org/draft/2020-12/schema", "$id": "example/t@v1",
               "type": "object", "properties": {"$schema": {"type": "string"}}}),
    )?;
    write("instances/a.json", json!({"$schema": "example/t@v1"}))?;
    let answer: Value = serde_json::from_str(&run(Command::new(cli)
        .arg("--skill")
        .arg(work.join("example"))
        .arg("check")
        .arg("--dir")
        .arg(work.join("instances")))?)
    .map_err(|error| format!("check の出力が JSON でない：{error}"))?;
    let _ = fs::remove_dir_all(&work);
    let errors = &answer["instances"][0]["errors"];
    Ok(match answer["ok"] == json!(true) && errors == &json!([]) {
        true => vec![],
        false => vec![format!(
            "--skill のスキーマを名前で引いて検査できない：{answer}"
        )],
    })
}

/// 展開した Skill のフォルダを確かめ、所見の文を返す。0件なら合格である。
pub fn verify(dir: &Path, tag: Option<&str>) -> Result<Vec<String>, String> {
    let recorded = read_json(&dir.join("dist.json"))?;
    let mut findings = dist_errors(&recorded)?;
    if !findings.is_empty() {
        return Ok(findings);
    }

    for file in recorded["files"].as_array().into_iter().flatten() {
        let path = file["path"].as_str().unwrap_or_default();
        let actual = fs::read(dir.join(path))
            .map(|bytes| sha256_hex(&bytes))
            .unwrap_or_else(|error| format!("読めない（{error}）"));
        if file["sha256"].as_str() != Some(actual.as_str()) {
            findings.push(format!("{path} の SHA-256 が dist.json と違う"));
        }
    }

    let suffix = exe_suffix(recorded["target"].as_str().unwrap_or_default());
    let cli = dir.join(format!("scripts/bin/{}{suffix}", BINARIES[0]));
    let answered: Value = serde_json::from_str(&run(Command::new(&cli).arg("--version"))?)
        .map_err(|error| format!("--version の出力が JSON でない：{error}"))?;
    let checked: Value = serde_json::from_str(&run(Command::new(&cli)
        .arg("check-schemas")
        .arg("--dir")
        .arg(dir.join("assets")))?)
    .map_err(|error| format!("check-schemas の出力が JSON でない：{error}"))?;
    if checked["ok"] != json!(true) {
        findings.push(format!("check-schemas が assets/ を読めない：{checked}"));
    }

    findings.extend(skill_schema_by_name(&cli)?);

    let (served, tools) = mcp(&dir.join(format!("scripts/bin/{}{suffix}", BINARIES[1])))?;
    if tools != TOOLS {
        findings.push(format!("MCP のツールの一覧が違う：{tools:?}"));
    }

    let mut sources = vec![
        (
            "dist.json",
            recorded["version"].as_str().unwrap_or_default(),
        ),
        (
            "--version",
            answered["data"]["version"].as_str().unwrap_or_default(),
        ),
        ("serverInfo", served.as_str()),
    ];
    let from_tag = tag.map(|tag| tag_version(tag).unwrap_or(tag));
    sources.extend(from_tag.map(|version| ("tag", version)));
    findings.extend(version_mismatches(VERSION, &sources));
    Ok(findings)
}

#[cfg(test)]
mod tests {
    //! 置き方と比べ方を、前もって計算した値の表で確かめる。ビルドと起動（skill と verify の本体）は、
    //! ワークフローが3つの OS で動かして確かめる。

    use super::*;

    fn words(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn parse_reads_the_two_tasks_and_rejects_the_rest() {
        let cases = [
            (
                vec!["skill", "--out", "o"],
                Ok(Task::Skill {
                    out: "o".into(),
                    target: None,
                }),
            ),
            (
                vec!["skill", "--out", "o", "--target", "t"],
                Ok(Task::Skill {
                    out: "o".into(),
                    target: Some("t".into()),
                }),
            ),
            (
                vec!["verify", "--dir", "d", "--tag", "schema-driven-v0.1.0"],
                Ok(Task::Verify {
                    dir: "d".into(),
                    tag: Some("schema-driven-v0.1.0".into()),
                }),
            ),
            (vec!["skill"], Err("skill には --out が必要".to_owned())),
            (
                vec!["verify", "--out", "o"],
                Err("verify は --out を受け付けない".to_owned()),
            ),
            (
                vec!["build"],
                Err("知らない作業: build（skill か verify）".to_owned()),
            ),
            (vec!["skill", "out"], Err("知らない引数: out".to_owned())),
            (vec!["skill", "--out"], Err("--out に値が無い".to_owned())),
        ];
        let got: Vec<_> = cases.iter().map(|(args, _)| parse(&words(args))).collect();
        let want: Vec<_> = cases.into_iter().map(|(_, want)| want).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn sha256_matches_the_published_test_vector() {
        // FIPS 180-2 の付録の "abc"
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn only_windows_targets_get_the_exe_suffix() {
        let targets = [
            "x86_64-pc-windows-msvc",
            "x86_64-unknown-linux-musl",
            UNIVERSAL,
        ];
        let got: Vec<_> = targets.iter().map(|target| exe_suffix(target)).collect();
        assert_eq!(got, vec![".exe", "", ""]);
    }

    #[test]
    fn tag_version_takes_the_part_after_the_prefix() {
        let tags = ["schema-driven-v0.1.0", "v0.1.0", "skills-creator-v0.1.0"];
        let got: Vec<_> = tags.iter().map(|tag| tag_version(tag)).collect();
        assert_eq!(got, vec![Some("0.1.0"), None, None]);
    }

    #[test]
    fn version_mismatches_names_every_source_that_differs() {
        let sources = [
            ("dist.json", "1.2.3"),
            ("--version", "1.2.4"),
            ("serverInfo", "1.2.3"),
            ("tag", "1.3.0"),
        ];
        assert_eq!(
            version_mismatches("1.2.3", &sources),
            vec![
                "--version の版が 1.2.4 で、Cargo.toml の版 1.2.3 と違う",
                "tag の版が 1.3.0 で、Cargo.toml の版 1.2.3 と違う",
            ]
        );
    }

    #[test]
    fn a_built_dist_follows_its_schema() {
        let value = dist(
            "x86_64-unknown-linux-musl",
            &["schema-driven/abstract@v1".to_owned()],
            &[(
                "scripts/bin/schema-driven".to_owned(),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_owned(),
            )],
        );
        assert_eq!(dist_errors(&value), Ok(vec![]));
        assert_eq!(value["version"], VERSION);
    }

    #[test]
    fn a_dist_with_a_short_hash_or_an_extra_field_is_rejected() {
        let short = json!({"$schema": "schema-driven/dist@v1", "name": NAME, "version": "0.1.0", "target": "t",
                           "abstract": [], "files": [{"path": "p", "sha256": "abc"}]});
        let extra = json!({"$schema": "schema-driven/dist@v1", "name": NAME, "version": "0.1.0", "target": "t",
                           "abstract": [], "files": [], "note": "x"});
        let counts: Vec<usize> = [short, extra]
            .iter()
            .map(|value| dist_errors(value).unwrap().len())
            .collect();
        assert_eq!(counts, vec![1, 1]);
    }
}
