//! 道具の一覧。CLI と MCP は、この一覧と `dispatch` から組む（呼び出し方を2か所に書かない）。

use schema_driven_core::application::instances::{UnfilledWithPrompt, UseCaseError};
use schema_driven_core::domain::values::ValidationError;
use schema_driven_core::ports::inbound::InstanceUseCases;
use serde_json::{json, Map, Value};

/// 道具1つ：名前 ・ 説明 ・ 引数（名前 ・ 説明 ・ 必須か）。
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [(&'static str, &'static str, bool)],
}

pub const TOOLS: &[ToolDef] = &[
    ToolDef {
        name: "create",
        description: "スキーマから、必須のプロパティを持たないインスタンスを作成し、未記入のプロパティと x-prompt を返す（UC-1）",
        args: &[("schema", "スキーマのファイルのパス", true), ("path", "作成するインスタンスのパス", true)],
    },
    ToolDef {
        name: "get",
        description: "JMESPath 式でインスタンスから値を取得する（UC-2）",
        args: &[("path", "インスタンスのパス", true), ("query", "JMESPath 式", true)],
    },
    ToolDef {
        name: "update",
        description: "インスタンスへ JSON Patch を適用し、未記入以外の検証エラーが無いときだけ書く（UC-3）",
        args: &[
            ("path", "インスタンスのパス", true),
            ("patch", "JSON Patch（RFC 6902）の文字列", true),
            ("hash", "読んだ時点のハッシュ値。違えば「ほかの更新と競合した」で拒む", false),
        ],
    },
    ToolDef { name: "delete", description: "インスタンスのファイルを削除する", args: &[("path", "インスタンスのパス", true)] },
    ToolDef {
        name: "prompt",
        description: "プロパティ（JSON Pointer）の x-prompt を返す（UC-7）",
        args: &[("schema", "スキーマのファイルのパス", true), ("property", "プロパティの JSON Pointer", true)],
    },
];

/// 道具の引数を JSON Schema で表す（MCP の inputSchema）。
pub fn input_schema(tool: &ToolDef) -> Map<String, Value> {
    let mut props = Map::new();
    for (name, desc, _) in tool.args {
        props.insert(
            (*name).to_owned(),
            json!({"type": "string", "description": desc}),
        );
    }
    let required: Vec<&str> = tool.args.iter().filter(|a| a.2).map(|a| a.0).collect();
    let schema = json!({"type": "object", "properties": props, "required": required, "additionalProperties": false});
    schema.as_object().cloned().unwrap_or_default()
}

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

/// 使い方の誤り（終了コード 2）。
pub fn misuse(detail: &str) -> (i32, Value) {
    let names: Vec<&str> = TOOLS.iter().map(|t| t.name).collect();
    (
        2,
        json!({"ok": false, "reason": "使い方が違う", "detail": detail, "tools": names}),
    )
}

/// 道具の名前と引数から、ユースケースを呼ぶ。返すのは終了コード（0 成功 ／ 1 失敗 ／ 2 使い方の誤り）と結果の JSON。
pub fn dispatch(name: &str, args: &Map<String, Value>, uc: &dyn InstanceUseCases) -> (i32, Value) {
    let Some(tool) = TOOLS.iter().find(|t| t.name == name) else {
        return misuse(&format!("知らない道具: {name}"));
    };
    for key in args.keys() {
        if !tool.args.iter().any(|a| a.0 == key) {
            return misuse(&format!("{name} は引数 {key} を受け付けない"));
        }
    }
    for (key, _, required) in tool.args {
        match args.get(*key) {
            None if *required => return misuse(&format!("{name} の引数 {key} が無い")),
            Some(v) if !v.is_string() => {
                return misuse(&format!("{name} の引数 {key} は文字列で渡す"))
            }
            _ => {}
        }
    }
    let s = |k: &str| args.get(k).and_then(Value::as_str).unwrap_or_default();
    let result = match name {
        "create" => uc.create(s("schema"), s("path")).map(|c| {
            json!({"ok": true, "path": c.path, "hash": c.hash, "unfilled": unfilled(&c.unfilled)})
        }),
        "get" => uc.get(s("path"), s("query")).map(|g| json!({"ok": true, "value": g.value, "found": g.found})),
        "update" => uc.update(s("path"), s("patch"), args.get("hash").and_then(Value::as_str)).map(|u| {
            json!({"ok": true, "hash": u.hash, "changed": u.changed, "errors": errors(&u.errors), "unfilled": unfilled(&u.unfilled)})
        }),
        "delete" => uc.delete(s("path")).map(|()| json!({"ok": true})),
        "prompt" => uc.prompt(s("schema"), s("property")).map(|p| json!({"ok": true, "prompt": p.prompt})),
        _ => return misuse(&format!("知らない道具: {name}")),
    };
    match result {
        Ok(v) => (0, v),
        Err(e) => failed(e),
    }
}
