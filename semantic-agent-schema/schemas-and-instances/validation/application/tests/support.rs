//! 検証 ・ 書く ・ 検索の手続きの試験が共に使う、ポートの代役。代役は試験の側に置く
//! （test-support に置くと、test-support が application に依存し、application の試験が test-support に依存して循環する）。
//! 書く ・ 検索のモジュールの試験は、#[path] でこのファイルを読む。
//! 文書の置き場所と形の検査のポートはモジュールごとに宣言するので、ここでは代役の振る舞いだけを持ち、
//! ポートへの当て方（impl）は読む側の試験が、そのモジュールのポートに書く。記録のポートも、モジュールごとに宣言したものに当てる。
//! 誰が打ったかは、ユースケースへ値（Caller）で渡す。

#![allow(dead_code, clippy::unwrap_used, clippy::panic)]

use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;
use validation_domain::values::{Judgment, ValidateRow};
use writing_domain::values::{Caller, ChangeRow, SchemaPlaces, Via, WriteRecord};

pub(crate) const ABSTRACT: &str = "urn:semantic-agent-schema:published-language:abstract:v1";
pub(crate) const UC: &str = "urn:example:t:uc:v1";

pub(crate) struct Files(pub(crate) RefCell<BTreeMap<String, String>>);
impl Files {
    pub(crate) fn read(&self, path: &str) -> Result<String, String> {
        self.0
            .borrow()
            .get(path)
            .cloned()
            .ok_or_else(|| format!("{path} が無い"))
    }
    pub(crate) fn write(&self, path: &str, text: &str) -> Result<(), String> {
        self.0.borrow_mut().insert(path.into(), text.into());
        Ok(())
    }
    pub(crate) fn remove(&self, path: &str) -> Result<(), String> {
        self.0
            .borrow_mut()
            .remove(path)
            .map(|_| ())
            .ok_or_else(|| format!("{path} が無い"))
    }
}

/// 形の検査はいつも通す代役。Abstract は根に rule を1つ持つ。
pub(crate) struct Lenient;
impl Lenient {
    pub(crate) fn shipped(&self, id: &str) -> Option<Value> {
        (id == ABSTRACT).then(|| json!({"x-prompt": {"rules": [{"id": "goal-is-ok"}]}}))
    }
    pub(crate) fn check(
        &self,
        _: &str,
        _: &Value,
        _: &[(String, Value)],
    ) -> Result<Vec<(String, String)>, String> {
        Ok(Vec::new())
    }
    /// 根の欄だけを見る代役：インスタンスにある根の欄ごとに、Concrete のその欄の定義を注釈として返す。
    pub(crate) fn annotations(
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

/// 人の呼び出し。
pub(crate) const HUMAN: Caller = Caller::Human;

/// ツールの環境変数で名乗ったセッション。
pub(crate) fn tool() -> Caller {
    Caller::Session {
        id: "agent".into(),
        tool: "claude-code".into(),
        folder: "/w".into(),
        via: Via::Env,
    }
}

/// 検証と変更を、足した順に1つの並びで持つ代役。
#[derive(Default)]
pub(crate) struct Log(
    pub(crate) Vec<ValidateRow>,
    pub(crate) Vec<(String, ChangeRow)>,
    /// 検証の操作の ID（0 と同じ並び。試験が 0 へ直に足した検証は、検証だけの並びの番号にする）
    pub(crate) Vec<String>,
);
impl Log {
    fn next(&self) -> String {
        format!("o-{}", self.0.len() + self.1.len() + 1)
    }
    pub(crate) fn append_validate(
        &mut self,
        _: Option<&str>,
        row: &ValidateRow,
    ) -> Result<String, String> {
        let id = self.next();
        self.0.push(row.clone());
        self.2.push(id.clone());
        Ok(id)
    }
    pub(crate) fn append_change(
        &mut self,
        _: Option<&str>,
        row: &ChangeRow,
    ) -> Result<String, String> {
        let id = self.next();
        self.1.push((id.clone(), row.clone()));
        Ok(id)
    }
    pub(crate) fn writes(&self, target_path: &str) -> Result<Vec<WriteRecord>, String> {
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
                same_worktree: true,
                row: r.clone(),
            })
            .collect())
    }
    pub(crate) fn instances_of(&self, schema: &str) -> Result<Vec<String>, String> {
        let mut last: BTreeMap<&str, &ChangeRow> = BTreeMap::new();
        for (_, r) in &self.1 {
            last.insert(r.target_path.as_str(), r);
        }
        Ok(last
            .into_iter()
            .filter(|(_, r)| r.kind != "delete" && r.schema.as_deref() == Some(schema))
            .map(|(p, _)| p.to_string())
            .collect())
    }
    pub(crate) fn last_judgments(&self, target_path: &str) -> Result<Vec<Judgment>, String> {
        // rule ごとに、その rule を最後に判定した検証の行（検証の ID は記録した操作の ID）
        let mut latest: BTreeMap<String, Vec<Judgment>> = BTreeMap::new();
        for (i, r) in self.0.iter().enumerate() {
            if r.target_path != target_path {
                continue;
            }
            let mut here: BTreeMap<String, Vec<Judgment>> = BTreeMap::new();
            for rule in &r.results {
                here.entry(rule.rule.clone()).or_default().push(Judgment {
                    operation: self.2.get(i).cloned().unwrap_or(format!("o-{}", i + 1)),
                    hash: r.hash.clone(),
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
        "goal": {"x-prompt": {"rules": [{"id": "goal-is-outcome"}]}}
    }})
}

/// rule を1つ持つ Concrete。goal の判定を打ったセッションから受け取る。
pub(crate) fn meaning() -> Value {
    json!({"$schema": ABSTRACT, "$id": UC,
           "properties": {
        "goal": {"x-prompt": {"rules": [{"id": "goal-is-outcome", "rule": "利用者が得る結果を書く。",
                 "why": "手段だと果たせたかを決められない。", "check": "結果を表す名詞句か。"}]}}
    }})
}

/// スキーマの置き場所（渡したファイルをそのまま読む）。
pub(crate) fn places(files: &[&str]) -> SchemaPlaces {
    let files: Vec<String> = files.iter().map(ToString::to_string).collect();
    SchemaPlaces::new(files.clone(), files)
}
