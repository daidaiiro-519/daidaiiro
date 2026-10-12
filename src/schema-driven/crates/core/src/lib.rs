//! schema-driven の中心。ドメイン ・ アプリケーション ・ 出ていく側のポートを持ち、入出力を直接扱わない。
#![deny(clippy::disallowed_methods, clippy::disallowed_types)]

pub mod application;
pub mod domain;
pub mod embedded;
pub mod ports;

#[cfg(test)]
mod tests;
