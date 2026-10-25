//! 任せる先（x-delegates。Abstract と Concrete の両方が持つ）

/// 任せる先：AI か CLI のプロセスに、判定か作業を任せる相手
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delegate {
    pub(in crate::aggregates) name: String,
    pub(in crate::aggregates) kind: String,
    pub(in crate::aggregates) has_argv: bool,
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

    /// CLI のプロセスとして動かすか（run.argv がある）
    pub fn has_argv(&self) -> bool {
        self.has_argv
    }
}
