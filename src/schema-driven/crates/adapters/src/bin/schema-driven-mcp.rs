//! MCP の実行ファイル。起動時の結合（compose）を MCP の受け口へ渡し、標準入出力で動かす。

use schema_driven_adapters::compose;
use schema_driven_adapters::inbound::mcp;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(mcp::serve(&args, compose::toolbox));
}
