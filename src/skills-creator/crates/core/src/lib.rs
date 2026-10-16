//! skills-creator の中心。domain（SKILL.md の読み取り ・ check の項目 ・ 公開された基盤の名前と照合）と、
//! application（scaffold ・ engine ・ check の手順）と、出ていく側の port を持つ。外の技術に依存しない。

pub mod application;
pub mod domain;
pub mod ports;

#[cfg(test)]
mod tests;
