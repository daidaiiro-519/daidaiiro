//! 記録のモジュールのポート IdentifyCaller を、引数と環境変数で実装する（semantic-agent-schema init の登録は記録のモジュールの手続きが引いて渡す）。
//! 人は --human で明示する。ツールの環境変数があるところでは人を名乗れない（AI のセッションの中で人の欄を書かせない）。打ったセッションは --session → ツールごとの環境変数（callers.json の表）→ このフォルダで
//! semantic-agent-schema init が最後に登録したセッション、の順で決め、どれも無ければ決めない（人とみなさない）。
//! 2つ以上のツールの環境変数に値があるとき（ツールの中から別のツールを起動した）は、どれが打ったかを決めない。

use recording_application::ports::{IdentifyCaller, NotIdentified, SessionRow};
use serde_json::Value;
use writing_domain::values::{Caller, Via};

/// ツールごとの環境変数の表（コードに書かず、表に置く）。
const CALLERS: &str = include_str!("../callers.json");

/// 表の行：ツールの名前と、セッション ID が入る環境変数の名前。
fn table() -> Vec<(String, String)> {
    serde_json::from_str::<Value>(CALLERS)
        .ok()
        .and_then(|v| v.get("tools").and_then(Value::as_array).cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|t| {
            Some((
                t.get("tool")?.as_str()?.to_string(),
                t.get("session_env")?.as_str()?.to_string(),
            ))
        })
        .collect()
}

/// 引数の --session と環境変数から決める呼び出し元。
pub struct FromArguments {
    /// --session の値
    session: Option<String>,
    /// --human を渡したか
    human: bool,
    /// 値のあるツールの環境変数（ツールの名前と値）
    found: Vec<(String, String)>,
    /// 打ったフォルダ
    folder: String,
}

impl FromArguments {
    /// --session の値（無ければ None）と、--human を渡したかと、環境変数を引く関数と、打ったフォルダから作る。
    pub fn new(
        session: Option<String>,
        human: bool,
        env: impl Fn(&str) -> Option<String>,
        folder: &str,
    ) -> Self {
        let found = table()
            .into_iter()
            .filter_map(|(tool, name)| env(&name).filter(|v| !v.is_empty()).map(|v| (tool, v)))
            .collect();
        Self {
            session,
            human,
            found,
            folder: folder.to_string(),
        }
    }
}

impl IdentifyCaller for FromArguments {
    /// --human なら人（ツールの環境変数に値があれば、人と受け取らずにそのツールの名前を返す）。ほかは --session → 環境変数 →
    /// このフォルダで semantic-agent-schema init が最後に登録したセッション（initialized）で、どれも無ければ None。
    /// --session が無く、2つ以上のツールの環境変数に値があれば、そのツールの名前を並べて返す。
    fn identify(&self, initialized: Option<SessionRow>) -> Result<Option<Caller>, NotIdentified> {
        if self.human {
            if !self.found.is_empty() {
                return Err(NotIdentified::HumanInTool(
                    self.found.iter().map(|(tool, _)| tool.clone()).collect(),
                ));
            }
            return Ok(Some(Caller::Human));
        }
        let at = |id: String, tool: String, via: Via| Caller::Session {
            id,
            tool,
            folder: self.folder.clone(),
            via,
        };
        Ok(match (self.session.clone(), self.found.as_slice()) {
            // --session を渡したときも、どのツールの中かは環境変数で分かる（1つに決まるときだけ）
            (Some(id), [(tool, _)]) => Some(at(id, tool.clone(), Via::Flag)),
            (Some(id), _) => Some(at(id, "unknown".to_string(), Via::Flag)),
            (None, [(tool, id)]) => Some(at(id.clone(), tool.clone(), Via::Env)),
            (None, []) => initialized.map(|row| at(row.id, row.tool, Via::Init)),
            (None, many) => {
                return Err(NotIdentified::Ambiguous(
                    many.iter().map(|(tool, _)| tool.clone()).collect(),
                ));
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::FromArguments;
    use recording_application::ports::{IdentifyCaller, NotIdentified, SessionRow};
    use writing_domain::values::{Caller, Via};

    fn with(name: &'static str, value: &'static str) -> impl Fn(&str) -> Option<String> {
        move |n| (n == name).then(|| value.to_string())
    }

    fn session(id: &str, tool: &str, via: Via) -> Caller {
        Caller::Session {
            id: id.to_string(),
            tool: tool.to_string(),
            folder: "/w".to_string(),
            via,
        }
    }

    fn registered() -> Option<SessionRow> {
        Some(SessionRow {
            id: "sas-1".to_string(),
            tool: "unknown".to_string(),
            folder: "/w".to_string(),
            via: "init".to_string(),
        })
    }

    #[test]
    fn session_flag_wins_over_the_environment() {
        let c = FromArguments::new(
            Some("ai-1".to_string()),
            false,
            with("CLAUDE_CODE_SESSION_ID", "c5c5"),
            "/w",
        );
        assert_eq!(
            c.identify(registered()),
            Ok(Some(session("ai-1", "claude-code", Via::Flag)))
        );
    }

    #[test]
    fn environment_names_the_session_when_no_flag() {
        let c = FromArguments::new(None, false, with("CLAUDE_CODE_SESSION_ID", "c5c5"), "/w");
        assert_eq!(
            c.identify(registered()),
            Ok(Some(session("c5c5", "claude-code", Via::Env)))
        );
    }

    #[test]
    fn registered_session_is_used_when_neither_flag_nor_environment() {
        assert_eq!(
            FromArguments::new(None, false, |_| None, "/w").identify(registered()),
            Ok(Some(session("sas-1", "unknown", Via::Init)))
        );
    }

    #[test]
    fn nothing_at_all_is_not_taken_as_a_human() {
        assert_eq!(
            FromArguments::new(None, false, |_| None, "/w").identify(None),
            Ok(None)
        );
        assert_eq!(
            FromArguments::new(None, true, |_| None, "/w").identify(registered()),
            Ok(Some(Caller::Human)),
            "--human は明示した人"
        );
        assert_eq!(
            FromArguments::new(None, true, with("CLAUDE_CODE_SESSION_ID", "c5c5"), "/w")
                .identify(registered()),
            Err(NotIdentified::HumanInTool(vec!["claude-code".to_string()])),
            "ツールの中では人を名乗れない"
        );
        assert_eq!(
            FromArguments::new(Some("ci".to_string()), false, |_| None, "/w").identify(None),
            Ok(Some(session("ci", "unknown", Via::Flag)))
        );
    }

    fn nested(n: &str) -> Option<String> {
        match n {
            "CLAUDE_CODE_SESSION_ID" => Some("c5c5".to_string()),
            "CODEX_SESSION_ID" => Some("0199".to_string()),
            _ => None,
        }
    }

    #[test]
    fn two_tools_in_the_environment_are_not_guessed() {
        assert_eq!(
            FromArguments::new(None, false, nested, "/w").identify(registered()),
            Err(NotIdentified::Ambiguous(vec![
                "claude-code".to_string(),
                "codex".to_string()
            ]))
        );
        assert_eq!(
            FromArguments::new(Some("0199".to_string()), false, nested, "/w").identify(None),
            Ok(Some(session("0199", "unknown", Via::Flag)))
        );
    }

    #[test]
    fn each_listed_tool_names_its_session() {
        for (name, tool) in [
            ("CODEX_SESSION_ID", "codex"),
            ("COPILOT_AGENT_SESSION_ID", "copilot"),
            ("KIRO_SESSION_ID", "kiro"),
        ] {
            assert_eq!(
                FromArguments::new(
                    None,
                    false,
                    move |n| (n == name).then(|| "x".to_string()),
                    "/w"
                )
                .identify(None),
                Ok(Some(session("x", tool, Via::Env)))
            );
        }
    }
}
