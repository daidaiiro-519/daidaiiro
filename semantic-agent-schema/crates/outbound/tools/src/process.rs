//! プロセスの起動：argv をシェルを通さずに起動し、標準入力に渡し、時間切れ ・ 出力の上限 ・ 環境変数の許可つきで標準出力を受け取る

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// 標準出力の上限（バイト）
pub const OUTPUT_LIMIT: usize = 1 << 20;

/// 誤りに添える標準エラーの末尾（バイト）
const STDERR_TAIL: usize = 2000;

/// どの起動にも渡す環境変数（プロセスを見つけて動かすのに要るもの）
const BASE_ENV: [&str; 4] = ["PATH", "HOME", "LANG", "TMPDIR"];

/// 起動のしかた
pub struct Launch<'a> {
    /// 引数の配列（先頭がプログラム）
    pub argv: &'a [String],
    /// 標準入力に渡すもの
    pub input: &'a [u8],
    /// 作業の場所
    pub root: &'a Path,
    /// 時間切れ
    pub timeout: Duration,
    /// BASE_ENV のほかに渡してよい環境変数の名前（秘密は、要る先にだけ渡す）
    pub env: &'a [&'a str],
    /// 正常とみなす終了コード
    pub ok_codes: &'a [i32],
}

/// 起動して、標準出力を受け取る
///
/// # Errors
/// 起動できない ・ 時間切れ ・ 出力が上限を超える ・ 終了コードが ok_codes に無いとき。標準エラーの末尾を添える
pub fn run(l: &Launch<'_>) -> Result<Vec<u8>, String> {
    let (program, args) = l.argv.split_first().ok_or("argv が空")?;
    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(l.root)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for name in BASE_ENV.iter().chain(l.env.iter()) {
        if let Ok(v) = std::env::var(name) {
            cmd.env(name, v);
        }
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("{program} を起動できない：{e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(l.input)
            .map_err(|e| format!("{program} に渡せない：{e}"))?;
    }
    let mut stdout = child.stdout.take().ok_or("標準出力を受け取れない")?;
    let mut stderr = child.stderr.take().ok_or("標準エラーを受け取れない")?;
    let out_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout
            .by_ref()
            .take(OUTPUT_LIMIT as u64 + 1)
            .read_to_end(&mut buf);
        buf
    });
    let err_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf);
        buf
    });
    let start = Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait().map_err(|e| e.to_string())? {
            break s;
        }
        if start.elapsed() > l.timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "{program} が {} 秒で終わらない（時間切れ）",
                l.timeout.as_secs()
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let out = out_reader
        .join()
        .map_err(|_| "標準出力を読めない".to_string())?;
    let err = err_reader.join().unwrap_or_default();
    let tail = {
        let from = err.len().saturating_sub(STDERR_TAIL);
        String::from_utf8_lossy(&err[from..]).trim().to_string()
    };
    if out.len() > OUTPUT_LIMIT {
        return Err(format!(
            "{program} の出力が上限 {OUTPUT_LIMIT} バイトを超えた"
        ));
    }
    if !status.code().is_some_and(|c| l.ok_codes.contains(&c)) {
        return Err(format!(
            "{program} が終了コード {:?} で終わった{}",
            status.code(),
            if tail.is_empty() {
                String::new()
            } else {
                format!("（標準エラーの末尾：{tail}）")
            }
        ));
    }
    Ok(out)
}
