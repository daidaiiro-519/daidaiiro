//! ドメイン。参照と導出値のドメインモデル（集約 インスタンス ・ 承認記録、ドメインサービス 検査する、値オブジェクト）を置く。

pub use check::Finding;
pub use schema_rules::SchemaFinding;
pub use values::{Hash, Html, JsonValue, Status, Unfilled, ValidationError};

pub mod approval;
pub mod catalog;
pub mod check;
pub mod contract;
pub mod design_system;
pub mod instance;
pub mod path;
pub mod published;
pub mod schema;
pub mod schema_name;
pub mod schema_rules;
pub mod values;
