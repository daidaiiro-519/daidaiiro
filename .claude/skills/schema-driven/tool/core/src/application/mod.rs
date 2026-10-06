//! アプリケーション。ユースケースを置く。読み書き ・ 描画 ・ 転写は、出ていく側のポートを直接使うトランザクションスクリプトである。

pub mod checks;
pub mod context;
pub mod instances;
pub mod page;
pub mod paths;
pub mod view;
