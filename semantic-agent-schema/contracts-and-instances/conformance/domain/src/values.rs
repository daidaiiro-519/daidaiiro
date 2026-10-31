//! 値オブジェクト。作ったら変えず、値が同じなら同じものとみなす。

mod diff;
mod fingerprint;
mod rule_result;
mod verdict;
mod write_violation;

pub use diff::{Change, Diff, FieldChange};
pub use fingerprint::Fingerprint;
pub use rule_result::{By, Counts, RuleResult};
pub use verdict::Verdict;
pub use write_violation::WriteViolation;
