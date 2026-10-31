//! ドメインサービス：いくつかのものにまたがる判定。

mod relationship;
mod rules;

pub use relationship::{check_relationships, check_targets, references_to, relationship_targets};
pub use rules::{AppliedRule, judge_others, rules_for_concrete, rules_for_instance};
