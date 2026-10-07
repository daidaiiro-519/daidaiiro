//! アプリケーション。ユースケースを置く。読み書き ・ 描画 ・ 転写は、出ていく側のポートを直接使うトランザクションスクリプトである。

// 具体が使ってよいもの：ユースケースと、その結果
pub mod checks;
pub mod instances;
pub mod renders;
pub mod transcriptions;

// 基盤の中だけで使うもの。具体からは見えない（基盤のテストは feature "internals" で開く）
#[cfg(feature = "internals")]
pub mod context;
#[cfg(not(feature = "internals"))]
pub(crate) mod context;
#[cfg(feature = "internals")]
pub mod page;
#[cfg(not(feature = "internals"))]
pub(crate) mod page;
#[cfg(feature = "internals")]
pub mod paths;
#[cfg(not(feature = "internals"))]
pub(crate) mod paths;
#[cfg(feature = "internals")]
pub mod view;
#[cfg(not(feature = "internals"))]
pub(crate) mod view;
