//! MCP の実行ファイル。アダプタを作り、ツールの一覧（adapters の tools）を MCP のツールとして出す。
//! rmcp と tokio はこの crate にだけ現れる（ACDR 0122）。

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
use schema_driven_core::application::transcriptions::Transcriptions;
use std::future::Future;
use std::sync::Arc;

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
        // 基盤だけで使うときは、具体のデザインが無いので、基盤の文書だけを描画する
        let renders = Renders::new(
            files.clone(),
            files.clone(),
            query,
            Arc::new(DocumentDesign),
            None,
            None,
        );
        let transcriptions = Transcriptions::new(files, master_root());
        let (code, out) = tools::Toolbox::base()
            .with_render(Arc::new(renders))
            .with_transcriptions(Arc::new(transcriptions))
            .dispatch(&request.name, &args, &use_cases, &checks);
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
            eprintln!("schema-driven-mcp: 起動できない: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = service.waiting().await {
        eprintln!("schema-driven-mcp: {error}");
        std::process::exit(1);
    }
}
