//! ユースケースの手続き（トランザクションスクリプト）。

pub mod approval;
pub mod approve;
pub mod create;
pub mod distribute;
pub mod update;
pub mod validate;
pub mod withdraw;
pub mod write_agent_md;

use crate::ports::AgentFiles;
use serde_json::Value;

/// Agent の項目のファイル名。
pub const AGENT_JSON: &str = "agent.json";

/// 始められなかった理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    /// 決まった名前（usage ・ not_found ・ invalid_input ・ sas_unavailable ・ skills_unavailable）
    pub code: &'static str,
    /// 理由
    pub message: String,
}

/// creator の検査の結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// 満たす
    Verified,
    /// 満たさない
    Violated,
    /// 確かめる材料を読めなかった
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
    /// 検査の名前
    pub check: &'static str,
    /// 結果
    pub outcome: Outcome,
    /// 違反に当たる所
    pub path: Option<String>,
    /// 理由
    pub reason: String,
}

/// 検査の結果1件を作る。
#[must_use]
pub fn finding(
    check: &'static str,
    outcome: Outcome,
    path: Option<String>,
    reason: String,
) -> Finding {
    Finding {
        check,
        outcome,
        path,
        reason,
    }
}

/// フォルダの経路の末尾の / を除く。
#[must_use]
pub fn trimmed(folder: &str) -> &str {
    let t = folder.trim_end_matches('/');
    if t.is_empty() { folder } else { t }
}

/// フォルダの agent.json を JSON として読む。
///
/// # Errors
/// 無い ・ 読めない（not_found）・ JSON でない（invalid_input）とき。
pub fn read_agent(files: &dyn AgentFiles, folder: &str) -> Result<Value, Refused> {
    let path = format!("{}/{AGENT_JSON}", trimmed(folder));
    let text = files.read(&path).map_err(|message| Refused {
        code: "not_found",
        message,
    })?;
    serde_json::from_str(&text).map_err(|e| Refused {
        code: "invalid_input",
        message: format!("{path} が JSON として読めない：{e}"),
    })
}

/// 文字列の並びを読む。
#[must_use]
pub fn strings(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect()
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
