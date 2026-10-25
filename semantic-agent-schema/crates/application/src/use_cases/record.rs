//! record ・ まとめを作る

use crate::ports::change_summaries::ChangeSummaries;
use crate::ports::changes::Changes;
use crate::ports::sessions::SessionRef;
use crate::use_cases::validate::{concrete_of, is_declaration, results_of};
use crate::use_cases::{Outcome, Ports};
use semantic_agent_schema_change::aggregates::change_record::factories::change_summary::change_summary;
use semantic_agent_schema_change::aggregates::change_record::{ChangeRecord, Item, Writer};
use semantic_agent_schema_change::values::change_id::ChangeId;
use semantic_agent_schema_change::values::file_state::FileState;
use semantic_agent_schema_declaration::values::target::Target;
use semantic_agent_schema_validation::services::validator::problems;
use serde_json::{Value, json};

/// 記録する変更の中身
pub struct RecordRequest<'a> {
    /// 変えたファイルの置き場所
    pub path: &'a str,
    /// 変えた項目（JSON Pointer）。空ならファイル全体
    pub pointers: &'a [String],
    /// 意図（1文）
    pub intent: &'a str,
    /// 目的の参照
    pub purpose: &'a str,
    /// 書いた者の種類（human ・ ai-agent）
    pub writer: &'a str,
    /// 置き換える意図の id
    pub replaces: Option<&'a str>,
    /// 記録の id（呼ぶ側が決めて渡す）
    pub id: &'a str,
    /// 手元のセッション
    pub session: Option<&'a str>,
}

/// 意図1つぶんの変更を、手元の DB に記録する。セッション ID は記録に入れず、手元の索引にだけ結ぶ（不変条件 I34）
pub fn record(ports: &Ports<'_>, r: &RecordRequest<'_>) -> Outcome {
    let document = match ports.instances.read(r.path) {
        Ok(d) => d,
        Err(e) => return Outcome::refused("", &e),
    };
    let target = Target::of_document(r.path, document.as_ref());
    let items: Result<Vec<Item>, String> = if r.pointers.is_empty() {
        Item::new(target.clone(), None).map(|i| vec![i])
    } else {
        r.pointers
            .iter()
            .map(|p| Item::new(target.clone(), Some(p.clone())))
            .collect()
    };
    let made = items.and_then(|items| {
        let writer = Writer::parse(r.writer)
            .ok_or_else(|| format!("書いた者「{}」は human か ai-agent", r.writer))?;
        let replaces = r.replaces.map(ChangeId::parse).transpose()?;
        let session = r.session.map(SessionRef::parse).transpose()?;
        ChangeRecord::new(
            ChangeId::parse(r.id)?,
            r.purpose,
            r.intent,
            writer,
            items,
            replaces,
        )
        .map(|c| (c, session))
    });
    let (change, session) = match made {
        Ok(m) => m,
        Err(e) => return Outcome::refused("", &e),
    };
    if let Err(e) = ports.records.add(&change, session.as_ref()) {
        return Outcome::refused("", &e);
    }
    Outcome::checked(
        &[],
        json!({"id": change.id().as_str(), "record": change.to_json()}),
    )
}

/// 前回のコミットからの変更を、まだまとめに入っていない変更の記録と両方向で突き合わせ、まとめを1件作って置く。
/// 入った記録にはまとめの id を付ける（不変条件 I31 ・ I35）。AI は書かない
pub fn summarize(
    ports: &Ports<'_>,
    changes: &dyn Changes,
    summaries: &dyn ChangeSummaries,
    id: &str,
) -> Outcome {
    match make_summary(ports, changes, summaries, id) {
        Ok(o) => o,
        Err(e) => Outcome::refused("", &e),
    }
}

fn make_summary(
    ports: &Ports<'_>,
    changes: &dyn Changes,
    summaries: &dyn ChangeSummaries,
    id: &str,
) -> Result<Outcome, String> {
    let id = ChangeId::parse(id)?;
    let head = changes.head()?.unwrap_or_default();
    let changed = changes.changed()?;
    if let Some(open) = changed
        .iter()
        .find(|f| f.path.starts_with(".semantic-agent-schema/changes/") && f.before.is_none())
    {
        return Err(format!(
            "コミットしていないまとめ {} がある。まとめはコミットごとに1件なので、コミットしてから作る",
            open.path
        ));
    }
    let mut files = Vec::new();
    for f in changed {
        let doc = f
            .after
            .as_deref()
            .or(f.before.as_deref())
            .filter(|_| f.path.ends_with(".json"))
            .and_then(|t| serde_json::from_str::<Value>(t).ok());
        let target = Target::of_document(&f.path, doc.as_ref());
        let decided = match &doc {
            Some(d) if !is_declaration(d) => concrete_of(ports, d)
                .ok()
                .flatten()
                .map(|c| c.decided_by_human().into_iter().map(|(at, _)| at).collect())
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        files.push(FileState::new(&f.path, target, f.before, f.after, decided));
    }
    let mut records = Vec::new();
    for r in ports.records.pending()? {
        let j = ports.judgments.intent_of(r.id())?;
        records.push((r, j));
    }
    if records.is_empty() && files.is_empty() {
        return Err("まとめる変更が無い（前回のコミットから変わったファイルも、まとめに入っていない記録も無い）".into());
    }
    let (summary, ids) = change_summary(&id, (&head, "working-tree"), &records, &files);
    let problems = problems(&results_of(ports, &summary, None)?);
    if !problems.is_empty() {
        return Ok(Outcome::checked(
            &problems,
            json!({"saved": false, "summary": summary}),
        ));
    }
    let path = summaries.put(id.as_str(), &summary)?;
    ports.records.summarized(&ids, id.as_str())?;
    let exceptions: usize = summary["exceptions"].as_array().map_or(0, |e| {
        e.iter()
            .filter_map(|x| x["count"].as_u64())
            .map(|n| usize::try_from(n).unwrap_or(0))
            .sum()
    });
    Ok(Outcome::checked(
        &[],
        json!({"id": id.as_str(), "path": path, "records": ids.len(), "exceptions": exceptions, "summary": summary}),
    ))
}
