//! UC5 配布する（Agent の側）：agents distribute。配る前に確かめ（validate の検査と sas validate に、承認がいまの中身に
//! 当たるか approval-current と、置き場所が空いているか destination-free を足す）、検出が無ければ、ツールと範囲ごとの
//! 置き場所に、agent.json から write-agent-md と同じ形で作った定義を写して置き、置いたファイルの中身が同じか
//! （placed-readable）を確かめる。
//!
//! 置き場所：Claude Code は .claude/agents/<name>.md（AGENT.md の写し）、Codex は .codex/agents/<name>.toml（codex.toml の
//! 写し）。範囲が repository なら打った場所、user なら HOME の下。Skill はフォルダへのリンクで置くが、Agent はリンクに
//! しない：ツールがこれらの置き場所のリンクのファイルを読むことは原文で確かめていない（確かめたのは Skill のフォルダの
//! リンクだけ）。置き場所のファイルの印（どの Agent のフォルダから作ったか）がこの Agent を指すなら、中身が古くても
//! 上書きして配り直す。印が無いか、ほかの Agent を指すなら置かない（最低保証）。
//! 印は置く範囲で変える：repository はリポジトリの根からのフォルダ（agents/<name>。リポジトリを複製しても使える）、
//! user は Agent のフォルダの絶対の経路（HOME の下は、どのリポジトリから置いたかを見分ける）。

use crate::ports::{AgentFiles, AgentHistory, AgentValidator, Destinations, SkillsCli};
use crate::use_cases::validate::{AgentChecked, validate};
use crate::use_cases::{AGENT_JSON, Finding, Outcome, Refused, finding, read_agent, trimmed};
use agents_domain::services::agent_md;
use serde_json::Value;

/// 配布先のツール。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    /// Claude Code（subagent）
    ClaudeCode,
    /// Codex（custom agent）
    Codex,
}

impl Tool {
    /// 名前から。
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "claude-code" => Some(Self::ClaudeCode),
            "codex" => Some(Self::Codex),
            _ => None,
        }
    }

    /// 出力に書く名前。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
        }
    }

    /// 置き場所のフォルダ（範囲の根から）。
    #[must_use]
    pub const fn dir(self) -> &'static str {
        match self {
            Self::ClaudeCode => ".claude/agents",
            Self::Codex => ".codex/agents",
        }
    }

    /// 置き場所のファイル名の拡張子。
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::ClaudeCode => "md",
            Self::Codex => "toml",
        }
    }

    /// Agent のフォルダの中の、このツールの定義のファイル。
    #[must_use]
    pub const fn source(self) -> &'static str {
        match self {
            Self::ClaudeCode => agent_md::FILE,
            Self::Codex => agent_md::CODEX_FILE,
        }
    }

    /// agent.json から、このツールの定義を作る（write-agent-md と同じ。folder は印に入れる Agent のフォルダ）。
    #[must_use]
    pub fn render(self, agent: &Value, folder: &str) -> String {
        match self {
            Self::ClaudeCode => agent_md::render(agent, folder),
            Self::Codex => agent_md::render_codex(agent, folder),
        }
    }
}

/// 配布する範囲。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// そのリポジトリ（打った場所）
    Repository,
    /// 利用者の全体（HOME）
    User,
}

impl Scope {
    /// この範囲に置く写しの印（Agent のフォルダ）：repository はリポジトリの根から、user は絶対の経路。
    #[must_use]
    pub fn mark(self, folder: &str, places: &dyn Destinations) -> Option<String> {
        match self {
            Self::Repository => Some(folder.to_string()),
            Self::User => places.absolute(folder),
        }
    }

    /// 出力に書く名前。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Repository => "repository",
            Self::User => "user",
        }
    }

    /// 名前から。
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "repository" => Some(Self::Repository),
            "user" => Some(Self::User),
            _ => None,
        }
    }
}

/// 置いた結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    /// 置いた所
    pub path: String,
    /// 写した元（Agent のフォルダの定義のファイル）
    pub source: String,
    /// placed（置いた）・ replaced（この Agent の前の版の写しを上書きした）・ already（同じ中身がもうあった）
    pub result: &'static str,
}

/// 配布した結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Distributed {
    /// 配る前の確かめ
    pub before: AgentChecked,
    /// 置いた結果
    pub placed: Option<Placed>,
    /// 置いたあとの確かめ
    pub after: Vec<Finding>,
}

impl Distributed {
    /// 配布できなかったか。
    #[must_use]
    pub fn detected(&self) -> bool {
        self.placed.is_none() || self.after.iter().any(|f| f.outcome != Outcome::Verified)
    }
}

fn how_to_approve(folder: &str) -> String {
    format!(
        "作る人が agents validate の結果を承認したら、人が自分の端末で sas update {folder}/{AGENT_JSON} --patch '[{{\"op\":\"add\",\"path\":\"/approved\",\"value\":\"<承認した validate の操作 ID>\"}}]' --if-match <いまの中身のハッシュ> を打つ（AI のセッションから打つと、SAS が human_field で断る）"
    )
}

/// 承認（approved の validate の操作）のあとに、承認を書いた人の操作のほかに Agent を書き換えていないか（Skill と同じ見方）。
fn approval_current(files: &dyn AgentFiles, history: &dyn AgentHistory, folder: &str) -> Finding {
    const CHECK: &str = "approval-current";
    let agent = match read_agent(files, folder) {
        Ok(v) => v,
        Err(r) => return finding(CHECK, Outcome::Error, None, r.message),
    };
    let Some(approved) = agent.get("approved").and_then(Value::as_str) else {
        return finding(
            CHECK,
            Outcome::Violated,
            None,
            format!(
                "承認していない（approved が無い）。{}",
                how_to_approve(folder)
            ),
        );
    };
    let rows: Vec<Value> = match history.log(&format!("{folder}/{AGENT_JSON}")) {
        Ok(r) if r.exit == 0 => match r.output {
            Value::Array(rows) => rows,
            other @ Value::Object(_) => vec![other],
            _ => Vec::new(),
        },
        Ok(r) => {
            return finding(
                CHECK,
                Outcome::Error,
                None,
                format!("sas log が断った：{}", r.output),
            );
        }
        Err(e) => return finding(CHECK, Outcome::Error, None, e),
    };
    let s = |v: &Value, k: &str| {
        v.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    let Some(at) = rows
        .iter()
        .position(|r| s(r, "operation") == approved && s(r, "op") == "validate")
    else {
        return finding(
            CHECK,
            Outcome::Violated,
            Some(approved.to_string()),
            format!(
                "approved の {approved} が、この Agent の確かめ（validate の操作）の記録に無い。{}",
                how_to_approve(folder)
            ),
        );
    };
    for row in &rows[..at] {
        let op = s(row, "op");
        if !matches!(op.as_str(), "create" | "update" | "delete") {
            continue;
        }
        let id = s(row, "operation");
        let only_approval = op == "update"
            && row.get("session").is_none_or(Value::is_null)
            && history.operation(&id).ok().is_some_and(|d| {
                d.output
                    .pointer("/diff/changes")
                    .and_then(Value::as_array)
                    .is_some_and(|c| {
                        !c.is_empty()
                            && c.iter().all(|x| {
                                x.get("pointer").and_then(Value::as_str) == Some("/approved")
                            })
                    })
            });
        if !only_approval {
            return finding(
                CHECK,
                Outcome::Violated,
                Some(id.clone()),
                format!(
                    "承認（{approved}）のあとに {id}（{op}）で書き換えた。承認は古い。{}",
                    how_to_approve(folder)
                ),
            );
        }
    }
    finding(
        CHECK,
        Outcome::Verified,
        None,
        format!("承認（{approved}）のあとに、承認を書いたほかに書き換えていない"),
    )
}

/// Agent を配布する。
///
/// # Errors
/// フォルダ ・ ツール ・ 範囲が無い（usage）・ HOME が分からない（usage）・ sas を呼べない（sas_unavailable）・ 置けない（not_found）とき。
#[allow(clippy::too_many_arguments)]
pub fn distribute(
    folder: Option<&str>,
    tool: Option<Tool>,
    scope: Option<Scope>,
    sas: &dyn AgentValidator,
    history: &dyn AgentHistory,
    cli: &dyn SkillsCli,
    files: &dyn AgentFiles,
    places: &dyn Destinations,
) -> Result<Distributed, Refused> {
    let usage = |m: &str| Refused {
        code: "usage",
        message: m.to_string(),
    };
    let folder = trimmed(folder.ok_or_else(|| usage("配布する Agent のフォルダを渡してください"))?);
    let tool = tool.ok_or_else(|| usage("--tool に claude-code か codex を渡してください"))?;
    let scope = scope.ok_or_else(|| usage("--scope に repository か user を渡してください"))?;
    let name = folder.rsplit('/').next().unwrap_or(folder);
    let source = format!("{folder}/{}", tool.source());
    let file = format!("{}/{name}.{}", tool.dir(), tool.extension());
    let path = match scope {
        Scope::Repository => file,
        Scope::User => {
            let home = places.home().ok_or_else(|| {
                usage("HOME が分からないので、利用者の全体の置き場所を決められない")
            })?;
            format!("{}/{file}", home.trim_end_matches('/'))
        }
    };
    let mut before = validate(&[folder.to_string()], sas, history, cli, files)?
        .agents
        .remove(0);
    before
        .creator
        .push(approval_current(files, history, folder));
    // 写す中身は、agent.json から write-agent-md と同じ形で作る。印は置く範囲で変える
    // （repository はリポジトリの根からのフォルダ、user は絶対の経路）
    let mark = scope.mark(folder, places);
    let text = read_agent(files, folder)
        .ok()
        .zip(mark.as_deref())
        .map(|(a, m)| tool.render(&a, m));
    let current = files.read(&path).ok();
    // 置き場所のファイルの印が、この範囲でのこの Agent を指すなら、この Agent の写し（中身が古くても配り直してよい）
    let ours = places.is_file(&path)
        && mark.is_some()
        && current.as_deref().and_then(agent_md::read_mark) == mark;
    let same = ours && current.is_some() && current == text;
    before.creator.push(if !places.occupied(&path) {
        finding("destination-free", Outcome::Verified, None, format!("{path} は空いている"))
    } else if ours {
        finding("destination-free", Outcome::Verified, None, format!("{path} は、この Agent（{}）から作った写し", mark.as_deref().unwrap_or(folder)))
    } else {
        finding("destination-free", Outcome::Violated, Some(path.clone()), format!("{path} に、この Agent から作ったものでないもの（印が無いか、ほかの Agent を指す）がある。置かない"))
    });
    let Some(text) = text.filter(|_| !before.detected()) else {
        return Ok(Distributed {
            before,
            placed: None,
            after: Vec::new(),
        });
    };
    let result = if same {
        "already"
    } else {
        places.place(&path, &text).map_err(|message| Refused {
            code: "not_found",
            message,
        })?;
        if ours { "replaced" } else { "placed" }
    };
    let readable = match files.read(&path) {
        Ok(a) if a == text => finding(
            "placed-readable",
            Outcome::Verified,
            None,
            format!(
                "{path} の中身が、write-agent-md で作る {} と同じ",
                tool.source()
            ),
        ),
        Ok(_) => finding(
            "placed-readable",
            Outcome::Violated,
            Some(path.clone()),
            format!(
                "{path} の中身が、write-agent-md で作る {} と違う",
                tool.source()
            ),
        ),
        Err(e) => finding("placed-readable", Outcome::Violated, Some(path.clone()), e),
    };
    Ok(Distributed {
        before,
        placed: Some(Placed {
            path,
            source,
            result,
        }),
        after: vec![readable],
    })
}
