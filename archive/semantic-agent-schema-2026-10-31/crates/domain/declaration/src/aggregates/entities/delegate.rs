//! 任せる先（x-delegates。Abstract と Concrete の両方が持つ）

use serde_json::Value;

/// 任せる先：AI か CLI のプロセスに、判定か作業を任せる相手
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delegate {
    pub(in crate::aggregates) name: String,
    pub(in crate::aggregates) kind: String,
    pub(in crate::aggregates) role: String,
    pub(in crate::aggregates) argv: Option<Vec<String>>,
    pub(in crate::aggregates) session: Option<String>,
    pub(in crate::aggregates) output: Option<Value>,
}

impl Delegate {
    /// 名前（x-delegates のキー）
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 種類（run.kind：judging ・ working）
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// 立場（role）
    pub fn role(&self) -> &str {
        &self.role
    }

    /// CLI のプロセスとして起動するときの引数（run.argv）。無ければコーディングエージェントで起動する
    pub fn argv(&self) -> Option<&[String]> {
        self.argv.as_deref()
    }

    /// CLI のプロセスとして動かすか（run.argv がある）
    pub fn has_argv(&self) -> bool {
        self.argv.is_some()
    }

    /// セッションの型（run.session：single ・ continuous。working だけ）
    pub fn session(&self) -> Option<&str> {
        self.session.as_deref()
    }

    /// 出力に足す details の形（output）
    pub fn output(&self) -> Option<&Value> {
        self.output.as_ref()
    }
}
