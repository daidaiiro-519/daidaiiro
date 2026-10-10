//! MCP の実行ファイル。アダプタを作り、ツールの一覧（adapters の tools）を MCP のツールとして出す。
//! rmcp と tokio はこの crate にだけ現れる。

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::{MaybeSendFuture, RequestContext};
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt};
use schema_driven_adapters::inbound::tools;
use schema_driven_adapters::outbound::catalog::Catalog;
use schema_driven_adapters::outbound::fs::FileSystem;
use schema_driven_adapters::outbound::jmespath::Jmespath;
use schema_driven_core::application::checks::Checks;
use schema_driven_core::application::instances::Instances;
use schema_driven_core::application::renders::Renders;
use std::future::Future;
use std::sync::Arc;

/// MCP のサーバー。--skill で渡した Skill のフォルダがあれば、その宣言したツールも出す。
struct Server {
    skill: Option<String>,
}

impl Server {
    fn toolbox(&self) -> tools::Toolbox {
        let toolbox = tools::Toolbox::base();
        match &self.skill {
            Some(dir) => toolbox.with_skill(dir),
            None => toolbox,
        }
    }
}

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
        let toolbox = self.toolbox();
        let list = toolbox
            .list()
            .iter()
            .map(|tool| Tool::new(tool.name, tool.description, tools::input_schema(tool)))
            .chain(toolbox.declared().iter().map(|tool| {
                Tool::new(
                    tool.name.clone(),
                    tool.description.clone(),
                    tools::declared_input_schema(tool),
                )
            }))
            .collect();
        std::future::ready(Ok(ListToolsResult::with_all_items(list)))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResponse, ErrorData>> + MaybeSendFuture + '_ {
        let (files, query) = (Arc::new(FileSystem), Arc::new(Jmespath));
        let mut use_cases = Instances::new(files.clone(), files.clone(), query.clone());
        let args = request.arguments.unwrap_or_default();
        let mut checks = Checks::new(files.clone(), files.clone(), query.clone());
        // 部品の見た目と操作は、基盤のカタログだけが持つ
        let mut renders = Renders::new(files.clone(), files.clone(), query, Arc::new(Catalog));
        // スキーマの名前は、--skill の Skill から引く
        if let Some(dir) = &self.skill {
            use_cases = use_cases.with_skill(dir);
            checks = checks.with_skill(dir);
            renders = renders.with_skill(dir);
        }
        let (code, out) = self.toolbox().with_render(Arc::new(renders)).dispatch(
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
    let args: Vec<String> = std::env::args().skip(1).collect();
    let skill = match args.as_slice() {
        [flag, dir, ..] if flag == "--skill" => Some(dir.clone()),
        _ => None,
    };
    let service = match (Server { skill }).serve(rmcp::transport::stdio()).await {
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
