//! 書いた変更に、書いたワークツリー（作業ツリーの根）を残す（ACDR 0224）。
//! 対象を書いた操作は、どのワークツリーのものも記録した順に読め、いまのワークツリーで書いたものに印が付く。
//! 表の形の版が違う記録は開かずに断る。

#![allow(clippy::unwrap_used)]

use outbound_sqlite::SqliteLog;
use rusqlite::Connection;
use writing_domain::values::ChangeRow;

fn update(before: &str, after: &str) -> ChangeRow {
    ChangeRow {
        kind: "update".into(),
        source: "sas".into(),
        target_path: "u1.json".into(),
        request_id: None,
        before: Some(before.into()),
        after: Some(after.into()),
        reason: None,
        schema: None,
        fields: Vec::new(),
    }
}

fn place(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("sas-worktree-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn writes_mark_the_ones_written_in_this_worktree() {
    let dir = place("mark");
    let path = dir.join("records.sqlite");
    let mut b = SqliteLog::open(&path).unwrap().in_worktree("/repo/b");
    b.append_change(None, &update("sha256:a", "sha256:b"))
        .unwrap();
    drop(b);
    let mut a = SqliteLog::open(&path).unwrap().in_worktree("/repo/a");
    a.append_change(None, &update("sha256:b", "sha256:c"))
        .unwrap();
    let seen: Vec<(String, bool)> = a
        .writes("u1.json")
        .unwrap()
        .into_iter()
        .map(|w| (w.operation, w.same_worktree))
        .collect();
    assert_eq!(
        seen,
        [("o-0001".to_string(), false), ("o-0002".to_string(), true)]
    );
    let stored: Vec<Option<String>> = a
        .connection()
        .prepare("SELECT worktree FROM changes ORDER BY rowid")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        stored,
        [Some("/repo/b".to_string()), Some("/repo/a".to_string())]
    );
    drop(a);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_database_of_another_shape_is_refused() {
    let dir = place("shape");
    let path = dir.join("records.sqlite");
    {
        let conn = Connection::open(&path).unwrap();
        conn.pragma_update(None, "user_version", outbound_sqlite::SHAPE + 1)
            .unwrap();
    }
    let message = SqliteLog::open(&path).unwrap_err();
    assert!(message.contains("表の形の版"), "{message}");
    std::fs::remove_dir_all(&dir).unwrap();
}
