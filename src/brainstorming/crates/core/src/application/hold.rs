//! ボードの保留を切り替えるユースケース。保留は利用者がボード全体を止める決定で、ボードの hold に日付と理由を書く。
//! ほかの AI が動いているあいだは切り替えない（ロックを取る）。

use crate::application::engine::call;
use crate::application::{lock, render, Env, Skill};
use crate::domain::board::{valid_id, Layout};
use crate::ports::{Clock, Files, Programs};
use serde_json::{json, Value};
use std::path::Path;

/// on なら保留にし、そうでなければ保留を解く。
pub fn set<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    root: &Path,
    board: &str,
    on: bool,
    why: &str,
) -> Result<Value, String> {
    if !valid_id(board) {
        return Err("ボードの名前が違う".to_owned());
    }
    let layout = Layout::new(root);
    let owner = format!("hold-{}", env.clock.uuid());
    lock::acquire(env, &layout, board, &owner, "hold")?;
    let done = write(env, skill, root, &layout, board, on, why);
    lock::release(env, &layout, board, &owner);
    done
}

fn write<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    root: &Path,
    layout: &Layout,
    board: &str,
    on: bool,
    why: &str,
) -> Result<Value, String> {
    let file = layout.board_file(board);
    let current = env.read_json(&file)?;
    let held = current.get("hold").is_some();
    let patch = match (on, held) {
        (true, _) => {
            let mut value = json!({"date": env.clock.today()});
            if !why.trim().is_empty() {
                value["why"] = json!(why.trim());
            }
            json!([{"op": "add", "path": "/hold", "value": value}])
        }
        (false, true) => json!([{"op": "remove", "path": "/hold"}]),
        (false, false) => return Ok(json!({"hold": false})),
    };
    let args = [
        "update".to_owned(),
        "--path".to_owned(),
        file.to_string_lossy().into_owned(),
        "--patch".to_owned(),
        patch.to_string(),
    ];
    let run = call(env, skill, &args)?;
    let out: Value = serde_json::from_str(&run.stdout)
        .map_err(|_| format!("update の出力を読めない：{}{}", run.stdout, run.stderr))?;
    if out.get("ok") != Some(&json!(true)) {
        return Err(format!("保留を書けなかった：{}", run.stdout.trim()));
    }
    render::run(env, skill, root)?;
    Ok(json!({"hold": on}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{env, passing_engine, skill, MemoryFiles};

    #[test]
    fn holding_calls_update_with_a_patch_and_releases_the_lock() {
        let env = env(
            MemoryFiles::with(&[("/r/.brainstorming/b.json", r#"{"id":"b"}"#)]),
            passing_engine(),
        );
        set(
            &env,
            &skill(),
            Path::new("/r/.brainstorming"),
            "b",
            true,
            "あとで",
        )
        .unwrap();
        let update = env
            .programs
            .calls()
            .into_iter()
            .find(|c| c.get(2).map(String::as_str) == Some("update"))
            .unwrap();
        assert!(update[6].contains(r#""path":"/hold""#));
        assert!(update[6].contains("あとで"));
        assert!(!env
            .files
            .exists(Path::new("/r/.brainstorming/.serve/locks/b.json")));
    }

    #[test]
    fn releasing_a_board_that_is_not_held_does_nothing() {
        let env = env(
            MemoryFiles::with(&[("/r/.brainstorming/b.json", r#"{"id":"b"}"#)]),
            passing_engine(),
        );
        assert_eq!(
            set(
                &env,
                &skill(),
                Path::new("/r/.brainstorming"),
                "b",
                false,
                ""
            )
            .unwrap(),
            json!({"hold": false})
        );
        assert!(env.programs.calls().is_empty());
    }
}
