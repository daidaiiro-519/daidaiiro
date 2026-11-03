//! ユースケースの手続き（トランザクションスクリプト）。

pub mod approval;
pub mod approve;
pub mod create;
pub mod distribute;
pub mod update;
pub mod validate;
pub mod withdraw;
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

/// SAS の手順を、creator のコマンドで書く形にする：command を creator のコマンドに置き換え、constraints の
/// 「command の sas」を「command の <creator のコマンド>」にする（sas を直に打つ形を出さない）。手順でなければ変えない。
pub fn as_creator_procedure(output: &mut serde_json::Value, command: String, bin: &str) {
    if output.get("kind").and_then(serde_json::Value::as_str) != Some("procedure") {
        return;
    }
    output["command"] = serde_json::Value::String(command);
    // 返す手順に無い欄（各 step の inner_rules ・ 手順の rules）を指す文を出さない
    let has_inner = output
        .get("steps")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|ss| {
            ss.iter().any(|s| {
                s.get("inner_rules")
                    .is_some_and(|r| r.as_array().is_some_and(|a| !a.is_empty()))
            })
        });
    let has_rules = output
        .get("rules")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|r| !r.is_empty());
    if let Some(cs) = output
        .get_mut("constraints")
        .and_then(serde_json::Value::as_array_mut)
    {
        for c in cs.iter_mut() {
            if let Some(t) = c.as_str() {
                let mut t = t.replace("command の sas", &format!("command の {bin}"));
                if !has_inner {
                    t = t.replace("と、inner_rules の rule（欄の中の所 pointer に当たる。* は配列の要素か名前を選ばない欄）", "");
                }
                if !has_rules {
                    t = t.replace("と、手順の rules（文書全体に当たる rule）", "");
                }
                *c = serde_json::Value::String(t);
            }
        }
    }
}
