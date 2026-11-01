//! 検証と書くの手続きの試験が共に使う、ポートの代役。代役は試験の側に置く
//! （test-support に置くと、test-support が application に依存し、application の試験が test-support に依存して循環する）。
//! 書くモジュールの試験は、#[path] でこのファイルを読む。

#![allow(dead_code, clippy::unwrap_used, clippy::panic)]

use recording_application::ports::{
    Caller, ChangeRow, IdentifyCaller, Judgment, OperationLog, SessionRow, StoredChange,
    ValidateRow, Via, WriteRecord,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;
use validation_application::ports::{DocumentStore, SchemaValidator};

pub(crate) const ABSTRACT: &str = "urn:semantic-agent-schema:published-language:abstract:v1";
pub(crate) const UC: &str = "urn:example:t:uc:v1";

pub(crate) struct Files(pub(crate) RefCell<BTreeMap<String, String>>);
impl DocumentStore for Files {
    fn list(&self) -> Result<Vec<String>, String> {
        Ok(self.0.borrow().keys().cloned().collect())
    }
    fn read(&self, path: &str) -> Result<String, String> {
        self.0
            .borrow()
            .get(path)
            .cloned()
            .ok_or_else(|| format!("{path} が無い"))
    }
    fn write(&self, path: &str, text: &str) -> Result<(), String> {
        self.0.borrow_mut().insert(path.into(), text.into());
        Ok(())
    }
    fn remove(&self, path: &str) -> Result<(), String> {
        self.0
            .borrow_mut()
            .remove(path)
            .map(|_| ())
            .ok_or_else(|| format!("{path} が無い"))
    }
}

/// 形の検査はいつも通す代役。Abstract は根に rule を1つ持つ。
pub(crate) struct Lenient;
impl SchemaValidator for Lenient {
    fn shipped(&self, id: &str) -> Option<Value> {
        (id == ABSTRACT).then(|| json!({"x-prompt": {"write": [{"id": "goal-is-ok"}]}}))
    }
    fn check(
        &self,
        _: &str,
        _: &Value,
        _: &[(String, Value)],
    ) -> Result<Vec<(String, String)>, String> {
        Ok(Vec::new())
    }
    /// 根の欄だけを見る代役：インスタンスにある根の欄ごとに、Concrete のその欄の定義を注釈として返す。
    fn annotations(
        &self,
        schema: &str,
        value: &Value,
        more: &[(String, Value)],
    ) -> Result<Vec<(String, Value)>, String> {
        let Some((_, doc)) = more.iter().find(|(id, _)| id == schema) else {
            return Ok(Vec::new());
        };
        Ok(doc
            .get("properties")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .filter(|(name, _)| value.get(name.as_str()).is_some())
            .map(|(name, field)| (format!("/{name}"), field.clone()))
            .collect())
    }
}

pub(crate) struct Human;
impl IdentifyCaller for Human {
    fn identify(&self) -> Caller {
        Caller::Human
    }
}

pub(crate) struct Tool;
impl IdentifyCaller for Tool {
    fn identify(&self) -> Caller {
        Caller::Session {
            id: "agent".into(),
            tool: "claude-code".into(),
            folder: "/w".into(),
            via: Via::Env,
        }
    }
}

/// 検証と変更を、足した順に1つの並びで持つ代役。
#[derive(Default)]
pub(crate) struct Log(
    pub(crate) Vec<ValidateRow>,
    pub(crate) Vec<(String, ChangeRow)>,
);
impl Log {
    fn next(&self) -> String {
        format!("o-{}", self.0.len() + self.1.len() + 1)
    }
}
impl OperationLog for Log {
    fn session(&mut self, caller: &Caller) -> Result<Option<String>, String> {
        Ok(match caller {
            Caller::Session { id, .. } => Some(id.clone()),
            Caller::Human => None,
        })
    }
    fn session_row(&self, _: &str) -> Result<Option<SessionRow>, String> {
        Ok(None)
    }
    fn initialized(&self, _: &str) -> Result<Option<SessionRow>, String> {
        Ok(None)
    }
    fn new_session_id(&self) -> String {
        "sas-1".into()
    }
    fn append_validate(&mut self, _: Option<&str>, row: &ValidateRow) -> Result<String, String> {
        let id = self.next();
        self.0.push(row.clone());
        Ok(id)
    }
    fn append_change(&mut self, _: Option<&str>, row: &ChangeRow) -> Result<String, String> {
        let id = self.next();
        self.1.push((id.clone(), row.clone()));
        Ok(id)
    }
    fn find_request(&self, request_id: &str) -> Result<Option<StoredChange>, String> {
        Ok(self
            .1
            .iter()
            .find(|(_, r)| r.request_id.as_deref() == Some(request_id))
            .map(|(id, r)| StoredChange {
                operation: id.clone(),
                session: None,
                row: r.clone(),
            }))
    }
    fn writes(&self, target_path: &str) -> Result<Vec<WriteRecord>, String> {
        let from = self
            .1
            .iter()
            .rposition(|(_, r)| r.target_path == target_path && r.kind == "delete")
            .map_or(0, |i| i + 1);
        Ok(self.1[from..]
            .iter()
            .filter(|(_, r)| {
                r.target_path == target_path && (r.kind == "create" || r.kind == "update")
            })
            .map(|(id, r)| WriteRecord {
                operation: id.clone(),
                session: None,
                at: "t".into(),
                row: r.clone(),
            })
            .collect())
    }
    fn last_judgments(&self, target_path: &str) -> Result<Vec<Judgment>, String> {
        // rule ごとに、その rule を最後に判定した検証の行（検証の ID は検証だけの並びの番号）
        let mut latest: BTreeMap<String, Vec<Judgment>> = BTreeMap::new();
        for (i, r) in self.0.iter().enumerate() {
            if r.target_path != target_path {
                continue;
            }
            let mut here: BTreeMap<String, Vec<Judgment>> = BTreeMap::new();
            for rule in &r.results {
                here.entry(rule.rule.clone()).or_default().push(Judgment {
                    operation: format!("o-{}", i + 1),
                    rule: rule.clone(),
                });
            }
            latest.extend(here);
        }
        Ok(latest.into_values().flatten().collect())
    }
}

pub(crate) fn files(entries: &[(&str, Value)]) -> Files {
    Files(RefCell::new(
        entries
            .iter()
            .map(|(p, v)| ((*p).to_string(), v.to_string()))
            .collect(),
    ))
}

pub(crate) fn concrete() -> Value {
    json!({"$schema": ABSTRACT, "$id": UC, "properties": {"$schema": {"const": UC}, "calls": {"items": {"type": "string"}}}})
}

pub(crate) fn decided() -> Value {
    json!({"$schema": ABSTRACT, "$id": UC, "properties": {
        "status": {"x-decided-by": "human", "default": "draft"},
        "goal": {"x-prompt": {"write": [{"id": "goal-is-outcome"}]}}
    }})
}

/// rule を1つ持つ Concrete。goal の判定を打ったセッションから受け取る。
pub(crate) fn meaning() -> Value {
    json!({"$schema": ABSTRACT, "$id": UC,
           "properties": {
        "goal": {"x-prompt": {"write": [{"id": "goal-is-outcome", "rule": "利用者が得る結果を書く。",
                 "why": "手段だと果たせたかを決められない。", "test": "結果を表す名詞句か。"}]}}
    }})
}
