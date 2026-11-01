//! 値オブジェクト。作ったら変えず、値が同じなら同じものとみなす。

mod content_hash;
mod diff;
mod write_violation;

pub use content_hash::ContentHash;
pub use diff::{Change, Diff, FieldChange};
pub use write_violation::WriteViolation;
