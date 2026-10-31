//! ポート OperationLog を SQLite で実装する。置き場所は作業ディレクトリの .sas/sas.db で、コミットしない。
//!
//! ID（s-0001 ・ o-0001）と時刻（UTC の RFC 3339）は、行を足すときにここで作る。application と domain は時刻に触らない。

use contracts_and_instances_operation_recording_application::ports::{
    Caller, OperationLog, ValidateRow,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// 表の作り方。版ごとに1つ足し、足したものは変えない。
const MIGRATIONS: [&str; 1] = [include_str!("../migrations/0001-validate.sql")];

/// SQLite の操作の記録。
#[derive(Debug)]
pub struct SqliteLog {
    conn: Connection,
}

fn err(e: &rusqlite::Error) -> String {
    format!("操作の記録：{e}")
}

impl SqliteLog {
    /// path の DB を開き、表が古ければ作る。親のフォルダが無ければ作る。
    ///
    /// # Errors
    /// 開けない ・ 表を作れないとき。
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("{} を作れない：{e}", dir.display()))?;
        }
        let conn = Connection::open(path).map_err(|e| err(&e))?;
        Self::with(conn)
    }

    /// メモリの中の DB（試験用）。
    ///
    /// # Errors
    /// 表を作れないとき。
    pub fn in_memory() -> Result<Self, String> {
        Self::with(Connection::open_in_memory().map_err(|e| err(&e))?)
    }

    fn with(conn: Connection) -> Result<Self, String> {
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(|e| err(&e))?;
        let version: usize = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(|e| err(&e))?;
        for (i, sql) in MIGRATIONS.iter().enumerate().skip(version) {
            conn.execute_batch(&format!(
                "BEGIN; {sql} PRAGMA user_version = {}; COMMIT;",
                i + 1
            ))
            .map_err(|e| err(&e))?;
        }
        Ok(Self { conn })
    }

    /// 生の接続（試験で決まりに反する行を直接書くため）。
    #[must_use]
    pub const fn connection(&self) -> &Connection {
        &self.conn
    }

    fn next_id(&self, table: &str, prefix: &str) -> Result<String, String> {
        let n: i64 = self
            .conn
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .map_err(|e| err(&e))?;
        Ok(format!("{prefix}-{:04}", n + 1))
    }
}

/// いまの時刻（UTC の RFC 3339、秒まで）。
fn now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let days = i64::try_from(secs / 86_400).unwrap_or_default();
    let rest = secs % 86_400;
    // 日数から年月日（Howard Hinnant の civil_from_days）
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

impl OperationLog for SqliteLog {
    fn caller_session(&mut self, caller: &Caller) -> Result<String, String> {
        let found: Option<(String, Option<String>)> = match caller {
            Caller::Tool { tool_ref } => self
                .conn
                .query_row("SELECT id, ended_at FROM sessions WHERE tool_ref = ?1", [tool_ref], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })
                .optional(),
            Caller::Human => self
                .conn
                .query_row(
                    "SELECT id, ended_at FROM sessions WHERE actor = 'human' AND ended_at IS NULL ORDER BY id LIMIT 1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional(),
        }
        .map_err(|e| err(&e))?;
        match found {
            Some((id, None)) => Ok(id),
            Some((id, Some(_))) => Err(format!("セッション {id} は閉じている")),
            None => {
                let id = self.next_id("sessions", "s")?;
                let (actor, tool_ref) = match caller {
                    Caller::Tool { tool_ref } => ("tool", Some(tool_ref.as_str())),
                    Caller::Human => ("human", None),
                };
                self.conn
                    .execute(
                        "INSERT INTO sessions (id, kind, actor, tool_ref, started_at) VALUES (?1, 'caller', ?2, ?3, ?4)",
                        params![id, actor, tool_ref, now()],
                    )
                    .map_err(|e| err(&e))?;
                Ok(id)
            }
        }
    }

    fn append_validate(&mut self, session: &str, row: &ValidateRow) -> Result<String, String> {
        let id = self.next_id("operations", "o")?;
        let tx = self.conn.transaction().map_err(|e| err(&e))?;
        let seq: i64 = tx
            .query_row(
                "SELECT coalesce(max(seq), 0) + 1 FROM operations WHERE session_id = ?1",
                [session],
                |r| r.get(0),
            )
            .map_err(|e| err(&e))?;
        tx.execute(
            "INSERT INTO operations (id, session_id, seq, kind, target_kind, target_path, before_hash, after_hash, at) \
             VALUES (?1, ?2, ?3, 'validate', ?4, ?5, ?6, ?7, ?8)",
            params![id, session, seq, row.target_kind, row.target_path, row.before, row.after, now()],
        )
        .map_err(|e| err(&e))?;
        for (ord, r) in row.results.iter().enumerate() {
            tx.execute(
                "INSERT INTO rule_results (operation_id, ord, rule_id, pointer, by, result, reason) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![id, i64::try_from(ord).unwrap_or(i64::MAX), r.rule, r.pointer, r.by, r.result, r.reason],
            )
            .map_err(|e| err(&e))?;
        }
        tx.commit().map_err(|e| err(&e))?;
        Ok(id)
    }
}
