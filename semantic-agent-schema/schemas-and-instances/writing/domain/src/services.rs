//! ドメインサービス：集約に属さない、スキーマの文書の読み方と引き方。

mod field_schema;
mod schema_index;

pub use field_schema::field_schema;
pub use schema_index::{Resolved, SchemaIndex, Unresolved, concrete_of, refs};
