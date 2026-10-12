//! アダプタ。入ってくる側は CLI と MCP の受け口、出ていく側はファイルシステム ・ jmespath ・ 部品のカタログ、
//! compose は起動時の結合である。

pub mod compose;
pub mod inbound;
pub mod outbound;

#[cfg(test)]
mod tests;
