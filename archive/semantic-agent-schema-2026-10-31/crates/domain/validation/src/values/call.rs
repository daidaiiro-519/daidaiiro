//! 依頼の一覧（呼び出し call）：基盤が任せる先を起動するときに渡す入力（call.schema.json の形）

use serde_json::{Value, json};

/// 判定させる rule 1件：id ・ 文 ・ なぜ ・ 確かめ方と、インスタンスの中の欄の場所
#[derive(Debug, Clone, PartialEq)]
pub struct CallRule {
    id: String,
    at: String,
    text: Value,
}

impl CallRule {
    /// 作る。text は Concrete の write の行（id ・ rule ・ why ・ test を写す）
    pub fn new(id: &str, at: &str, row: &Value) -> Self {
        let pick = |k: &str| row.get(k).cloned().unwrap_or(Value::Null);
        CallRule {
            id: id.to_string(),
            at: at.to_string(),
            text: json!({"id": id, "rule": pick("rule"), "why": pick("why"), "test": pick("test")}),
        }
    }

    /// rule の id
    pub fn id(&self) -> &str {
        &self.id
    }

    /// インスタンスの中の欄の場所（結果を返す場所）
    pub fn at(&self) -> &str {
        &self.at
    }
}

/// 読む値の写し1件：場所 ・ 値 ・ その欄の説明と読み方
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    at: String,
    value: Value,
    description: Value,
    read: Value,
}

impl Input {
    /// 作る
    pub fn new(at: &str, value: Value, description: Value, read: Value) -> Self {
        Input {
            at: at.to_string(),
            value,
            description,
            read,
        }
    }

    /// 場所
    pub fn at(&self) -> &str {
        &self.at
    }
}

/// 呼び出し：任せる先1つ × 対象1件 × その任せる先に判定させる rule の組
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    delegate: String,
    target: String,
    rules: Vec<CallRule>,
    inputs: Vec<Input>,
}

impl Call {
    /// 作る
    pub fn new(delegate: &str, target: &str, rules: Vec<CallRule>, inputs: Vec<Input>) -> Self {
        Call {
            delegate: delegate.to_string(),
            target: target.to_string(),
            rules,
            inputs,
        }
    }

    /// 任せる先の名前
    pub fn delegate(&self) -> &str {
        &self.delegate
    }

    /// 対象の置き場所
    pub fn target(&self) -> &str {
        &self.target
    }

    /// 判定させる rule
    pub fn rules(&self) -> &[CallRule] {
        &self.rules
    }

    /// 読む値の写し
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    /// call.schema.json の形の JSON
    pub fn to_json(&self) -> Value {
        json!({
            "delegate": self.delegate,
            "target": self.target,
            "rules": self.rules.iter().map(|r| r.text.clone()).collect::<Vec<_>>(),
            "inputs": self.inputs.iter().map(|i| {
                let mut v = json!({"at": i.at, "value": i.value});
                if !i.description.is_null() { v["description"] = i.description.clone(); }
                if !i.read.is_null() { v["read"] = i.read.clone(); }
                v
            }).collect::<Vec<_>>()
        })
    }
}
