//! 固有のツールの宣言（Skill のフォルダの tools.json）を読む。形は abstract スキーマの宣言の形で検証し、
//! 形では書けない決まり（コマンドが Skill のフォルダの中にある ・ 名前が重ならない）をここで確かめる。

use crate::domain::contract;
use crate::domain::schema::Schema;
use serde_json::Value;
use std::collections::BTreeSet;

/// 制限時間を書かなかったときの秒数。
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 30;

/// 引数1つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredArg {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// 宣言したツール1つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredTool {
    pub name: String,
    pub description: String,
    pub args: Vec<DeclaredArg>,
    /// 先頭は Skill のフォルダからの相対パス
    pub command: Vec<String>,
    pub sha256: String,
    pub timeout_seconds: u64,
    pub env: Vec<String>,
}

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn strings(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

/// コマンドの先頭が、Skill のフォルダの中を指す相対パスか。
fn inside_skill(path: &str) -> bool {
    let windows_absolute = path.chars().nth(1) == Some(':');
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !windows_absolute
        && !path.split(['/', '\\']).any(|part| part == "..")
}

/// 宣言を読む。reserved は共通ツールの名前（宣言の名前と重なってはいけない）。
/// 契約に合わなければ、どこが合わないかを返す。
pub fn read_declarations(value: &Value, reserved: &[&str]) -> Result<Vec<DeclaredTool>, String> {
    let shape = contract::abstract_schema(contract::TOOLS_V1).unwrap_or(Value::Null);
    let validation = Schema::new("tools.schema.json", shape, vec![])
        .validate(value)
        .map_err(|error| error.to_string())?;
    if let Some(error) = validation.errors.first() {
        return Err(format!("{} {}", error.property(), error.reason()));
    }
    if let Some(unfilled) = validation.unfilled.first() {
        return Err(format!("{} が無い", unfilled.property()));
    }
    let mut names = BTreeSet::new();
    let mut tools = Vec::new();
    for tool in value["tools"].as_array().into_iter().flatten() {
        let name = text(tool, "name");
        if reserved.contains(&name.as_str()) {
            return Err(format!("{name} は共通ツールの名前と重なる"));
        }
        if !names.insert(name.clone()) {
            return Err(format!("{name} が2回宣言されている"));
        }
        let command = strings(tool, "command");
        if !inside_skill(&command[0]) {
            return Err(format!(
                "{name} のコマンド {} は Skill のフォルダの中を指していない",
                command[0]
            ));
        }
        let args = tool["args"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|arg| DeclaredArg {
                name: text(arg, "name"),
                description: text(arg, "description"),
                required: arg.get("required").and_then(Value::as_bool) == Some(true),
            })
            .collect();
        tools.push(DeclaredTool {
            name,
            description: text(tool, "description"),
            args,
            command,
            sha256: text(tool, "sha256"),
            timeout_seconds: tool
                .get("timeout_seconds")
                .and_then(Value::as_u64)
                .unwrap_or(DEFAULT_TIMEOUT_SECONDS),
            env: strings(tool, "env"),
        });
    }
    Ok(tools)
}
