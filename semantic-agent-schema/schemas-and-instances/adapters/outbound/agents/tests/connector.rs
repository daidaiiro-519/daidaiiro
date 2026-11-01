//! 接続の定義で起動するアダプターの試験。子セッションは sh の小さな台本で、実際の AI は使わない。

#![allow(clippy::unwrap_used, clippy::panic)]

use outbound_agents::Connector;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use validation_application::ports::{ReadMeaning, Reply};

fn work(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sas-agents-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 受け取った標準入力を答えにして返す子セッション。
fn echo(dir: &Path, limit: u64) -> Connector {
    let script = dir.join("echo.sh");
    std::fs::write(
        &script,
        "#!/bin/sh\nin=$(cat)\nprintf '{\"session_id\":\"c-1\",\"result\":%s}' \"$(printf '%s' \"$in\" | sed 's/\"/\\\\\"/g; s/^/\"/; s/$/\"/')\"\n",
    )
    .unwrap();
    let cmd =
        json!({"argv": ["sh", script.to_string_lossy()], "stdin": "{prompt.text}|{tool_ref}"});
    Connector::new(
        json!({"name": "echo", "version": "1", "commands": {"start": cmd, "continue": cmd},
               "answer": {"json_pointer": "/result"}, "tool_ref": {"json_pointer": "/session_id"},
               "time_limit_s": limit, "capabilities": {"resume": true}}),
        Vec::new(),
        dir.join("work"),
    )
}

#[test]
fn placeholders_inside_the_prompt_are_left_as_they_are() {
    let dir = work("fill");
    let c = echo(&dir, 20);
    let Reply::Answered { answer, .. } = c.resume(
        "c-1",
        &Value::Null,
        "本文の {tool_ref} と {prompt.text}",
        &json!({}),
    ) else {
        panic!("答えが返るはず");
    };
    assert_eq!(answer, "本文の {tool_ref} と {prompt.text}|c-1");
}

#[test]
fn a_child_held_by_another_open_is_busy_and_a_left_file_is_not() {
    let dir = work("lock");
    let c = echo(&dir, 20);
    let lock = dir.join("work/children/c-1.lock");
    std::fs::create_dir_all(lock.parent().unwrap()).unwrap();
    // 止められた sas が残したファイル（鍵は外れている）では止まらない
    std::fs::write(&lock, "").unwrap();
    assert!(matches!(
        c.resume("c-1", &Value::Null, "a", &json!({})),
        Reply::Answered { .. }
    ));
    // 別の起動が鍵を持っている間は送らない
    let held = std::fs::OpenOptions::new().write(true).open(&lock).unwrap();
    // 同じ試験のプロセスで別の試験が子セッションを起動する瞬間は、記述子が exec までの間だけ複製され、鍵が一時的に取れない
    let mut tries = 0;
    while held.try_lock().is_err() {
        tries += 1;
        assert!(tries < 500, "鍵が外れない");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(matches!(
        c.resume("c-1", &Value::Null, "a", &json!({})),
        Reply::Busy
    ));
    drop(held);
    assert!(matches!(
        c.resume("c-1", &Value::Null, "a", &json!({})),
        Reply::Answered { .. }
    ));
}

#[test]
fn a_child_that_never_reads_a_large_prompt_still_times_out() {
    let dir = work("stdin");
    let cmd = json!({"argv": ["sleep", "30"], "stdin": "{prompt.text}"});
    let c = Connector::new(
        json!({"name": "slow", "version": "1", "commands": {"start": cmd},
               "answer": {"json_pointer": "/result"}, "tool_ref": {"json_pointer": "/session_id"},
               "time_limit_s": 1}),
        Vec::new(),
        dir.join("work"),
    );
    let big = "x".repeat(1 << 20);
    let started = std::time::Instant::now();
    assert!(matches!(c.start(&big, &json!({})), Reply::Timeout { .. }));
    assert!(started.elapsed().as_secs() < 10);
}

#[test]
fn prompt_files_are_written_only_when_used_and_removed_after() {
    let dir = work("keep");
    let c = echo(&dir, 20);
    c.start("a", &json!({}));
    assert!(
        !dir.join("work/prompts").exists(),
        "path を使わない定義では置かない"
    );
    let script = dir.join("cat.sh");
    std::fs::write(
        &script,
        "#!/bin/sh\nprintf '{\"session_id\":\"c-2\",\"result\":\"%s\"}' \"$(cat \"$1\")\"\n",
    )
    .unwrap();
    let cmd = json!({"argv": ["sh", script.to_string_lossy(), "{prompt.path}"]});
    let c = Connector::new(
        json!({"name": "file", "version": "1", "commands": {"start": cmd},
               "answer": {"json_pointer": "/result"}, "tool_ref": {"json_pointer": "/session_id"}}),
        Vec::new(),
        dir.join("work"),
    );
    let Reply::Answered { answer, .. } = c.start("ファイルで渡す", &json!({})) else {
        panic!("答えが返るはず");
    };
    assert_eq!(answer, "ファイルで渡す");
    assert_eq!(
        std::fs::read_dir(dir.join("work/prompts")).unwrap().count(),
        0,
        "終われば消す"
    );
}
