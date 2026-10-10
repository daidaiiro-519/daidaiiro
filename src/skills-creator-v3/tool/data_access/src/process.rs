// SPDX-License-Identifier: MIT
//! 外部の道具の起動。**どの Skill も同じファイルを複製して使う** ── `contract.rs` と同じ扱いである。
//!
//! データアクセス層が持つ。**業務ロジック層は `std::process` を直接呼ばず、ここを通す**（ACDR 0058）。
//! 子プロセスの規律（契約の「子プロセスを起こすときの規律」）はここで1回だけ実装する
//! ── 標準入力を閉じる ・ 制限時間で止める ・ 標準出力と標準エラーの受け先を決める。
//! **起動するコマンドの名前を持たない** ── 名前は tool.json からサービス層が読み、引数で渡る。

use std::io::{BufRead, BufReader, Read, Write};

/// 標準出力と標準エラーを受け取る上限（バイト）。**超えたぶんは読み捨てる** ── まとめて
/// 受け取ると、道具が出した量がそのまま記憶に載る（契約の「出力をまとめて受け取らない」）。
pub const MAX_OUTPUT: u64 = 16 * 1024 * 1024;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// 起動の失敗。**文言を持たない** ── どう伝えるかは業務ロジック層が決める。
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Failed {
    /// 起動できない。OS が返した理由を持つ。
    Spawn(String),
    /// 制限時間を過ぎたので止めた。
    Timeout,
    /// 標準入出力を受け渡せない。
    Pipe(String),
}

/// 1回の起動の結果。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Ran {
    /// 終了コード。シグナルで止まったときは -1 である。
    pub code: i32,
    /// 標準出力。
    pub stdout: String,
    /// 標準エラー。
    pub stderr: String,
}

/// 起動して、終わるまで待つ。**標準入力は閉じ、制限時間を過ぎたら止める。**
///
/// # Errors
///
/// 起動できないか、制限時間を過ぎたときに返す。
pub fn run(
    command: impl AsRef<Path>,
    args: &[String],
    cwd: impl AsRef<Path>,
    limit: Duration,
) -> Result<Ran, Failed> {
    let command = command.as_ref();
    let mut child = Command::new(command)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Failed::Spawn(e.to_string()))?;
    let out = drain(child.stdout.take());
    let err = drain(child.stderr.take());
    let code = wait_limited(&mut child, limit)?;
    Ok(Ran {
        code,
        stdout: out.join().unwrap_or_default(),
        stderr: err.join().unwrap_or_default(),
    })
}

fn drain<R: Read + Send + 'static>(from: Option<R>) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut kept = Vec::new();
        if let Some(mut r) = from {
            let _ = r.by_ref().take(MAX_OUTPUT).read_to_end(&mut kept);
            // **残りも読み切る** ── 読まずに閉じると、書き続ける子プロセスが止まる
            let _ = std::io::copy(&mut r, &mut std::io::sink());
        }
        String::from_utf8_lossy(&kept).into_owned()
    })
}

fn wait_limited(child: &mut Child, limit: Duration) -> Result<i32, Failed> {
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|e| Failed::Pipe(e.to_string()))? {
            return Ok(status.code().unwrap_or(-1));
        }
        if start.elapsed() > limit {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Failed::Timeout);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// 行を送り、行を受ける対話。**何を送り、どの応答を待つかは持たない** ── それは
/// 業務ロジック層の判定である。捨てると子プロセスを止める。
pub struct Session {
    child: Child,
    stdin: ChildStdin,
    lines: mpsc::Receiver<String>,
}

impl Session {
    /// 起動する。標準エラーは捨てる。
    ///
    /// # Errors
    ///
    /// 起動できないときに返す。
    pub fn open(
        command: impl AsRef<Path>,
        args: &[String],
        cwd: impl AsRef<Path>,
    ) -> Result<Self, Failed> {
        let mut child = Command::new(command.as_ref())
            .args(args)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| Failed::Spawn(e.to_string()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Failed::Pipe("標準入力を渡せない".to_owned()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Failed::Pipe("標準出力を受けられない".to_owned()))?;
        let (tx, lines) = mpsc::channel::<String>();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            stdin,
            lines,
        })
    }

    /// 1行を送る。
    ///
    /// # Errors
    ///
    /// 書けないときに返す。
    pub fn send(&mut self, line: &str) -> Result<(), Failed> {
        writeln!(self.stdin, "{line}")
            .and_then(|()| self.stdin.flush())
            .map_err(|e| Failed::Pipe(e.to_string()))
    }

    /// 1行を受ける。**待つ時間を過ぎたら None を返す。**
    #[must_use]
    pub fn recv(&self, wait: Duration) -> Option<String> {
        self.lines.recv_timeout(wait).ok()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
