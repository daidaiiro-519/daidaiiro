//! UC1 作るの write-skill-md：Skill の項目（skill.json）から SKILL.md を決まった形で作り、同じフォルダに書く。
//! 同じ項目からは同じ SKILL.md になる。前と同じ中身なら書かない。skill.json は変えない。

use crate::ports::{GeneratedFiles, SkillFiles};
use crate::use_cases::{Refused, read_skill, trimmed};
use skills_domain::services::skill_md;

/// 作った結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Built {
    /// 渡されたフォルダ（渡されたまま）
    pub folder: String,
    /// 書いた SKILL.md の経路
    pub path: String,
    /// 書く前の SKILL.md と中身が違ったか（無かったときも true）
    pub changed: bool,
}

/// SKILL.md を作って書く。
///
/// # Errors
/// フォルダが無い（usage）・ skill.json を読めない（not_found）・ JSON でない（invalid_input）・ 書けない（not_found）とき。
pub fn write_skill_md(
    folder: Option<&str>,
    files: &dyn SkillFiles,
    out: &dyn GeneratedFiles,
) -> Result<Built, Refused> {
    let Some(folder) = folder else {
        return Err(Refused {
            code: "usage",
            message: "SKILL.md を作る Skill のフォルダを渡してください".to_string(),
        });
    };
    let skill = read_skill(files, folder)?;
    let text = skill_md::render(&skill);
    let path = format!("{}/{}", trimmed(folder), skill_md::FILE);
    let changed = files.read(&path).ok().as_deref() != Some(text.as_str());
    if changed {
        out.write(&path, &text).map_err(|message| Refused {
            code: "not_found",
            message,
        })?;
    }
    Ok(Built {
        folder: folder.to_string(),
        path,
        changed,
    })
}
