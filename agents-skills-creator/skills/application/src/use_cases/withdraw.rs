//! UC6 使わなくする：agents-skills-creator skills withdraw。配布を取り下げる（distribute の対）。
//!
//! 影響を示す：distribute が置く4か所（Claude Code ・ Codex × repository ・ user）のうち、いま在るものを、creator が置いた
//! リンク（Skill のフォルダを指すシンボリックリンク）か、ほかのものかに分けて返す。
//! 外すのは、人が打ったときだけ（入口が、AI のセッションの環境変数が無いことで見分けて human を渡す）。AI のセッションから
//! 打たれたら外さず、影響と、人が打つコマンドを返す。外すのは creator が置いたリンクだけで、実体のフォルダや別の先への
//! リンクは外さない。Skill の実体（skills/<name>）は消さない（UC6 で使わなくするのは配布だけ）。
//! リンクしている Agent を示すことは、Agent（UC2）がまだ無いので作らない。

use crate::ports::Destinations;
use crate::use_cases::distribute::{Scope, Tool};
use crate::use_cases::validate::{Finding, Outcome, finding};
use crate::use_cases::{Refused, trimmed};

/// 置き場所1つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    /// 経路
    pub path: String,
    /// ツール（claude-code ・ codex）
    pub tool: &'static str,
    /// 範囲（repository ・ user）
    pub scope: &'static str,
    /// linked（creator が置いた、Skill のフォルダを指すリンク）・ other（ほかのもの。外さない）
    pub state: &'static str,
}

/// 取り下げた結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Withdrawn {
    /// 渡したフォルダ
    pub folder: String,
    /// 理由（作る人が伝えたもの）
    pub reason: String,
    /// 外す前に在った置き場所
    pub places: Vec<Place>,
    /// 外したリンク
    pub removed: Vec<String>,
    /// 取り下げたか（人が打ったとき）
    pub withdrawn: bool,
    /// AI のセッションから打たれたときの、人が打つコマンド
    pub next: Option<String>,
    /// 外したあとの確かめ（unlinked）
    pub after: Vec<Finding>,
}

const TOOLS: [(Tool, &str); 2] = [(Tool::ClaudeCode, "claude-code"), (Tool::Codex, "codex")];

/// 4か所のうち、いま在る置き場所。
fn places(folder: &str, name: &str, dest: &dyn Destinations) -> Vec<Place> {
    let me = dest.canonical(folder);
    let mut roots: Vec<(Scope, &'static str, String)> =
        vec![(Scope::Repository, "repository", String::new())];
    if let Some(home) = dest.home() {
        roots.push((
            Scope::User,
            "user",
            format!("{}/", home.trim_end_matches('/')),
        ));
    }
    let mut out = Vec::new();
    for (_, scope, root) in &roots {
        for (tool, tool_name) in TOOLS {
            let path = format!("{root}{}/{name}", tool.dir());
            if !dest.occupied(&path) {
                continue;
            }
            let linked = dest.is_link(&path) && me.is_some() && dest.canonical(&path) == me;
            out.push(Place {
                path,
                tool: tool_name,
                scope,
                state: if linked { "linked" } else { "other" },
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
    dest: &dyn Destinations,
) -> Result<Withdrawn, Refused> {
    let usage = |m: &str| Refused {
        code: "usage",
        message: m.to_string(),
    };
    let folder =
        trimmed(folder.ok_or_else(|| usage("取り下げる Skill のフォルダを渡してください"))?);
    let reason = reason
        .filter(|r| !r.trim().is_empty())
        .ok_or_else(|| usage("--reason に、使わなくする理由を渡してください"))?;
    let name = folder.rsplit('/').next().unwrap_or(folder);
    let found = places(folder, name, dest);
    if !human {
        return Ok(Withdrawn {
            folder: folder.to_string(),
            reason: reason.to_string(),
            places: found,
            removed: Vec::new(),
            withdrawn: false,
            next: Some(format!(
                "作る人が影響（places）を見て承認したら、人が自分の端末で agents-skills-creator skills withdraw {folder} --reason \"{reason}\" を打つ（AI のセッションから打つと外さない）"
            )),
            after: Vec::new(),
        });
    }
    let mut removed = Vec::new();
    for p in found.iter().filter(|p| p.state == "linked") {
        dest.unlink(&p.path).map_err(|message| Refused {
            code: "not_found",
            message,
        })?;
        removed.push(p.path.clone());
    }
    let left: Vec<String> = places(folder, name, dest)
        .into_iter()
        .filter(|p| p.state == "linked")
        .map(|p| p.path)
        .collect();
    let after = if left.is_empty() {
        finding(
            "unlinked",
            Outcome::Verified,
            None,
            "どの置き場所にも、この Skill へのリンクが残っていない".to_string(),
        )
    } else {
        finding(
            "unlinked",
            Outcome::Violated,
            Some(left.join(" ・ ")),
            "この Skill へのリンクが残っている".to_string(),
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
