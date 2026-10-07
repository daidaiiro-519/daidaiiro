//! ブレストボードの MCP。転写した schema-driven のツールの一覧を、ボードの Design で使う。
//! rmcp と tokio はこの crate にだけ現れる（ACDR 0122）。

use brainstorming_board::design::BoardDesign;
use brainstorming_board::tools::BoardTools;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::{MaybeSendFuture, RequestContext};
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt};
use schema_driven_adapters::inbound::tools;
use schema_driven_adapters::outbound::document_design::DocumentDesign;
use schema_driven_adapters::outbound::fs::{master_root, FileSystem};
use schema_driven_adapters::outbound::jmespath::Jmespath;
use schema_driven_core::application::checks::Checks;
use schema_driven_core::application::instances::Instances;
use schema_driven_core::application::renders::Renders;
use std::future::Future;
use std::sync::Arc;

struct Server;

/// 基盤のツールに、ボードに固有のツールを足した一覧。名前は重ならない（重なれば起動の時点で止まる）。
fn toolbox() -> tools::Toolbox {
    tools::Toolbox::with(Arc::new(BoardTools::new(master_root()))).unwrap_or_else(|error| {
        eprintln!("brainstorming-board-mcp: {error}");
        std::process::exit(1)
    })
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build()).with_server_info(
            Implementation::new("brainstorming-board", env!("CARGO_PKG_VERSION")),
        )
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + MaybeSendFuture + '_ {
        let list = toolbox()
            .list()
            .iter()
            .map(|tool| Tool::new(tool.name, tool.description, tools::input_schema(tool)))
            .collect();
        std::future::ready(Ok(ListToolsResult::with_all_items(list)))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResponse, ErrorData>> + MaybeSendFuture + '_ {
        let (files, query) = (Arc::new(FileSystem), Arc::new(Jmespath));
        let use_cases = Instances::new(files.clone(), files.clone(), query.clone());
        let args = request.arguments.unwrap_or_default();
        let checks = Checks::new(files.clone(), files.clone(), query.clone());
        let renders = Renders::new(
            files.clone(),
            files,
            query,
            Arc::new(DocumentDesign),
            Some(Arc::new(BoardDesign)),
            None,
        );
        let (code, out) = toolbox().with_render(Arc::new(renders)).dispatch(
            &request.name,
            &args,
            &use_cases,
            &checks,
        );
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
        Ok(service) => service,
        Err(error) => {
            eprintln!("brainstorming-board-mcp: 起動できない: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = service.waiting().await {
        eprintln!("brainstorming-board-mcp: {error}");
        std::process::exit(1);
    }
}
