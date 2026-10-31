//! semantic-agent-schema-coding-agents：コーディングエージェント（Claude Code ・ Codex ・ GitHub Copilot ・ Gemini CLI ・ Kiro）のアダプター。
//! 基盤の呼び出し（call）を各コーディングエージェントのプロンプトと起動に直し、答えを基盤の出力の形で受け取る。delegates と agent_sessions を実装する
pub mod claude_code;
pub mod codex;
pub mod copilot;
pub mod gemini;
pub mod kiro;
pub mod prompt;
pub use semantic_agent_schema_application;

use semantic_agent_schema_application::ports::delegates::{
    Answer, DelegateError, Delegates, Provenance,
};
use semantic_agent_schema_application::semantic_agent_schema_declaration::aggregates::entities::delegate::Delegate;
use semantic_agent_schema_application::semantic_agent_schema_validation::values::call::Call;
use semantic_agent_schema_tools::process::{Launch, run};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

/// 起動するコーディングエージェント
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Agent {
    /// Claude Code（道具を全部外し、スキーマに沿った答えを受け取れることを原文で確かめた）
    ClaudeCode {
        /// 渡すモデル（無ければ Claude Code の既定）
        model: Option<String>,
    },
    /// 呼ぶ側が渡す argv（プロンプトを標準入力に渡し、出力の JSON をそのまま受け取る。試験の代役にも使う）
    Command(Vec<String>),
    /// 判定に使えないコーディングエージェント（名前）。道具を全部外して起動できることを原文で確かめられていない
    Unsupported(String),
}

impl Agent {
    /// 名前から選ぶ：claude ・ claude-code はClaude Code、ほか（codex ・ copilot ・ gemini ・ kiro）は判定に使えない
    pub fn by_name(name: &str, model: Option<String>) -> Self {
        match name {
            "claude" | "claude-code" => Agent::ClaudeCode { model },
            other => Agent::Unsupported(other.to_string()),
        }
    }
}

/// argv の無い任せる先を、コーディングエージェントで起動する
pub struct CodingAgents {
    agent: Agent,
    root: PathBuf,
    timeout: Duration,
    version: OnceLock<Option<String>>,
}

impl CodingAgents {
    /// 起動するコーディングエージェント ・ 作業の根 ・ 時間切れを渡して作る
    pub fn new(agent: Agent, root: PathBuf, timeout: Duration) -> Self {
        CodingAgents {
            agent,
            root,
            timeout,
            version: OnceLock::new(),
        }
    }
}

impl Delegates for CodingAgents {
    fn call(
        &self,
        delegate: &Delegate,
        call: &Call,
        output_schema: &Value,
    ) -> Result<Answer, DelegateError> {
        if delegate.has_argv() {
            return Err(DelegateError::Unsupported(format!(
                "任せる先 {} は argv を持つので、外のツールとして起動する",
                delegate.name()
            )));
        }
        if delegate.kind() != "judging" {
            return Err(DelegateError::Unsupported(format!(
                "任せる先 {} は {}。working の起動はセッションの扱い（ステップ9）と一緒に入れる",
                delegate.name(),
                delegate.kind()
            )));
        }
        let prompt = prompt::judging(delegate.role(), &call.to_json(), output_schema);
        match &self.agent {
            Agent::ClaudeCode { model } => {
                let (output, used) = claude_code::judge(
                    &prompt,
                    output_schema,
                    model.as_deref(),
                    &self.root,
                    self.timeout,
                )?;
                let version = self
                    .version
                    .get_or_init(|| claude_code::version(&self.root))
                    .clone();
                Ok(Answer {
                    output,
                    provenance: Provenance::Agent {
                        agent: "claude-code".to_string(),
                        version,
                        model: used,
                        prompt,
                    },
                })
            }
            Agent::Command(argv) => {
                let out = run(&Launch {
                    argv,
                    input: prompt.as_bytes(),
                    root: &self.root,
                    timeout: self.timeout,
                    env: &[],
                    ok_codes: &[0],
                })
                .map_err(DelegateError::Failed)?;
                let output: Value = serde_json::from_slice(&out)
                    .map_err(|e| DelegateError::Failed(format!("答えが JSON でない：{e}")))?;
                Ok(Answer {
                    output,
                    provenance: Provenance::Agent {
                        agent: argv.first().cloned().unwrap_or_default(),
                        version: None,
                        model: None,
                        prompt,
                    },
                })
            }
            Agent::Unsupported(name) => Err(DelegateError::Unsupported(format!(
                "{name} は判定に使えない（道具を全部外して起動できることを原文で確かめられていない）"
            ))),
        }
    }
}
