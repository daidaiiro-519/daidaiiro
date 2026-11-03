//! 移行 0008（ACDR 0224）：書いた変更に、書いたワークツリー（作業ツリーの根）を残す。
//! 対象を書いた操作は、どのワークツリーのものも記録した順に読め、いまのワークツリーで書いたものに印が付く。
//! 版7 までの DB を開くと、前の行はどのワークツリーのものでもない（NULL）まま版8 になる。

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
fn rows_before_version_8_belong_to_no_worktree() {
    let dir = place("migrate");
    let path = dir.join("records.sqlite");
    {
        let conn = Connection::open(&path).unwrap();
        let up_to_7 = [
            include_str!("../migrations/0001-validate.sql"),
            include_str!("../migrations/0002-change.sql"),
            include_str!("../migrations/0003-adopt.sql"),
            include_str!("../migrations/0004-delete.sql"),
            include_str!("../migrations/0005-read.sql"),
            include_str!("../migrations/0006-reason.sql"),
            include_str!("../migrations/0007-not-judged.sql"),
        ];
        for (i, sql) in up_to_7.iter().enumerate() {
            conn.execute_batch(&format!(
                "BEGIN; {sql} PRAGMA user_version = {}; COMMIT;",
                i + 1
            ))
            .unwrap();
        }
        conn.execute_batch(
            "INSERT INTO changes (id, at, kind, target_path, before_hash, after_hash) \
             VALUES ('o-0001', '2026-11-01T00:00:00Z', 'update', 'u1.json', 'sha256:a', 'sha256:b');",
        )
        .unwrap();
    }
    let log = SqliteLog::open(&path).unwrap().in_worktree("/repo/a");
    let version: i64 = log
        .connection()
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 8);
    let writes = log.writes("u1.json").unwrap();
    assert_eq!(writes.len(), 1);
    assert!(!writes[0].same_worktree);
    drop(log);
    std::fs::remove_dir_all(&dir).unwrap();
}
