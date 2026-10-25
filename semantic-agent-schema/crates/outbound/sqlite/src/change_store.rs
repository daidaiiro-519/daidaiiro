//! 変更の記録と意図の判定

use rusqlite::{Connection, OptionalExtension, params};
use semantic_agent_schema_application::ports::change_records::ChangeRecords;
use semantic_agent_schema_application::ports::judgments::Judgments;
use semantic_agent_schema_application::ports::sessions::SessionRef;
use semantic_agent_schema_application::semantic_agent_schema_change::aggregates::change_record::{
    ChangeRecord, Item, Writer,
};
use semantic_agent_schema_application::semantic_agent_schema_change::aggregates::judgment::IntentJudgment;
use semantic_agent_schema_application::semantic_agent_schema_change::values::change_id::ChangeId;
use semantic_agent_schema_application::semantic_agent_schema_change::values::verdict::Verdict;
use semantic_agent_schema_application::semantic_agent_schema_declaration::values::by::By;
use semantic_agent_schema_application::semantic_agent_schema_declaration::values::target::Target;
use std::path::Path;

/// 手元の DB の形（決めたことの全体の4節の DDL。消す表は無く、消すことはトリガーで断る）
const DDL: &str = "
CREATE TABLE IF NOT EXISTS changes (
  id                 TEXT PRIMARY KEY,
  purpose_ref        TEXT NOT NULL,
  intent             TEXT NOT NULL,
  writer_kind        TEXT NOT NULL CHECK (writer_kind IN ('human','ai-agent')),
  replaces_intent_id TEXT,
  summary_id         TEXT,
  created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE TABLE IF NOT EXISTS change_items (
  change_id TEXT NOT NULL REFERENCES changes(id),
  seq       INTEGER NOT NULL,
  target    TEXT NOT NULL,
  pointer   TEXT,
  PRIMARY KEY (change_id, seq)
);
CREATE TABLE IF NOT EXISTS intent_judgments (
  change_id  TEXT PRIMARY KEY REFERENCES changes(id),
  judge_kind TEXT NOT NULL CHECK (judge_kind IN ('ai','human')),
  result     TEXT NOT NULL CHECK (result IN ('verified','violated','unverified')),
  meaning    TEXT NOT NULL,
  reason     TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS session_changes (
  tool TEXT NOT NULL, id TEXT NOT NULL, change_id TEXT NOT NULL REFERENCES changes(id),
  PRIMARY KEY (tool, id, change_id)
);
CREATE TRIGGER IF NOT EXISTS no_delete_changes BEFORE DELETE ON changes BEGIN SELECT RAISE(ABORT, '変更の記録は消さない'); END;
CREATE TRIGGER IF NOT EXISTS no_delete_change_items BEFORE DELETE ON change_items BEGIN SELECT RAISE(ABORT, '変更の項目は消さない'); END;
";

/// 手元の DB（SQLite 1ファイル）
pub struct LocalDb {
    conn: Connection,
}

impl LocalDb {
    /// 開く。無ければフォルダごと作り、表を用意する
    ///
    /// # Errors
    /// 開けないか、表を作れないとき
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("{} を作れない：{e}", dir.display()))?;
        }
        let conn = Connection::open(path)
            .map_err(|e| format!("手元の DB {} を開けない：{e}", path.display()))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| e.to_string())?;
        conn.execute_batch(DDL)
            .map_err(|e| format!("手元の DB の表を作れない：{e}"))?;
        Ok(LocalDb { conn })
    }

    fn load(&self, id: &str) -> Result<Option<ChangeRecord>, String> {
        let row = self
            .conn
            .query_row("SELECT purpose_ref, intent, writer_kind, replaces_intent_id FROM changes WHERE id = ?1", params![id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, Option<String>>(3)?))
            })
            .optional()
            .map_err(|e| e.to_string())?;
        let Some((purpose, intent, writer, replaces)) = row else {
            return Ok(None);
        };
        let mut stmt = self
            .conn
            .prepare("SELECT target, pointer FROM change_items WHERE change_id = ?1 ORDER BY seq")
            .map_err(|e| e.to_string())?;
        let items = stmt
            .query_map(params![id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
            })
            .map_err(|e| e.to_string())?
            .map(|r| {
                r.map_err(|e| e.to_string())
                    .and_then(|(t, p)| Item::new(Target::from_key(&t), p))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let writer = Writer::parse(&writer).ok_or("書いた者の種類が読めない")?;
        let replaces = replaces.map(|r| ChangeId::parse(&r)).transpose()?;
        ChangeRecord::new(
            ChangeId::parse(id)?,
            &purpose,
            &intent,
            writer,
            items,
            replaces,
        )
        .map(Some)
    }
}

impl ChangeRecords for LocalDb {
    fn add(&self, record: &ChangeRecord, session: Option<&SessionRef>) -> Result<(), String> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO changes (id, purpose_ref, intent, writer_kind, replaces_intent_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![record.id().as_str(), record.purpose_ref(), record.intent(), record.writer().as_str(), record.replaces().map(ChangeId::as_str)],
        )
        .map_err(|e| format!("変更の記録を書けない：{e}"))?;
        for (seq, item) in record.items().iter().enumerate() {
            tx.execute(
                "INSERT INTO change_items (change_id, seq, target, pointer) VALUES (?1, ?2, ?3, ?4)",
                params![record.id().as_str(), i64::try_from(seq).unwrap_or(i64::MAX), item.target().key(), item.field()],
            )
            .map_err(|e| e.to_string())?;
        }
        if let Some(s) = session {
            tx.execute(
                "INSERT INTO session_changes (tool, id, change_id) VALUES (?1, ?2, ?3)",
                params![s.tool, s.id, record.id().as_str()],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }

    fn get(&self, id: &ChangeId) -> Result<Option<ChangeRecord>, String> {
        self.load(id.as_str())
    }

    fn touching(&self, target: &Target) -> Result<Vec<ChangeRecord>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT c.id FROM changes c JOIN change_items i ON i.change_id = c.id WHERE i.target = ?1 ORDER BY c.created_at, c.id")
            .map_err(|e| e.to_string())?;
        let ids = stmt
            .query_map(params![target.key()], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        ids.iter()
            .filter_map(|id| self.load(id).transpose())
            .collect()
    }
}

impl Judgments for LocalDb {
    fn put_intent(&self, j: &IntentJudgment) -> Result<(), String> {
        let kind = if j.by() == By::Human { "human" } else { "ai" };
        self.conn
            .execute(
                "INSERT INTO intent_judgments (change_id, judge_kind, result, meaning, reason) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(change_id) DO UPDATE SET judge_kind = excluded.judge_kind, result = excluded.result, meaning = excluded.meaning, reason = excluded.reason",
                params![j.change_id().as_str(), kind, j.result().as_str(), j.meaning(), j.reason()],
            )
            .map(|_| ())
            .map_err(|e| format!("意図の判定を書けない：{e}"))
    }

    fn intent_of(&self, change_id: &ChangeId) -> Result<Option<IntentJudgment>, String> {
        let row = self
            .conn
            .query_row("SELECT judge_kind, result, meaning, reason FROM intent_judgments WHERE change_id = ?1", params![change_id.as_str()], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?))
            })
            .optional()
            .map_err(|e| e.to_string())?;
        let Some((kind, result, meaning, reason)) = row else {
            return Ok(None);
        };
        let by = if kind == "human" { By::Human } else { By::Ai };
        let result = match result.as_str() {
            "verified" => Verdict::Verified,
            "violated" => Verdict::Violated,
            _ => Verdict::Unverified,
        };
        IntentJudgment::new(change_id.clone(), by, result, &meaning, &reason).map(Some)
    }
}
