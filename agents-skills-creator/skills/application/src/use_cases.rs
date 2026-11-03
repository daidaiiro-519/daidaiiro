//! ユースケースの手続き（トランザクションスクリプト）。

pub mod approval;
pub mod create;
pub mod distribute;
pub mod update;
pub mod validate;
pub mod write_skill_md;

use crate::ports::SkillFiles;
use serde_json::Value;

/// 始められなかった理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    /// 決まった名前（usage ・ not_found ・ invalid_input ・ sas_unavailable）
    pub code: &'static str,
    /// 理由
    pub message: String,
}

/// フォルダの経路の末尾の / を除く。
#[must_use]
pub fn trimmed(folder: &str) -> &str {
    let t = folder.trim_end_matches('/');
    if t.is_empty() { folder } else { t }
}

/// フォルダの skill.json を JSON として読む。
///
/// # Errors
/// 無い ・ 読めない（not_found）・ JSON でない（invalid_input）とき。
pub fn read_skill(files: &dyn SkillFiles, folder: &str) -> Result<Value, Refused> {
    let path = format!("{}/skill.json", trimmed(folder));
    let text = files.read(&path).map_err(|message| Refused {
        code: "not_found",
        message,
    })?;
    serde_json::from_str(&text).map_err(|e| Refused {
        code: "invalid_input",
        message: format!("{path} が JSON として読めない：{e}"),
    })
}
