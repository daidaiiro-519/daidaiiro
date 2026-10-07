//! CLI の受け口。`<ツール> --引数 値 …` をツールの一覧の呼び出しへ変換する。

use crate::inbound::tools::{self, Toolbox};
use schema_driven_core::ports::inbound::{CheckUseCases, InstanceUseCases};
use serde_json::{Map, Value};

/// 引数を読み、基盤のツールを呼び出して、終了コードと出力を返す。
pub fn run(
    args: &[String],
    instances: &dyn InstanceUseCases,
    checks: &dyn CheckUseCases,
) -> (i32, Value) {
    run_in(&Toolbox::base(), args, instances, checks)
}

/// 引数を読み、ツールの一覧（具体のツールを追加したものでもよい）から呼び出す。
pub fn run_in(
    toolbox: &Toolbox,
    args: &[String],
    instances: &dyn InstanceUseCases,
    checks: &dyn CheckUseCases,
) -> (i32, Value) {
    let Some((command, rest)) = args.split_first() else {
        return tools::misuse("ツールの名前が無い");
    };
    let mut map = Map::new();
    let mut words = rest.iter();
    while let Some(key) = words.next() {
        let Some(name) = key.strip_prefix("--") else {
            return tools::misuse(&format!("知らない引数: {key}"));
        };
        let Some(value) = words.next() else {
            return tools::misuse(&format!("--{name} に値が無い"));
        };
        map.insert(name.to_owned(), Value::String(value.clone()));
    }
    toolbox.dispatch(command, &map, instances, checks)
}
