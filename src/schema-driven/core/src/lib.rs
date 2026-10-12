//! schema-driven の中心。ドメイン ・ アプリケーション ・ ポートを持ち、入出力を直接扱わない。
#![deny(clippy::disallowed_methods, clippy::disallowed_types)]

pub mod application;
pub mod domain;
pub mod ports;
