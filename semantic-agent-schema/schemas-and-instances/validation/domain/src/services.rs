//! 業務サービス：集約を読んで rule に照らす判定。

mod abstract_rules;
mod rules;

pub use abstract_rules::builtin_checks;
pub use rules::{AppliedRule, rules_for_concrete, rules_for_instance, unjudged};
