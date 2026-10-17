//! brainstorming の Skill のフォルダを作り（skill）、確かめる（verify）ツール。製品の実行ファイルには入らない。
//!
//! ```text
//! cargo xtask skill --out <Skill のフォルダ> --creator <skills-creator の実行ファイル> --engine <基盤の版> [--target <target>]
//! cargo xtask verify --dir <Skill のフォルダ> [--tag brainstorming-v<版>] [--creator <skills-creator の実行ファイル>]
//! ```
//!
//! skill は、brainstorming をビルドして scripts/bin/ に置き、基盤のエンジンは skills-creator の engine で入れる。
//! SKILL.md ・ LICENSE ・ assets/ は git で追跡しているものをそのまま使い、触らない。
//! verify は、版 ・ トークンの正本との一致 ・ スキーマ ・ 練習のボードの inspect ・ check ・ advance ・ render ・ serve の受け口を確かめる。
//! --creator を渡すと、skills-creator の check も通す。

use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// macOS 向けに、2つの CPU の実行ファイルを1つにまとめた target の名前。
pub const UNIVERSAL: &str = "universal2-apple-darwin";
const MAC_SIDES: [&str; 2] = ["aarch64-apple-darwin", "x86_64-apple-darwin"];
/// tag の名前の頭。
pub const TAG_PREFIX: &str = "brainstorming-v";
/// Cargo.toml の [workspace.package] の version。brainstorming の版の正本である。
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// 練習のボード。
const PRACTICE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/fixtures/practice.json"
));

/// xtask の作業。
#[derive(Debug, PartialEq)]
pub enum Task {
    Skill {
        out: PathBuf,
        creator: PathBuf,
        engine: String,
        target: Option<String>,
    },
    Verify {
        dir: PathBuf,
        tag: Option<String>,
        creator: Option<PathBuf>,
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
        "skill" => &["out", "creator", "engine", "target"],
        "verify" => &["dir", "tag", "creator"],
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
            creator: PathBuf::from(required("creator")?),
            engine: required("engine")?,
            target: flags.get("target").cloned(),
        },
        _ => Task::Verify {
            dir: PathBuf::from(required("dir")?),
            tag: flags.get("tag").cloned(),
            creator: flags.get("creator").map(PathBuf::from),
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
            creator,
            engine,
            target,
        }) => match skill(&out, &creator, &engine, target.as_deref()) {
            Ok(data) => (0, json!({"ok": true, "findings": [], "data": data})),
            Err(error) => (1, json!({"ok": false, "findings": [error]})),
        },
        Ok(Task::Verify { dir, tag, creator }) => {
            match verify(&dir, tag.as_deref(), creator.as_deref()) {
                Ok(findings) => (
                    i32::from(!findings.is_empty()),
                    json!({"ok": findings.is_empty(), "findings": findings}),
                ),
                Err(error) => (1, json!({"ok": false, "findings": [error]})),
            }
        }
    }
}

/// 基盤の配布物の target。brainstorming の target と同じ OS の配布物を選ぶ。
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
fn call(program: &Path, args: &[&str], cwd: Option<&Path>) -> Result<(i32, Value), String> {
    let mut command = Command::new(program);
    command.args(args).stdin(Stdio::null());
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    let output = command
        .output()
        .map_err(|error| format!("{} を起動できない：{error}", program.display()))?;
    let value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&output.stdout)}));
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

/// brainstorming を target 向けにビルドし、実行ファイルの場所を返す。universal2 は lipo でまとめる。
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
            "brainstorming-adapters",
            "--target",
            side,
        ]))?;
    }
    if target != UNIVERSAL {
        return Ok(release(target).join(format!("brainstorming{}", exe_suffix(target))));
    }
    let out = release(UNIVERSAL);
    fs::create_dir_all(&out).map_err(|error| format!("{} を作れない：{error}", out.display()))?;
    run(Command::new("lipo")
        .arg("-create")
        .arg("-output")
        .arg(out.join("brainstorming"))
        .args(MAC_SIDES.map(|side| release(side).join("brainstorming"))))?;
    Ok(out.join("brainstorming"))
}

/// brainstorming の Skill のフォルダを作る。ビルド → scripts/bin/ に置く → skills-creator engine で基盤のエンジンを入れる。
pub fn skill(
    out: &Path,
    creator: &Path,
    engine: &str,
    target: Option<&str>,
) -> Result<Value, String> {
    let target = match target {
        Some(target) => target.to_owned(),
        None => host()?,
    };
    let built = build(&target)?;
    let bin = out.join("scripts").join("bin");
    fs::create_dir_all(&bin).map_err(|error| format!("{} を作れない：{error}", bin.display()))?;
    let placed = bin.join(format!("brainstorming{}", exe_suffix(&target)));
    fs::copy(&built, &placed)
        .map_err(|error| format!("{} を置けない：{error}", placed.display()))?;
    let out_text = out.to_string_lossy();
    let (code, installed) = call(
        creator,
        &[
            "engine",
            &out_text,
            "--version",
            engine,
            "--target",
            engine_target(&target),
        ],
        None,
    )?;
    if code != 0 {
        return Err(format!("基盤のエンジンを入れられない：{installed}"));
    }
    Ok(json!({"brainstorming": VERSION, "target": target, "engine": installed["data"]}))
}

/// check の結果から、不合格か判定不能の項目の id を並べる（skills-creator の check の形）。
pub fn failing(report: &Value) -> Vec<String> {
    report["items"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|item| matches!(item["result"].as_str(), Some("fail") | Some("unknown")))
        .filter_map(|item| item["id"].as_str().map(str::to_owned))
        .collect()
}

/// 基盤の check の出力のうち、合格でないものの数。
pub fn problems(report: &Value) -> usize {
    let errors: usize = report["instances"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|item| {
            item["errors"].as_array().map_or(0, Vec::len)
                + item["unfilled"].as_array().map_or(0, Vec::len)
        })
        .sum();
    let findings = report["findings"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|f| f["status"] != "合格")
        .count();
    errors + findings
}

/// HTTP の要求を1つ送り、応答の状態の番号とヘッダーを返す。
fn http(port: u16, raw: &str) -> Result<(u16, Vec<String>), String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .map_err(|error| format!("serve につながらない：{error}"))?;
    stream
        .write_all(raw.as_bytes())
        .map_err(|error| error.to_string())?;
    let mut reader = BufReader::new(stream);
    let mut status = String::new();
    reader
        .read_line(&mut status)
        .map_err(|error| error.to_string())?;
    let code = status
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);
    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        if reader
            .read_line(&mut line)
            .map_err(|error| error.to_string())?
            == 0
            || line.trim().is_empty()
        {
            break;
        }
        headers.push(line.trim().to_owned());
    }
    let mut rest = Vec::new();
    let _ = reader.read_to_end(&mut rest);
    Ok((code, headers))
}

/// serve を起動して、合言葉 ・ cookie ・ Host ・ Origin を確かめる。
fn verify_serve(entry: &Path, root: &Path, findings: &mut Vec<String>) -> Result<(), String> {
    let mut child = Command::new(entry)
        .args(["serve", "--dir", &root.to_string_lossy(), "--port", "0"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("serve を起動できない：{error}"))?;
    let mut first = String::new();
    if let Some(stdout) = child.stdout.take() {
        let _ = BufReader::new(stdout).read_line(&mut first);
    }
    let started: Value = serde_json::from_str(&first).unwrap_or(Value::Null);
    let url = started["data"]["url"].as_str().unwrap_or("").to_owned();
    let port = started["data"]["port"].as_u64().unwrap_or(0) as u16;
    let token = url.split("k=").nth(1).unwrap_or("").to_owned();
    let host = format!("Host: 127.0.0.1:{port}\r\n");
    let checks: Result<Vec<(String, u16, u16)>, String> = (|| {
        let cookie = format!("Cookie: bs{port}={token}\r\n");
        Ok(vec![
            ("合言葉の無い要求を断る".to_owned(), http(port, &format!("GET /dashboard.html HTTP/1.1\r\n{host}\r\n"))?.0, 403),
            ("合言葉を cookie に移す".to_owned(), http(port, &format!("GET /?k={token} HTTP/1.1\r\n{host}\r\n"))?.0, 303),
            ("cookie があればページを配る".to_owned(), http(port, &format!("GET /dashboard.html HTTP/1.1\r\n{host}{cookie}\r\n"))?.0, 200),
            ("Host が違えば断る".to_owned(), http(port, &format!("GET /dashboard.html HTTP/1.1\r\nHost: evil.example:{port}\r\n{cookie}\r\n"))?.0, 403),
            (
                "Origin が違う POST を断る".to_owned(),
                http(port, &format!("POST /api/answer HTTP/1.1\r\n{host}{cookie}Origin: http://evil.example\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{{}}"))?.0,
                403,
            ),
        ])
    })();
    let _ = child.kill();
    let _ = child.wait();
    if !url.starts_with("http://127.0.0.1:") {
        findings.push(format!("serve が 127.0.0.1 の URL を返さない：{first}"));
    }
    for (what, got, want) in checks? {
        if got != want {
            findings.push(format!("serve：{what}（{got} が返った。{want} のはず）"));
        }
    }
    Ok(())
}

/// 作った Skill のフォルダを確かめ、所見の文を返す。0件なら合格である。
pub fn verify(
    dir: &Path,
    tag: Option<&str>,
    creator: Option<&Path>,
) -> Result<Vec<String>, String> {
    let mut findings = Vec::new();
    let suffix = if cfg!(windows) { ".exe" } else { "" };
    let entry = dir
        .join("scripts")
        .join("bin")
        .join(format!("brainstorming{suffix}"));

    let (_, answered) = call(&entry, &["--version"], None)?;
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
    let (code, same) = call(&entry, &["tokens", "--check", "1"], None)?;
    if code != 0 {
        findings.push(format!("design-system.json がトークンの正本と違う：{same}"));
    }
    let assets = dir.join("assets");
    let (_, schemas) = call(
        &entry,
        &["check-schemas", "--dir", &assets.to_string_lossy()],
        None,
    )?;
    if schemas["findings"].as_array().is_none_or(|f| !f.is_empty()) {
        findings.push(format!(
            "assets/ のスキーマが基盤の契約を満たさない：{schemas}"
        ));
    }
    if let Some(creator) = creator {
        let (_, report) = call(creator, &["check", &dir.to_string_lossy()], None)?;
        let failed = failing(&report);
        if !failed.is_empty() {
            findings.push(format!(
                "skills-creator の check で合格しない項目：{}（{report}）",
                failed.join(" ・ ")
            ));
        }
    }

    // 練習のボードで、inspect ・ check ・ advance ・ render ・ serve を通す
    let work = std::env::temp_dir().join(format!("brainstorming-verify-{}", std::process::id()));
    let _ = fs::remove_dir_all(&work);
    let root = work.join(".brainstorming");
    fs::create_dir_all(&root).map_err(|error| format!("{} を作れない：{error}", root.display()))?;
    let board = root.join("practice.json");
    fs::write(&board, PRACTICE)
        .map_err(|error| format!("{} を書けない：{error}", board.display()))?;
    let board_text = board.to_string_lossy().into_owned();
    let root_text = root.to_string_lossy().into_owned();
    let (code, inspected) = call(&entry, &["inspect", "--board", &board_text], None)?;
    if code != 0 {
        findings.push(format!("練習のボードの inspect が0件でない：{inspected}"));
    }
    let (_, checked) = call(&entry, &["check", "--dir", &root_text], None)?;
    if problems(&checked) != 0 || checked["ok"] != true {
        findings.push(format!("練習のボードの check が合格しない：{checked}"));
    }
    let (code, advanced) = call(&entry, &["advance", "--board", &board_text], None)?;
    if code != 0 || !root.join("practice").join("rounds").join("1.json").exists() {
        findings.push(format!("練習のボードを advance できない：{advanced}"));
    }
    let (code, rendered) = call(&entry, &["render", "--dir", &root_text], None)?;
    for page in ["practice.html", "dashboard.html"] {
        if code != 0 || !root.join("site").join(page).exists() {
            findings.push(format!("render が {page} を描かない：{rendered}"));
        }
    }
    let page = fs::read_to_string(root.join("site").join("practice.html")).unwrap_or_default();
    let tabs: Vec<&str> = ["figure", "screen", "code", "diff", "tree", "run"]
        .into_iter()
        .filter(|kind| page.contains(&format!("data-kind=\"{kind}\"")))
        .collect();
    if tabs != ["tree", "run"] {
        findings.push(format!(
            "在る種類だけがタブになっていない：{tabs:?}（tree と run のはず）"
        ));
    }
    if page.contains("VERIFY-NOT-RENDERED") {
        findings.push("書き手の検証の記録（verification）がページに出ている".to_owned());
    }
    // 前の回の記録は、次の回へ進めても1バイトも変わらない
    let first = root.join("practice").join("rounds").join("1.json");
    let before = fs::read(&first).unwrap_or_default();
    let (code, again) = call(&entry, &["advance", "--board", &board_text], None)?;
    if code != 0
        || fs::read(&first).unwrap_or_default() != before
        || !root.join("practice").join("rounds").join("2.json").exists()
    {
        findings.push(format!(
            "2回目の advance で前の回の記録が変わったか、新しい記録が無い：{again}"
        ));
    }
    verify_broken(&entry, &work, &mut findings)?;
    verify_contrast(dir, &work, &mut findings)?;
    verify_serve(&entry, &root, &mut findings)?;
    let _ = fs::remove_dir_all(&work);
    Ok(findings)
}

/// 壊したボードが、1件ずつ check で不合格になる。エンジンの失敗は終了コードのまま返る。
fn verify_broken(entry: &Path, work: &Path, findings: &mut Vec<String>) -> Result<(), String> {
    let good: Value = serde_json::from_str(PRACTICE).map_err(|error| error.to_string())?;
    let breaks: [(&str, &str, Value); 6] = [
        ("答えが無い", "/topics/0/answer", Value::Null),
        (
            "根拠に出どころが無い",
            "/topics/0/grounds/0/source",
            Value::Null,
        ),
        (
            "採らなかった案に理由が無い",
            "/topics/0/rejected/0/reason",
            Value::Null,
        ),
        (
            "表の種類の完成イメージ",
            "/topics/0/images/0/kind",
            json!("table"),
        ),
        (
            "終了コードの無い実行例",
            "/topics/0/images/1/exit",
            Value::Null,
        ),
        (
            "どこが違うかの無い差し戻し",
            "/answer",
            json!({"decision": "return"}),
        ),
    ];
    for (n, (what, pointer, value)) in breaks.into_iter().enumerate() {
        let dir = work.join(format!("broken-{n}"));
        fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        let (file, instance) = if pointer == "/answer" {
            let answer = json!({"$schema": "brainstorming/answer@v1", "kind": "answer", "id": "1-Q1-1", "board": "practice", "round": 1, "topic": "Q1", "decision": value["decision"], "at": "2026-10-17T00:00:00Z"});
            (dir.join("1-Q1-1.json"), answer)
        } else {
            let mut board = good.clone();
            match value {
                Value::Null => remove(&mut board, pointer),
                other => {
                    if let Some(slot) = board.pointer_mut(pointer) {
                        *slot = other;
                    }
                }
            }
            (dir.join("practice.json"), board)
        };
        fs::write(&file, instance.to_string()).map_err(|error| error.to_string())?;
        let (_, report) = call(entry, &["check", "--dir", &dir.to_string_lossy()], None)?;
        if report["ok"] != true || problems(&report) == 0 {
            findings.push(format!(
                "壊したボード（{what}）が check で不合格にならない：{report}"
            ));
        }
    }
    let (code, missing) = call(
        entry,
        &[
            "get",
            "--path",
            &work.join("none.json").to_string_lossy(),
            "--query",
            "id",
        ],
        None,
    )?;
    if code == 0 {
        findings.push(format!(
            "無いファイルの get が終了コード 0 で返った：{missing}"
        ));
    }
    Ok(())
}

fn remove(value: &mut Value, pointer: &str) {
    let (parent, key) = pointer.rsplit_once('/').unwrap_or(("", pointer));
    if let Some(Value::Object(object)) = value.pointer_mut(parent) {
        object.remove(key);
    }
}

/// 文字のコントラストが下限を割るトークンにすると、描かない。
fn verify_contrast(dir: &Path, work: &Path, findings: &mut Vec<String>) -> Result<(), String> {
    let copy = work.join("low-contrast");
    copy_tree(dir, &copy)?;
    let path = copy
        .join("assets")
        .join("catalog")
        .join("design-system.json");
    let mut design: Value =
        serde_json::from_str(&fs::read_to_string(&path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    design["tokens"]["color-text-body"] = json!("{color-surface-block}");
    fs::write(&path, design.to_string()).map_err(|error| error.to_string())?;
    let root = work.join("contrast").join(".brainstorming");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    fs::write(root.join("practice.json"), PRACTICE).map_err(|error| error.to_string())?;
    let suffix = if cfg!(windows) { ".exe" } else { "" };
    let entry = copy
        .join("scripts")
        .join("bin")
        .join(format!("brainstorming{suffix}"));
    let (code, out) = call(&entry, &["render", "--dir", &root.to_string_lossy()], None)?;
    if code == 0 || root.join("site").join("practice.html").exists() {
        findings.push(format!("文字が背景と同じ色のトークンでも描いた：{out}"));
    }
    Ok(())
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(from)
        .map_err(|error| error.to_string())?
        .flatten()
    {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
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
                vec!["skill", "--out", "o", "--creator", "c", "--engine", "0.1.1"],
                Ok(Task::Skill {
                    out: "o".into(),
                    creator: "c".into(),
                    engine: "0.1.1".into(),
                    target: None,
                }),
            ),
            (
                vec!["verify", "--dir", "d", "--tag", "brainstorming-v0.1.0"],
                Ok(Task::Verify {
                    dir: "d".into(),
                    tag: Some("brainstorming-v0.1.0".into()),
                    creator: None,
                }),
            ),
            (
                vec!["skill", "--out", "o", "--engine", "1"],
                Err("skill には --creator が必要".to_owned()),
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
            "x86_64-pc-windows-msvc",
            UNIVERSAL,
        ]
        .iter()
        .map(|t| engine_target(t))
        .collect();
        assert_eq!(
            got,
            vec![
                "x86_64-unknown-linux-musl",
                "x86_64-pc-windows-msvc",
                UNIVERSAL
            ]
        );
    }

    #[test]
    fn problems_count_what_is_not_passing() {
        let report = json!({"instances": [{"errors": ["e"], "unfilled": []}], "findings": [{"status": "合格"}, {"status": "ずれ"}]});
        assert_eq!(problems(&report), 2);
        assert_eq!(
            failing(
                &json!({"items": [{"id": "a", "result": "skip"}, {"id": "b", "result": "unknown"}]})
            ),
            vec!["b"]
        );
    }

    #[test]
    fn the_practice_board_is_json_with_one_open_topic() {
        let board: Value = serde_json::from_str(PRACTICE).unwrap();
        assert_eq!(board["topics"][0]["status"], "open");
    }
}
