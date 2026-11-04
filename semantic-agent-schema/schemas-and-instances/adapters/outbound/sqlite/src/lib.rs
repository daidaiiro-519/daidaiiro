//! 操作の記録のポートを SQLite で実装する。置き場所は入口が決める（<git-common-dir>/semantic-agent-schema/records.sqlite で、コミットしない）。
//! 書いた変更には、打ったワークツリー（作業ツリーの根）を残し、対象を書いた操作を引くときに同じワークツリーのものかを返す。
//!
//! 記録のポートは使うモジュールがそれぞれ宣言する（書く ChangeLog ・ 検証 ValidationLog ・ 記録 SessionLog ・ OperationReader）。手続きはこの型のメソッドに1つずつ持ち、各ポートはそれを呼ぶ。
//!
//! 操作の ID（o-0001。changes と validations で同じ採番）と時刻（UTC の RFC 3339）は、行を足すときにここで作る。
//! application と domain は時刻に触らない。セッションの ID はツールのもので、ツールから取れないとき（semantic-agent-schema init）だけここで作る。

use recording_application::ports::{
    OperationDetail, OperationFilter, OperationReader, OperationSummary, SessionLog, SessionRow,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::hash::{BuildHasher, Hasher};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use validation_domain::values::{Judgment, RuleRow, ValidateRow};
use writing_domain::values::{Caller, ChangeRow, FieldRow, WriteRecord};

/// 表の作り方。表の形を変えるときは、この1つを書き換えて SHAPE を1つ上げる（CODING.md）。
const RECORDS: &str = include_str!("../sql/records.sql");

/// 表の形の版（user_version）。
pub const SHAPE: i64 = 9;

/// SQLite の操作の記録。
#[derive(Debug)]
pub struct SqliteLog {
    conn: Connection,
    /// 打ったワークツリー（作業ツリーの根の絶対の経路）。書いた変更に残し、同じワークツリーの操作を見分ける
    worktree: Option<String>,
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
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(|e| err(&e))?;
        if version == 0 {
            conn.execute_batch(&format!(
                "BEGIN; {RECORDS} PRAGMA user_version = {SHAPE}; COMMIT;"
            ))
            .map_err(|e| err(&e))?;
        } else if version != SHAPE {
            return Err(format!(
                "操作の記録：表の形の版が {version} で、このコマンドの版 {SHAPE} と違う。記録は作り直せるので、記録のファイルを消してから打ち直す"
            ));
        }
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(|e| err(&e))?;
        Ok(Self {
            conn,
            worktree: None,
        })
    }

    /// 打ったワークツリー（作業ツリーの根の絶対の経路）を決める。書いた変更に残し、対象を書いた操作のうち同じワークツリーのものを見分ける。
    #[must_use]
    pub fn in_worktree(mut self, worktree: impl Into<String>) -> Self {
        self.worktree = Some(worktree.into());
        self
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
                "SELECT kind, source, target_path, before_hash, after_hash, reason, schema \
                 FROM changes WHERE id = ?1",
                [operation],
                |r| {
                    Ok(ChangeRow {
                        kind: r.get(0)?,
                        source: r.get(1)?,
                        target_path: r.get(2)?,
                        before: r.get(3)?,
                        after: r.get(4)?,
                        reason: r.get(5)?,
                        schema: r.get(6)?,
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

/// 記録の手続き。各モジュールの記録のポートは、これを呼ぶ。
impl SqliteLog {
    /// 打ったセッションの ID。初めて見たセッションなら、sessions に1行足す（行は書き換えない）。人なら None。
    ///
    /// # Errors
    /// 置き場所を読めない ・ 書けないとき。
    pub fn session(&mut self, caller: &Caller) -> Result<Option<String>, String> {
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

    /// 記録にあるセッションの行。無ければ None。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    pub fn session_row(&self, id: &str) -> Result<Option<SessionRow>, String> {
        self.conn
            .query_row(
                "SELECT id, tool, folder, via FROM sessions WHERE id = ?1",
                [id],
                session_row,
            )
            .optional()
            .map_err(|e| err(&e))
    }

    /// 新しいセッション ID（semantic-agent-schema init が作る。頭は semantic-agent-schema-）。一意で、時刻だけから作らない。
    #[must_use]
    pub fn new_session_id(&self) -> String {
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
        format!("semantic-agent-schema-{:016x}{:016x}", part(1), part(2))
    }

    /// 検証を1件足し（rule ごとの結果も一緒に）、その ID を返す。
    ///
    /// # Errors
    /// 置き場所が拒んだとき（決まりに反する行）。
    pub fn append_validate(
        &mut self,
        session: Option<&str>,
        row: &ValidateRow,
    ) -> Result<String, String> {
        let id = self.next_operation()?;
        let tx = self.conn.transaction().map_err(|e| err(&e))?;
        tx.execute(
            "INSERT INTO validations (id, at, target_path, hash, session_id, schema) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, now(), row.target_path, row.hash, session, row.schema],
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

    /// 書いた作成 ・ 更新 ・ 削除を1件足し（変わった欄も一緒に）、その ID を返す。
    ///
    /// # Errors
    /// 置き場所が拒んだとき（決まりに反する行）。
    pub fn append_change(
        &mut self,
        session: Option<&str>,
        row: &ChangeRow,
    ) -> Result<String, String> {
        let id = self.next_operation()?;
        let tx = self.conn.transaction().map_err(|e| err(&e))?;
        tx.execute(
            "INSERT INTO changes (id, at, kind, target_path, before_hash, after_hash, reason, source, \
             session_id, schema, worktree) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                id, now(), row.kind, row.target_path, row.before, row.after, row.reason,
                row.source, session, row.schema, self.worktree
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

    /// 経路の対象を書いた操作（作成か更新）を、記録した順に。最後に消したより後のものだけ。
    /// どのワークツリーで書いたものも返し、いまのワークツリーで書いたものに印を付ける。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    pub fn writes(&self, target_path: &str) -> Result<Vec<WriteRecord>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, session_id, at, worktree FROM changes \
                 WHERE kind IN ('create', 'update') AND target_path = ?1 \
                   AND rowid > coalesce((SELECT max(rowid) FROM changes \
                                         WHERE kind = 'delete' AND target_path = ?1), 0) \
                 ORDER BY rowid",
            )
            .map_err(|e| err(&e))?;
        type Found = (String, Option<String>, String, Option<String>);
        let found: Vec<Found> = stmt
            .query_map([target_path], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })
            .map_err(|e| err(&e))?
            .collect::<Result<_, _>>()
            .map_err(|e| err(&e))?;
        found
            .into_iter()
            .map(|(operation, session, at, worktree)| {
                Ok(WriteRecord {
                    row: self.change_row(&operation)?,
                    operation,
                    session,
                    at,
                    same_worktree: worktree.is_some() && worktree == self.worktree,
                })
            })
            .collect()
    }

    /// 記録の最後の変更で、$id が schema の Concrete に従うと書いた文書の経路（消した文書は除く。経路の順）。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    pub fn instances_of(&self, schema: &str) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT c.target_path FROM changes c \
                 WHERE c.rowid = (SELECT max(c2.rowid) FROM changes c2 WHERE c2.target_path = c.target_path) \
                   AND c.kind <> 'delete' AND c.schema = ?1 \
                 ORDER BY c.target_path",
            )
            .map_err(|e| err(&e))?;
        stmt.query_map([schema], |r| r.get(0))
            .map_err(|e| err(&e))?
            .collect::<Result<_, _>>()
            .map_err(|e| err(&e))
    }

    /// 経路の対象の、rule ごとの最後の判定。最後に消したより後の検証だけ。
    ///
    /// # Errors
    /// 置き場所を読めないとき。
    pub fn last_judgments(&self, target_path: &str) -> Result<Vec<Judgment>, String> {
        // rule ごとに、その rule を最後に判定した検証を選び、その検証の行を返す
        let mut stmt = self
            .conn
            .prepare(
                "SELECT v.id, r.rule_id, r.pointer, r.by, r.result, r.reason, r.reads_hash, v.hash \
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
                hash: r.get(7)?,
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

impl SqliteLog {
    /// 検証1件の行（rule ごとの結果も）を、操作の ID から読む。無ければ None。
    fn validate_row(
        &self,
        operation: &str,
    ) -> Result<Option<(String, Option<String>, ValidateRow)>, String> {
        let found = self
            .conn
            .query_row(
                "SELECT at, session_id, target_path, hash, schema FROM validations WHERE id = ?1",
                [operation],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, Option<String>>(1)?,
                        ValidateRow {
                            target_path: r.get(2)?,
                            hash: r.get(3)?,
                            schema: r.get(4)?,
                            results: Vec::new(),
                        },
                    ))
                },
            )
            .optional()
            .map_err(|e| err(&e))?;
        let Some((at, session, mut row)) = found else {
            return Ok(None);
        };
        let mut stmt = self
            .conn
            .prepare(
                "SELECT rule_id, pointer, by, result, reason, reads_hash FROM rule_results \
                 WHERE validation_id = ?1 ORDER BY ord",
            )
            .map_err(|e| err(&e))?;
        row.results = stmt
            .query_map([operation], |r| {
                Ok(RuleRow {
                    rule: r.get(0)?,
                    pointer: r.get(1)?,
                    by: r.get(2)?,
                    result: r.get(3)?,
                    reason: r.get(4)?,
                    reads: r.get(5)?,
                })
            })
            .map_err(|e| err(&e))?
            .collect::<Result<_, _>>()
            .map_err(|e| err(&e))?;
        Ok(Some((at, session, row)))
    }

    /// 検証1件の、判定の区分ごとの件数（区分の名前の順）。
    fn result_counts(&self, operation: &str) -> Result<Vec<(String, usize)>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT result, count(*) FROM rule_results WHERE validation_id = ?1 \
                 GROUP BY result ORDER BY result",
            )
            .map_err(|e| err(&e))?;
        stmt.query_map([operation], |r| {
            Ok((
                r.get::<_, String>(0)?,
                usize::try_from(r.get::<_, i64>(1)?).unwrap_or_default(),
            ))
        })
        .map_err(|e| err(&e))?
        .collect::<Result<_, _>>()
        .map_err(|e| err(&e))
    }
}

impl OperationReader for SqliteLog {
    fn operations(&self, filter: &OperationFilter) -> Result<Vec<OperationSummary>, String> {
        // 絞り方は SQL の条件にして「かつ」で並べ、値は引数で渡す。新しい順は採番（o-NNNN）の数の大きい順
        let mut conditions: Vec<String> = Vec::new();
        let mut args: Vec<String> = Vec::new();
        if !filter.paths.is_empty() {
            let marks: Vec<String> = filter
                .paths
                .iter()
                .map(|p| {
                    args.push(p.clone());
                    format!("?{}", args.len())
                })
                .collect();
            conditions.push(format!("o.target_path IN ({})", marks.join(", ")));
        }
        if let Some(by) = &filter.by {
            args.push(by.clone());
            conditions.push(format!("o.session_id = ?{}", args.len()));
        }
        if let Some(folder) = &filter.folder {
            args.push(folder.clone());
            conditions.push(format!("s.folder = ?{}", args.len()));
        }
        if let Some(since) = &filter.since {
            args.push(since.clone());
            conditions.push(format!("o.at >= ?{}", args.len()));
        }
        let mut sql =
            "SELECT o.id, o.at, o.kind, o.target_path, o.reason, s.id, s.tool, s.folder, s.via \
                       FROM operations o LEFT JOIN sessions s ON s.id = o.session_id"
                .to_string();
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY CAST(substr(o.id, 3) AS INTEGER) DESC");
        if let Some(n) = filter.limit {
            sql.push_str(&format!(" LIMIT {n}"));
        }
        let mut stmt = self.conn.prepare(&sql).map_err(|e| err(&e))?;
        let rows: Vec<OperationSummary> = stmt
            .query_map(rusqlite::params_from_iter(args.iter()), |r| {
                let session = match r.get::<_, Option<String>>(5)? {
                    Some(id) => Some(SessionRow {
                        id,
                        tool: r.get(6)?,
                        folder: r.get(7)?,
                        via: r.get(8)?,
                    }),
                    None => None,
                };
                Ok(OperationSummary {
                    operation: r.get(0)?,
                    at: r.get(1)?,
                    op: r.get(2)?,
                    path: r.get(3)?,
                    session,
                    reason: r.get(4)?,
                    counts: Vec::new(),
                })
            })
            .map_err(|e| err(&e))?
            .collect::<Result<_, _>>()
            .map_err(|e| err(&e))?;
        rows.into_iter()
            .map(|mut o| {
                if o.op == "validate" {
                    o.counts = self.result_counts(&o.operation)?;
                }
                Ok(o)
            })
            .collect()
    }

    fn operation(&self, id: &str) -> Result<Option<OperationDetail>, String> {
        if let Some((at, session, row)) = self.validate_row(id)? {
            return Ok(Some(OperationDetail::Validate {
                operation: id.to_string(),
                at,
                session,
                row,
            }));
        }
        let found: Option<(String, Option<String>)> = self
            .conn
            .query_row(
                "SELECT at, session_id FROM changes WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| err(&e))?;
        let Some((at, session)) = found else {
            return Ok(None);
        };
        Ok(Some(OperationDetail::Change {
            operation: id.to_string(),
            at,
            session,
            row: self.change_row(id)?,
        }))
    }
}

impl SessionLog for SqliteLog {
    fn session(&mut self, caller: &Caller) -> Result<Option<String>, String> {
        Self::session(self, caller)
    }
    fn session_row(&self, id: &str) -> Result<Option<SessionRow>, String> {
        Self::session_row(self, id)
    }
    fn new_session_id(&self) -> String {
        Self::new_session_id(self)
    }
}

impl writing_application::ports::ChangeLog for SqliteLog {
    fn append_change(&mut self, session: Option<&str>, row: &ChangeRow) -> Result<String, String> {
        Self::append_change(self, session, row)
    }
    fn writes(&self, target_path: &str) -> Result<Vec<WriteRecord>, String> {
        Self::writes(self, target_path)
    }
    fn instances_of(&self, schema: &str) -> Result<Vec<String>, String> {
        Self::instances_of(self, schema)
    }
}

impl validation_application::ports::ValidationLog for SqliteLog {
    fn append_validate(
        &mut self,
        session: Option<&str>,
        row: &ValidateRow,
    ) -> Result<String, String> {
        Self::append_validate(self, session, row)
    }
    fn writes(&self, target_path: &str) -> Result<Vec<WriteRecord>, String> {
        Self::writes(self, target_path)
    }
    fn last_judgments(&self, target_path: &str) -> Result<Vec<Judgment>, String> {
        Self::last_judgments(self, target_path)
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
