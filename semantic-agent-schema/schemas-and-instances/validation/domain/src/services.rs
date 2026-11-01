//! 業務サービス：集約を読んで rule に照らす判定。

mod abstract_rules;
mod judged;
mod rules;

pub use abstract_rules::builtin_checks;
pub use judged::{Judged, concrete_rules, instance_rules, reads_hash, reads_of, rule_reads, whole};
pub use rules::{AppliedRule, rules_for_concrete, rules_for_instance, unjudged};
