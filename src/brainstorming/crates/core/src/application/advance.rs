//! 回を進めるユースケース。前の回の記録と回答を読み、domain の advance に渡し、ボードと回の記録を書く。
//! 書いたあとに基盤の check を通し、通らなければボードを元に戻して回の記録を消す。

use crate::application::engine::check_dir;
use crate::application::{Env, Skill};
use crate::domain::advance::advance;
use crate::domain::board::{pretty, text, Layout};
use crate::ports::{Clock, Files, Programs};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// フォルダの中の `<数>.json` のうち、いちばん大きい数のもの。
pub fn latest_round<F: Files>(files: &F, dir: &Path) -> Option<(u64, PathBuf)> {
    files
        .list(dir)
        .into_iter()
        .filter_map(|path| {
            let stem = path.file_stem()?.to_str()?.to_owned();
            let ext = path.extension()?.to_str()?.to_owned();
            (ext == "json").then_some(())?;
            Some((stem.parse::<u64>().ok()?, path))
        })
        .max_by_key(|(n, _)| *n)
}

/// フォルダの中の JSON をすべて読む（読めないものは飛ばさずに失敗にする）。
pub fn read_all<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    dir: &Path,
) -> Result<Vec<Value>, String> {
    env.files
        .list(dir)
        .into_iter()
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("json"))
        .map(|path| env.read_json(&path))
        .collect()
}

/// ボードの本体 board_file を次の回へ進める。
pub fn run<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    board_file: &Path,
) -> Result<Value, String> {
    let root = board_file.parent().ok_or("ボードの置き場所が分からない")?;
    let layout = Layout::new(root);
    let original = env.files.read(board_file)?;
    let current: Value = serde_json::from_str(&original)
        .map_err(|error| format!("ボードを JSON として読めない：{error}"))?;
    let id = text(&current, "id").to_owned();
    let stem = board_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if id != stem {
        return Err(format!("ボードの id（{id}）とファイル名（{stem}）が違う"));
    }
    let previous = match latest_round(&env.files, &layout.rounds(&id)) {
        Some((_, path)) => Some(env.read_json(&path)?),
        None => None,
    };
    let answers = read_all(env, &layout.answers(&id))?;
    let done = advance(
        &current,
        previous.as_ref(),
        &answers,
        &env.clock.today(),
        &env.clock.now(),
    )?;
    let record_path = layout.rounds(&id).join(format!("{}.json", done.round));
    if env.files.exists(&record_path) {
        return Err(format!(
            "第{}回の記録はもうある。回の記録は書き換えない",
            done.round
        ));
    }
    env.files.write(board_file, &pretty(&done.board))?;
    env.files.write(&record_path, &pretty(&done.record))?;
    let problems = check_dir(env, skill, root)
        .and_then(|a| check_dir(env, skill, &layout.rounds(&id)).map(|b| a + b));
    match problems {
        Ok(0) => Ok(json!({"board": id, "round": done.round, "changes": done.changes})),
        other => {
            env.files.write(board_file, &original)?;
            env.files.remove_file(&record_path)?;
            Err(match other {
                Ok(n) => format!("進めたボードが check を通らない（{n}件）。元に戻した"),
                Err(error) => format!("{error}。元に戻した"),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{env, passing_engine, skill, MemoryFiles};

    #[test]
    fn the_latest_round_is_the_largest_number() {
        let files = MemoryFiles::with(&[
            ("/r/b/rounds/1.json", "{}"),
            ("/r/b/rounds/10.json", "{}"),
            ("/r/b/rounds/2.json", "{}"),
            ("/r/b/rounds/x.json", "{}"),
        ]);
        assert_eq!(
            latest_round(&files, Path::new("/r/b/rounds")).map(|(n, _)| n),
            Some(10)
        );
    }

    #[test]
    fn advancing_writes_the_board_and_a_frozen_record() {
        let board = r#"{"id":"b","status":"answer","round":1,"queue":[{"topic":"Q1","why":"w"}],"topics":[{"id":"Q1","status":"open","question":"q","answer":{"text":"a"}}]}"#;
        let env = env(MemoryFiles::with(&[("/r/b.json", board)]), passing_engine());
        let done = run(&env, &skill(), Path::new("/r/b.json")).unwrap();
        assert_eq!(done["round"], 1);
        assert!(env.files.exists(Path::new("/r/b/rounds/1.json")));
        let again = run(&env, &skill(), Path::new("/r/b.json")).unwrap();
        assert_eq!(again["round"], 2);
        assert!(env.files.exists(Path::new("/r/b/rounds/2.json")));
    }

    #[test]
    fn a_record_that_already_exists_is_not_overwritten() {
        let board = r#"{"id":"b","status":"answer","round":1,"topics":[]}"#;
        let env = env(
            MemoryFiles::with(&[
                ("/r/b.json", board),
                ("/r/b/rounds/1.json", r#"{"round":0,"snapshot":{}}"#),
            ]),
            passing_engine(),
        );
        assert_eq!(
            run(&env, &skill(), Path::new("/r/b.json")).unwrap_err(),
            "第1回の記録はもうある。回の記録は書き換えない"
        );
    }

    #[test]
    fn a_failing_check_restores_the_board_and_removes_the_record() {
        let board = r#"{"id":"b","status":"answer","round":1,"topics":[]}"#;
        let env = env(
            MemoryFiles::with(&[("/r/b.json", board)]),
            crate::tests::failing_engine(),
        );
        assert!(run(&env, &skill(), Path::new("/r/b.json")).is_err());
        assert_eq!(env.files.read(Path::new("/r/b.json")).unwrap(), board);
        assert!(!env.files.exists(Path::new("/r/b/rounds/1.json")));
    }

    #[test]
    fn a_board_whose_id_differs_from_its_file_name_is_refused() {
        let env = env(
            MemoryFiles::with(&[("/r/c.json", r#"{"id":"b","topics":[]}"#)]),
            passing_engine(),
        );
        assert_eq!(
            run(&env, &skill(), Path::new("/r/c.json")).unwrap_err(),
            "ボードの id（b）とファイル名（c）が違う"
        );
    }
}
