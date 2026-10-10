//! 出ていく側。宣言した固有のツールを、Skill のフォルダの中の別の実行ファイルとして呼ぶ。
//! 呼ぶ前に SHA-256 を確かめ、シェルを通さず、宣言した環境変数だけを渡す。呼んだあとは、
//! 制限時間 ・ 出力の大きさ ・ 出力の形 ・ 終了コードを確かめる。再試行はしない。

use schema_driven_core::application::declared::DeclaredTool;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 出力の大きさの上限（1 MiB）。
pub const OUTPUT_LIMIT: usize = 1024 * 1024;

/// 宣言のファイルの名前。Skill のフォルダの直下に置く。
pub const DECLARATIONS: &str = "tools.json";

fn failure(reason: &str, detail: impl Into<String>) -> (i32, Value) {
    (
        1,
        json!({"ok": false, "reason": reason, "detail": detail.into(), "errors": []}),
    )
}

/// Skill のフォルダの宣言を読む。宣言のファイルが無ければ None。
pub fn load_declarations(skill: &Path) -> Result<Option<Value>, String> {
    let path = skill.join(DECLARATIONS);
    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|error| format!("{}: {error}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("{}: {error}", path.display())),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// 宣言したツールを呼ぶ。返すのは終了コードと結果の JSON（共通ツールと同じ形）。
pub fn run_declared(skill: &Path, tool: &DeclaredTool, args: &Map<String, Value>) -> (i32, Value) {
    let executable = skill.join(&tool.command[0]);
    let bytes = match std::fs::read(&executable) {
        Ok(bytes) => bytes,
        Err(error) => {
            return failure(
                "実行ファイルを読めない",
                format!("{}: {error}", executable.display()),
            )
        }
    };
    if hex(&Sha256::digest(&bytes)) != tool.sha256 {
        return failure(
            "実行ファイルの SHA-256 が宣言と合わない",
            executable.display().to_string(),
        );
    }
    let mut command = Command::new(&executable);
    command
        .args(&tool.command[1..])
        .current_dir(skill)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    for name in &tool.env {
        if let Ok(value) = std::env::var(name) {
            command.env(name, value);
        }
    }
    // Windows の実行ファイルは、SYSTEMROOT が無いと起動できないことがある
    if cfg!(windows) {
        if let Ok(value) = std::env::var("SYSTEMROOT") {
            command.env("SYSTEMROOT", value);
        }
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => return failure("実行ファイルを起動できない", error.to_string()),
    };
    let request = json!({"tool": tool.name, "args": args}).to_string();
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(request.as_bytes());
    }
    let over = Arc::new(AtomicBool::new(false));
    let reader = {
        let over = over.clone();
        let mut stdout = child.stdout.take().expect("stdout は piped にした");
        std::thread::spawn(move || {
            let (mut out, mut buffer) = (Vec::new(), [0u8; 8192]);
            while let Ok(read) = stdout.read(&mut buffer) {
                if read == 0 {
                    break;
                }
                out.extend_from_slice(&buffer[..read]);
                if out.len() > OUTPUT_LIMIT {
                    over.store(true, Ordering::SeqCst);
                    break;
                }
            }
            out
        })
    };
    let deadline = Instant::now() + Duration::from_secs(tool.timeout_seconds);
    let status = loop {
        if over.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = child.wait();
            return failure("出力が大きすぎる", format!("{OUTPUT_LIMIT} バイトを超えた"));
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return failure("制限時間を超えた", format!("{} 秒", tool.timeout_seconds));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(error) => return failure("実行ファイルの終わりを待てない", error.to_string()),
        }
    };
    let out = reader.join().unwrap_or_default();
    if out.len() > OUTPUT_LIMIT {
        return failure("出力が大きすぎる", format!("{OUTPUT_LIMIT} バイトを超えた"));
    }
    let value: Value = match serde_json::from_slice(&out) {
        Ok(value @ Value::Object(_)) if value.get("ok").is_some_and(Value::is_boolean) => value,
        _ => {
            return failure(
                "出力が契約に合わない",
                "ok を持つ JSON のオブジェクトを1つ返していない",
            )
        }
    };
    match status.code() {
        Some(code @ 0..=2) => (code, value),
        other => (
            1,
            json!({"ok": false, "reason": "終了コードが契約に合わない", "detail": format!("{other:?}"), "errors": [], "output": value}),
        ),
    }
}
