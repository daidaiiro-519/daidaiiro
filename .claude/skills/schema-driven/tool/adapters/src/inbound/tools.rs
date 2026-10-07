//! ツールの一覧。CLI と MCP は、この一覧と `dispatch` から作る（呼び出し方を2か所に書かない）。

use schema_driven_core::application::instances::{UnfilledWithPrompt, UseCaseError};
use schema_driven_core::domain::check::Finding;
use schema_driven_core::domain::values::Unfilled;
use schema_driven_core::domain::values::ValidationError;
use schema_driven_core::ports::inbound::{CheckUseCases, InstanceUseCases, RenderUseCases};
use serde_json::{json, Map, Value};
use std::sync::Arc;

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
    ToolDef {
        name: "delete",
        description: "インスタンスのファイルを削除し、まだそれを指している参照を返す（UC-4）",
        args: &[("path", "インスタンスのパス", true)],
    },
    ToolDef {
        name: "prompt",
        description: "プロパティ（JSON Pointer）の x-prompt を返す（UC-7）",
        args: &[("schema", "スキーマのファイルのパス", true), ("property", "プロパティの JSON Pointer", true)],
    },
    ToolDef {
        name: "check",
        description: "ディレクトリのインスタンスを検証し、x-ref と x-derive の検査をする（UC-5）",
        args: &[("dir", "インスタンスのディレクトリ", true)],
    },
    ToolDef {
        name: "check-schemas",
        description: "ディレクトリの具体のスキーマ（*.schema.json）を、メタスキーマ（references/meta-schema.json）と注釈の仕様で、すべての深さまで検査する",
        args: &[("dir", "スキーマのディレクトリ", true)],
    },
    ToolDef {
        name: "approve",
        description: "検査を通ったディレクトリのインスタンスのパスとハッシュ値を、承認記録（approval.json）へ書く（UC-8）",
        args: &[("dir", "インスタンスのディレクトリ", true)],
    },
    ToolDef {
        name: "render",
        description: "ディレクトリのインスタンスをページへ描画する（UC-6）。具体のページテンプレートとデザインがあればそれで、無い種類は基盤の既定のページで描画する。検証エラーがあれば描画しない。同じ入力からは同じページを書く",
        args: &[
            ("dir", "インスタンスのディレクトリ", true),
            ("pages", "ページテンプレートのディレクトリ。無ければ、すべての種類を基盤の既定のページで描画する", false),
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

/// 使い方の誤り（終了コード 2）。基盤のツールだけを並べる。
pub fn misuse(detail: &str) -> (i32, Value) {
    misuse_in(TOOLS, detail)
}

fn misuse_in(list: &[ToolDef], detail: &str) -> (i32, Value) {
    let names: Vec<&str> = list.iter().map(|tool| tool.name).collect();
    (
        2,
        json!({"ok": false, "reason": "使い方が違う", "detail": detail, "tools": names}),
    )
}

/// 具体が追加するツール（ボード schema-driven-build の論点5 C）。引数の検査は基盤が行ってから call を呼び出す。
pub trait ExtraTools {
    fn tools(&self) -> Vec<ToolDef>;
    /// ツールを呼び出す。返すのは終了コードと結果の JSON（基盤のツールと同じ形）。
    fn call(&self, name: &str, args: &Map<String, Value>) -> (i32, Value);
}

/// 基盤のツールの一覧に、具体のツールを追加したもの。CLI と MCP はこれから作る。
pub struct Toolbox {
    list: Vec<ToolDef>,
    extra: Option<Arc<dyn ExtraTools>>,
    render: Option<Arc<dyn RenderUseCases>>,
}

impl Toolbox {
    /// 基盤のツールだけ。
    pub fn base() -> Self {
        Self {
            list: TOOLS.to_vec(),
            extra: None,
            render: None,
        }
    }

    /// 描画のユースケース（具体のデザインを持つもの）を渡す。渡さなければ、render は理由を返して失敗する。
    pub fn with_render(mut self, render: Arc<dyn RenderUseCases>) -> Self {
        self.render = Some(render);
        self
    }

    /// 具体のツールを、基盤のツールのあとに追加する。基盤のツールと同じ名前は追加できない。
    pub fn with(extra: Arc<dyn ExtraTools>) -> Result<Self, String> {
        let mut list = TOOLS.to_vec();
        for tool in extra.tools() {
            if list.iter().any(|base| base.name == tool.name) {
                return Err(format!(
                    "基盤のツールと同じ名前は追加できない: {}",
                    tool.name
                ));
            }
            list.push(tool);
        }
        Ok(Self {
            list,
            extra: Some(extra),
            render: None,
        })
    }

    pub fn list(&self) -> &[ToolDef] {
        &self.list
    }

    /// ツールの名前と引数から、ユースケース（または具体のツール）を呼び出す。
    /// 返すのは終了コード（0 成功 ／ 1 失敗 ／ 2 使い方の誤り）と結果の JSON。
    pub fn dispatch(
        &self,
        name: &str,
        args: &Map<String, Value>,
        instances: &dyn InstanceUseCases,
        checks: &dyn CheckUseCases,
    ) -> (i32, Value) {
        let Some(tool) = self.list.iter().find(|tool| tool.name == name) else {
            return misuse_in(&self.list, &format!("知らないツール: {name}"));
        };
        for key in args.keys() {
            if !tool.args.iter().any(|arg| arg.0 == key) {
                return misuse_in(&self.list, &format!("{name} は引数 {key} を受け付けない"));
            }
        }
        for (key, _, required) in tool.args {
            match args.get(*key) {
                None if *required => {
                    return misuse_in(&self.list, &format!("{name} の引数 {key} が無い"))
                }
                Some(value) if !value.is_string() => {
                    return misuse_in(&self.list, &format!("{name} の引数 {key} は文字列で渡す"))
                }
                _ => {}
            }
        }
        if name == "render" {
            let arg = |key: &str| args.get(key).and_then(Value::as_str).unwrap_or_default();
            let Some(render) = &self.render else {
                return (
                    1,
                    json!({"ok": false, "reason": "描画のユースケースが渡されていない", "detail": "render は、Toolbox に描画のユースケース（Renders）を渡したツールで使う"}),
                );
            };
            return match render.render(arg("dir"), arg("pages"), arg("out")) {
                Ok(done) => {
                    let pages: Vec<Value> = done
                        .pages
                        .iter()
                        .map(|page| json!({"path": page.path, "changed": page.changed, "design": page.design}))
                        .collect();
                    (0, json!({"ok": true, "pages": pages}))
                }
                Err(error) => failed(error),
            };
        }
        if !TOOLS.iter().any(|tool| tool.name == name) {
            if let Some(extra) = &self.extra {
                return extra.call(name, args);
            }
        }
        call_base(name, args, instances, checks)
    }
}

/// ツールの名前と引数から、基盤のユースケースを呼び出す（基盤のツールだけの Toolbox）。
pub fn dispatch(
    name: &str,
    args: &Map<String, Value>,
    instances: &dyn InstanceUseCases,
    checks: &dyn CheckUseCases,
) -> (i32, Value) {
    Toolbox::base().dispatch(name, args, instances, checks)
}

/// 引数を検査したあとで、基盤のユースケースを呼ぶ。
fn call_base(
    name: &str,
    args: &Map<String, Value>,
    instances: &dyn InstanceUseCases,
    checks: &dyn CheckUseCases,
) -> (i32, Value) {
    let arg = |key: &str| args.get(key).and_then(Value::as_str).unwrap_or_default();
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
                .map(|finding| json!({"schema": finding.schema, "at": finding.at, "rule": finding.rule, "message": finding.message}))
                .collect();
            json!({"ok": true, "schemas": checked.schemas, "findings": list})
        }),
        "approve" => checks.approve(arg("dir")).map(|approved| {
            let list: Vec<Value> = approved.instances.iter().map(|(path, hash)| json!({"path": path, "hash": hash})).collect();
            json!({"ok": true, "changed": approved.changed, "instances": list})
        }),
        "prompt" => instances.prompt(arg("schema"), arg("property")).map(|prompted| json!({"ok": true, "prompt": prompted.prompt})),
        _ => return misuse(&format!("知らないツール: {name}")),
    };
    match result {
        Ok(value) => (0, value),
        Err(error) => failed(error),
    }
}
