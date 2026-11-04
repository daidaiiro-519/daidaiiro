//! コマンドを指定しない ・ 知らないコマンドは、使い方の誤り（終了コード 2）。--help は使い方を出して終わる。

use std::process::Command;

#[test]
fn no_arguments_is_usage_error() {
    let out = Command::new(env!("CARGO_BIN_EXE_semantic-agent-schema"))
        .output()
        .expect("semantic-agent-schema を起動できること");
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&out.stderr).lines().count(), 1);
}

#[test]
fn unknown_command_is_usage_error() {
    let out = Command::new(env!("CARGO_BIN_EXE_semantic-agent-schema"))
        .arg("frobnicate")
        .output()
        .expect("semantic-agent-schema を起動できること");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&out.stderr).lines().count(), 1);
}

#[test]
fn help_prints_the_usage_and_succeeds() {
    for flag in ["--help", "help"] {
        let out = Command::new(env!("CARGO_BIN_EXE_semantic-agent-schema"))
            .arg(flag)
            .output()
            .expect("semantic-agent-schema を起動できること");
        assert_eq!(out.status.code(), Some(0), "{flag}");
        let text = String::from_utf8_lossy(&out.stdout);
        for word in ["--schemas", "--range", "--human", "--base", "Git の根"] {
            assert!(text.contains(word), "{flag} の使い方に {word} が無い");
        }
    }
}
