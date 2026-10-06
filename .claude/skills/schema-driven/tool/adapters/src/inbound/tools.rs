//! ツールの一覧。CLI と MCP は、この一覧と `dispatch` から作る（呼び出し方を2か所に書かない）。

use schema_driven_core::application::instances::{UnfilledWithPrompt, UseCaseError};
use schema_driven_core::domain::check::Finding;
use schema_driven_core::domain::values::Unfilled;
use schema_driven_core::domain::values::ValidationError;
use schema_driven_core::ports::inbound::{CheckUseCases, InstanceUseCases};
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
        name: "approve",
        description: "検査を通ったディレクトリのインスタンスのパスとハッシュ値を、承認記録（approval.json）へ書く（UC-8）",
        args: &[("dir", "インスタンスのディレクトリ", true)],
    },
];

/// ツールの引数を JSON Schema で表す（MCP の inputSchema）。
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

fn findings(list: &[Finding]) -> Value {
    list.iter()
        .map(|f| json!({"status": f.status.label(), "check": f.check, "from": f.from, "to": f.to, "message": f.message}))
        .collect()
}

fn unfilled_only(list: &[Unfilled]) -> Value {
    list.iter()
        .map(|u| json!({"property": u.property()}))
        .collect()
}

fn failed(e: UseCaseError) -> (i32, Value) {
    (
        1,
        json!({"ok": false, "reason": e.reason, "detail": e.detail, "errors": errors(&e.errors)}),
    )
}

/// 使い方の誤り（終了コード 2）。基盤のツールだけを並べる。
pub fn misuse(detail: &str) -> (i32, Value) {
    misuse_in(TOOLS, detail)
}

fn misuse_in(list: &[ToolDef], detail: &str) -> (i32, Value) {
    let names: Vec<&str> = list.iter().map(|t| t.name).collect();
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
pub struct Toolbox<'a> {
    list: Vec<ToolDef>,
    extra: Option<&'a dyn ExtraTools>,
}

impl<'a> Toolbox<'a> {
    /// 基盤のツールだけ。
    pub fn base() -> Self {
        Self {
            list: TOOLS.to_vec(),
            extra: None,
        }
    }

    /// 具体のツールを、基盤のツールのあとに追加する。基盤のツールと同じ名前は追加できない。
    pub fn with(extra: &'a dyn ExtraTools) -> Result<Self, String> {
        let mut list = TOOLS.to_vec();
        for t in extra.tools() {
            if list.iter().any(|b| b.name == t.name) {
                return Err(format!("基盤のツールと同じ名前は追加できない: {}", t.name));
            }
            list.push(t);
        }
        Ok(Self {
            list,
            extra: Some(extra),
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
        uc: &dyn InstanceUseCases,
        cc: &dyn CheckUseCases,
    ) -> (i32, Value) {
        let Some(tool) = self.list.iter().find(|t| t.name == name) else {
            return misuse_in(&self.list, &format!("知らないツール: {name}"));
        };
        for key in args.keys() {
            if !tool.args.iter().any(|a| a.0 == key) {
                return misuse_in(&self.list, &format!("{name} は引数 {key} を受け付けない"));
            }
        }
        for (key, _, required) in tool.args {
            match args.get(*key) {
                None if *required => {
                    return misuse_in(&self.list, &format!("{name} の引数 {key} が無い"))
                }
                Some(v) if !v.is_string() => {
                    return misuse_in(&self.list, &format!("{name} の引数 {key} は文字列で渡す"))
                }
                _ => {}
            }
        }
        if !TOOLS.iter().any(|t| t.name == name) {
            if let Some(extra) = self.extra {
                return extra.call(name, args);
            }
        }
        call_base(name, args, uc, cc)
    }
}

/// ツールの名前と引数から、基盤のユースケースを呼び出す（基盤のツールだけの Toolbox）。
pub fn dispatch(
    name: &str,
    args: &Map<String, Value>,
    uc: &dyn InstanceUseCases,
    cc: &dyn CheckUseCases,
) -> (i32, Value) {
    Toolbox::base().dispatch(name, args, uc, cc)
}

/// 引数を検査したあとで、基盤のユースケースを呼ぶ。
fn call_base(
    name: &str,
    args: &Map<String, Value>,
    uc: &dyn InstanceUseCases,
    cc: &dyn CheckUseCases,
) -> (i32, Value) {
    let s = |k: &str| args.get(k).and_then(Value::as_str).unwrap_or_default();
    let result = match name {
        "create" => uc.create(s("schema"), s("path")).map(|c| {
            json!({"ok": true, "path": c.path, "hash": c.hash, "unfilled": unfilled(&c.unfilled)})
        }),
        "get" => uc.get(s("path"), s("query")).map(|g| json!({"ok": true, "value": g.value, "found": g.found})),
        "update" => uc.update(s("path"), s("patch"), args.get("hash").and_then(Value::as_str)).map(|u| {
            json!({"ok": true, "hash": u.hash, "changed": u.changed, "errors": errors(&u.errors), "unfilled": unfilled(&u.unfilled)})
        }),
        "delete" => cc.delete(s("path")).map(|d| json!({"ok": true, "remaining": findings(&d.remaining)})),
        "check" => cc.check(s("dir")).map(|c| {
            let instances: Vec<Value> = c
                .instances
                .iter()
                .map(|i| json!({"path": i.path, "hash": i.hash, "errors": errors(&i.errors), "unfilled": unfilled_only(&i.unfilled)}))
                .collect();
            json!({"ok": true, "instances": instances, "findings": findings(&c.findings)})
        }),
        "approve" => cc.approve(s("dir")).map(|a| {
            let list: Vec<Value> = a.instances.iter().map(|(p, h)| json!({"path": p, "hash": h})).collect();
            json!({"ok": true, "changed": a.changed, "instances": list})
        }),
        "prompt" => uc.prompt(s("schema"), s("property")).map(|p| json!({"ok": true, "prompt": p.prompt})),
        _ => return misuse(&format!("知らないツール: {name}")),
    };
    match result {
        Ok(v) => (0, v),
        Err(e) => failed(e),
    }
}
