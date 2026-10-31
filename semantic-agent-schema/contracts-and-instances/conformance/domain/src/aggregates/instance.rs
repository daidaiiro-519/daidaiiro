//! 集約インスタンス：Concrete に従って書いた JSON。
//!
//! 書く条件（ステップ2）：id を変えない ・ 導出値が一致する ・ 人の欄は人の入口から変える。
//! 指紋が一致したときだけ書く条件は、いまの中身を読む手続き（use_cases）の側で確かめる。
//! 欄は根の properties だけを見る（入れ子の欄はまだ見ない）。

use crate::aggregates::escape;
use crate::values::WriteViolation;
use serde_json::Value;

/// 書く依頼がどの入口から来たか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    /// 人（名乗らなかった呼び出し）
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

    /// インスタンスの id（参照の先として指される名前）。
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.document.get("id").and_then(Value::as_str)
    }

    /// 文書。
    #[must_use]
    pub const fn document(&self) -> &Value {
        &self.document
    }

    /// この中身を書いてよいか。before は直す前の中身（作成なら None）、concrete は従う Concrete の文書。
    ///
    /// # Errors
    /// id を変えた ・ 人の欄を人でない入口から決めた ・ 導出値と違う値を書いたとき（最初に見つけた1つ）。
    pub fn check_write(
        &self,
        before: Option<&Self>,
        concrete: &Value,
        entry: Entry,
    ) -> Result<(), WriteViolation> {
        if let Some(b) = before
            && b.id() != self.id()
        {
            return Err(WriteViolation::new(
                "id_changed",
                "/id",
                &format!(
                    "id を {} から {} へ変えようとした。id はほかのインスタンスから指される名前なので変えない",
                    b.id().unwrap_or("（無し）"),
                    self.id().unwrap_or("（無し）")
                ),
            ));
        }
        let Some(properties) = concrete.get("properties").and_then(Value::as_object) else {
            return Ok(());
        };
        for (name, field) in properties {
            let at = format!("/{}", escape(name));
            let now = self.document.get(name);
            if entry == Entry::Tool
                && field.get("x-decided-by").and_then(Value::as_str) == Some("human")
            {
                let changed = match before {
                    Some(b) => b.document.get(name) != now,
                    None => now.is_some() && now != field.get("default"),
                };
                if changed {
                    return Err(WriteViolation::new(
                        "human_field",
                        &at,
                        &format!(
                            "{at} は x-decided-by: human の欄で、人の入口（--session を付けない呼び出し）からだけ決める"
                        ),
                    ));
                }
            }
            if let Some(rule) = field.get("x-derived-from") {
                let target = rule.get("declared").and_then(Value::as_str).unwrap_or(name);
                let derived = derive(rule, &self.document);
                if let Some(written) = self.document.get(target)
                    && Some(written) != derived
                {
                    let at = format!("/{}", escape(target));
                    return Err(WriteViolation::new(
                        "derived_mismatch",
                        &at,
                        &format!(
                            "{at} は {written} だが、x-derived-from で導く値は {}",
                            derived.map_or_else(|| "（無し）".to_string(), Value::to_string)
                        ),
                    ));
                }
            }
        }
        Ok(())
    }
}

/// x-derived-from の決まりを上から当て、最初に当たった then、どれにも当たらなければ otherwise を返す。
fn derive<'a>(rule: &'a Value, document: &Value) -> Option<&'a Value> {
    let hit = rule
        .get("rules")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|r| {
            r.get("when")
                .and_then(Value::as_object)
                .is_some_and(|when| when.iter().all(|(k, v)| document.get(k) == Some(v)))
        });
    match hit {
        Some(r) => r.get("then"),
        None => rule.get("otherwise"),
    }
}

#[cfg(test)]
mod tests {
    use super::{Entry, Instance};
    use serde_json::{Value, json};

    fn concrete() -> Value {
        json!({"properties": {
            "status": {"x-decided-by": "human", "default": "draft"},
            "review": {"x-derived-from": {"rules": [{"when": {"risk": "high"}, "then": "by-two"}], "otherwise": "by-one"}}
        }})
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
    fn id_is_kept() {
        let b = inst(&json!({"$schema": "c", "id": "D1"}));
        let a = inst(&json!({"$schema": "c", "id": "D2"}));
        assert_eq!(
            code(a.check_write(Some(&b), &concrete(), Entry::Human)),
            "id_changed"
        );
    }

    #[test]
    fn human_field_from_tool_only_default_on_create() {
        let draft = inst(&json!({"$schema": "c", "id": "D1", "status": "draft"}));
        let approved = inst(&json!({"$schema": "c", "id": "D1", "status": "approved"}));
        assert_eq!(
            code(draft.check_write(None, &concrete(), Entry::Tool)),
            "ok"
        );
        assert_eq!(
            code(approved.check_write(None, &concrete(), Entry::Tool)),
            "human_field"
        );
        assert_eq!(
            code(approved.check_write(Some(&draft), &concrete(), Entry::Tool)),
            "human_field"
        );
        assert_eq!(
            code(approved.check_write(Some(&draft), &concrete(), Entry::Human)),
            "ok"
        );
    }

    #[test]
    fn derived_value_must_match() {
        let ok = inst(&json!({"$schema": "c", "risk": "high", "review": "by-two"}));
        let bad = inst(&json!({"$schema": "c", "risk": "high", "review": "by-one"}));
        let absent = inst(&json!({"$schema": "c", "risk": "high"}));
        assert_eq!(code(ok.check_write(None, &concrete(), Entry::Human)), "ok");
        assert_eq!(
            code(bad.check_write(None, &concrete(), Entry::Human)),
            "derived_mismatch"
        );
        assert_eq!(
            code(absent.check_write(None, &concrete(), Entry::Human)),
            "ok"
        );
    }
}
