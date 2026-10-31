//! 任せる先を呼ぶ：Concrete の x-delegates に宣言された任せる先を、宣言どおりに起動して出力を受け取る
//! （argv があれば CLI のプロセス、無ければコーディングエージェント。どちらで起動するかはアダプターが決め、使う場面は知らない）

use semantic_agent_schema_declaration::aggregates::entities::delegate::Delegate;
use semantic_agent_schema_validation::values::call::Call;
use serde_json::Value;

/// 出どころ：誰が答えたか。判定の記録に残し、同じ入力の判定を使い直すかを決める材料にする
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provenance {
    /// コーディングエージェント（名前 ・ 版 ・ 実際に答えたモデル ・ 渡したプロンプト）
    Agent {
        /// コーディングエージェントの名前（claude-code など）
        agent: String,
        /// コーディングエージェントの版（分からなければ None）
        version: Option<String>,
        /// 実際に答えたモデル（分からなければ None）
        model: Option<String>,
        /// 渡したプロンプト
        prompt: String,
    },
    /// CLI のプロセス（起動した argv 全体）
    Process {
        /// 起動した argv
        argv: Vec<String>,
    },
}

/// 任せる先の答え：出力（種別の出力の形。judging なら output-judging）と出どころ
#[derive(Debug, Clone, PartialEq)]
pub struct Answer {
    /// 出力
    pub output: Value,
    /// 出どころ
    pub provenance: Provenance,
}

/// 任せる先を呼べなかった理由
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegateError {
    /// この任せる先は起動できない（宣言のしかた ・ コーディングエージェントが条件を満たさない）。呼び直しても変わらない
    Unsupported(String),
    /// 起動したが答えを得られなかった（起動できない ・ 時間切れ ・ 終了コード ・ 出力が JSON でない）
    Failed(String),
}

/// 任せる先を呼ぶ
pub trait Delegates {
    /// 任せる先に呼び出し（call.schema.json の形）を渡し、出力の形（自己完結の JSON Schema）で答えさせる
    ///
    /// # Errors
    /// 起動できない先は Unsupported、起動したが答えを得られなければ Failed
    fn call(
        &self,
        delegate: &Delegate,
        call: &Call,
        output_schema: &Value,
    ) -> Result<Answer, DelegateError>;
}
