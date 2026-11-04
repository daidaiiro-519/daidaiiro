//! 集約 Instance：Concrete に従って書いた JSON。
//!
//! 書く条件：人の欄（x-decided-by: human）は人の入口から変える。人の欄の所は入れ子のどこでも、
//! 手続き（use_cases）がバリデーターの注釈から集めて渡す。導出値は JSON Schema（if ・ then）が見る。
//! ハッシュが一致したときだけ書く条件は、いまの中身を読む手続きの側で確かめる。

use crate::values::WriteViolation;
use serde_json::Value;

/// 書く依頼がどの入口から来たか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    /// 人（--human で明示した呼び出し）
    Human,
    /// 名乗ったツール（AI のエージェント ・ CI ・ オーケストレーター）
    Tool,
}

/// インスタンスの文書。
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    document: Value,
}

impl Instance {
    /// 文書から作る。$schema が文字列でなければ None。
    #[must_use]
    pub fn read(document: &Value) -> Option<Self> {
        document.get("$schema")?.as_str()?;
        Some(Self {
            document: document.clone(),
        })
    }

    /// 従う Concrete の $id。
    #[must_use]
    pub fn concrete(&self) -> &str {
        self.document
            .get("$schema")
            .and_then(Value::as_str)
            .unwrap_or_default()
    }

    /// 文書。
    #[must_use]
    pub const fn document(&self) -> &Value {
        &self.document
    }

    /// この中身を書いてよいか。before は直す前の中身（作成なら None）、human は人が決める欄
    /// （x-decided-by: human が当たったインスタンスの所と、その default）。所は入れ子のどこでもよい。
    ///
    /// # Errors
    /// 人の欄を人でない入口から決めたとき（最初に見つけた1つ）。
    pub fn check_write(
        &self,
        before: Option<&Self>,
        human: &[(String, Option<Value>)],
        entry: Entry,
    ) -> Result<(), WriteViolation> {
        if entry != Entry::Tool {
            return Ok(());
        }
        for (at, default) in human {
            let now = self.document.pointer(at);
            let changed = match before {
                Some(b) => b.document.pointer(at) != now,
                None => now.is_some() && now != default.as_ref(),
            };
            if changed {
                return Err(WriteViolation::new(
                    "human_field",
                    at,
                    &format!(
                        "{at} は x-decided-by: human の欄で、人の入口（--session を付けない呼び出し）からだけ決める"
                    ),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Entry, Instance};
    use serde_json::{Value, json};

    /// 人の欄：根の status と、入れ子の steps/0/approved。
    fn human() -> Vec<(String, Option<Value>)> {
        vec![
            ("/status".to_string(), Some(json!("draft"))),
            ("/steps/0/approved".to_string(), Some(json!(false))),
        ]
    }

    fn inst(v: &Value) -> Instance {
        Instance::read(v).unwrap_or_else(|| Instance {
            document: v.clone(),
        })
    }

    fn code(r: Result<(), crate::values::WriteViolation>) -> &'static str {
        r.err().map_or("ok", |v| v.code())
    }

    #[test]
    fn human_field_from_tool_only_default_on_create() {
        let draft = inst(&json!({"$schema": "c", "id": "D1", "status": "draft"}));
        let approved = inst(&json!({"$schema": "c", "id": "D1", "status": "approved"}));
        assert_eq!(code(draft.check_write(None, &human(), Entry::Tool)), "ok");
        assert_eq!(
            code(approved.check_write(None, &human(), Entry::Tool)),
            "human_field"
        );
        assert_eq!(
            code(approved.check_write(Some(&draft), &human(), Entry::Tool)),
            "human_field"
        );
        assert_eq!(
            code(approved.check_write(Some(&draft), &human(), Entry::Human)),
            "ok"
        );
    }

    #[test]
    fn nested_human_field_is_kept_too() {
        let before =
            inst(&json!({"$schema": "c", "status": "draft", "steps": [{"approved": false}]}));
        let after =
            inst(&json!({"$schema": "c", "status": "draft", "steps": [{"approved": true}]}));
        assert_eq!(
            code(after.check_write(Some(&before), &human(), Entry::Tool)),
            "human_field"
        );
        let removed = inst(&json!({"$schema": "c", "status": "draft", "steps": [{}]}));
        assert_eq!(
            code(removed.check_write(Some(&before), &human(), Entry::Tool)),
            "human_field",
            "消すのも変えること"
        );
    }
}
