//! ボードのロックの読み書き。決まりは domain::lock が持つ。

use crate::application::Env;
use crate::domain::board::{pretty, Layout};
use crate::domain::lock::{self, may_write, refusal, stale, CLI};
use crate::ports::{Clock, Files, Programs};
use serde_json::{json, Value};

/// いまのロック。古いものは外して None を返す。
pub fn current<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    layout: &Layout,
    board: &str,
) -> Option<Value> {
    let path = layout.lock(board);
    let found = env.read_json(&path).ok()?;
    if stale(&found, env.clock.epoch_secs()) {
        let _ = env.files.remove_file(&path);
        return None;
    }
    Some(found)
}

/// owner としてロックを取る。ほかが持っていれば断る。同じ持ち主なら、始めた時刻を変えずに beat を書き直す。
pub fn acquire<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    layout: &Layout,
    board: &str,
    owner: &str,
    what: &str,
) -> Result<(), String> {
    let path = layout.lock(board);
    let now = env.clock.epoch_secs();
    let made = lock::make(board, owner, what, &env.clock.now(), now);
    if env.files.create_new(&path, &pretty(&made))? {
        return Ok(());
    }
    let found = env.read_json(&path).ok();
    if !may_write(found.as_ref(), owner, now) {
        return Err(found.as_ref().map(refusal).unwrap_or_default());
    }
    let mut next = made;
    if let Some(old) = found.filter(|old| old.get("owner") == Some(&json!(owner))) {
        next["since"] = old["since"].clone();
    }
    env.files.write(&path, &pretty(&next))
}

/// owner のロックなら beat を書き直す。
pub fn beat<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    layout: &Layout,
    board: &str,
    owner: &str,
) {
    let path = layout.lock(board);
    if let Ok(mut found) = env.read_json(&path) {
        if found.get("owner") == Some(&json!(owner)) {
            found["beat"] = json!(env.clock.epoch_secs());
            let _ = env.files.write(&path, &pretty(&found));
        }
    }
}

/// owner のロックなら外す。
pub fn release<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    layout: &Layout,
    board: &str,
    owner: &str,
) {
    let path = layout.lock(board);
    if let Ok(found) = env.read_json(&path) {
        if found.get("owner") == Some(&json!(owner)) {
            let _ = env.files.remove_file(&path);
        }
    }
}

/// ターミナルから呼んだ update ・ advance の前に呼ぶ。run は環境変数で受けた依頼の ID（serve の AI の中なら Some）。
/// serve の AI の中なら、その依頼のロックの下で書く。そうでなければ cli として取る。
pub fn enter_cli<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    layout: &Layout,
    board: &str,
    run: Option<&str>,
) -> Result<(), String> {
    if let (Some(run), Some(found)) = (run, current(env, layout, board)) {
        if found.get("owner") == Some(&json!(run)) {
            return Ok(());
        }
    }
    acquire(env, layout, board, CLI, "cli")
}

/// path がボードの本体なら、その置き場所と id を返す（中身の $schema で見分ける）。
pub fn board_of<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    path: &std::path::Path,
) -> Option<(Layout, String)> {
    let found = env.read_json(path).ok()?;
    if found.get("$schema") != Some(&json!("brainstorming/board@v1")) {
        return None;
    }
    let id = found.get("id")?.as_str()?.to_owned();
    Some((Layout::new(path.parent()?), id))
}

/// ターミナルのロックを外す（advance のあと ・ unlock）。
pub fn leave_cli<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    layout: &Layout,
    board: &str,
) {
    release(env, layout, board, CLI);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{env, passing_engine, MemoryFiles, EPOCH};
    use std::path::Path;

    fn layout() -> Layout {
        Layout::new(Path::new("/r/.brainstorming"))
    }

    #[test]
    fn a_second_owner_is_refused_until_the_first_releases() {
        let env = env(MemoryFiles::with(&[]), passing_engine());
        acquire(&env, &layout(), "b", "R1", "send").unwrap();
        let refused = acquire(&env, &layout(), "b", "R2", "ask").unwrap_err();
        assert!(refused.starts_with("このボードは更新中（送った回答をもとに更新"));
        assert!(enter_cli(&env, &layout(), "b", None).is_err());
        release(&env, &layout(), "b", "R2");
        assert!(current(&env, &layout(), "b").is_some());
        release(&env, &layout(), "b", "R1");
        acquire(&env, &layout(), "b", "R2", "ask").unwrap();
    }

    #[test]
    fn the_ai_started_by_serve_writes_under_its_own_lock() {
        let env = env(MemoryFiles::with(&[]), passing_engine());
        acquire(&env, &layout(), "b", "R1", "send").unwrap();
        enter_cli(&env, &layout(), "b", Some("R1")).unwrap();
        assert!(enter_cli(&env, &layout(), "b", Some("R2")).is_err());
        assert_eq!(current(&env, &layout(), "b").unwrap()["owner"], "R1");
    }

    #[test]
    fn a_terminal_lock_blocks_serve_and_is_left_by_advance() {
        let env = env(MemoryFiles::with(&[]), passing_engine());
        enter_cli(&env, &layout(), "b", None).unwrap();
        enter_cli(&env, &layout(), "b", None).unwrap();
        assert!(acquire(&env, &layout(), "b", "R1", "send").is_err());
        leave_cli(&env, &layout(), "b");
        assert!(current(&env, &layout(), "b").is_none());
    }

    #[test]
    fn only_a_board_file_is_found_as_a_board() {
        let env = env(
            MemoryFiles::with(&[
                (
                    "/r/.brainstorming/b.json",
                    r#"{"$schema":"brainstorming/board@v1","id":"b"}"#,
                ),
                (
                    "/r/.brainstorming/b/answers/1-Q0-1.json",
                    r#"{"$schema":"brainstorming/answer@v1","id":"x"}"#,
                ),
            ]),
            passing_engine(),
        );
        let (found, id) = board_of(&env, Path::new("/r/.brainstorming/b.json")).unwrap();
        assert_eq!((found, id.as_str()), (layout(), "b"));
        assert!(board_of(&env, Path::new("/r/.brainstorming/b/answers/1-Q0-1.json")).is_none());
    }

    #[test]
    fn a_stale_lock_is_taken_over() {
        let old = lock::make("b", "R0", "send", "t", EPOCH - 3600);
        let env = env(
            MemoryFiles::with(&[("/r/.brainstorming/.serve/locks/b.json", &old.to_string())]),
            passing_engine(),
        );
        assert!(current(&env, &layout(), "b").is_none());
        acquire(&env, &layout(), "b", "R1", "ask").unwrap();
        assert_eq!(current(&env, &layout(), "b").unwrap()["owner"], "R1");
    }
}
