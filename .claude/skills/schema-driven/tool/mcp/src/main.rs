//! MCP の実行ファイル。アダプタを作り、ツールの一覧（adapters の tools）を MCP のツールとして出す。
//! rmcp と tokio はこの crate にだけ現れる（ACDR 0122）。

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::{MaybeSendFuture, RequestContext};
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt};
use schema_driven_adapters::inbound::tools;
use schema_driven_adapters::outbound::{fs::FileSystem, jmespath::Jmespath};
use schema_driven_core::application::checks::Checks;
use schema_driven_core::application::instances::Instances;
use std::future::Future;

struct Server;

impl ServerHandler for Server {
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
        let list = tools::Toolbox::base()
            .list()
            .iter()
            .map(|t| Tool::new(t.name, t.description, tools::input_schema(t)))
            .collect();
        std::future::ready(Ok(ListToolsResult::with_all_items(list)))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResponse, ErrorData>> + MaybeSendFuture + '_ {
        let (files, query) = (FileSystem, Jmespath);
        let use_cases = Instances::new(&files, &files, &query);
        let args = request.arguments.unwrap_or_default();
        let checks = Checks::new(&files, &files, &query);
        let (code, out) =
            tools::Toolbox::base().dispatch(&request.name, &args, &use_cases, &checks);
        let content = vec![ContentBlock::text(out.to_string())];
        let result = if code == 0 {
            CallToolResult::success(content)
        } else {
            CallToolResult::error(content)
        };
        std::future::ready(Ok(result.into()))
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let service = match Server.serve(rmcp::transport::stdio()).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("schema-driven-mcp: 起動できない: {e}");
            std::process::exit(1);
        }
    };
    if let Err(e) = service.waiting().await {
        eprintln!("schema-driven-mcp: {e}");
        std::process::exit(1);
    }
}
