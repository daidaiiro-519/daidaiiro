//! ツールの一覧。CLI と MCP は、この一覧と `dispatch` から作る（呼び出し方を2か所に書かない）。

use schema_driven_core::application::checks::Checks;
use schema_driven_core::application::instances::{Instances, UnfilledWithPrompt, UseCaseError};
use schema_driven_core::application::renders::Renders;
use schema_driven_core::domain::{Finding, Unfilled, ValidationError};
use schema_driven_core::ports::{Files, Query, Schemas};
use serde_json::{json, Map, Value};

/// ツール1つ：名前 ・ 説明 ・ 引数（名前 ・ 説明 ・ 必須か）。
#[derive(Debug, Clone, Copy)]
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [(&'static str, &'static str, bool)],
}

pub const TOOLS: &[ToolDef] = &[
    ToolDef {
        name: "create",
        description: "スキーマから、必須のプロパティを持たないインスタンスを作成し、未記入のプロパティと x-prompt を返す",
        args: &[("schema", "スキーマのファイルのパス", true), ("path", "作成するインスタンスのパス", true)],
    },
    ToolDef {
        name: "get",
        description: "JMESPath 式でインスタンスから値を取得する",
        args: &[("path", "インスタンスのパス", true), ("query", "JMESPath 式", true)],
    },
    ToolDef {
        name: "update",
        description: "インスタンスへ JSON Patch を適用し、未記入以外の検証エラーが無いときだけ書く",
        args: &[
            ("path", "インスタンスのパス", true),
            ("patch", "JSON Patch（RFC 6902）の文字列", true),
            ("hash", "読んだ時点のハッシュ値。違えば「ほかの更新と競合した」で拒む", false),
        ],
    },
    ToolDef {
        name: "delete",
        description: "インスタンスのファイルを削除し、まだそれを指している参照を返す",
        args: &[("path", "インスタンスのパス", true)],
    },
    ToolDef {
        name: "prompt",
        description: "プロパティ（JSON Pointer）の x-prompt を返す",
        args: &[("schema", "スキーマのファイルのパス", true), ("property", "プロパティの JSON Pointer", true)],
    },
    ToolDef {
        name: "check",
        description: "ディレクトリのインスタンスを検証し、x-ref と x-derive の検査をする",
        args: &[("dir", "インスタンスのディレクトリ", true)],
    },
    ToolDef {
        name: "check-schemas",
        description: "ディレクトリの具体のスキーマ（*.schema.json）を、メタスキーマ（schemas/meta-schema.json）と注釈の仕様で、すべての深さまで検査する。公開の記録（published.json）があれば、公開した版の書き換えと消失も指摘し、まだ記録に無い名前を指紋つきで返す",
        args: &[("dir", "スキーマのディレクトリ", true)],
    },
    ToolDef {
        name: "approve",
        description: "検査を通ったディレクトリのインスタンスのパスとハッシュ値を、承認記録（approval.json）へ書く",
        args: &[("dir", "インスタンスのディレクトリ", true)],
    },
    ToolDef {
        name: "render",
        description: "ディレクトリのインスタンスをページへ描画する。kind が document なら基盤の文書の契約で、そのほかの種類はその種類のページテンプレートで、部品のカタログを使って描画する。ページテンプレートの無い種類があれば、1ページも書かずに失敗する。検証エラーがあれば描画しない。同じ入力からは同じページを書く",
        args: &[
            ("dir", "インスタンスのディレクトリ", true),
            ("pages", "具体のページテンプレートのディレクトリ。無ければ、kind が document のインスタンスだけを描画できる", false),
            ("out", "ページを書き出すディレクトリ", true),
        ],
    },
];

/// ツールの引数を JSON Schema で表す（MCP の inputSchema）。
pub fn input_schema(tool: &ToolDef) -> Map<String, Value> {
    let mut properties = Map::new();
    for (name, desc, _) in tool.args {
        properties.insert(
            (*name).to_owned(),
            json!({"type": "string", "description": desc}),
        );
    }
    let required: Vec<&str> = tool
        .args
        .iter()
        .filter(|arg| arg.2)
        .map(|arg| arg.0)
        .collect();
    let schema = json!({"type": "object", "properties": properties, "required": required, "additionalProperties": false});
    schema.as_object().cloned().unwrap_or_default()
}

fn errors(list: &[ValidationError]) -> Value {
    list.iter()
        .map(|error| json!({"property": error.property(), "reason": error.reason()}))
        .collect()
}

fn unfilled(list: &[UnfilledWithPrompt]) -> Value {
    list.iter()
        .map(|unfilled| json!({"property": unfilled.property, "prompt": unfilled.prompt}))
        .collect()
}

fn findings(list: &[Finding]) -> Value {
    list.iter()
        .map(|finding| json!({"status": finding.status.label(), "check": finding.check, "from": finding.from, "to": finding.to, "message": finding.message}))
        .collect()
}

fn unfilled_only(list: &[Unfilled]) -> Value {
    list.iter()
        .map(|unfilled| json!({"property": unfilled.property()}))
        .collect()
}

fn failed(error: UseCaseError) -> (i32, Value) {
    (
        1,
        json!({"ok": false, "reason": error.reason, "detail": error.detail, "errors": errors(&error.errors)}),
    )
}

/// 使い方の誤り（終了コード 2）。基盤のツールの名前を並べて返す。
pub fn misuse(detail: &str) -> (i32, Value) {
    let names: Vec<&str> = TOOLS.iter().map(|tool| tool.name).collect();
    (
        2,
        json!({"ok": false, "reason": "使い方が違う", "detail": detail, "tools": names}),
    )
}

/// ツールの一覧を data.tools で返す。
pub fn catalog() -> Value {
    let tools: Vec<Value> = TOOLS
        .iter()
        .map(|tool| {
            let args: Vec<Value> = tool
                .args
                .iter()
                .map(|(name, _, required)| json!({"name": name, "required": required}))
                .collect();
            json!({"name": tool.name, "summary": tool.description, "args": args})
        })
        .collect();
    json!({"ok": true, "findings": [], "data": {"tools": tools}})
}

/// 引数を、ツールの引数の宣言で確かめる。合わなければ使い方の誤りを返す。
fn misused(tool: &ToolDef, args: &Map<String, Value>) -> Option<(i32, Value)> {
    let name = tool.name;
    if let Some(key) = args
        .keys()
        .find(|key| !tool.args.iter().any(|arg| arg.0 == key.as_str()))
    {
        return Some(misuse(&format!("{name} は引数 {key} を受け付けない")));
    }
    tool.args
        .iter()
        .find_map(|(key, _, required)| match args.get(*key) {
            None if *required => Some(misuse(&format!("{name} の引数 {key} が無い"))),
            Some(value) if !value.is_string() => {
                Some(misuse(&format!("{name} の引数 {key} は文字列で渡す")))
            }
            _ => None,
        })
}

/// 基盤のユースケースと、ツールの一覧からの振り分け。CLI と MCP は、これを呼ぶ。
pub struct Toolbox<F, S, Q> {
    instances: Instances<F, S, Q>,
    checks: Checks<F, S, Q>,
    renders: Renders<F, S, Q>,
}

impl<F: Files, S: Schemas, Q: Query> Toolbox<F, S, Q> {
    pub fn new(
        instances: Instances<F, S, Q>,
        checks: Checks<F, S, Q>,
        renders: Renders<F, S, Q>,
    ) -> Self {
        Self {
            instances,
            checks,
            renders,
        }
    }

    /// ツールの名前と引数から、ユースケースを呼び出す。
    /// 返すのは終了コード（0 成功 ／ 1 失敗 ／ 2 使い方の誤り）と結果の JSON。
    pub fn dispatch(&self, name: &str, args: &Map<String, Value>) -> (i32, Value) {
        let Some(tool) = TOOLS.iter().find(|tool| tool.name == name) else {
            return misuse(&format!("知らないツール: {name}"));
        };
        if let Some(misused) = misused(tool, args) {
            return misused;
        }
        let arg = |key: &str| args.get(key).and_then(Value::as_str).unwrap_or_default();
        let (instances, checks) = (&self.instances, &self.checks);
        let result = match name {
            "create" => instances.create(arg("schema"), arg("path")).map(|created| {
                json!({"ok": true, "path": created.path, "hash": created.hash, "unfilled": unfilled(&created.unfilled)})
            }),
            "get" => instances.get(arg("path"), arg("query")).map(|got| json!({"ok": true, "value": got.value, "found": got.found})),
            "update" => instances.update(arg("path"), arg("patch"), args.get("hash").and_then(Value::as_str)).map(|updated| {
                json!({"ok": true, "hash": updated.hash, "changed": updated.changed, "errors": errors(&updated.errors), "unfilled": unfilled(&updated.unfilled)})
            }),
            "delete" => checks.delete(arg("path")).map(|deleted| json!({"ok": true, "remaining": findings(&deleted.remaining)})),
            "check" => checks.check(arg("dir")).map(|checked| {
                let validated: Vec<Value> = checked
                    .instances
                    .iter()
                    .map(|instance| json!({"path": instance.path, "hash": instance.hash, "errors": errors(&instance.errors), "unfilled": unfilled_only(&instance.unfilled)}))
                    .collect();
                json!({"ok": true, "instances": validated, "findings": findings(&checked.findings)})
            }),
            "check-schemas" => checks.check_schemas(arg("dir")).map(|checked| {
                let list: Vec<Value> = checked
                    .findings
                    .iter()
                    .map(|finding| {
                        let mut out = json!({"schema": finding.schema, "at": finding.at, "rule": finding.rule, "message": finding.message});
                        if let Some(prompt) = &finding.prompt {
                            out["prompt"] = prompt.clone();
                        }
                        out
                    })
                    .collect();
                let unpublished: Vec<Value> = checked
                    .unpublished
                    .iter()
                    .map(|item| json!({"name": item.name, "sha256": item.sha256}))
                    .collect();
                json!({"ok": true, "schemas": checked.schemas, "findings": list, "unpublished": unpublished})
            }),
            "approve" => checks.approve(arg("dir")).map(|approved| {
                let list: Vec<Value> = approved.instances.iter().map(|(path, hash)| json!({"path": path, "hash": hash})).collect();
                json!({"ok": true, "changed": approved.changed, "instances": list})
            }),
            "prompt" => instances.prompt(arg("schema"), arg("property")).map(|prompted| json!({"ok": true, "prompt": prompted.prompt})),
            "render" => self.renders.render(arg("dir"), arg("pages"), arg("out")).map(|done| {
                let pages: Vec<Value> = done
                    .pages
                    .iter()
                    .map(|page| json!({"path": page.path, "changed": page.changed, "design": page.design}))
                    .collect();
                json!({"ok": true, "pages": pages})
            }),
            _ => return misuse(&format!("知らないツール: {name}")),
        };
        match result {
            Ok(value) => (0, value),
            Err(error) => failed(error),
        }
    }
}

#[cfg(test)]
mod tests {
    //! MCP の変換（ツールの引数の宣言 → inputSchema）。
    use super::{input_schema, TOOLS};
    use serde_json::{json, Value};

    #[test]
    fn input_schema_lists_string_arguments_and_the_required_ones() {
        let update = TOOLS.iter().find(|tool| tool.name == "update").unwrap();
        let schema = Value::Object(input_schema(update));
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["required"], json!(["path", "patch"]));
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["properties"]["hash"]["type"], "string");
    }
}
