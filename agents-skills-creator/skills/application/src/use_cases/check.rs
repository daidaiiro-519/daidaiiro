//! UC4 確かめる（Skill だけ）：渡した Skill のフォルダごとに、SAS の検証の結果（書き換えない）と、SAS が文書1つでは
//! 判定できない creator の検査3つの結果を返す。確かめても Skill のファイルは変えない（SAS の記録 .git/sas/sas.db には
//! 検証の操作が1件足される）。
//!
//! creator の検査：
//! - name-is-folder：skill.json の name がフォルダ名と同じ（Agent Skills の仕様。SAS は文書1つしか見ない）
//! - name-unique：同じ親フォルダにある Skill のあいだで name が重ならない（複数の Skill をまたぐ）
//! - links-resolve：フォルダそのものと、フォルダの中のシンボリックリンクの先がある（ファイルの外の事実）

use crate::ports::{SasRun, SkillFiles, SkillValidator};
use serde_json::Value;

/// creator の検査の結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// 満たす
    Verified,
    /// 満たさない
    Violated,
    /// 確かめる材料を読めなかった（skill.json が無い ・ JSON でない ・ name が無い）
    Error,
}

impl Outcome {
    /// 出力に書く名前（SAS の判定の区分と同じ語）。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Violated => "violated",
            Self::Error => "error",
        }
    }
}

/// creator の検査の結果1件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// 検査の名前（name-is-folder ・ name-unique ・ links-resolve）
    pub check: &'static str,
    /// 結果
    pub outcome: Outcome,
    /// 違反に当たる所（重なった相手のフォルダ ・ 先の無いリンク）
    pub path: Option<String>,
    /// 理由
    pub reason: String,
}

/// Skill のフォルダ1つの結果。
#[derive(Debug, Clone, PartialEq)]
pub struct SkillChecked {
    /// 渡されたフォルダの経路（渡されたまま）
    pub folder: String,
    /// SAS の検証の結果（書き換えない）
    pub sas: SasRun,
    /// creator の検査の結果（検査の順）
    pub creator: Vec<Finding>,
}

impl SkillChecked {
    /// SAS か creator のどちらかで検出があったか。
    #[must_use]
    pub fn detected(&self) -> bool {
        self.sas.exit != 0 || self.creator.iter().any(|f| f.outcome != Outcome::Verified)
    }
}

/// 確かめた結果（渡した順）。
#[derive(Debug, Clone, PartialEq)]
pub struct Checked {
    /// Skill ごとの結果
    pub skills: Vec<SkillChecked>,
}

impl Checked {
    /// どれかの Skill で検出があったか。
    #[must_use]
    pub fn detected(&self) -> bool {
        self.skills.iter().any(SkillChecked::detected)
    }
}

/// 始められなかった理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    /// 決まった名前（usage ・ sas_unavailable）
    pub code: &'static str,
    /// 理由
    pub message: String,
}

/// フォルダの経路の末尾の / を除く。
fn trimmed(folder: &str) -> &str {
    let t = folder.trim_end_matches('/');
    if t.is_empty() { folder } else { t }
}

/// フォルダ名（経路の最後の部分）。
fn folder_name(folder: &str) -> &str {
    trimmed(folder).rsplit('/').next().unwrap_or_default()
}

/// フォルダの skill.json の name。読めなければ理由。
fn skill_name(files: &dyn SkillFiles, folder: &str) -> Result<String, String> {
    let path = format!("{}/skill.json", trimmed(folder));
    let text = files.read(&path)?;
    let value: Value =
        serde_json::from_str(&text).map_err(|e| format!("{path} が JSON として読めない：{e}"))?;
    value
        .get("name")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("{path} に name が無い"))
}

fn finding(check: &'static str, outcome: Outcome, path: Option<String>, reason: String) -> Finding {
    Finding {
        check,
        outcome,
        path,
        reason,
    }
}

/// フォルダそのものと、フォルダの中のシンボリックリンクの先があるか。
fn links_resolve(files: &dyn SkillFiles, folder: &str) -> Vec<Finding> {
    const CHECK: &str = "links-resolve";
    let folder = trimmed(folder);
    if !files.exists(folder) {
        return vec![finding(
            CHECK,
            Outcome::Violated,
            Some(folder.to_string()),
            format!("フォルダ {folder} が無い（先の無いリンクか、無い経路）"),
        )];
    }
    match files.dangling_links(folder) {
        Ok(broken) if broken.is_empty() => vec![finding(
            CHECK,
            Outcome::Verified,
            None,
            "フォルダと、フォルダの中のリンクの先がある".to_string(),
        )],
        Ok(broken) => broken
            .into_iter()
            .map(|l| {
                let reason = format!("リンク {l} の先が無い");
                finding(CHECK, Outcome::Violated, Some(l), reason)
            })
            .collect(),
        Err(e) => vec![finding(CHECK, Outcome::Error, None, e)],
    }
}

/// name がフォルダ名と同じか、同じ親フォルダのほかの Skill と name が重ならないか。
fn name_checks(files: &dyn SkillFiles, folder: &str) -> Vec<Finding> {
    let name = match skill_name(files, folder) {
        Ok(n) => n,
        Err(e) => {
            return vec![
                finding("name-is-folder", Outcome::Error, None, e.clone()),
                finding("name-unique", Outcome::Error, None, e),
            ];
        }
    };
    let dir = folder_name(folder);
    let is_folder = if name == dir {
        finding(
            "name-is-folder",
            Outcome::Verified,
            None,
            format!("name {name} がフォルダ名と同じ"),
        )
    } else {
        finding(
            "name-is-folder",
            Outcome::Violated,
            None,
            format!("name {name} がフォルダ名 {dir} と違う"),
        )
    };
    // 同じ親フォルダのほかのフォルダのうち、skill.json の name を読めるものが Skill
    let unique = match files.siblings(trimmed(folder)) {
        Ok(others) => {
            let same: Vec<Finding> = others
                .into_iter()
                .filter(|o| skill_name(files, o).is_ok_and(|n| n == name))
                .map(|o| {
                    let reason = format!("name {name} が、同じ親フォルダの {o} と重なる");
                    finding("name-unique", Outcome::Violated, Some(o), reason)
                })
                .collect();
            if same.is_empty() {
                vec![finding(
                    "name-unique",
                    Outcome::Verified,
                    None,
                    format!("name {name} は、同じ親フォルダのほかの Skill と重ならない"),
                )]
            } else {
                same
            }
        }
        Err(e) => vec![finding("name-unique", Outcome::Error, None, e)],
    };
    let mut out = vec![is_folder];
    out.extend(unique);
    out
}

/// UC4 確かめる（Skill だけ）：フォルダごとに creator の検査3つと SAS の検証をして、渡した順に返す。Skill のファイルは変えない。
///
/// # Errors
/// フォルダが1つも無い（usage）・ sas を呼べない（sas_unavailable）とき。
pub fn check(
    folders: &[String],
    sas: &dyn SkillValidator,
    files: &dyn SkillFiles,
) -> Result<Checked, Refused> {
    if folders.is_empty() {
        return Err(Refused {
            code: "usage",
            message: "確かめる Skill のフォルダを1つ以上渡してください".to_string(),
        });
    }
    let mut skills = Vec::new();
    for folder in folders {
        let mut creator = name_checks(files, folder);
        creator.extend(links_resolve(files, folder));
        let run = sas
            .validate(&format!("{}/skill.json", trimmed(folder)))
            .map_err(|message| Refused {
                code: "sas_unavailable",
                message,
            })?;
        skills.push(SkillChecked {
            folder: folder.clone(),
            sas: run,
            creator,
        });
    }
    Ok(Checked { skills })
}
