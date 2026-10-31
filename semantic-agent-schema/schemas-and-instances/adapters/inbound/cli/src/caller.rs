//! ポート IdentifyCaller を、引数と環境変数で実装する。
//! 親のセッション ID は --session → ツールごとの環境変数（callers.json の表）→ どちらも無ければ人、の順で決める。
//! 2つ以上のツールの環境変数に値があるとき（ツールの中から別のツールを起動した）は、どちらが打ったかを決めない。

use schemas_and_instances_operation_recording_application::ports::{Caller, IdentifyCaller};
use serde_json::Value;

/// ツールごとの環境変数の表（コードに書かず、表に置く）。
const CALLERS: &str = include_str!("../callers.json");

/// 表の行：ツールの名前と、親のセッション ID が入る環境変数の名前。
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
    caller: Caller,
}

impl FromArguments {
    /// --session の値（無ければ None）と、環境変数を引く関数から決める。
    /// --session が無く、2つ以上のツールの環境変数に値があれば、そのツールの名前を並べて返す。
    pub fn new(
        session: Option<String>,
        env: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, Vec<String>> {
        let found: Vec<(String, String)> = table()
            .into_iter()
            .filter_map(|(tool, name)| env(&name).filter(|v| !v.is_empty()).map(|v| (tool, v)))
            .collect();
        let caller = match (session, found.as_slice()) {
            // --session を渡したときも、どのツールの中かは環境変数で分かる（1つに決まるときだけ）
            (Some(id), [(tool, _)]) => Caller::Parent {
                id,
                tool: tool.clone(),
            },
            (Some(id), _) => Caller::Parent {
                id,
                tool: "unknown".to_string(),
            },
            (None, [(tool, id)]) => Caller::Parent {
                id: id.clone(),
                tool: tool.clone(),
            },
            (None, []) => Caller::Human,
            (None, many) => return Err(many.iter().map(|(tool, _)| tool.clone()).collect()),
        };
        Ok(Self { caller })
    }
}

impl IdentifyCaller for FromArguments {
    fn identify(&self) -> Caller {
        self.caller.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::FromArguments;
    use schemas_and_instances_operation_recording_application::ports::{Caller, IdentifyCaller};

    fn with(name: &'static str, value: &'static str) -> impl Fn(&str) -> Option<String> {
        move |n| (n == name).then(|| value.to_string())
    }

    #[test]
    fn session_flag_wins_over_the_environment() {
        let c = FromArguments::new(
            Some("ai-1".to_string()),
            with("CLAUDE_CODE_SESSION_ID", "c5c5"),
        )
        .unwrap_or_else(|_| unreachable!());
        assert_eq!(
            c.identify(),
            Caller::Parent {
                id: "ai-1".to_string(),
                tool: "claude-code".to_string()
            }
        );
    }

    #[test]
    fn environment_names_the_parent_when_no_flag() {
        let c = FromArguments::new(None, with("CLAUDE_CODE_SESSION_ID", "c5c5"))
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(
            c.identify(),
            Caller::Parent {
                id: "c5c5".to_string(),
                tool: "claude-code".to_string()
            }
        );
    }

    #[test]
    fn neither_flag_nor_environment_is_a_human() {
        assert_eq!(
            FromArguments::new(None, |_| None).map(|c| c.identify()),
            Ok(Caller::Human)
        );
        assert_eq!(
            FromArguments::new(Some("ci".to_string()), |_| None).map(|c| c.identify()),
            Ok(Caller::Parent {
                id: "ci".to_string(),
                tool: "unknown".to_string()
            })
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
            FromArguments::new(None, nested).map(|c| c.identify()),
            Err(vec!["claude-code".to_string(), "codex".to_string()])
        );
        assert_eq!(
            FromArguments::new(Some("0199".to_string()), nested).map(|c| c.identify()),
            Ok(Caller::Parent {
                id: "0199".to_string(),
                tool: "unknown".to_string()
            })
        );
    }

    #[test]
    fn each_listed_tool_names_its_parent() {
        for (name, tool) in [
            ("CODEX_SESSION_ID", "codex"),
            ("COPILOT_AGENT_SESSION_ID", "copilot"),
            ("KIRO_SESSION_ID", "kiro"),
        ] {
            assert_eq!(
                FromArguments::new(None, move |n| (n == name).then(|| "x".to_string()))
                    .map(|c| c.identify()),
                Ok(Caller::Parent {
                    id: "x".to_string(),
                    tool: tool.to_string()
                })
            );
        }
    }
}
