//! record ・ まとめを作る

use crate::ports::sessions::SessionRef;
use crate::use_cases::{Outcome, Ports};
use semantic_agent_schema_change::aggregates::change_record::{ChangeRecord, Item, Writer};
use semantic_agent_schema_change::values::change_id::ChangeId;
use semantic_agent_schema_declaration::values::target::Target;
use serde_json::json;

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
