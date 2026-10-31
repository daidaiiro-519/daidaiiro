//! ステップ0 の sas：コマンドを持たないので、どう呼んでも使い方の誤り（終了コード 2）を1行返す。

use std::process::Command;

#[test]
fn no_arguments_is_usage_error() {
    let out = Command::new(env!("CARGO_BIN_EXE_sas"))
        .output()
        .expect("sas を起動できること");
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&out.stderr).lines().count(), 1);
}

#[test]
fn unknown_command_is_usage_error() {
    let out = Command::new(env!("CARGO_BIN_EXE_sas"))
        .arg("validate")
        .output()
        .expect("sas を起動できること");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&out.stderr).lines().count(), 1);
}
