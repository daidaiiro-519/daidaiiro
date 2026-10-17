//! 画面が数秒ごとに聞く、ボードごとのステータス。ロックはどのプロセスが持っていても見えるので、ターミナルから直しているあいだも更新中になる。

use crate::application::{lock, Env};
use crate::domain::board::{text, Layout};
use crate::domain::lock::what_label;
use crate::domain::status::{board_status, open_count};
use crate::ports::{Clock, Files, Programs};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::path::Path;

/// root（.brainstorming/）のボードごとのステータス ・ 回答待ちの数 ・ ロック ・ 本体の版（中身の SHA-256）。
pub fn boards<F: Files, P: Programs, C: Clock>(env: &Env<F, P, C>, root: &Path) -> Value {
    let layout = Layout::new(root);
    let mut out = Map::new();
    for path in env.files.list(root) {
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(raw) = env.files.read(&path) else {
            continue;
        };
        let Ok(board) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        if text(&board, "$schema") != "brainstorming/board@v1" {
            continue;
        }
        let id = text(&board, "id").to_owned();
        let held = lock::current(env, &layout, &id);
        let rev: String = Sha256::digest(raw.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let lock = held.as_ref().map(|l| {
            let what = text(l, "what");
            json!({"what": what, "label": what_label(what), "since": text(l, "since")})
        });
        out.insert(
            id,
            json!({
                "status": board_status(&board, held.is_some()).name(),
                "open": open_count(&board),
                "lock": lock,
                "rev": rev,
            }),
        );
    }
    Value::Object(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::lock::acquire;
    use crate::tests::{env, passing_engine, MemoryFiles};

    #[test]
    fn each_board_reports_its_status_and_a_lock_makes_it_updating() {
        let a = r#"{"$schema":"brainstorming/board@v1","id":"a","topics":[{"id":"Q0","status":"open"}]}"#;
        let b = r#"{"$schema":"brainstorming/board@v1","id":"b","topics":[{"id":"Q0","status":"settled"}]}"#;
        let env = env(
            MemoryFiles::with(&[
                ("/r/.brainstorming/a.json", a),
                ("/r/.brainstorming/b.json", b),
                (
                    "/r/.brainstorming/dashboard.json",
                    r#"{"$schema":"brainstorming/dashboard@v1"}"#,
                ),
            ]),
            passing_engine(),
        );
        let root = Path::new("/r/.brainstorming");
        let got = boards(&env, root);
        assert_eq!(got["a"]["status"], "answer");
        assert_eq!(got["a"]["open"], 1);
        assert_eq!(got["b"]["status"], "done");
        assert!(got.get("dashboard").is_none());
        acquire(&env, &Layout::new(root), "a", "R1", "ask").unwrap();
        let got = boards(&env, root);
        assert_eq!(got["a"]["status"], "updating");
        assert_eq!(got["a"]["lock"]["label"], "AI に聞く");
    }
}
