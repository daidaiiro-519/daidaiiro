//! 値オブジェクト。作ったら変えず、値が同じなら同じものとみなす。

mod caller;
mod change_record;
mod content_hash;
mod diff;
mod schema_places;
mod target_kind;
mod write_violation;

pub use caller::{Caller, Via};
pub use change_record::{ChangeRow, FieldRow, WriteRecord, unrecorded_since};
pub use content_hash::ContentHash;
pub use diff::{Change, Diff, FieldChange};
pub use schema_places::SchemaPlaces;
pub use target_kind::{TargetKind, Unclassified};
pub use write_violation::WriteViolation;
