// SPDX-License-Identifier: MIT
//! 規則の道具を、**出力をファイルへ流して**起動する。この Skill に固有の入出力である。
//!
//! `process::run` を使わない理由 ── `process::run` は標準出力と標準エラーを分けて記憶に
//! 受け取る。規則の道具は、**2つを1本のファイルへ出た順のまま流し**、記憶に載せない
//! （実測 2026-09-24、100MB を出す道具で最大常駐 306MB）。**起動できない理由の種類
//! （道具が見つからない）も業務ロジック層が判定に使う**ので、`io::Error` のまま返す。
//! 標準入力を閉じる ・ 制限時間で止める規律は `process` と同じである。
//!
//! **判定を置かない** ── どの道具を起動し、出力をどう読むかは、業務ロジック層が決める。

use std::fs::File;
use std::io::{self, Read as _, Seek as _, SeekFrom};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// 起動の失敗。**どの段で失敗したかを分ける** ── 業務ロジック層が、段ごとに扱いを変える。
#[derive(Debug)]
pub enum Failed {
    /// 出力を流す先を作れない。
    Sink(io::Error),
    /// 起動できない。
    Spawn(io::Error),
    /// 待てない ・ 止められない。
    Wait(io::Error),
}

/// 起動して、終わるまで待つ。**標準出力と標準エラーを `log` へ流し、標準入力は閉じる。**
/// 制限時間を過ぎたら止めて `None` を返す。
///
/// # Errors
///
/// 出力の先を作れないとき、起動できないとき、待てないときに返す。
pub fn run(
    command: impl AsRef<Path>,
    args: &[String],
    cwd: impl AsRef<Path>,
    log: impl AsRef<Path>,
    limit: Duration,
) -> Result<Option<i32>, Failed> {
    let sink = File::create(log).map_err(Failed::Sink)?;
    let err = sink.try_clone().map_err(Failed::Sink)?;
    let mut child = Command::new(command.as_ref())
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(sink)
        .stderr(err)
        .spawn()
        .map_err(Failed::Spawn)?;
    wait_for(&mut child, limit).map_err(Failed::Wait)
}

/// 子を待つ。**制限時間を超えたら終わらせる。**
fn wait_for(child: &mut Child, limit: Duration) -> io::Result<Option<i32>> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status.code().unwrap_or(-1)));
        }
        if started.elapsed() >= limit {
            child.kill()?;
            let _ = child.wait()?;
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// `offset` から、最大 `length` バイトを読む。**全体を記憶に載せない。**
///
/// # Errors
///
/// 開けないとき、または読めないときに返す。
pub fn read_range(path: impl AsRef<Path>, offset: u64, length: usize) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;
    let mut buf = Vec::new();
    file.take(length as u64).read_to_end(&mut buf)?;
    Ok(buf)
}

/// 複写する。**在れば置き換える。**
///
/// # Errors
///
/// 読めないか、書けないときに返す。
pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<u64> {
    std::fs::copy(from, to)
}

/// フォルダを、中身ごと消す。
///
/// # Errors
///
/// 消せないときに返す。
pub fn remove_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    std::fs::remove_dir_all(path)
}

/// この process の番号。
#[must_use]
pub fn process_id() -> u32 {
    std::process::id()
}
