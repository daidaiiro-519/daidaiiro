//! ユースケース駆動の concrete。基盤の7つの契約を使い、ドリフト検知を足す。
use crate::base;
use serde_json::Value;
use std::collections::BTreeMap;

/// テスト仕様：ユースケースのシナリオと、集約の条件。ID は「宣言の ID.項目の ID」。
pub fn spec_items(decls: &[Value]) -> Vec<String> {
    let mut ids = vec![];
    for d in decls {
        let owner = base::get(d, "id");
        let items = base::get(d, "[scenarios[].id, conditions[].id][] | [?@ != null]");
        for i in items.as_array().unwrap() {
            ids.push(format!("{}.{}", owner.as_str().unwrap(), i.as_str().unwrap()));
        }
    }
    ids
}

pub struct Drift { pub missing: Vec<String>, pub extra: Vec<String>, pub failing: Vec<String> }

/// ドリフト検知の契約：テスト仕様と、テスト実装の報告（1行に ID と pass ・ fail）を突き合わせる。
pub fn drift(spec: &[String], report: &str) -> Drift {
    let tests: BTreeMap<&str, &str> = report.lines().filter_map(|l| l.split_once('\t')).collect();
    Drift {
        missing: spec.iter().filter(|s| !tests.contains_key(s.as_str())).cloned().collect(),
        extra: tests.keys().filter(|t| !spec.iter().any(|s| s == *t)).map(|t| t.to_string()).collect(),
        failing: tests.iter().filter(|(t, r)| **r != "pass" && spec.iter().any(|s| s == *t)).map(|(t, _)| t.to_string()).collect(),
    }
}

/// 削除した宣言を指す参照を探す。
pub fn dangling(decls: &[Value], removed: &str) -> Vec<String> {
    decls.iter()
        .filter(|d| base::get(d, "writes").as_array().is_some_and(|w| w.iter().any(|x| x == removed)))
        .map(|d| base::get(d, "id").as_str().unwrap().to_string())
        .collect()
}
