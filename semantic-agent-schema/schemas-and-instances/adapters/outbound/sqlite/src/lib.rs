//! ポート OperationLog を SQLite で実装する。置き場所は入口が決める（既定は <git-common-dir>/sas/sas.db で、コミットしない）。
//!
//! 操作の ID（o-0001。changes と validations で同じ採番）と時刻（UTC の RFC 3339）は、行を足すときにここで作る。
//! application と domain は時刻に触らない。セッションの ID はツールのもので、ツールから取れないとき（sas init）だけここで作る。

use recording_application::ports::{
    Caller, ChangeRow, FieldRow, Judgment, OperationLog, RuleRow, SessionRow, StoredChange,
    ValidateRow, WriteRecord,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::hash::{BuildHasher, Hasher};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// 表の作り方。版ごとに1つ足し、足したものは変えない（公開する前の書き直しはステップ4b の1回だけ：CODING.md）。
const MIGRATIONS: [&str; 4] = [
    include_str!("../migrations/0001-validate.sql"),
    include_str!("../migrations/0002-change.sql"),
    include_str!("../migrations/0003-adopt.sql"),
    include_str!("../migrations/0004-delete.sql"),
];

/// SQLite の操作の記録。
#[derive(Debug)]
pub struct SqliteLog {
    conn: Connection,
}

fn err(e: &rusqlite::Error) -> String {
    format!("操作の記録：{e}")
}

impl SqliteLog {
    /// path の DB を開き、表が古ければ作る。置き場所のフォルダが無ければ作る。
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
        let version: usize = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(|e| err(&e))?;
        // 移すあいだは外部キーの検査を止め、コミットの前に外れが無いかを確かめる
        // （SQLite の ALTER TABLE の文書の手順。PRAGMA foreign_keys はトランザクションの外で変える）
        conn.pragma_update(None, "foreign_keys", false)
            .map_err(|e| err(&e))?;
        for (i, sql) in MIGRATIONS.iter().enumerate().skip(version) {
            conn.execute_batch(&format!("BEGIN; {sql} PRAGMA user_version = {};", i + 1))
                .map_err(|e| err(&e))?;
            let broken: i64 = conn
                .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| {
                    r.get(0)
                })
                .map_err(|e| err(&e))?;
            if broken > 0 {
                let _ = conn.execute_batch("ROLLBACK;");
                return Err(format!(
                    "操作の記録：版 {} へ移すと {broken} 件が外部キーに外れるので、移さない",
                    i + 1
                ));
            }
            conn.execute_batch("COMMIT;").map_err(|e| err(&e))?;
        }
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(|e| err(&e))?;
        Ok(Self { conn })
    }

    /// 生の接続（試験で決まりに反する行を直接書くため）。
    #[must_use]
    pub const fn connection(&self) -> &Connection {
        &self.conn
    }

    /// 書いた作成 ・ 更新 ・ 削除の行を、操作の ID から読む。
    fn change_row(&self, operation: &str) -> Result<ChangeRow, String> {
        let mut row = self
            .conn
            .query_row(
                "SELECT kind, source, target_path, request_id, before_hash, after_hash, intent \
                 FROM changes WHERE id = ?1",
                [operation],
                |r| {
                    Ok(ChangeRow {
                        kind: r.get(0)?,
                        source: r.get(1)?,
                        target_path: r.get(2)?,
                        request_id: r.get(3)?,
                        before: r.get(4)?,
                        after: r.get(5)?,
                        intent: r.get(6)?,
                        fields: Vec::new(),
                    })
                },
            )
            .map_err(|e| err(&e))?;
        let mut stmt = self
            .conn
            .prepare("SELECT pointer, change, before, after, before_unavailable FROM field_changes WHERE change_id = ?1 ORDER BY pointer")
            .map_err(|e| err(&e))?;
        row.fields = stmt
            .query_map([operation], |r| {
                Ok(FieldRow {
                    pointer: r.get(0)?,
                    change: r.get(1)?,
                    before: r.get(2)?,
                    after: r.get(3)?,
                    before_unavailable: r.get::<_, i64>(4)? == 1,
                })
            })
            .map_err(|e| err(&e))?
            .collect::<Result<_, _>>()
            .map_err(|e| err(&e))?;
        Ok(row)
    }

    /// 操作の次の ID。changes と validations は同じ採番から振る（結果の operation の形を1つにする）。
    fn next_operation(&self) -> Result<String, String> {
        let n: i64 = self
            .conn
            .query_row(
                "SELECT (SELECT count(*) FROM changes) + (SELECT count(*) FROM validations)",
                [],
                |r| r.get(0),
            )
            .map_err(|e| err(&e))?;
        Ok(format!("o-{:04}", n + 1))
    }

    /// 操作の ID と打ったセッションから、記録にある変更を組む。
    fn stored(&self, found: Vec<(String, Option<String>)>) -> Result<Vec<StoredChange>, String> {
        found
            .into_iter()
            .map(|(operation, session)| {
                Ok(StoredChange {
                    row: self.change_row(&operation)?,
                    operation,
                    session,
                })
            })
            .collect()
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
    fn session(&mut self, caller: &Caller) -> Result<Option<String>, String> {
        let Caller::Session {
            id,
            tool,
            folder,
            via,
        } = caller
        else {
            return Ok(None);
        };
        // 初めて見たセッションだけ1行足す（行は書き換えない）
        self.conn
            .execute(
                "INSERT INTO sessions (id, tool, folder, via, first_seen) VALUES (?1, ?2, ?3, ?4, ?5) \
                 ON CONFLICT (id) DO NOTHING",
                params![id, tool, folder, via.as_str(), now()],
            )
            .map_err(|e| err(&e))?;
        Ok(Some(id.clone()))
    }

    fn session_row(&self, id: &str) -> Result<Option<SessionRow>, String> {
        self.conn
            .query_row(
                "SELECT id, tool, folder, via FROM sessions WHERE id = ?1",
                [id],
                session_row,
            )
            .optional()
            .map_err(|e| err(&e))
    }

    fn initialized(&self, folder: &str) -> Result<Option<SessionRow>, String> {
        self.conn
            .query_row(
                "SELECT id, tool, folder, via FROM sessions WHERE folder = ?1 AND via = 'init' \
                 ORDER BY rowid DESC LIMIT 1",
                [folder],
                session_row,
            )
            .optional()
            .map_err(|e| err(&e))
    }

    fn new_session_id(&self) -> String {
        // 時刻とプロセスに、プロセスごとに乱数で決まる鍵のハッシュを2つ重ねる（推測できる時刻だけにしない）
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let part = |salt: u64| {
            let mut h = std::collections::hash_map::RandomState::new().build_hasher();
            h.write_u128(nanos);
            h.write_u32(std::process::id());
            h.write_u64(salt);
            h.finish()
        };
        format!("sas-{:016x}{:016x}", part(1), part(2))
    }

    fn append_validate(
        &mut self,
        session: Option<&str>,
        row: &ValidateRow,
    ) -> Result<String, String> {
        let id = self.next_operation()?;
        let tx = self.conn.transaction().map_err(|e| err(&e))?;
        tx.execute(
            "INSERT INTO validations (id, at, target_path, hash, session_id) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, now(), row.target_path, row.hash, session],
        )
        .map_err(|e| err(&e))?;
        for (ord, r) in row.results.iter().enumerate() {
            tx.execute(
                "INSERT INTO rule_results (validation_id, ord, rule_id, pointer, by, result, reason, reads_hash) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![id, i64::try_from(ord).unwrap_or(i64::MAX), r.rule, r.pointer, r.by, r.result, r.reason, r.reads],
            )
            .map_err(|e| err(&e))?;
        }
        tx.commit().map_err(|e| err(&e))?;
        Ok(id)
    }

    fn append_change(&mut self, session: Option<&str>, row: &ChangeRow) -> Result<String, String> {
        let id = self.next_operation()?;
        let tx = self.conn.transaction().map_err(|e| err(&e))?;
        tx.execute(
            "INSERT INTO changes (id, at, kind, target_path, before_hash, after_hash, intent, request_id, source, \
             session_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id, now(), row.kind, row.target_path, row.before, row.after, row.intent, row.request_id,
                row.source, session
            ],
        )
        .map_err(|e| err(&e))?;
        for f in &row.fields {
            tx.execute(
                "INSERT INTO field_changes (change_id, pointer, change, before, after, before_unavailable) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, f.pointer, f.change, f.before, f.after, i64::from(f.before_unavailable)],
            )
            .map_err(|e| err(&e))?;
        }
        tx.commit().map_err(|e| err(&e))?;
        Ok(id)
    }

    fn find_request(&self, request_id: &str) -> Result<Option<StoredChange>, String> {
        let found: Option<(String, Option<String>)> = self
            .conn
            .query_row(
                "SELECT id, session_id FROM changes WHERE request_id = ?1",
                [request_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| err(&e))?;
        Ok(self.stored(found.into_iter().collect())?.pop())
    }

    fn writes(&self, target_path: &str) -> Result<Vec<WriteRecord>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, session_id, at FROM changes \
                 WHERE kind IN ('create', 'update') AND target_path = ?1 \
                   AND rowid > coalesce((SELECT max(rowid) FROM changes \
                                         WHERE kind = 'delete' AND target_path = ?1), 0) \
                 ORDER BY rowid",
            )
            .map_err(|e| err(&e))?;
        let found: Vec<(String, Option<String>, String)> = stmt
            .query_map([target_path], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(|e| err(&e))?
            .collect::<Result<_, _>>()
            .map_err(|e| err(&e))?;
        found
            .into_iter()
            .map(|(operation, session, at)| {
                Ok(WriteRecord {
                    row: self.change_row(&operation)?,
                    operation,
                    session,
                    at,
                })
            })
            .collect()
    }

    fn last_judgments(&self, target_path: &str) -> Result<Vec<Judgment>, String> {
        // rule ごとに、その rule を最後に判定した検証を選び、その検証の行を返す
        let mut stmt = self
            .conn
            .prepare(
                "SELECT v.id, r.rule_id, r.pointer, r.by, r.result, r.reason, r.reads_hash \
                 FROM rule_results r JOIN validations v ON v.id = r.validation_id \
                 WHERE v.target_path = ?1 \
                   AND CAST(substr(v.id, 3) AS INTEGER) > coalesce((SELECT max(CAST(substr(id, 3) AS INTEGER)) FROM changes \
                                         WHERE kind = 'delete' AND target_path = ?1), 0) \
                   AND v.rowid = (SELECT max(v2.rowid) FROM rule_results r2 JOIN validations v2 ON v2.id = r2.validation_id \
                                  WHERE v2.target_path = ?1 AND r2.rule_id = r.rule_id) \
                 ORDER BY v.rowid, r.ord",
            )
            .map_err(|e| err(&e))?;
        stmt.query_map([target_path], |r| {
            Ok(Judgment {
                operation: r.get(0)?,
                rule: RuleRow {
                    rule: r.get(1)?,
                    pointer: r.get(2)?,
                    by: r.get(3)?,
                    result: r.get(4)?,
                    reason: r.get(5)?,
                    reads: r.get(6)?,
                },
            })
        })
        .map_err(|e| err(&e))?
        .collect::<Result<_, _>>()
        .map_err(|e| err(&e))
    }
}

/// sessions の行を読む（id ・ tool ・ folder ・ via の順に選んだ行）。
fn session_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<SessionRow> {
    Ok(SessionRow {
        id: r.get(0)?,
        tool: r.get(1)?,
        folder: r.get(2)?,
        via: r.get(3)?,
    })
}
