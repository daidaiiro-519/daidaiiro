//! CLI の受け口。引数を読み、ユースケースを呼び、結果を JSON で標準出力へ書く。
//! 終了コードは 0 成功 ／ 1 ユースケースの失敗 ／ 2 使い方の誤り。

use schema_driven_core::application::instances::{UnfilledWithPrompt, UseCaseError};
use schema_driven_core::domain::values::ValidationError;
use schema_driven_core::ports::inbound::InstanceUseCases;
use serde_json::{json, Value};
use std::collections::BTreeMap;

const USAGE: &str = "使い方: schema-driven <create|get|update|delete|prompt> --キー 値 …";

fn errors(list: &[ValidationError]) -> Value {
    list.iter()
        .map(|e| json!({"property": e.property(), "reason": e.reason()}))
        .collect()
}

fn unfilled(list: &[UnfilledWithPrompt]) -> Value {
    list.iter()
        .map(|u| json!({"property": u.property, "prompt": u.prompt}))
        .collect()
}

fn failed(e: UseCaseError) -> (i32, Value) {
    (
        1,
        json!({"ok": false, "reason": e.reason, "detail": e.detail, "errors": errors(&e.errors)}),
    )
}

fn misuse(detail: &str) -> (i32, Value) {
    (
        2,
        json!({"ok": false, "reason": "使い方が違う", "detail": detail, "usage": USAGE}),
    )
}

/// 引数を読み、ユースケースを呼んで、終了コードと出力を返す。
pub fn run(args: &[String], uc: &dyn InstanceUseCases) -> (i32, Value) {
    let Some((command, rest)) = args.split_first() else {
        return misuse("サブコマンドが無い");
    };
    let mut opts = BTreeMap::new();
    let mut it = rest.iter();
    while let Some(key) = it.next() {
        let Some(name) = key.strip_prefix("--") else {
            return misuse(&format!("知らない引数: {key}"));
        };
        let Some(value) = it.next() else {
            return misuse(&format!("--{name} に値が無い"));
        };
        opts.insert(name.to_owned(), value.clone());
    }
    let need = |k: &str| opts.get(k).cloned();
    macro_rules! req {
        ($k:expr) => {
            match need($k) {
                Some(v) => v,
                None => return misuse(&format!("--{} が無い", $k)),
            }
        };
    }
    match command.as_str() {
        "create" => match uc.create(&req!("schema"), &req!("path")) {
            Ok(c) => (
                0,
                json!({"ok": true, "path": c.path, "hash": c.hash, "unfilled": unfilled(&c.unfilled)}),
            ),
            Err(e) => failed(e),
        },
        "get" => match uc.get(&req!("path"), &req!("query")) {
            Ok(g) => (0, json!({"ok": true, "value": g.value, "found": g.found})),
            Err(e) => failed(e),
        },
        "update" => match uc.update(
            &req!("path"),
            &req!("patch"),
            opts.get("hash").map(String::as_str),
        ) {
            Ok(u) => (
                0,
                json!({"ok": true, "hash": u.hash, "changed": u.changed, "errors": errors(&u.errors), "unfilled": unfilled(&u.unfilled)}),
            ),
            Err(e) => failed(e),
        },
        "delete" => match uc.delete(&req!("path")) {
            Ok(()) => (0, json!({"ok": true})),
            Err(e) => failed(e),
        },
        "prompt" => match uc.prompt(&req!("schema"), &req!("property")) {
            Ok(p) => (0, json!({"ok": true, "prompt": p.prompt})),
            Err(e) => failed(e),
        },
        other => misuse(&format!("知らないサブコマンド: {other}")),
    }
}
