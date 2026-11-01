//! リポジトリの文書と、そこから読んだ Concrete の一覧。

use crate::aggregates::ConcreteSchema;
use serde_json::Value;
use std::collections::BTreeMap;

/// リポジトリの文書1つ。
#[derive(Debug, Clone, PartialEq)]
pub struct Doc {
    /// 作業ディレクトリからの経路
    pub path: String,
    /// 読んだ文字列
    pub text: String,
    /// 読んだ JSON
    pub value: Value,
}

/// リポジトリの文書と、そこから読んだ Concrete ・ インスタンスの一覧。
#[derive(Debug, Clone, PartialEq)]
pub struct Corpus {
    /// 読めた文書
    pub docs: Vec<Doc>,
    /// $id → Concrete
    pub concretes: BTreeMap<String, ConcreteSchema>,
    /// バリデーターに渡す Concrete（$id と文書）
    pub more: Vec<(String, Value)>,
}

impl Corpus {
    /// 読んだ文書（経路と文字列）から作る。JSON として読めない文書は除く。
    #[must_use]
    pub fn read(texts: impl IntoIterator<Item = (String, String)>) -> Self {
        let docs: Vec<Doc> = texts
            .into_iter()
            .filter_map(|(path, text)| {
                let value = serde_json::from_str::<Value>(&text).ok()?;
                Some(Doc { path, text, value })
            })
            .collect();
        let concretes: BTreeMap<String, ConcreteSchema> = docs
            .iter()
            .filter_map(|d| ConcreteSchema::read(&d.value))
            .filter_map(|c| Some((c.id()?.to_string(), c)))
            .collect();
        let more: Vec<(String, Value)> = concretes
            .iter()
            .map(|(id, c)| (id.clone(), c.document().clone()))
            .collect();
        Self {
            docs,
            concretes,
            more,
        }
    }
}
