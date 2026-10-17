//! brainstorming の中心（domain ・ application ・ 出ていく側の port）。外の技術に依存しない。

pub mod application;
pub mod domain;
pub mod ports;

#[cfg(test)]
mod tests;
