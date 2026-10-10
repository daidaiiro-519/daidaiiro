//! ツールの一覧。CLI と MCP は、この一覧と `dispatch` から作る（呼び出し方を2か所に書かない）。

use crate::outbound::process;
use schema_driven_core::application::declared::{read_declarations, DeclaredTool};
use schema_driven_core::application::instances::{UnfilledWithPrompt, UseCaseError};
use schema_driven_core::domain::Finding;
use schema_driven_core::domain::Unfilled;
use schema_driven_core::domain::ValidationError;
use schema_driven_core::ports::inbound::{CheckUseCases, InstanceUseCases, RenderUseCases};
use serde_json::{json, Map, Value};
use std::path::PathBuf;
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
        description: "ディレクトリの具体のスキーマ（*.schema.json）を、メタスキーマ（references/meta-schema.json）と注釈の仕様で、すべての深さまで検査する。公開の記録（published.json）があれば、公開した版の書き換えと消失も指摘し、まだ記録に無い名前を指紋つきで返す",
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

/// 共通ツールと、Skill が宣言したツールの一覧。CLI と MCP はこれから作る。
pub struct Toolbox {
    list: Vec<ToolDef>,
    render: Option<Arc<dyn RenderUseCases>>,
    /// --skill で渡した Skill のフォルダ
    skill: Option<PathBuf>,
    /// その Skill の tools.json で宣言したツール
    declared: Vec<DeclaredTool>,
    /// 宣言が契約に合わないときの理由。あれば、どのツールも呼ばずにこれを返す
    declaration_error: Option<String>,
}

impl Toolbox {
    /// 基盤のツールだけ。
    pub fn base() -> Self {
        Self {
            list: TOOLS.to_vec(),
            render: None,
            skill: None,
            declared: Vec::new(),
            declaration_error: None,
        }
    }

    /// Skill のフォルダを渡す。そのフォルダの tools.json で宣言したツールを、共通ツールのあとに並べる。
    pub fn with_skill(mut self, dir: &str) -> Self {
        let path = PathBuf::from(dir);
        let reserved: Vec<&str> = self.list.iter().map(|tool| tool.name).collect();
        match process::load_declarations(&path) {
            Ok(None) => {}
            Ok(Some(value)) => match read_declarations(&value, &reserved) {
                Ok(declared) => self.declared = declared,
                Err(error) => self.declaration_error = Some(error),
            },
            Err(error) => self.declaration_error = Some(error),
        }
        self.skill = Some(path);
        self
    }

    /// 宣言が契約に合わないとき、その失敗を返す。
    pub fn declaration_failure(&self) -> Option<(i32, Value)> {
        self.declaration_error.as_ref().map(|error| {
            (
                1,
                json!({"ok": false, "reason": "宣言が契約に合わない", "detail": error, "errors": []}),
            )
        })
    }

    /// 宣言したツール。
    pub fn declared(&self) -> &[DeclaredTool] {
        &self.declared
    }

    /// 描画のユースケースを渡す。渡さなければ、render は理由を返して失敗する。
    pub fn with_render(mut self, render: Arc<dyn RenderUseCases>) -> Self {
        self.render = Some(render);
        self
    }

    pub fn list(&self) -> &[ToolDef] {
        &self.list
    }

    /// ツールの一覧を、ほかの Skill の CLI と同じ形（data.tools ・ data.skill_root）で返す。
    pub fn catalog(&self, skill_root: &str) -> Value {
        let tools: Vec<Value> = self
            .list
            .iter()
            .map(|tool| {
                let args: Vec<Value> = tool
                    .args
                    .iter()
                    .map(|(name, _, required)| json!({"name": name, "required": required}))
                    .collect();
                json!({"name": tool.name, "summary": tool.description, "args": args})
            })
            .chain(self.declared.iter().map(|tool| {
                let args: Vec<Value> = tool
                    .args
                    .iter()
                    .map(|arg| json!({"name": arg.name, "required": arg.required}))
                    .collect();
                json!({"name": tool.name, "summary": tool.description, "args": args})
            }))
            .collect();
        json!({"ok": true, "findings": [], "data": {"tools": tools, "skill_root": skill_root}})
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
        if let Some(failure) = self.declaration_failure() {
            return failure;
        }
        if let Some(tool) = self.declared.iter().find(|tool| tool.name == name) {
            return self.call_declared(tool, args);
        }
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
        _ => return misuse(&format!("知らないツール: {name}")),
    };
    match result {
        Ok(value) => (0, value),
        Err(error) => failed(error),
    }
}

impl Toolbox {
    /// 宣言したツールの引数を確かめてから、別の実行ファイルとして呼ぶ。
    fn call_declared(&self, tool: &DeclaredTool, args: &Map<String, Value>) -> (i32, Value) {
        let name = &tool.name;
        for key in args.keys() {
            if !tool.args.iter().any(|arg| &arg.name == key) {
                return self.misuse_all(&format!("{name} は引数 {key} を受け付けない"));
            }
        }
        for arg in &tool.args {
            match args.get(&arg.name) {
                None if arg.required => {
                    return self.misuse_all(&format!("{name} の引数 {} が無い", arg.name))
                }
                Some(value) if !value.is_string() => {
                    return self.misuse_all(&format!("{name} の引数 {} は文字列で渡す", arg.name))
                }
                _ => {}
            }
        }
        let skill = self.skill.clone().unwrap_or_else(|| PathBuf::from("."));
        process::run_declared(&skill, tool, args)
    }

    /// 使い方の誤り。共通ツールと宣言したツールの名前を並べて返す。
    fn misuse_all(&self, detail: &str) -> (i32, Value) {
        let names: Vec<&str> = self
            .list
            .iter()
            .map(|tool| tool.name)
            .chain(self.declared.iter().map(|tool| tool.name.as_str()))
            .collect();
        (
            2,
            json!({"ok": false, "reason": "使い方が違う", "detail": detail, "tools": names}),
        )
    }
}

/// 宣言したツールの引数を JSON Schema で表す（MCP の inputSchema）。
pub fn declared_input_schema(tool: &DeclaredTool) -> Map<String, Value> {
    let mut properties = Map::new();
    for arg in &tool.args {
        properties.insert(
            arg.name.clone(),
            json!({"type": "string", "description": arg.description}),
        );
    }
    let required: Vec<&str> = tool
        .args
        .iter()
        .filter(|arg| arg.required)
        .map(|arg| arg.name.as_str())
        .collect();
    let schema = json!({"type": "object", "properties": properties, "required": required, "additionalProperties": false});
    schema.as_object().cloned().unwrap_or_default()
}
