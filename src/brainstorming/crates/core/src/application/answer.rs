//! 回答を受けるユースケース。画面から来た値を確かめ、answers/ に1件ずつインスタンスとして書き、基盤の check を通す。
//! 回答はボードでまとめて1回で送る。1件でも受けられなければ、1件も書かない。serve はボードを書かない。

use crate::application::engine::check_dir;
use crate::application::{Env, Skill};
use crate::domain::answer::accept;
use crate::domain::board::{pretty, text, valid_id, Layout};
use crate::ports::{Clock, Files, Programs};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// 回答をまとめて受けて書く。返すのは書いた回答の id。
pub fn submit_all<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    root: &Path,
    board_id: &str,
    round: u64,
    items: &[Value],
) -> Result<Vec<String>, String> {
    if !valid_id(board_id) {
        return Err("ボードの名前が違う".to_owned());
    }
    if items.is_empty() {
        return Err("選んだ回答が無い".to_owned());
    }
    let layout = Layout::new(root);
    let board = env.read_json(&layout.board_file(board_id))?;
    let mut existing: Vec<String> = env
        .files
        .list(&layout.answers(board_id))
        .iter()
        .filter_map(|path| path.file_stem().and_then(|s| s.to_str()).map(str::to_owned))
        .collect();
    let mut made = Vec::new();
    for item in items {
        let mut input = item.clone();
        input["board"] = json!(board_id);
        input["round"] = json!(round);
        let topic = text(&input, "topic").to_owned();
        if made.iter().any(|a: &Value| text(a, "topic") == topic) {
            return Err(format!("{topic} への回答が2つある"));
        }
        let answer = accept(&board, &input, &existing, &env.clock.now())?;
        existing.push(text(&answer, "id").to_owned());
        made.push(answer);
    }
    let mut written: Vec<PathBuf> = Vec::new();
    for answer in &made {
        let path = layout
            .answers(board_id)
            .join(format!("{}.json", text(answer, "id")));
        env.files.write(&path, &pretty(answer))?;
        written.push(path);
    }
    match check_dir(env, skill, &layout.answers(board_id)) {
        Ok(0) => Ok(made.iter().map(|a| text(a, "id").to_owned()).collect()),
        other => {
            for path in &written {
                env.files.remove_file(path)?;
            }
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

    const BOARD: &str = r#"{"id":"b","round":1,"topics":[{"id":"Q1","status":"open"},{"id":"Q2","status":"open"}]}"#;

    fn items(list: &[(&str, &str, &str)]) -> Vec<Value> {
        list.iter()
            .map(|(topic, decision, place)| json!({"topic": topic, "decision": decision, "where": place}))
            .collect()
    }

    #[test]
    fn answers_are_written_together_with_their_own_sequence_numbers() {
        let env = env(MemoryFiles::with(&[("/r/b.json", BOARD)]), passing_engine());
        let sent = items(&[("Q1", "approve", ""), ("Q2", "return", "答えの1文")]);
        let ids = submit_all(&env, &skill(), Path::new("/r"), "b", 1, &sent).unwrap();
        assert_eq!(ids, vec!["1-Q1-1", "1-Q2-1"]);
        submit_all(&env, &skill(), Path::new("/r"), "b", 1, &sent[..1]).unwrap();
        assert!(env.files.exists(Path::new("/r/b/answers/1-Q1-2.json")));
        assert_eq!(env.files.read(Path::new("/r/b.json")).unwrap(), BOARD);
    }

    #[test]
    fn one_refused_answer_leaves_no_file() {
        let env1 = env(MemoryFiles::with(&[("/r/b.json", BOARD)]), passing_engine());
        let bad = items(&[("Q1", "approve", ""), ("Q2", "return", "  ")]);
        assert!(submit_all(&env1, &skill(), Path::new("/r"), "b", 1, &bad).is_err());
        let twice = items(&[("Q1", "approve", ""), ("Q1", "return", "x")]);
        assert_eq!(
            submit_all(&env1, &skill(), Path::new("/r"), "b", 1, &twice).unwrap_err(),
            "Q1 への回答が2つある"
        );
        let env2 = env(MemoryFiles::with(&[("/r/b.json", BOARD)]), failing_engine());
        let good = items(&[("Q1", "approve", "")]);
        assert!(submit_all(&env2, &skill(), Path::new("/r"), "b", 1, &good).is_err());
        assert!(env1.files.list(Path::new("/r/b/answers")).is_empty());
        assert!(env2.files.list(Path::new("/r/b/answers")).is_empty());
    }

    #[test]
    fn an_empty_send_and_a_board_name_with_a_path_are_refused() {
        let env = env(MemoryFiles::with(&[]), passing_engine());
        assert_eq!(
            submit_all(&env, &skill(), Path::new("/r"), "b", 1, &[]).unwrap_err(),
            "選んだ回答が無い"
        );
        assert_eq!(
            submit_all(
                &env,
                &skill(),
                Path::new("/r"),
                "a/b",
                1,
                &items(&[("Q1", "approve", "")])
            )
            .unwrap_err(),
            "ボードの名前が違う"
        );
    }
}
