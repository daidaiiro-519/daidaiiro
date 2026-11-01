//! 値オブジェクト。作ったら変えず、値が同じなら同じものとみなす。

mod diff;
mod fingerprint;
mod write_violation;

pub use diff::{Change, Diff, FieldChange};
pub use fingerprint::Fingerprint;
pub use write_violation::WriteViolation;
