//! MCP の受け口。ツールの一覧（tools）を MCP のツールとして出し、呼び出しを振り分けへ渡す。
//! rmcp と tokio は、adapters の中でもこのファイルにだけ現れる。

use crate::inbound::cli::split_skill;
use crate::inbound::tools::{input_schema, Toolbox, TOOLS};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::{MaybeSendFuture, RequestContext};
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt};
use schema_driven_core::ports::{Files, Query, Schemas};
use std::future::Future;

/// MCP のサーバー。起動時に組んだツールの一覧を持つ。
pub struct Server<F, S, Q> {
    toolbox: Toolbox<F, S, Q>,
}

impl<F: Files, S: Schemas, Q: Query> ServerHandler for Server<F, S, Q> {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build()).with_server_info(
            Implementation::new("schema-driven", env!("CARGO_PKG_VERSION")),
        )
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + MaybeSendFuture + '_ {
        let list = TOOLS
            .iter()
            .map(|tool| Tool::new(tool.name, tool.description, input_schema(tool)))
            .collect();
        std::future::ready(Ok(ListToolsResult::with_all_items(list)))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResponse, ErrorData>> + MaybeSendFuture + '_ {
        let args = request.arguments.unwrap_or_default();
        let (code, out) = self.toolbox.dispatch(&request.name, &args);
        let content = vec![ContentBlock::text(out.to_string())];
        let result = if code == 0 {
            CallToolResult::success(content)
        } else {
            CallToolResult::error(content)
        };
        std::future::ready(Ok(result.into()))
    }
}

/// 引数（`--skill <Skill のフォルダ>` だけを読む）から compose でツールの一覧を組み、
/// 標準入出力で MCP のサーバーを動かす。終わったら、プロセスの終了コードを返す。
pub fn serve<F: Files, S: Schemas, Q: Query>(
    args: &[String],
    compose: impl Fn(Option<&str>) -> Toolbox<F, S, Q>,
) -> i32 {
    let (skill, _) = split_skill(args);
    let server = Server {
        toolbox: compose(skill),
    };
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("schema-driven-mcp: 起動できない: {error}");
            return 1;
        }
    };
    runtime.block_on(async {
        let service = match server.serve(rmcp::transport::stdio()).await {
            Ok(service) => service,
            Err(error) => {
                eprintln!("schema-driven-mcp: 起動できない: {error}");
                return 1;
            }
        };
        match service.waiting().await {
            Ok(_) => 0,
            Err(error) => {
                eprintln!("schema-driven-mcp: {error}");
                1
            }
        }
    })
}
