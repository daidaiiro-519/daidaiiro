//! 差分：前後の中身から、JSON Pointer ごとに求めた変わった欄。Git の行の差分は使わない。

use crate::aggregates::escape;
use serde_json::Value;

/// 欄の変わり方。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// 前に無かった欄が足された
    Add,
    /// 前にあった欄が無くなった
    Remove,
    /// 欄の値が変わった
    Replace,
}

impl Change {
    /// 公開された言葉での名前。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Remove => "remove",
            Self::Replace => "replace",
        }
    }

    /// 名前から読む。知らない名前は None。
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "add" => Some(Self::Add),
            "remove" => Some(Self::Remove),
            "replace" => Some(Self::Replace),
            _ => None,
        }
    }
}

/// 変わった欄1つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldChange {
    pointer: String,
    change: Change,
    before: Option<Value>,
    after: Option<Value>,
}

impl FieldChange {
    /// 欄の変わり方を作る。add は before を、remove は after を持たない。
    #[must_use]
    pub fn new(pointer: &str, change: Change, before: Option<Value>, after: Option<Value>) -> Self {
        let (before, after) = match change {
            Change::Add => (None, after),
            Change::Remove => (before, None),
            Change::Replace => (before, after),
        };
        Self {
            pointer: pointer.to_string(),
            change,
            before,
            after,
        }
    }

    /// 変わった欄（JSON Pointer）。
    #[must_use]
    pub fn pointer(&self) -> &str {
        &self.pointer
    }

    /// 変わり方。
    #[must_use]
    pub const fn change(&self) -> Change {
        self.change
    }

    /// 前の値（remove ・ replace）。
    #[must_use]
    pub const fn before(&self) -> Option<&Value> {
        self.before.as_ref()
    }

    /// 後の値（add ・ replace）。
    #[must_use]
    pub const fn after(&self) -> Option<&Value> {
        self.after.as_ref()
    }
}

/// 前後の中身の差分（欄の並びは JSON Pointer の順）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diff(Vec<FieldChange>);

impl Diff {
    /// 前後の中身から求める。前が無い（作成）ときは、空のオブジェクトからの差分にする。
    /// オブジェクトはキーごと、配列は位置ごとに降り、それ以外は値が違えば replace にする。
    #[must_use]
    pub fn between(before: Option<&Value>, after: &Value) -> Self {
        let empty = Value::Object(serde_json::Map::new());
        let mut out = Vec::new();
        walk(before.unwrap_or(&empty), after, "", &mut out);
        out.sort_by(|a, b| a.pointer.cmp(&b.pointer));
        Self(out)
    }

    /// 記録から戻す。
    #[must_use]
    pub const fn from_changes(changes: Vec<FieldChange>) -> Self {
        Self(changes)
    }

    /// 変わった欄。
    #[must_use]
    pub fn changes(&self) -> &[FieldChange] {
        &self.0
    }

    /// 変わった欄が無いか。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn walk(before: &Value, after: &Value, at: &str, out: &mut Vec<FieldChange>) {
    match (before, after) {
        (Value::Object(b), Value::Object(a)) => {
            for (k, bv) in b {
                let here = format!("{at}/{}", escape(k));
                match a.get(k) {
                    Some(av) => walk(bv, av, &here, out),
                    None => out.push(FieldChange::new(
                        &here,
                        Change::Remove,
                        Some(bv.clone()),
                        None,
                    )),
                }
            }
            for (k, av) in a {
                if !b.contains_key(k) {
                    let here = format!("{at}/{}", escape(k));
                    out.push(FieldChange::new(&here, Change::Add, None, Some(av.clone())));
                }
            }
        }
        (Value::Array(b), Value::Array(a)) => {
            for i in 0..b.len().max(a.len()) {
                let here = format!("{at}/{i}");
                match (b.get(i), a.get(i)) {
                    (Some(bv), Some(av)) => walk(bv, av, &here, out),
                    (Some(bv), None) => {
                        out.push(FieldChange::new(
                            &here,
                            Change::Remove,
                            Some(bv.clone()),
                            None,
                        ));
                    }
                    (None, Some(av)) => {
                        out.push(FieldChange::new(&here, Change::Add, None, Some(av.clone())));
                    }
                    (None, None) => {}
                }
            }
        }
        _ if before != after => out.push(FieldChange::new(
            at,
            Change::Replace,
            Some(before.clone()),
            Some(after.clone()),
        )),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{Change, Diff};
    use serde_json::json;

    #[test]
    fn create_is_add_per_root_key() {
        let d = Diff::between(None, &json!({"b": 1, "a": [1]}));
        let p: Vec<_> = d
            .changes()
            .iter()
            .map(|c| (c.pointer(), c.change()))
            .collect();
        assert_eq!(p, vec![("/a", Change::Add), ("/b", Change::Add)]);
    }

    #[test]
    fn only_changed_fields() {
        let d = Diff::between(
            Some(&json!({"id": "U1", "goal": "x", "calls": ["U2"], "a/b": 1})),
            &json!({"id": "U1", "goal": "y", "calls": [], "a/b": 1}),
        );
        let p: Vec<_> = d
            .changes()
            .iter()
            .map(|c| (c.pointer(), c.change()))
            .collect();
        assert_eq!(
            p,
            vec![("/calls/0", Change::Remove), ("/goal", Change::Replace)]
        );
        assert_eq!(d.changes()[1].before(), Some(&json!("x")));
        assert_eq!(d.changes()[0].after(), None);
    }

    #[test]
    fn null_is_a_value() {
        let d = Diff::between(Some(&json!({"a": null})), &json!({"a": 1}));
        assert_eq!(d.changes()[0].before(), Some(&json!(null)));
        assert!(Diff::between(Some(&json!({"a": 1})), &json!({"a": 1})).is_empty());
    }
}
