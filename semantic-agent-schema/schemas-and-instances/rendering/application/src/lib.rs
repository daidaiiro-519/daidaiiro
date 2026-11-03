//! 業務領域「描画」のモジュール（トランザクションスクリプト）。
//!
//! UC5 描画する：使う側が渡すデザインシステム（テンプレートとデザイントークン）と、Concrete に従うインスタンスの JSON を
//! 受け取り、インスタンスの値をテンプレートに埋め込んで、決められたデザインで描く。同じ入力からは同じページになる。
//! デザインを組み立てる部品を持たず、判定 ・ 記録は扱わない。描いたページは記録しない（コミットもしない）。
//! ポート TemplateRenderer（テンプレートの処理系）・ ScriptReader（JS を読む）・ PageWriter（書き出し先）と、描画が使う文書を読む
//! DocumentStore ・ 形を照らす SchemaValidator と、デザインシステムの検証（semantic-agent-schema validate）を記録する ValidationLog を ports.rs に宣言する。
//! 誰が打ったか（Caller）は入口が決めて値で渡す。

pub mod ports;
pub mod use_cases;
