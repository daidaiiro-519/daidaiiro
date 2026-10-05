//! CLI の受け口。`<道具> --引数 値 …` を道具の一覧の呼び出しへ直す。

use crate::inbound::tools;
use schema_driven_core::ports::inbound::{CheckUseCases, InstanceUseCases};
use serde_json::{Map, Value};

/// 引数を読み、道具を呼んで、終了コードと出力を返す。
pub fn run(args: &[String], uc: &dyn InstanceUseCases, cc: &dyn CheckUseCases) -> (i32, Value) {
    let Some((command, rest)) = args.split_first() else {
        return tools::misuse("道具の名前が無い");
    };
    let mut map = Map::new();
    let mut it = rest.iter();
    while let Some(key) = it.next() {
        let Some(name) = key.strip_prefix("--") else {
            return tools::misuse(&format!("知らない引数: {key}"));
        };
        let Some(value) = it.next() else {
            return tools::misuse(&format!("--{name} に値が無い"));
        };
        map.insert(name.to_owned(), Value::String(value.clone()));
    }
    tools::dispatch(command, &map, uc, cc)
}
