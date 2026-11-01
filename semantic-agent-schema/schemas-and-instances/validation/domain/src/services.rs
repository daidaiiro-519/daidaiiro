//! 業務サービス：集約を読んで rule に照らす判定。

mod abstract_rules;
mod relationship;
mod rules;

pub use abstract_rules::judge_concrete;
pub use relationship::{check_relationships, check_targets};
pub use rules::{AppliedRule, judge_others, rules_for_concrete, rules_for_instance};
