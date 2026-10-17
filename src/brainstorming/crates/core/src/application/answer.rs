//! 回答を受けるユースケース。画面から来た値を確かめ、answers/ に1件のインスタンスとして書き、基盤の check を通す。
//! serve はボードを書かない。

use crate::application::engine::check_dir;
use crate::application::{Env, Skill};
use crate::domain::answer::accept;
use crate::domain::board::{pretty, text, valid_id, Layout};
use crate::ports::{Clock, Files, Programs};
use serde_json::{json, Value};
use std::path::Path;

/// 回答を受けて書く。
pub fn submit<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    root: &Path,
    input: &Value,
) -> Result<Value, String> {
    let id = text(input, "board");
    if !valid_id(id) {
        return Err("ボードの名前が違う".to_owned());
    }
    let layout = Layout::new(root);
    let board = env.read_json(&layout.board_file(id))?;
    let existing: Vec<String> = env
        .files
        .list(&layout.answers(id))
        .iter()
        .filter_map(|path| path.file_stem().and_then(|s| s.to_str()).map(str::to_owned))
        .collect();
    let answer = accept(&board, input, &existing, &env.clock.now())?;
    let path = layout
        .answers(id)
        .join(format!("{}.json", text(&answer, "id")));
    env.files.write(&path, &pretty(&answer))?;
    match check_dir(env, skill, &layout.answers(id)) {
        Ok(0) => Ok(
            json!({"answer": answer["id"], "next": "AI に「回答した」と一言伝えると、次の回へ進む。"}),
        ),
        other => {
            env.files.remove_file(&path)?;
            Err(match other {
                Ok(n) => format!("回答が check を通らない（{n}件）"),
                Err(error) => error,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{env, failing_engine, passing_engine, skill, MemoryFiles};

    const BOARD: &str = r#"{"id":"b","round":1,"topics":[{"id":"Q1","status":"open"}]}"#;

    #[test]
    fn an_answer_is_written_once_per_submission() {
        let env = env(MemoryFiles::with(&[("/r/b.json", BOARD)]), passing_engine());
        let input = json!({"board": "b", "round": 1, "topic": "Q1", "decision": "approve"});
        submit(&env, &skill(), Path::new("/r"), &input).unwrap();
        submit(&env, &skill(), Path::new("/r"), &input).unwrap();
        assert!(env.files.exists(Path::new("/r/b/answers/1-Q1-1.json")));
        assert!(env.files.exists(Path::new("/r/b/answers/1-Q1-2.json")));
        assert_eq!(env.files.read(Path::new("/r/b.json")).unwrap(), BOARD);
    }

    #[test]
    fn a_refused_or_failing_answer_leaves_no_file() {
        let env1 = env(MemoryFiles::with(&[("/r/b.json", BOARD)]), passing_engine());
        let bad = json!({"board": "b", "round": 1, "topic": "Q1", "decision": "return"});
        assert!(submit(&env1, &skill(), Path::new("/r"), &bad).is_err());
        let env2 = env(MemoryFiles::with(&[("/r/b.json", BOARD)]), failing_engine());
        let good = json!({"board": "b", "round": 1, "topic": "Q1", "decision": "approve"});
        assert!(submit(&env2, &skill(), Path::new("/r"), &good).is_err());
        assert!(env1.files.list(Path::new("/r/b/answers")).is_empty());
        assert!(env2.files.list(Path::new("/r/b/answers")).is_empty());
    }

    #[test]
    fn a_board_name_with_a_path_is_refused() {
        let env = env(MemoryFiles::with(&[]), passing_engine());
        let input = json!({"board": "a/b", "round": 1, "topic": "Q1", "decision": "approve"});
        assert_eq!(
            submit(&env, &skill(), Path::new("/r"), &input).unwrap_err(),
            "ボードの名前が違う"
        );
    }
}
