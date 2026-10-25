//! 変更の記録。欄は pub(in crate::aggregates::change_record)
pub mod factories;

use crate::values::change_id::ChangeId;
use semantic_agent_schema_declaration::values::target::Target;
use serde_json::{Value, json};

/// 書いた者の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Writer {
    /// 人
    Human,
    /// AI の Agent
    AiAgent,
}

impl Writer {
    /// 文字列から読む
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "human" => Some(Writer::Human),
            "ai-agent" => Some(Writer::AiAgent),
            _ => None,
        }
    }

    /// 文字列にする
    pub fn as_str(self) -> &'static str {
        match self {
            Writer::Human => "human",
            Writer::AiAgent => "ai-agent",
        }
    }
}

/// 変更の項目：対象と、変えた欄（JSON Pointer。ファイル全体なら無し）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub(in crate::aggregates::change_record) target: Target,
    pub(in crate::aggregates::change_record) field: Option<String>,
}

impl Item {
    /// 対象と欄から作る
    ///
    /// # Errors
    /// 欄が JSON Pointer の形でないか、JSON でないファイルに欄を書いたとき
    pub fn new(target: Target, field: Option<String>) -> Result<Self, String> {
        if let Some(f) = &field {
            if !f.is_empty() && !f.starts_with('/') {
                return Err(format!("欄「{f}」は JSON Pointer（/ で始まる）で書く"));
            }
            if let Target::Path(p) = &target
                && !p.ends_with(".json")
            {
                return Err(format!("JSON でないファイル {p} には欄を書かない"));
            }
        }
        Ok(Item { target, field })
    }

    /// 対象
    pub fn target(&self) -> &Target {
        &self.target
    }

    /// 変えた欄
    pub fn field(&self) -> Option<&str> {
        self.field.as_deref()
    }
}

/// 変更の記録：意図1つぶんの変更。前後の値とセッション ID は持たない（不変条件 I34）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRecord {
    pub(in crate::aggregates::change_record) id: ChangeId,
    pub(in crate::aggregates::change_record) purpose_ref: String,
    pub(in crate::aggregates::change_record) intent: String,
    pub(in crate::aggregates::change_record) writer: Writer,
    pub(in crate::aggregates::change_record) items: Vec<Item>,
    pub(in crate::aggregates::change_record) replaces: Option<ChangeId>,
}

impl ChangeRecord {
    /// 作る。意図は1文（120字まで）、項目は1つ以上
    ///
    /// # Errors
    /// 目的の参照か意図が空、意図が120字を超える、項目が無いとき
    pub fn new(
        id: ChangeId,
        purpose_ref: &str,
        intent: &str,
        writer: Writer,
        items: Vec<Item>,
        replaces: Option<ChangeId>,
    ) -> Result<Self, String> {
        if purpose_ref.trim().is_empty() {
            return Err("目的の参照（--purpose）が空".to_string());
        }
        if intent.trim().is_empty() {
            return Err("意図（--intent）が空".to_string());
        }
        if intent.chars().count() > 120 {
            return Err("意図は120字までの1文で書く".to_string());
        }
        if items.is_empty() {
            return Err("変更の項目が無い".to_string());
        }
        Ok(ChangeRecord {
            id,
            purpose_ref: purpose_ref.to_string(),
            intent: intent.to_string(),
            writer,
            items,
            replaces,
        })
    }

    /// id
    pub fn id(&self) -> &ChangeId {
        &self.id
    }

    /// 目的の参照
    pub fn purpose_ref(&self) -> &str {
        &self.purpose_ref
    }

    /// 意図
    pub fn intent(&self) -> &str {
        &self.intent
    }

    /// 書いた者の種類
    pub fn writer(&self) -> Writer {
        self.writer
    }

    /// 変更の項目
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// 置き換える意図
    pub fn replaces(&self) -> Option<&ChangeId> {
        self.replaces.as_ref()
    }

    /// この対象のこの欄を変えた記録か
    pub fn touches(&self, target: &Target, field: &str) -> bool {
        self.items.iter().any(|i| {
            &i.target == target
                && i.field.as_deref().is_none_or(|f| {
                    f.is_empty() || f == field || field.starts_with(&format!("{f}/"))
                })
        })
    }

    /// change.schema.json の形
    pub fn to_json(&self) -> Value {
        let mut v = json!({
            "$schema": "urn:semantic-agent-schema:base:change:v1",
            "kind": "change",
            "id": self.id.as_str(),
            "purpose_ref": self.purpose_ref,
            "intent": self.intent,
            "writer_kind": self.writer.as_str(),
            "items": self.items.iter().map(|i| {
                let mut o = json!({"target": i.target.to_json()});
                if let Some(f) = &i.field { o["field"] = json!(f); }
                o
            }).collect::<Vec<_>>()
        });
        if let Some(r) = &self.replaces {
            v["replaces"] = json!(r.as_str());
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::{ChangeRecord, Item, Writer};
    use crate::values::change_id::ChangeId;
    use semantic_agent_schema_declaration::values::target::Target;

    fn target() -> Target {
        Target::Instance {
            id: "r-1".into(),
            kind: "urn:semantic-agent-schema:t:record:v1".into(),
        }
    }

    fn record(intent: &str, items: Vec<Item>) -> Result<ChangeRecord, String> {
        ChangeRecord::new(
            ChangeId::from_parts(1, 1),
            "0199",
            intent,
            Writer::AiAgent,
            items,
            None,
        )
    }

    /// traces: U5:事後
    #[test]
    fn record_touches_the_field_it_names() {
        let r = record(
            "理由を書き直す",
            vec![Item::new(target(), Some("/why".into())).unwrap()],
        )
        .unwrap();
        assert!(r.touches(&target(), "/why") && !r.touches(&target(), "/status"));
    }

    /// traces: U5:失敗時
    #[test]
    fn record_without_items_is_refused() {
        assert!(record("意図", vec![]).is_err());
    }

    /// traces: U5:失敗時
    #[test]
    fn intent_longer_than_120_is_refused() {
        assert!(record(&"あ".repeat(121), vec![Item::new(target(), None).unwrap()]).is_err());
    }

    /// traces: U13:事後
    #[test]
    fn record_has_no_session_id() {
        let r = record("意図", vec![Item::new(target(), None).unwrap()]).unwrap();
        assert!(r.to_json().get("session").is_none());
    }
}
