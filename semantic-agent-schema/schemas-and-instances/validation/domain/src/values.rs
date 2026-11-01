//! 値オブジェクト。作ったら変えず、値が同じなら同じものとみなす。

mod rule_result;
mod validation_record;
mod verdict;

pub use rule_result::{By, Counts, RuleResult};
pub use validation_record::{Judgment, RuleRow, ValidateRow};
pub use verdict::Verdict;
