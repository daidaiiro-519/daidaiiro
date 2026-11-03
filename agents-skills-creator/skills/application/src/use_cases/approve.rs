//! 承認（人が打つ）：agents-skills-creator approve <Skill のフォルダ>。判定まで済んだ確かめ（最後に書いたあとの、
//! not-judged ・ violated ・ error が0の validate の操作）の ID と、そのときの中身のハッシュを SAS の記録から引き、
//! semantic-agent-schema update で /approved に書かせる。承認できなければ理由を返して書かない。AI のセッションから打つと SAS が断る
//! （/approved は人が決める欄）ので、そのときは人が打つコマンドを返す。

use crate::ports::{SasRun, SkillFiles, SkillHistory, SkillWriter};
use crate::use_cases::validate::judged_validation;
use crate::use_cases::{Refused, read_skill, trimmed};
use serde_json::Value;

/// 承認を書かせた結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Approved {
    /// 渡されたフォルダ
    pub folder: String,
    /// 承認した確かめの操作 ID
    pub operation: String,
    /// SAS の結果（書き換えない）
    pub sas: SasRun,
    /// SAS が断ったとき（AI のセッションから打たれたなど）の、人が打つコマンド
    pub next: Option<String>,
}

/// 承認を書かせる。
///
/// # Errors
/// フォルダが無い（usage）・ skill.json を読めない（not_found ・ invalid_input）・ 判定まで済んだ確かめが無い
/// （not_approvable）・ semantic-agent-schema を呼べない（semantic_agent_schema_unavailable）とき。
pub fn approve(
    folder: Option<&str>,
    files: &dyn SkillFiles,
    history: &dyn SkillHistory,
    writer: &dyn SkillWriter,
) -> Result<Approved, Refused> {
    let folder = trimmed(folder.ok_or_else(|| Refused {
        code: "usage",
        message: "承認する Skill のフォルダを渡してください".to_string(),
    })?);
    let skill = read_skill(files, folder)?;
    let path = format!("{folder}/skill.json");
    let not = |message: String| Refused {
        code: "not_approvable",
        message,
    };
    let op = judged_validation(history, &path).ok_or_else(|| {
        not(format!(
            "意味の判定まで済んだ確かめが無い（agents-skills-creator skills validate {folder} の承認の材料の delegate の a で判定して semantic-agent-schema validate --answer で渡し、validate を打ち直す）"
        ))
    })?;
    let hash = history
        .operation(&op)
        .ok()
        .and_then(|r| {
            r.output
                .pointer("/targets/0/hash")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .ok_or_else(|| {
            not(format!(
                "確かめ {op} の中身のハッシュを、SAS の記録から引けない"
            ))
        })?;
    let verb = if skill.get("approved").is_some() {
        "replace"
    } else {
        "add"
    };
    let patch = format!("[{{\"op\":\"{verb}\",\"path\":\"/approved\",\"value\":\"{op}\"}}]");
    let run = writer
        .update_patch(&path, &patch, &hash, &format!("承認：{op}"))
        .map_err(|message| Refused {
            code: "semantic_agent_schema_unavailable",
            message,
        })?;
    let next = (run.exit != 0).then(|| {
        format!("作る人が承認するなら、人が自分の端末で agents-skills-creator approve {folder} を打つ（AI のセッションからは SAS が断る）")
    });
    Ok(Approved {
        folder: folder.to_string(),
        operation: op,
        sas: run,
        next,
    })
}
