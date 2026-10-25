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

use semantic_agent_schema_change::values::instance_state::InstanceState;
use semantic_agent_schema_change::values::verdict::{Finding, Verdict};
use serde_json::{Value, json};

/// 結果の1行：どこで ・ 何が（refused か検証の5つの値）・ 根拠 ・ 判定した者
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// rule の id（形の検証は json-schema、検査①の断りは空）
    pub rule: String,
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
                rule: String::new(),
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
            findings: lines(findings),
            data,
        }
    }

    /// rule ごとの検証結果から作る。findings は violated と error だけ、data.results に全部、data.state に状態（決定表）を入れる
    pub fn validated(results: &[Finding], data: Value) -> Self {
        let problems: Vec<Finding> = results
            .iter()
            .filter(|f| matches!(f.verdict(), Verdict::Violated | Verdict::Error))
            .cloned()
            .collect();
        let state = InstanceState::from_results(results);
        let mut data = data;
        data["results"] = json!(lines(results).iter().map(Line::to_json).collect::<Vec<_>>());
        data["state"] = json!({"name": state.as_str(), "label": state.label()});
        Outcome {
            ok: problems.is_empty(),
            findings: lines(&problems),
            data,
        }
    }
}

impl Line {
    /// JSON にする
    pub fn to_json(&self) -> Value {
        json!({"rule": self.rule, "pointer": self.pointer, "result": self.result, "reason": self.reason, "by": self.by})
    }
}

fn lines(findings: &[Finding]) -> Vec<Line> {
    findings
        .iter()
        .map(|f| Line {
            rule: f.rule().to_string(),
            pointer: f.pointer().to_string(),
            result: f.verdict().as_str().to_string(),
            reason: f.reason().to_string(),
            by: f.by().as_str().to_string(),
        })
        .collect()
}
