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

/// 先頭の `--skill <Skill のフォルダ>` を取り出す。残りの引数と一緒に返す。
pub fn split_skill(args: &[String]) -> (Option<String>, Vec<String>) {
    match args {
        [flag, dir, rest @ ..] if flag == "--skill" => (Some(dir.clone()), rest.to_vec()),
        _ => (None, args.to_vec()),
    }
}

/// 引数を読み、ツールの一覧（具体のツールを追加したものでもよい）から呼び出す。
pub fn run_in(
    toolbox: &Toolbox,
    args: &[String],
    instances: &dyn InstanceUseCases,
    checks: &dyn CheckUseCases,
) -> (i32, Value) {
    // 出力はもともとすべて JSON なので、--json はどこにあっても受け付けて取り除く
    let as_json = args.iter().any(|arg| arg == "--json");
    let args: Vec<&String> = args.iter().filter(|arg| *arg != "--json").collect();
    let Some((command, rest)) = args.split_first() else {
        // 動詞なしの --json は、ツールの一覧を返す
        if as_json {
            if let Some(failure) = toolbox.declaration_failure() {
                return failure;
            }
            return (0, toolbox.catalog());
        }
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
        map.insert(name.to_owned(), Value::String((*value).clone()));
    }
    toolbox.dispatch(command, &map, instances, checks)
}
