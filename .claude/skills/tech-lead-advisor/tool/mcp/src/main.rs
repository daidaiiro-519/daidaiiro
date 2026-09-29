// SPDX-License-Identifier: MIT
//! tech-lead-advisor の MCP の面。**同じ宣言から組む** ── 能力は1行も複製しない。
//!
//! **`#[tool]` マクロを使わない。** マクロは道具をその場で宣言するので、能力の正本が
//! 2か所になる。代わりに `ServerHandler` を手で実装し、`list_tools` と `call_tool` を
//! 宣言から組む。
//!
//! **標準出力へ1バイトも書かない。** 原典が禁じている ── `The server MUST NOT
//! write anything to its stdout that is not a valid MCP message.`
//! （modelcontextprotocol.io/specification/2025-06-18/basic/transports:33、2026-09-24 取得）。
//! ログは標準エラーへ書く（同 31行が許す）。
//!
//! **`ok` が偽なら `is_error` を立てる。** 原典が2つを分けている ──
//! `Tool Execution Errors: Reported in tool results with isError: true`
//! （同 server/tools:389）。**検出（findings）は誤りではない**ので、通常の結果で返す。

use std::borrow::Cow;
use std::sync::Arc;

use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParam, CallToolResult, Content, Implementation, ListToolsResult,
    PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::stdio;
use rmcp::{ErrorData as McpError, ServiceExt};
use serde_json::{json, Map, Value};
use tla_declare::{tools, Given};

/// 宣言から、入力の形を組む。**引数を1つずつ公開する** ── まとめて受けると、
/// 呼ぶ側がどの引数を渡せばよいかを認知できない。
fn input_schema(tool: &tla_declare::Tool) -> Arc<Map<String, Value>> {
    let mut properties = Map::new();
    let mut required = Vec::new();
    for a in &tool.args {
        // **まとめて受ける引数は、並びとして公開する** ── 文字列1本にすると、呼ぶ側が
        // 区切りを推測することになる（実測 ── 配列がそのまま1つの値になった）
        let shape = if a.many {
            json!({ "type": "array", "items": { "type": "string" }, "description": a.summary })
        } else {
            json!({ "type": "string", "description": a.summary })
        };
        properties.insert(a.name.to_owned(), shape);
        if a.required {
            required.push(Value::String(a.name.to_owned()));
        }
    }
    let mut schema = Map::new();
    schema.insert("type".to_owned(), Value::String("object".to_owned()));
    schema.insert("properties".to_owned(), Value::Object(properties));
    schema.insert("required".to_owned(), Value::Array(required));
    Arc::new(schema)
}

#[derive(Clone, Debug)]
struct Handler;

impl ServerHandler for Handler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "tech-lead-advisor".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                title: None,
                icons: None,
                website_url: None,
            },
            instructions: Some("コード配置 ・ レイヤー境界 ・ 依存方向の判断基準に根拠を示して、判断の相談に回答する".to_owned()),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let listed = tools()
            .iter()
            .map(|t| Tool {
                name: Cow::Owned(t.name.to_owned()),
                title: None,
                description: Some(Cow::Owned(t.summary.to_owned())),
                input_schema: input_schema(t),
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            })
            .collect();
        Ok(ListToolsResult {
            tools: listed,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let all = tools();
        let Some(tool) = all.iter().find(|t| t.name == request.name.as_ref()) else {
            return Err(McpError::invalid_params(
                format!("その道具は無い: {}", request.name),
                None,
            ));
        };
        let mut given = Given::default();
        for (key, value) in request.arguments.unwrap_or_default() {
            // **型は緩く受ける** ── 呼ぶ側は JSON の値を渡すので、数を文字列で
            // 包むことを強制しない。**並びは1件ずつ足す** ── まとめると区切りが消える
            match value {
                Value::Array(items) => {
                    for item in items {
                        let text = match item {
                            Value::String(s) => s,
                            other => other.to_string(),
                        };
                        given.push(&key, text);
                    }
                }
                Value::String(s) => given.push(&key, s),
                other => given.push(&key, other.to_string()),
            }
        }
        for a in &tool.args {
            if !given.has(a.name) {
                if let Some(d) = a.default {
                    given.push(a.name, d.to_owned());
                } else if a.required {
                    return Err(McpError::invalid_params(
                        format!("引数が足りない: {} は {} を要する", tool.name, a.name),
                        None,
                    ));
                }
            }
        }
        let out = (tool.run)(&given);
        let body = out.to_json();
        let text = serde_json::to_string(&body).unwrap_or_default();
        Ok(CallToolResult {
            content: vec![Content::text(text)],
            structured_content: Some(body),
            is_error: Some(!out.ok),
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = Handler.serve(stdio()).await.inspect_err(|e| {
        // **標準エラーへ書く。** 標準出力は JSON-RPC の専用である
        eprintln!("立てられない: {e}");
    })?;
    service.waiting().await?;
    Ok(())
}
