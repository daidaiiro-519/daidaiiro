//! 変わったファイルの前後（まとめの材料）

use semantic_agent_schema_declaration::values::target::Target;

/// 変わったファイルの前後：置き場所 ・ 対象 ・ 前の中身 ・ 後の中身 ・ 人が決める項目（x-decided-by の JSON Pointer）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileState {
    path: String,
    target: Target,
    before: Option<String>,
    after: Option<String>,
    decided: Vec<String>,
}

impl FileState {
    /// 作る
    pub fn new(
        path: &str,
        target: Target,
        before: Option<String>,
        after: Option<String>,
        decided: Vec<String>,
    ) -> Self {
        FileState {
            path: path.to_string(),
            target,
            before,
            after,
            decided,
        }
    }

    /// 置き場所
    pub fn path(&self) -> &str {
        &self.path
    }

    /// 対象
    pub fn target(&self) -> &Target {
        &self.target
    }

    /// 前の中身
    pub fn before(&self) -> Option<&str> {
        self.before.as_deref()
    }

    /// 後の中身
    pub fn after(&self) -> Option<&str> {
        self.after.as_deref()
    }

    /// 人が決める項目
    pub fn decided(&self) -> &[String] {
        &self.decided
    }
}
