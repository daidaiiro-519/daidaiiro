//! CLI の受け口。`[--skill <Skill のフォルダ>] <ツール> --引数 値 …` をツールの一覧の呼び出しへ変換する。
//! `--version` だけを渡すと、エンジンの名前と版を返す。版は Cargo.toml の [workspace.package] の1か所から来る。

use crate::inbound::tools::{self, Toolbox};
use schema_driven_core::ports::{Files, Query, Schemas};
use serde_json::{Map, Value};

/// 先頭の `--skill <Skill のフォルダ>` を取り出す。残りの引数と一緒に返す。
pub fn split_skill(args: &[String]) -> (Option<&str>, &[String]) {
    match args {
        [flag, dir, rest @ ..] if flag == "--skill" => (Some(dir.as_str()), rest),
        _ => (None, args),
    }
}

/// エンジンの名前と版を data で返す。MCP の serverInfo と同じ値である。
pub fn version() -> Value {
    serde_json::json!({"ok": true, "findings": [], "data": {"name": "schema-driven", "version": env!("CARGO_PKG_VERSION")}})
}

/// 引数を読み、compose でツールの一覧を組んで呼び出し、終了コードと出力を返す。
pub fn run<F: Files, S: Schemas, Q: Query>(
    args: &[String],
    compose: impl Fn(Option<&str>) -> Toolbox<F, S, Q>,
) -> (i32, Value) {
    let (skill, rest) = split_skill(args);
    call(&compose(skill), rest)
}

/// `--skill` を取り除いた引数を読み、ツールの一覧から呼び出す。
pub fn call<F: Files, S: Schemas, Q: Query>(
    toolbox: &Toolbox<F, S, Q>,
    args: &[String],
) -> (i32, Value) {
    // 出力はもともとすべて JSON なので、--json はどこにあっても受け付けて取り除く
    let as_json = args.iter().any(|arg| arg == "--json");
    let args: Vec<&String> = args.iter().filter(|arg| *arg != "--json").collect();
    if args.len() == 1 && args[0] == "--version" {
        return (0, version());
    }
    let Some((command, rest)) = args.split_first() else {
        // 動詞なしの --json は、ツールの一覧を返す
        if as_json {
            return (0, tools::catalog());
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
    toolbox.dispatch(command, &map)
}

#[cfg(test)]
mod tests {
    //! CLI の変換（引数 ↔ 呼び出し）。動詞なしの `--json` で、ツールの一覧を返す（AI エージェントはこれでツールを調べる）。
    //! 出力はもともとすべて JSON なので、動詞のあとに付けた `--json` は受け付けて、何も変えない。

    use super::{call, split_skill};
    use crate::compose;
    use serde_json::Value;

    fn run(args: &[&str]) -> (i32, Value) {
        let args: Vec<String> = args.iter().map(|arg| (*arg).to_owned()).collect();
        call(&compose::toolbox(None), &args)
    }

    fn strings(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn json_without_a_tool_name_lists_every_tool_with_its_arguments() {
        let (code, out) = run(&["--json"]);
        assert_eq!(code, 0, "{out}");
        let tools = out["data"]["tools"].as_array().unwrap();
        let names: Vec<&str> = tools
            .iter()
            .map(|tool| tool["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            vec![
                "create",
                "get",
                "update",
                "delete",
                "prompt",
                "check",
                "check-schemas",
                "approve",
                "render"
            ]
        );
        let update = &tools[2];
        assert!(!update["summary"].as_str().unwrap().is_empty());
        assert_eq!(
            update["args"],
            serde_json::json!([
                {"name": "path", "required": true},
                {"name": "patch", "required": false},
                {"name": "patch_file", "required": false},
                {"name": "hash", "required": false}
            ])
        );
    }

    #[test]
    fn json_after_a_tool_name_is_accepted_and_changes_nothing() {
        assert_eq!(
            run(&["get", "--path", "missing.json", "--query", "id", "--json"]),
            run(&["get", "--path", "missing.json", "--query", "id"])
        );
    }

    #[test]
    fn version_alone_returns_the_name_and_the_version() {
        let (code, out) = run(&["--version"]);
        assert_eq!(code, 0, "{out}");
        assert_eq!(out["data"]["name"], "schema-driven");
        assert_eq!(out["data"]["version"], env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn version_after_a_tool_name_is_a_misuse() {
        assert_eq!(run(&["get", "--version"]).0, 2);
    }

    #[test]
    fn no_arguments_at_all_is_still_a_misuse() {
        let (code, out) = run(&[]);
        assert_eq!(code, 2, "{out}");
    }

    #[test]
    fn a_word_without_dashes_or_a_flag_without_a_value_is_a_misuse() {
        assert_eq!(run(&["get", "path"]).0, 2);
        assert_eq!(run(&["get", "--path"]).0, 2);
    }

    #[test]
    fn skill_is_taken_only_from_the_front() {
        let front = strings(&["--skill", "d", "check", "--dir", "x"]);
        assert_eq!(split_skill(&front), (Some("d"), &front[2..]));
        let later = strings(&["check", "--skill", "d"]);
        assert_eq!(split_skill(&later), (None, &later[..]));
    }
}
