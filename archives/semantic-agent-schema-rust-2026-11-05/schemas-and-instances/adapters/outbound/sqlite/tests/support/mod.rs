//! 試験の記録：一時フォルダのファイルに開いた記録と、決まりに反する行を直接書くための生の接続。終わったらファイルを消す。

#![allow(clippy::unwrap_used, dead_code)]

use outbound_sqlite::SqliteLog;
use rusqlite::Connection;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// 試験ごとに新しい記録。
pub struct TestLog {
    log: SqliteLog,
    raw: Connection,
    path: PathBuf,
}

/// 空の記録を一時フォルダに作って開く。
pub fn log() -> TestLog {
    let path = std::env::temp_dir().join(format!(
        "semantic-agent-schema-sqlite-test-{}-{}.sqlite",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_file(&path);
    let log = SqliteLog::open(&path).unwrap();
    let raw = Connection::open(&path).unwrap();
    raw.pragma_update(None, "foreign_keys", true).unwrap();
    TestLog { log, raw, path }
}

impl TestLog {
    /// 生の接続（決まりに反する行を直接書く ・ 表を直接読む）。
    pub const fn connection(&self) -> &Connection {
        &self.raw
    }
}

impl Deref for TestLog {
    type Target = SqliteLog;
    fn deref(&self) -> &SqliteLog {
        &self.log
    }
}

impl DerefMut for TestLog {
    fn deref_mut(&mut self) -> &mut SqliteLog {
        &mut self.log
    }
}

impl Drop for TestLog {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
