//! rule（書くときに守る決まりの1件）

use crate::values::by::By;

/// rule：Concrete の中で id が一意な、書くときに守る決まりの1件
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub(in crate::aggregates::concrete_schema) id: String,
    pub(in crate::aggregates::concrete_schema) pointer: String,
    pub(in crate::aggregates::concrete_schema) by: Option<By>,
    pub(in crate::aggregates::concrete_schema) agent: Option<String>,
    pub(in crate::aggregates::concrete_schema) reads: Vec<String>,
}

impl Rule {
    /// id
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Concrete の中で rule を書いた場所（JSON Pointer）
    pub fn pointer(&self) -> &str {
        &self.pointer
    }

    /// 判定する者（書いていなければ None）
    pub fn by(&self) -> Option<By> {
        self.by
    }

    /// by が ai のときの任せる先の名前
    pub fn agent(&self) -> Option<&str> {
        self.agent.as_deref()
    }

    /// 読む欄の JSON Pointer。空ならインスタンスの全体
    pub fn reads(&self) -> &[String] {
        &self.reads
    }
}
