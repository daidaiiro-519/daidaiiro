//! UC6 使わなくする（Agent の側）：agents-skills-creator agents withdraw。配布を取り下げる（distribute の対）。Skill の取り下げと同じ考えで、
//! distribute が置く4か所（Claude Code ・ Codex × repository ・ user）のうち在るものを、この Agent から作った写し（ふつうの
//! ファイルで、印がこの Agent のフォルダを指す。中身が古い版でもよい）か、ほかのもの（印が無い ・ ほかの Agent を指す ・
//! リンク ・ フォルダ）かに分けて示し、人が打ったときだけ、この Agent から作った写しを外す。直してから取り下げても、
//! 前の版の写しが外れる。印は置く範囲ごとに見る（repository はリポジトリの根からのフォルダ、user は絶対の経路）。
//! Agent の実体（agents/<name>）は消さない。

use crate::ports::{AgentFiles, Destinations};
use crate::use_cases::distribute::{Scope, Tool};
use crate::use_cases::{Finding, Outcome, Refused, finding, trimmed};
use agents_domain::services::agent_md;

/// 置き場所1つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    /// 経路
    pub path: String,
    /// ツール
    pub tool: &'static str,
    /// 範囲
    pub scope: &'static str,
    /// placed（この Agent から作った写し。印で見分ける。外す）・ other（ほかのもの。外さない）
    pub state: &'static str,
}

/// 取り下げた結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Withdrawn {
    /// 渡したフォルダ
    pub folder: String,
    /// 理由
    pub reason: String,
    /// 外す前に在った置き場所
    pub places: Vec<Place>,
    /// 外したリンク
    pub removed: Vec<String>,
    /// 取り下げたか
    pub withdrawn: bool,
    /// AI のセッションから打たれたときの、人が打つコマンド
    pub next: Option<String>,
    /// 外したあとの確かめ（unlinked）
    pub after: Vec<Finding>,
}

fn places(folder: &str, name: &str, files: &dyn AgentFiles, dest: &dyn Destinations) -> Vec<Place> {
    let mut roots: Vec<(Scope, String)> = vec![(Scope::Repository, String::new())];
    if let Some(home) = dest.home() {
        roots.push((Scope::User, format!("{}/", home.trim_end_matches('/'))));
    }
    let mut out = Vec::new();
    for (scope, root) in &roots {
        // 印は置く範囲で変わる（repository はリポジトリの根から、user は絶対の経路）
        let mark = scope.mark(folder, dest);
        for tool in [Tool::ClaudeCode, Tool::Codex] {
            let path = format!("{root}{}/{name}.{}", tool.dir(), tool.extension());
            if !dest.occupied(&path) {
                continue;
            }
            let ours = dest.is_file(&path)
                && files
                    .read(&path)
                    .ok()
                    .and_then(|t| agent_md::read_mark(&t))
                    .is_some_and(|m| Some(m) == mark);
            out.push(Place {
                path,
                tool: tool.name(),
                scope: scope.name(),
                state: if ours { "placed" } else { "other" },
            });
        }
    }
    out
}

/// 配布を取り下げる。human が false（AI のセッションから打たれた）なら外さない。
///
/// # Errors
/// フォルダか理由が無い（usage）・ リンクを外せない（not_found）とき。
pub fn withdraw(
    folder: Option<&str>,
    reason: Option<&str>,
    human: bool,
    files: &dyn AgentFiles,
    dest: &dyn Destinations,
) -> Result<Withdrawn, Refused> {
    let usage = |m: &str| Refused {
        code: "usage",
        message: m.to_string(),
    };
    let folder =
        trimmed(folder.ok_or_else(|| usage("取り下げる Agent のフォルダを渡してください"))?);
    let reason = reason
        .filter(|r| !r.trim().is_empty())
        .ok_or_else(|| usage("--reason に、使わなくする理由を渡してください"))?;
    let name = folder.rsplit('/').next().unwrap_or(folder);
    let found = places(folder, name, files, dest);
    if !human {
        return Ok(Withdrawn {
            folder: folder.to_string(),
            reason: reason.to_string(),
            places: found,
            removed: Vec::new(),
            withdrawn: false,
            next: Some(format!(
                "作る人が影響（places）を見て承認したら、人が自分の端末で agents-skills-creator agents withdraw {folder} --reason \"{reason}\" を打つ（AI のセッションから打つと外さない）"
            )),
            after: Vec::new(),
        });
    }
    let mut removed = Vec::new();
    for p in found.iter().filter(|p| p.state == "placed") {
        dest.remove(&p.path).map_err(|message| Refused {
            code: "not_found",
            message,
        })?;
        removed.push(p.path.clone());
    }
    let left: Vec<String> = places(folder, name, files, dest)
        .into_iter()
        .filter(|p| p.state == "placed")
        .map(|p| p.path)
        .collect();
    let after = if left.is_empty() {
        finding(
            "unlinked",
            Outcome::Verified,
            None,
            "どの置き場所にも、この Agent の写しが残っていない".to_string(),
        )
    } else {
        finding(
            "unlinked",
            Outcome::Violated,
            Some(left.join(" ・ ")),
            "この Agent の写しが残っている".to_string(),
        )
    };
    Ok(Withdrawn {
        folder: folder.to_string(),
        reason: reason.to_string(),
        places: found,
        removed,
        withdrawn: true,
        next: None,
        after: vec![after],
    })
}
