//! 文脈 agents のドメインモデル。
//!
//! サービス1つ：Agent の項目（agent.json）から、Claude Code の subagent（AGENT.md）と Codex の custom agent（TOML）を
//! 決まった形で作る（agent_md）。書く（write-agent-md）と確かめる（validate の agent-md-built）が同じ理由で使う。

pub mod services;
