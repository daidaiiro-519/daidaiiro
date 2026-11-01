//! 値オブジェクト。作ったら変えず、値が同じなら同じものとみなす。

mod rule_result;
mod verdict;

pub use rule_result::{By, Counts, RuleResult};
pub use verdict::Verdict;
