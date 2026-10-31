//! ポート OperationLog を SQLite で実装する。置き場所は作業ディレクトリの .sas/sas.db で、コミットしない。
//!
//! ID（s-0001 ・ o-0001）と時刻（UTC の RFC 3339）は、行を足すときにここで作る。application と domain は時刻に触らない。

use rusqlite::{Connection, OptionalExtension, params};
use schemas_and_instances_operation_recording_application::ports::{
    Caller, ChangeRow, FieldRow, Judgment, OperationLog, RuleRow, StoredChange, ValidateRow,
    WriteRecord,
};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// 表の作り方。版ごとに1つ足し、足したものは変えない（公開する前の CHECK を広げる変更だけは元の移行に入れた：CODING.md）。
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
        let version: usize = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(|e| err(&e))?;
        // 表を作り直す移行があるので、移すあいだは外部キーの検査を止め、コミットの前に外れが無いかを確かめる
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

    /// 書いた ・ 断った作成か更新の行を、操作の ID から読む。
    fn change_row(&self, operation: &str) -> Result<ChangeRow, String> {
        let mut row = self
            .conn
            .query_row(
                "SELECT kind, target_kind, target_path, concrete_ref, request_id, before_hash, after_hash, refusal_code, refusal, intent, source, cause_id \
                 FROM operations WHERE id = ?1",
                [operation],
                |r| {
                    let code: Option<String> = r.get(7)?;
                    let reason: Option<String> = r.get(8)?;
                    Ok(ChangeRow {
                        kind: r.get(0)?,
                        source: r.get(10)?,
                        target_kind: r.get(1)?,
                        target_path: r.get(2)?,
                        concrete: r.get(3)?,
                        request_id: r.get(4)?,
                        before: r.get(5)?,
                        after: r.get(6)?,
                        refusal: code.zip(reason),
                        intent: r.get(9)?,
                        cause: r.get(11)?,
                        fields: Vec::new(),
                    })
                },
            )
            .map_err(|e| err(&e))?;
        let mut stmt = self
            .conn
            .prepare("SELECT pointer, change, before, after, before_unavailable FROM field_changes WHERE operation_id = ?1 ORDER BY pointer")
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
                        "INSERT INTO sessions (id, actor, tool_ref, started_at) VALUES (?1, ?2, ?3, ?4)",
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
                "INSERT INTO rule_results (operation_id, ord, rule_id, pointer, by, result, reason, reads_hash) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![id, i64::try_from(ord).unwrap_or(i64::MAX), r.rule, r.pointer, r.by, r.result, r.reason, r.reads],
            )
            .map_err(|e| err(&e))?;
        }
        tx.commit().map_err(|e| err(&e))?;
        Ok(id)
    }

    fn append_change(&mut self, session: &str, row: &ChangeRow) -> Result<String, String> {
        let id = self.next_id("operations", "o")?;
        let tx = self.conn.transaction().map_err(|e| err(&e))?;
        let seq: i64 = tx
            .query_row(
                "SELECT coalesce(max(seq), 0) + 1 FROM operations WHERE session_id = ?1",
                [session],
                |r| r.get(0),
            )
            .map_err(|e| err(&e))?;
        let (code, reason) = row
            .refusal
            .as_ref()
            .map_or((None, None), |(c, m)| (Some(c.as_str()), Some(m.as_str())));
        tx.execute(
            "INSERT INTO operations (id, session_id, seq, request_id, kind, target_kind, target_path, concrete_ref, \
             before_hash, after_hash, refusal_code, refusal, intent, at, source, cause_id) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                id, session, seq, row.request_id, row.kind, row.target_kind, row.target_path, row.concrete,
                row.before, row.after, code, reason, row.intent, now(), row.source, row.cause
            ],
        )
        .map_err(|e| err(&e))?;
        for f in &row.fields {
            tx.execute(
                "INSERT INTO field_changes (operation_id, pointer, change, before, after, before_unavailable) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, f.pointer, f.change, f.before, f.after, i64::from(f.before_unavailable)],
            )
            .map_err(|e| err(&e))?;
        }
        tx.commit().map_err(|e| err(&e))?;
        Ok(id)
    }

    fn find_request(&self, request_id: &str) -> Result<Option<StoredChange>, String> {
        let found: Option<(String, String)> = self
            .conn
            .query_row(
                "SELECT id, session_id FROM operations WHERE request_id = ?1",
                [request_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| err(&e))?;
        found
            .map(|(operation, session)| {
                Ok(StoredChange {
                    row: self.change_row(&operation)?,
                    operation,
                    session,
                })
            })
            .transpose()
    }

    fn caused_by(&self, operation: &str) -> Result<Vec<StoredChange>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, session_id FROM operations WHERE cause_id = ?1 ORDER BY rowid")
            .map_err(|e| err(&e))?;
        let found: Vec<(String, String)> = stmt
            .query_map([operation], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| err(&e))?
            .collect::<Result<_, _>>()
            .map_err(|e| err(&e))?;
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

    fn writes(&self, target_path: &str) -> Result<Vec<WriteRecord>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, session_id, at FROM operations \
                 WHERE kind IN ('create', 'update') AND refusal_code IS NULL AND target_path = ?1 \
                   AND rowid > coalesce((SELECT max(rowid) FROM operations \
                                         WHERE kind = 'delete' AND refusal_code IS NULL AND target_path = ?1), 0) \
                 ORDER BY rowid",
            )
            .map_err(|e| err(&e))?;
        let found: Vec<(String, String, String)> = stmt
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
        // rule ごとに、その rule を最後に判定した検証の操作を選び、その操作の行を返す
        let mut stmt = self
            .conn
            .prepare(
                "SELECT o.id, r.rule_id, r.pointer, r.by, r.result, r.reason, r.reads_hash \
                 FROM rule_results r JOIN operations o ON o.id = r.operation_id \
                 WHERE o.kind = 'validate' AND o.target_path = ?1 \
                   AND o.rowid = (SELECT max(o2.rowid) FROM rule_results r2 JOIN operations o2 ON o2.id = r2.operation_id \
                                  WHERE o2.kind = 'validate' AND o2.target_path = ?1 AND r2.rule_id = r.rule_id) \
                 ORDER BY o.rowid, r.ord",
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
