//! UC2 Agent を作るの write-agent-md：Agent の項目（agent.json）から、Claude Code の subagent（AGENT.md）と Codex の
//! custom agent（codex.toml）を決まった形で作り、同じフォルダに書く。前と同じ中身なら書かない。agent.json は変えない。

use crate::ports::{AgentFiles, GeneratedFiles};
use crate::use_cases::{Refused, read_agent, trimmed};
use agents_domain::services::agent_md;

/// 書いた結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    /// 渡されたフォルダ
    pub folder: String,
    /// 書いたファイル（AGENT.md ・ codex.toml）と、前と中身が違ったか
    pub files: Vec<(String, bool)>,
}

/// AGENT.md と codex.toml を書く。
///
/// # Errors
/// フォルダが無い（usage）・ agent.json を読めない（not_found ・ invalid_input）・ 書けない（not_found）とき。
pub fn write_agent_md(
    folder: Option<&str>,
    files: &dyn AgentFiles,
    out: &dyn GeneratedFiles,
) -> Result<Written, Refused> {
    let folder = folder.ok_or_else(|| Refused {
        code: "usage",
        message: "AGENT.md を書く Agent のフォルダを渡してください".to_string(),
    })?;
    let agent = read_agent(files, folder)?;
    let mut written = Vec::new();
    for (name, text) in [
        (agent_md::FILE, agent_md::render(&agent, trimmed(folder))),
        (
            agent_md::CODEX_FILE,
            agent_md::render_codex(&agent, trimmed(folder)),
        ),
    ] {
        let path = format!("{}/{name}", trimmed(folder));
        let changed = files.read(&path).ok().as_deref() != Some(text.as_str());
        if changed {
            out.write(&path, &text).map_err(|message| Refused {
                code: "not_found",
                message,
            })?;
        }
        written.push((path, changed));
    }
    Ok(Written {
        folder: folder.to_string(),
        files: written,
    })
}
