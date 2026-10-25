//! 実行ファイル semantic-agent-schema（CLI）
fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    semantic_agent_schema_cli::compose::run(&args)
}
