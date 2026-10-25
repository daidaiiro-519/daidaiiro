//! 使う場面（コマンドの組ごとに1ファイル）
pub mod create;
pub mod history;
pub mod import;
pub mod judge;
pub mod patch;
pub mod record;
pub mod release;
pub mod render;
pub mod search;
pub mod semantic;
pub mod session;
pub mod validate;

use semantic_agent_schema_change::values::verdict::{Finding, Verdict};
use serde_json::Value;

/// 結果の1行：どこで ・ 何が（refused か検証の5つの値）・ 根拠 ・ 判定した者
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// 場所（JSON Pointer。入力の外の断りは空）
    pub pointer: String,
    /// refused（検査①で断った）か、検証の5つの値
    pub result: String,
    /// 根拠
    pub reason: String,
    /// 判定した者
    pub by: String,
}

/// コマンドの結果：ok ・ findings ・ data
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    /// violated と refused が無い
    pub ok: bool,
    /// 結果の行
    pub findings: Vec<Line>,
    /// コマンドが返す値
    pub data: Value,
}

impl Outcome {
    /// 検査①で断った（状態に触れていない）
    pub fn refused(pointer: &str, reason: &str) -> Self {
        Outcome {
            ok: false,
            findings: vec![Line {
                pointer: pointer.to_string(),
                result: "refused".to_string(),
                reason: reason.to_string(),
                by: "machine".to_string(),
            }],
            data: Value::Null,
        }
    }

    /// 検証結果から作る。violated か error が1件でもあれば ok でない
    pub fn checked(findings: &[Finding], data: Value) -> Self {
        Outcome {
            ok: !findings
                .iter()
                .any(|f| matches!(f.verdict(), Verdict::Violated | Verdict::Error)),
            findings: findings
                .iter()
                .map(|f| Line {
                    pointer: f.pointer().to_string(),
                    result: f.verdict().as_str().to_string(),
                    reason: f.reason().to_string(),
                    by: f.by().as_str().to_string(),
                })
                .collect(),
            data,
        }
    }
}
