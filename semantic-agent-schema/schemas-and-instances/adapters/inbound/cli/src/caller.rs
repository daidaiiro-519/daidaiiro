//! 記録のモジュールのポート IdentifyCaller を、引数と環境変数で実装する（semantic-agent-schema init の登録は記録のモジュールの手続きが引いて渡す）。
//! 打ったセッションは --session → ツールごとの環境変数（callers.json の表）→ このフォルダで semantic-agent-schema init が最後に登録したセッション →
//! どれも無ければ人、の順で決める。
//! 2つ以上のツールの環境変数に値があるとき（ツールの中から別のツールを起動した）は、どれが打ったかを決めない。

use recording_application::ports::{IdentifyCaller, SessionRow};
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
    /// 値のあるツールの環境変数（ツールの名前と値）
    found: Vec<(String, String)>,
    /// 打ったフォルダ
    folder: String,
}

impl FromArguments {
    /// --session の値（無ければ None）と、環境変数を引く関数と、打ったフォルダから作る。
    pub fn new(
        session: Option<String>,
        env: impl Fn(&str) -> Option<String>,
        folder: &str,
    ) -> Self {
        let found = table()
            .into_iter()
            .filter_map(|(tool, name)| env(&name).filter(|v| !v.is_empty()).map(|v| (tool, v)))
            .collect();
        Self {
            session,
            found,
            folder: folder.to_string(),
        }
    }
}

impl IdentifyCaller for FromArguments {
    /// --session → 環境変数 → このフォルダで semantic-agent-schema init が最後に登録したセッション（initialized）→ 人。
    /// --session が無く、2つ以上のツールの環境変数に値があれば、そのツールの名前を並べて返す。
    fn identify(&self, initialized: Option<SessionRow>) -> Result<Caller, Vec<String>> {
        let at = |id: String, tool: String, via: Via| Caller::Session {
            id,
            tool,
            folder: self.folder.clone(),
            via,
        };
        Ok(match (self.session.clone(), self.found.as_slice()) {
            // --session を渡したときも、どのツールの中かは環境変数で分かる（1つに決まるときだけ）
            (Some(id), [(tool, _)]) => at(id, tool.clone(), Via::Flag),
            (Some(id), _) => at(id, "unknown".to_string(), Via::Flag),
            (None, [(tool, id)]) => at(id.clone(), tool.clone(), Via::Env),
            (None, []) => match initialized {
                Some(row) => at(row.id, row.tool, Via::Init),
                None => Caller::Human,
            },
            (None, many) => return Err(many.iter().map(|(tool, _)| tool.clone()).collect()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::FromArguments;
    use recording_application::ports::{IdentifyCaller, SessionRow};
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
            with("CLAUDE_CODE_SESSION_ID", "c5c5"),
            "/w",
        );
        assert_eq!(
            c.identify(registered()),
            Ok(session("ai-1", "claude-code", Via::Flag))
        );
    }

    #[test]
    fn environment_names_the_session_when_no_flag() {
        let c = FromArguments::new(None, with("CLAUDE_CODE_SESSION_ID", "c5c5"), "/w");
        assert_eq!(
            c.identify(registered()),
            Ok(session("c5c5", "claude-code", Via::Env))
        );
    }

    #[test]
    fn registered_session_is_used_when_neither_flag_nor_environment() {
        assert_eq!(
            FromArguments::new(None, |_| None, "/w").identify(registered()),
            Ok(session("sas-1", "unknown", Via::Init))
        );
    }

    #[test]
    fn nothing_at_all_is_a_human() {
        assert_eq!(
            FromArguments::new(None, |_| None, "/w").identify(None),
            Ok(Caller::Human)
        );
        assert_eq!(
            FromArguments::new(Some("ci".to_string()), |_| None, "/w").identify(None),
            Ok(session("ci", "unknown", Via::Flag))
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
            FromArguments::new(None, nested, "/w").identify(registered()),
            Err(vec!["claude-code".to_string(), "codex".to_string()])
        );
        assert_eq!(
            FromArguments::new(Some("0199".to_string()), nested, "/w").identify(None),
            Ok(session("0199", "unknown", Via::Flag))
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
                FromArguments::new(None, move |n| (n == name).then(|| "x".to_string()), "/w")
                    .identify(None),
                Ok(session("x", tool, Via::Env))
            );
        }
    }
}
