//! アダプタ。入ってくる側は CLI と MCP の受け口、出ていく側はファイルシステムと jmespath である。

pub mod inbound;
pub mod outbound;
