//! ドメイン。参照と導出値のドメインモデル（集約 インスタンス ・ 承認記録、ドメインサービス 検査する、値オブジェクト）を置く。

// 具体が使ってよいもの。ポートとユースケースの結果に出てくる値だけを出す
pub use check::Finding;
pub use schema_rules::SchemaFinding;
pub use values::{Hash, Html, JsonValue, Status, Unfilled, ValidationError};

// 基盤の中だけで使うもの。具体からは見えない（基盤のテストは feature "internals" で開く）
#[cfg(feature = "internals")]
pub mod approval;
#[cfg(not(feature = "internals"))]
pub(crate) mod approval;
#[cfg(feature = "internals")]
pub mod check;
#[cfg(not(feature = "internals"))]
pub(crate) mod check;
#[cfg(feature = "internals")]
pub mod instance;
#[cfg(not(feature = "internals"))]
pub(crate) mod instance;
#[cfg(feature = "internals")]
pub mod schema;
#[cfg(not(feature = "internals"))]
pub(crate) mod schema;
#[cfg(feature = "internals")]
pub mod schema_rules;
#[cfg(not(feature = "internals"))]
pub(crate) mod schema_rules;
#[cfg(feature = "internals")]
pub mod values;
#[cfg(not(feature = "internals"))]
pub(crate) mod values;
