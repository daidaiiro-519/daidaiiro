// SPDX-License-Identifier: MIT
//! 原稿から、枚ごとの音声と尺を作る。
//!
//! **同じ文なら作り直さない。** 文 ・ 声 ・ 合成器のどれかが変わればキーも変わる ──
//! 変わっていないものを作り直すと、外へ出す回数だけ費用が増える。
//!
//! **外の道具を呼ぶのは合成だけである。** 尺は音声そのものから測る（`mp3`）。

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{json, Value};
use sha2::{Digest as _, Sha256};

use crate::mp3;

/// 読みの印の種類。**1つの引数として渡す** ── 分けると、続く引数まで飲み込む。
const MARK_TYPES: &str = "--speech-mark-types=[\"word\", \"sentence\"]";

/// 合成の制限時間（秒）。
const TIMEOUT: u64 = 300;

/// 原稿1件。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Item {
    /// 枚の識別子。
    pub id: String,
    /// 読み上げる文。
    pub text: String,
}

/// 原稿の全体。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Script {
    /// 読み上げる声。
    pub voice: String,
    /// 合成器。
    pub engine: String,
    /// 読みの辞書。
    pub lexicons: Vec<String>,
    /// 取り出しの置き場（原稿の場所からの相対）。
    pub cache: String,
    /// 枚。
    pub items: Vec<Item>,
}

/// キーを作る。**文 ・ 声 ・ 合成器のどれかが変われば、キーも変わる。**
#[must_use]
pub fn key_of(text: &str, voice: &str, engine: &str) -> String {
    let digest = Sha256::digest(format!("{text}|{voice}|{engine}").as_bytes());
    digest.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

/// 原稿を読む。
///
/// # Errors
///
/// 読めないとき、または形が違うときに返す。
pub fn load(dir: &Path) -> io::Result<Script> {
    let body = std::fs::read_to_string(dir.join("narration.json"))?;
    let parsed: Value = serde_json::from_str(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let text = |key: &str, default: &str| {
        parsed
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or(default)
            .to_owned()
    };
    let items = parsed
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "items が無い"))?
        .iter()
        .filter_map(|x| {
            Some(Item {
                id: x.get("id")?.as_str()?.to_owned(),
                text: x.get("text")?.as_str()?.to_owned(),
            })
        })
        .collect();
    Ok(Script {
        voice: text("voice", "Takumi"),
        engine: text("engine", "neural"),
        lexicons: parsed
            .get("lexicons")
            .and_then(Value::as_array)
            .map(|x| {
                x.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        cache: text("cache", "cache"),
        items,
    })
}

/// 1枚ぶんの見立て。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Planned {
    /// 枚の識別子。
    pub id: String,
    /// キー。
    pub key: String,
    /// 読み上げる文。
    pub text: String,
    /// 音声の置き場（原稿の場所からの相対）。
    pub audio: String,
    /// 読みの印の置き場。
    pub marks: String,
    /// 既に在るか。
    pub cached: bool,
}

/// 枚ごとに、キーと、合成が要るかどうかを出す。**1件も合成しない。**
#[must_use]
pub fn plan(dir: &Path, script: &Script, voice: &str) -> Vec<Planned> {
    let cache = PathBuf::from(&script.cache);
    script
        .items
        .iter()
        .map(|item| {
            let key = key_of(&item.text, voice, &script.engine);
            let audio = cache.join(format!("{key}.mp3"));
            Planned {
                id: item.id.clone(),
                key: key.clone(),
                text: item.text.clone(),
                cached: dir.join(&audio).exists(),
                audio: audio.display().to_string(),
                marks: cache
                    .join(format!("{key}.marks.json"))
                    .display()
                    .to_string(),
            }
        })
        .collect()
}

/// Amazon Polly を呼ぶ。**コマンドは引数で受け取る**（tool.json の external から、宣言の層が渡す）。
fn polly(aws: &str, args: &[&str], out: &Path) -> io::Result<()> {
    let mut command = Command::new(aws);
    command
        .args(["polly", "synthesize-speech"])
        .args(args)
        .arg(out);
    let done = run_limited(command, TIMEOUT)?;
    if done.status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "合成できない ── {}",
        String::from_utf8_lossy(&done.stderr).trim()
    )))
}

/// 音声と、語と文の時刻を作る。**どちらも同じ操作から得られる。**
///
/// **値を複数取る引数は `=` で1つにまとめる** ── 分けて渡すと、後ろに続く出力先の
/// 指定まで飲み込み、出力先が無いという誤りになる。
///
/// # Errors
///
/// 合成に失敗したときに返す。
pub fn synthesize(
    aws: &str,
    text: &str,
    voice: &str,
    engine: &str,
    dest: &Path,
    lexicons: &[String],
) -> io::Result<()> {
    let joined = lexicons.join(" ");
    let mut head: Vec<&str> = vec!["--voice-id", voice, "--engine", engine];
    let with_lexicons = format!("--lexicon-names={joined}");
    if !lexicons.is_empty() {
        head.push(&with_lexicons);
    }
    let mut audio = head.clone();
    audio.extend(["--output-format", "mp3", "--text", text]);
    polly(aws, &audio, &dest.with_extension("mp3"))?;

    let mut marks = head;
    marks.extend(["--output-format", "json", MARK_TYPES, "--text", text]);
    let name = dest
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    polly(
        aws,
        &marks,
        &dest.with_file_name(format!("{name}.marks.json")),
    )
}

/// 作った結果。
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Made {
    /// 新しく合成した枚。
    pub made: Vec<String>,
    /// 取り出した枚。
    pub taken: Vec<String>,
    /// 枚ごとの尺（ミリ秒）。
    pub durations: Vec<(String, u64)>,
}

/// キーが在る枚は取り出し、無い枚だけを合成して、継ぎ目の出力を書く。
///
/// # Errors
///
/// 合成に失敗したとき、または書けないときに返す。
pub fn run(aws: &str, dir: &Path, script: &Script, voice: &str) -> io::Result<Made> {
    let cache = dir.join(&script.cache);
    std::fs::create_dir_all(&cache)?;
    let mut out = Made::default();
    let mut items = Vec::new();
    for row in plan(dir, script, voice) {
        let dest = cache.join(&row.key);
        if dest.with_extension("mp3").exists() {
            out.taken.push(row.id.clone());
        } else {
            synthesize(
                aws,
                &row.text,
                voice,
                &script.engine,
                &dest,
                &script.lexicons,
            )?;
            out.made.push(row.id.clone());
        }
        let data = std::fs::read(dest.with_extension("mp3")).unwrap_or_default();
        let ms = mp3::duration_ms(&data);
        out.durations.push((row.id.clone(), ms));
        items.push(json!({
            "id": row.id, "key": row.key, "audio": row.audio,
            "marks": row.marks, "format": "mp3", "durationMs": ms,
        }));
    }
    let body = json!({ "voice": voice, "engine": script.engine, "items": items });
    let text = serde_json::to_string_pretty(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    std::fs::write(dir.join("narration.out.json"), text + "\n")?;
    Ok(out)
}

/// 読みの辞書を登録する。**合成の前に、入力へ適用される。**
///
/// # Errors
///
/// 登録に失敗したときに返す。
pub fn put_lexicon(aws: &str, path: &Path, name: &str) -> io::Result<()> {
    let content = format!("file://{}", path.display());
    let mut command = Command::new(aws);
    command.args([
        "polly",
        "put-lexicon",
        "--name",
        name,
        "--content",
        &content,
    ]);
    let done = run_limited(command, 60)?;
    if done.status.success() {
        return Ok(());
    }
    let why: String = String::from_utf8_lossy(&done.stderr)
        .trim()
        .chars()
        .take(300)
        .collect();
    Err(io::Error::other(why))
}

/// 外部の道具を、制限時間つきで実行する。**timeout コマンドを使わない** ── macOS の標準に無く、
/// Windows では別の意味のコマンドである（ACDR 0029）。時間を超えたら子プロセスを終了させる。
///
/// # Errors
///
/// 起動できないとき、または制限時間を超えたときに返す。
fn run_limited(mut command: Command, secs: u64) -> io::Result<std::process::Output> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    // **標準エラーは別の糸で読み切る** ── 読まずに待つと、出力が詰まって子が止まる
    let mut err = child.stderr.take();
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(pipe) = err.as_mut() {
            let _ = io::Read::read_to_end(pipe, &mut buf);
        }
        buf
    });
    let start = std::time::Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed().as_secs() >= secs {
            let _ = child.kill();
            let _ = child.wait();
            let _ = reader.join();
            return Err(io::Error::other(format!("{secs}秒で終わらない")));
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    };
    let stderr = reader.join().unwrap_or_default();
    Ok(std::process::Output {
        status,
        stdout: Vec::new(),
        stderr,
    })
}
