//! 値オブジェクト。作ったら変えず、値が同じなら同じものとみなす。

mod caller;
mod change_record;
mod content_hash;
mod corpus;
mod diff;
mod target_kind;
mod write_violation;

pub use caller::{Caller, Via};
pub use change_record::{ChangeRow, FieldRow, StoredChange, WriteRecord, untracked_since};
pub use content_hash::ContentHash;
pub use corpus::{Corpus, Doc};
pub use diff::{Change, Diff, FieldChange};
pub use target_kind::{TargetKind, Unclassified};
pub use write_violation::WriteViolation;
