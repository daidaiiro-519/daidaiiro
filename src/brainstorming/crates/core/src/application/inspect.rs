//! 本文を読む検査のユースケース。

use crate::application::Env;
use crate::domain::inspect::inspect;
use crate::ports::{Clock, Files, Programs};
use serde_json::{json, Value};
use std::path::Path;

/// ボードの本体を読み、見つけたことを返す。
pub fn run<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    board_file: &Path,
) -> Result<Value, String> {
    let board = env.read_json(board_file)?;
    let findings = inspect(&board);
    Ok(json!({"ok": findings.is_empty(), "findings": findings}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{env, passing_engine, MemoryFiles};

    #[test]
    fn findings_come_back_with_ok_false() {
        let board = r#"{"id":"b","status":"answer","queue":[],"topics":[{"id":"Q1","status":"open","answer":{"text":"a"},"images":[{"kind":"run"}],"verification":{"out_of_scope":[{"item":"外の作業"}]}}]}"#;
        let env = env(MemoryFiles::with(&[("/r/b.json", board)]), passing_engine());
        let out = run(&env, Path::new("/r/b.json")).unwrap();
        assert_eq!(out["ok"], false);
        assert_eq!(
            out["findings"][0],
            "回答待ちの論点があるのに、queue に いま見る論点が無い"
        );
    }
}
