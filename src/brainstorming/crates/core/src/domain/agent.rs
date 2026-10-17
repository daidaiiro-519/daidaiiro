//! AI のコーディング用 CLI（Claude Code ・ Codex ・ Kiro）の呼び方と、出来事を共通の形に直すこと。
//! 権限を外す旗は使わない。Claude Code は許すツールを挙げ、聞くはずの呼び出しを拒否させる。
//! 何を許すかはコードに持たず、許可の設定（brainstorming/agents@v1）から受け取る。書き換えの範囲は serve の検査でも確かめる。

use serde_json::Value;

/// AI の CLI。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Agent {
    Claude,
    Codex,
    Kiro,
}

impl Agent {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "claude" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            "kiro" => Some(Self::Kiro),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Kiro => "kiro",
        }
    }
    /// 起動する実行ファイルの名前。
    pub fn program(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Kiro => "kiro-cli",
        }
    }
    pub const ALL: [Agent; 3] = [Agent::Claude, Agent::Codex, Agent::Kiro];
}

/// 起動のしかた。prompt は、stdin が Some ならそこへ、None なら引数に入っている。
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub program: String,
    pub args: Vec<String>,
    pub stdin: Option<String>,
    /// 子のプロセスに足す環境変数。ロックの持ち主を、AI が呼ぶ brainstorming へ伝えるのに使う
    pub env: Vec<(String, String)>,
}

/// AI が呼ぶ brainstorming に、どの依頼の中で動いているかを伝える環境変数の名前。
pub const RUN_VAR: &str = "BRAINSTORMING_RUN";

/// 何についての依頼か。
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    /// リポジトリの根からの、Skill の SKILL.md
    pub skill_md: String,
    /// リポジトリの根からの、Skill の実行ファイル
    pub entry: String,
    /// リポジトリの根からの、ボードの本体
    pub board_file: String,
    /// リポジトリの根からの、ボードのフォルダ
    pub board_dir: String,
    /// 作業するフォルダ（リポジトリの根）
    pub root: String,
    /// ボードを直してよいか。読むだけの AI には false を渡す
    pub writable: bool,
    pub round: u64,
    pub topic: String,
    pub text: String,
    /// この依頼の ID。ロックの持ち主になる。空なら環境変数を渡さない
    pub run: String,
    /// 許可の設定（brainstorming/agents@v1）から読んだ、AI に許すこと
    pub allowed: Allowed,
}

/// AI に許すこと。値の決まり（ボードを直すツールを必ず許す ・ sandbox を外さない ・ Kiro は読むだけ）は、
/// スキーマ brainstorming/agents@v1 が持ち、読む前に基盤の check で確かめる。ここは形を読むだけにする。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Allowed {
    /// Claude Code の --allowedTools。${entry} を含む
    pub claude: Vec<String>,
    /// Codex の sandbox
    pub codex_sandbox: String,
    /// Kiro の --trust-tools
    pub kiro_trust: Vec<String>,
    /// 新しいセッションの最初の依頼に足す文。${entry} ・ ${board_dir} を含む
    pub guide: Vec<String>,
}

/// 許可の設定のインスタンスを読む。
pub fn allowed(value: &Value) -> Result<Allowed, String> {
    let strings = |pointer: &str| -> Result<Vec<String>, String> {
        match value.pointer(pointer) {
            None => Ok(vec![]),
            Some(Value::Array(items)) => items
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| format!("許可の設定の {pointer} に文字列でない値がある"))
                })
                .collect(),
            Some(_) => Err(format!("許可の設定の {pointer} が配列でない")),
        }
    };
    Ok(Allowed {
        claude: strings("/claude/allow")?,
        codex_sandbox: value
            .pointer("/codex/sandbox")
            .and_then(Value::as_str)
            .ok_or("許可の設定に /codex/sandbox が無い")?
            .to_owned(),
        kiro_trust: strings("/kiro/trust")?,
        guide: strings("/guide")?,
    })
}

/// 設定の値の ${entry} ・ ${board_dir} を、この依頼の値に置き換える。
fn fill(text: &str, request: &Request) -> String {
    text.replace("${entry}", &request.entry)
        .replace("${board_dir}", &request.board_dir)
}

/// 依頼の文。新しいセッションの最初の依頼には、Skill の使い方を指す1文を必ず入れる。
pub fn prompt(request: &Request, new_session: bool) -> String {
    let mut out = String::new();
    if new_session && request.writable {
        out.push_str(&format!(
            "{} を読み、その手順で {} を使って作業する。ボードの JSON は {} の update で直し、ファイルを直接書き換えない。シェルで実行してよいのは {} で始まるコマンドだけで、ほかのファイルは読むツールで読む。\n",
            request.skill_md, request.entry, request.entry, request.entry
        ));
    }
    if new_session && request.writable {
        for line in &request.allowed.guide {
            out.push_str(&fill(line, request));
            out.push('\n');
        }
    }
    if new_session && !request.writable {
        out.push_str(&format!(
            "{} を読んで手順を知る。この AI は読むだけで、ボードもほかのファイルも書き換えない。直す案は文で示す。\n",
            request.skill_md
        ));
    }
    out.push_str(&format!(
        "作業するフォルダは {}。対象は {}（回の記録 ・ 回答は {}）、回は {}、論点は {}。\n\n{}",
        request.root,
        request.board_file,
        request.board_dir,
        request.round,
        if request.topic.is_empty() {
            "指定なし"
        } else {
            &request.topic
        },
        request.text
    ));
    out
}

/// 起動のしかたを決める。session は続きに使う ID（無ければ新しいセッション）、fresh は Claude Code に渡す新しい ID。
pub fn plan(
    agent: Agent,
    session: Option<&str>,
    fresh: &str,
    root: &str,
    request: &Request,
) -> Plan {
    let text = prompt(request, session.is_none());
    let env: Vec<(String, String)> = if request.run.is_empty() {
        vec![]
    } else {
        vec![(RUN_VAR.to_owned(), request.run.clone())]
    };
    let own = |args: &[&str]| {
        args.iter()
            .map(|a| (*a).to_owned())
            .collect::<Vec<String>>()
    };
    match agent {
        Agent::Claude => {
            let mut args = own(&["-p", "--output-format", "stream-json", "--verbose"]);
            match session {
                Some(id) => args.extend(own(&["--resume", id])),
                None => args.extend(own(&["--session-id", fresh])),
            }
            args.extend(own(&["--permission-mode", "dontAsk", "--allowedTools"]));
            args.extend(
                request
                    .allowed
                    .claude
                    .iter()
                    .map(|tool| fill(tool, request)),
            );
            Plan {
                program: agent.program().to_owned(),
                args,
                stdin: Some(text),
                env: env.clone(),
            }
        }
        Agent::Codex => {
            let args = match session {
                Some(id) => own(&[
                    "exec",
                    "resume",
                    "--json",
                    "-c",
                    &format!("sandbox_mode=\"{}\"", request.allowed.codex_sandbox),
                    id,
                    "-",
                ]),
                None => own(&[
                    "exec",
                    "--json",
                    "--sandbox",
                    &request.allowed.codex_sandbox,
                    "-C",
                    root,
                    "-",
                ]),
            };
            Plan {
                program: agent.program().to_owned(),
                args,
                stdin: Some(text),
                env: env.clone(),
            }
        }
        Agent::Kiro => {
            let mut args = own(&[
                "chat",
                "--agent-engine",
                "v2",
                "--output-format",
                "stream-json",
            ]);
            args.push(format!(
                "--trust-tools={}",
                request.allowed.kiro_trust.join(",")
            ));
            if let Some(id) = session {
                args.extend(own(&["--resume-id", id]));
            }
            args.push(text);
            Plan {
                program: agent.program().to_owned(),
                args,
                stdin: None,
                env,
            }
        }
    }
}

/// 共通の形の出来事。
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// AI の文
    Text(String),
    /// 作業の記録（read ・ edit ・ run）と、その対象
    Work(&'static str, String),
    /// 続きに使うセッションの ID
    Session(String),
    /// 権限が無くて断られたツール
    Denied(String),
    /// 終わり。ok は成功したか、text は失敗の説明
    Done(bool, String),
    /// 使ったトークン（入力の合計 ・ うちキャッシュから読んだ分 ・ 出力）
    Usage(u64, u64, u64),
}

fn n(value: &Value, pointer: &str) -> u64 {
    value.pointer(pointer).and_then(Value::as_u64).unwrap_or(0)
}

fn s<'v>(value: &'v Value, pointer: &str) -> &'v str {
    value.pointer(pointer).and_then(Value::as_str).unwrap_or("")
}

/// 作業の対象を、リポジトリの根からの場所にして短くする。
fn short(target: &str, root: &str) -> String {
    let root = root.trim_end_matches('/');
    target
        .strip_prefix(root)
        .map(|rest| rest.trim_start_matches('/'))
        .unwrap_or(target)
        .to_owned()
}

/// `/bin/bash -lc '…'` や `/bin/bash -lc "…"` で包まれたコマンドから中身を出す。
fn unwrap_shell(command: &str) -> String {
    ["/bin/bash -lc ", "bash -lc ", "/bin/sh -c "]
        .iter()
        .find_map(|head| command.strip_prefix(head))
        .and_then(|rest| {
            let quote = rest.chars().next().filter(|c| *c == '\'' || *c == '"')?;
            rest[1..].strip_suffix(quote)
        })
        .unwrap_or(command)
        .to_owned()
}

/// 1行の JSON を、その CLI の形から共通の形に直す。読めない行は何も返さない。
pub fn events(agent: Agent, line: &str, root: &str) -> Vec<Event> {
    let Ok(value) = serde_json::from_str::<Value>(line) else {
        return vec![];
    };
    match agent {
        Agent::Claude => claude(&value, root),
        Agent::Codex => codex(&value, root),
        Agent::Kiro => kiro(&value, root),
    }
}

fn claude(value: &Value, root: &str) -> Vec<Event> {
    match (s(value, "/type"), s(value, "/subtype")) {
        ("system", "init") => vec![Event::Session(s(value, "/session_id").to_owned())],
        ("system", "permission_denied") => vec![Event::Denied(s(value, "/tool_name").to_owned())],
        ("assistant", _) => value
            .pointer("/message/content")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|item| match s(item, "/type") {
                "text" => Some(Event::Text(s(item, "/text").to_owned())),
                "tool_use" => {
                    let input = item.get("input").cloned().unwrap_or(Value::Null);
                    let (act, target) = match s(item, "/name") {
                        "Read" => ("read", short(s(&input, "/file_path"), root)),
                        "Grep" | "Glob" => ("read", s(&input, "/pattern").to_owned()),
                        "Bash" => ("run", short(s(&input, "/command"), root)),
                        "Write" | "Edit" | "MultiEdit" | "NotebookEdit" => {
                            ("edit", short(s(&input, "/file_path"), root))
                        }
                        other => ("run", other.to_owned()),
                    };
                    Some(Event::Work(act, target))
                }
                _ => None,
            })
            .collect(),
        ("result", _) => vec![
            Event::Usage(
                n(value, "/usage/input_tokens")
                    + n(value, "/usage/cache_creation_input_tokens")
                    + n(value, "/usage/cache_read_input_tokens"),
                n(value, "/usage/cache_read_input_tokens"),
                n(value, "/usage/output_tokens"),
            ),
            Event::Done(
                !value
                    .get("is_error")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                if value.get("is_error").and_then(Value::as_bool) == Some(true) {
                    s(value, "/result").to_owned()
                } else {
                    String::new()
                },
            ),
        ],
        _ => vec![],
    }
}

fn codex(value: &Value, root: &str) -> Vec<Event> {
    match s(value, "/type") {
        "thread.started" => vec![Event::Session(s(value, "/thread_id").to_owned())],
        "item.started" if s(value, "/item/type") == "command_execution" => {
            vec![Event::Work(
                "run",
                short(&unwrap_shell(s(value, "/item/command")), root),
            )]
        }
        "item.completed" => match s(value, "/item/type") {
            "agent_message" => vec![Event::Text(format!(
                "{}\n",
                s(value, "/item/text").trim_end()
            ))],
            "file_change" => value
                .pointer("/item/changes")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|change| Event::Work("edit", short(s(change, "/path"), root)))
                .collect(),
            _ => vec![],
        },
        "turn.completed" => vec![
            Event::Usage(
                n(value, "/usage/input_tokens"),
                n(value, "/usage/cached_input_tokens"),
                n(value, "/usage/output_tokens"),
            ),
            Event::Done(true, String::new()),
        ],
        "turn.failed" => vec![Event::Done(false, s(value, "/error/message").to_owned())],
        "error" => vec![Event::Done(false, s(value, "/message").to_owned())],
        _ => vec![],
    }
}

fn kiro(value: &Value, root: &str) -> Vec<Event> {
    match s(value, "/type") {
        "metadata" => vec![Event::Session(s(value, "/data/sessionId").to_owned())],
        "sessionUpdate" => {
            let update = value
                .pointer("/data/update")
                .cloned()
                .unwrap_or(Value::Null);
            match s(&update, "/sessionUpdate") {
                "agent_message_chunk" => vec![Event::Text(s(&update, "/content/text").to_owned())],
                "tool_call" => {
                    let path = s(&update, "/locations/0/path");
                    let (act, target) = match s(&update, "/kind") {
                        "read" => (
                            "read",
                            short(
                                if path.is_empty() {
                                    s(&update, "/title")
                                } else {
                                    path
                                },
                                root,
                            ),
                        ),
                        "edit" => (
                            "edit",
                            short(
                                if path.is_empty() {
                                    s(&update, "/title")
                                } else {
                                    path
                                },
                                root,
                            ),
                        ),
                        _ => {
                            let command = s(&update, "/rawInput/command");
                            (
                                "run",
                                short(
                                    if command.is_empty() {
                                        s(&update, "/title")
                                    } else {
                                        command
                                    },
                                    root,
                                ),
                            )
                        }
                    };
                    vec![Event::Work(act, target)]
                }
                _ => vec![],
            }
        }
        "runFinished" => vec![Event::Done(
            s(value, "/data/status") == "success",
            String::new(),
        )],
        "runError" => vec![Event::Done(false, s(value, "/data/message").to_owned())],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> Request {
        Request {
            skill_md: ".claude/skills/brainstorming/SKILL.md".to_owned(),
            entry: ".claude/skills/brainstorming/scripts/bin/brainstorming".to_owned(),
            board_file: ".brainstorming/b.json".to_owned(),
            board_dir: ".brainstorming/b/".to_owned(),
            root: "/r".to_owned(),
            writable: true,
            round: 2,
            topic: "Q1".to_owned(),
            text: "前提を足して".to_owned(),
            run: "R1".to_owned(),
            allowed: allowed(&serde_json::json!({
                "claude": {"allow": ["Read", "Grep", "Glob", "Bash(${entry}:*)"]},
                "codex": {"sandbox": "workspace-write"},
                "kiro": {"trust": ["fs_read"]}
            }))
            .unwrap(),
        }
    }

    #[test]
    fn each_cli_gets_exactly_what_the_settings_allow() {
        let wide = Request {
            allowed: allowed(&serde_json::json!({
                "claude": {"allow": ["Read", "WebSearch", "Bash(${entry}:*)", "Bash(.claude/skills/fact-check/bin/fact-check:*)"]},
                "codex": {"sandbox": "read-only"},
                "kiro": {"trust": ["fs_read"]},
                "guide": ["原文は ${board_dir}sources/ に置き、${entry} で直す。"]
            }))
            .unwrap(),
            ..request()
        };
        let claude = plan(Agent::Claude, None, "U", "/r", &wide).args;
        let tools = &claude[claude.iter().position(|a| a == "--allowedTools").unwrap() + 1..];
        assert_eq!(
            tools,
            [
                "Read",
                "WebSearch",
                "Bash(.claude/skills/brainstorming/scripts/bin/brainstorming:*)",
                "Bash(.claude/skills/fact-check/bin/fact-check:*)"
            ]
        );
        assert!(plan(Agent::Codex, None, "U", "/r", &wide)
            .args
            .contains(&"read-only".to_owned()));
        assert!(plan(Agent::Codex, Some("S"), "U", "/r", &wide)
            .args
            .contains(&"sandbox_mode=\"read-only\"".to_owned()));
        assert!(prompt(&wide, true).contains(
            "原文は .brainstorming/b/sources/ に置き、.claude/skills/brainstorming/scripts/bin/brainstorming で直す。"
        ));
        assert!(!prompt(&wide, false).contains("原文は"));
    }

    #[test]
    fn settings_of_the_wrong_shape_are_refused() {
        assert!(allowed(
            &serde_json::json!({"claude": {"allow": "Read"}, "codex": {"sandbox": "read-only"}})
        )
        .is_err());
        assert!(allowed(&serde_json::json!({"claude": {"allow": []}})).is_err());
    }

    #[test]
    fn the_run_id_is_passed_to_the_ai_as_an_environment_variable() {
        let made = plan(Agent::Claude, None, "U", "/r", &request());
        assert_eq!(made.env, vec![(RUN_VAR.to_owned(), "R1".to_owned())]);
        let none = Request {
            run: String::new(),
            ..request()
        };
        assert!(plan(Agent::Codex, None, "U", "/r", &none).env.is_empty());
    }

    #[test]
    fn only_a_new_session_is_told_where_the_skill_is() {
        assert!(
            prompt(&request(), true).starts_with(".claude/skills/brainstorming/SKILL.md を読み")
        );
        assert!(prompt(&request(), false)
            .starts_with("作業するフォルダは /r。対象は .brainstorming/b.json"));
        assert!(prompt(&request(), false).ends_with("前提を足して"));
    }

    #[test]
    fn plans_use_no_flag_that_drops_permissions() {
        let plans: Vec<Plan> = Agent::ALL
            .iter()
            .flat_map(|agent| {
                [
                    plan(*agent, None, "U", "/r", &request()),
                    plan(*agent, Some("S"), "U", "/r", &request()),
                ]
            })
            .collect();
        let joined: String = plans
            .iter()
            .map(|p| p.args.join(" "))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!joined.contains("dangerously"));
        assert!(!joined.contains("bypass"));
        assert!(!joined.contains("trust-all-tools"));
        assert!(!joined.contains("yolo"));
    }

    #[test]
    fn each_cli_starts_new_and_resumes_by_id() {
        let claude_new = plan(Agent::Claude, None, "U", "/r", &request());
        let claude_again = plan(Agent::Claude, Some("S"), "U", "/r", &request());
        let codex_new = plan(Agent::Codex, None, "U", "/r", &request());
        let codex_again = plan(Agent::Codex, Some("S"), "U", "/r", &request());
        let kiro_again = plan(Agent::Kiro, Some("S"), "U", "/r", &request());
        assert_eq!(
            claude_new.args[4..6],
            ["--session-id".to_owned(), "U".to_owned()]
        );
        assert_eq!(
            claude_again.args[4..6],
            ["--resume".to_owned(), "S".to_owned()]
        );
        assert_eq!(
            claude_new.args.last().map(String::as_str),
            Some("Bash(.claude/skills/brainstorming/scripts/bin/brainstorming:*)")
        );
        assert_eq!(
            codex_new.args,
            [
                "exec",
                "--json",
                "--sandbox",
                "workspace-write",
                "-C",
                "/r",
                "-"
            ]
        );
        assert_eq!(
            codex_again.args[..3],
            ["exec".to_owned(), "resume".to_owned(), "--json".to_owned()]
        );
        assert_eq!(codex_again.args[5..], ["S".to_owned(), "-".to_owned()]);
        assert_eq!(kiro_again.stdin, None);
        assert_eq!(
            kiro_again.args[6..8],
            ["--resume-id".to_owned(), "S".to_owned()]
        );
    }

    /// 2026-10-17 に手元で実測した行（ボード brainstorming-on-base の measured/probe/）。
    #[test]
    fn measured_lines_become_common_events() {
        let cases: [(Agent, &str, Vec<Event>); 12] = [
            (
                Agent::Claude,
                r#"{"type":"system","subtype":"init","session_id":"436d"}"#,
                vec![Event::Session("436d".to_owned())],
            ),
            (
                Agent::Claude,
                r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Read","input":{"file_path":"/r/.agents/x.md"}}]}}"#,
                vec![Event::Work("read", ".agents/x.md".to_owned())],
            ),
            (
                Agent::Claude,
                r#"{"type":"assistant","message":{"content":[{"type":"text","text":"AMBERFOX"}]}}"#,
                vec![Event::Text("AMBERFOX".to_owned())],
            ),
            (
                Agent::Claude,
                r#"{"type":"system","subtype":"permission_denied","tool_name":"Write"}"#,
                vec![Event::Denied("Write".to_owned())],
            ),
            (
                Agent::Claude,
                r#"{"type":"result","subtype":"success","is_error":false,"result":"x","usage":{"input_tokens":28,"cache_creation_input_tokens":50380,"cache_read_input_tokens":694918,"output_tokens":9780}}"#,
                vec![
                    Event::Usage(745326, 694918, 9780),
                    Event::Done(true, String::new()),
                ],
            ),
            (
                Agent::Codex,
                r#"{"type":"thread.started","thread_id":"01a1"}"#,
                vec![Event::Session("01a1".to_owned())],
            ),
            (
                Agent::Codex,
                r#"{"type":"item.started","item":{"id":"item_1","type":"command_execution","command":"/bin/bash -lc 'ls -a'","status":"in_progress"}}"#,
                vec![Event::Work("run", "ls -a".to_owned())],
            ),
            (
                Agent::Codex,
                r#"{"type":"item.completed","item":{"id":"item_3","type":"agent_message","text":"DONE"}}"#,
                vec![Event::Text("DONE\n".to_owned())],
            ),
            (
                Agent::Kiro,
                r#"{"type":"metadata","data":{"sessionId":"e799"}}"#,
                vec![Event::Session("e799".to_owned())],
            ),
            (
                Agent::Kiro,
                r#"{"type":"sessionUpdate","data":{"update":{"sessionUpdate":"tool_call","kind":"execute","title":"Running: ls -a","rawInput":{"command":"ls -a"}}}}"#,
                vec![Event::Work("run", "ls -a".to_owned())],
            ),
            (
                Agent::Kiro,
                r#"{"type":"sessionUpdate","data":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"OK"}}}}"#,
                vec![Event::Text("OK".to_owned())],
            ),
            (
                Agent::Kiro,
                r#"{"type":"runError","data":{"sessionId":null,"stage":"engine","message":"v1 では使えない"}}"#,
                vec![Event::Done(false, "v1 では使えない".to_owned())],
            ),
        ];
        let got: Vec<Vec<Event>> = cases.iter().map(|(a, l, _)| events(*a, l, "/r")).collect();
        let want: Vec<Vec<Event>> = cases.iter().map(|(_, _, w)| w.clone()).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn kiro_only_reads_and_is_told_so() {
        let read_only = Request {
            writable: false,
            ..request()
        };
        assert!(plan(Agent::Kiro, None, "U", "/r", &read_only)
            .args
            .contains(&"--trust-tools=fs_read".to_owned()));
        assert!(!plan(Agent::Kiro, None, "U", "/r", &read_only)
            .args
            .join(" ")
            .contains("execute_bash"));
        assert!(prompt(&read_only, true).contains("この AI は読むだけ"));
    }

    #[test]
    fn shell_wrappers_are_removed_in_both_quotings() {
        assert_eq!(unwrap_shell("/bin/bash -lc 'ls -a'"), "ls -a");
        assert_eq!(unwrap_shell("/bin/bash -lc \"pwd; ls\""), "pwd; ls");
        assert_eq!(unwrap_shell("ls -a"), "ls -a");
        assert_eq!(
            unwrap_shell("/bin/bash -lc 'unclosed"),
            "/bin/bash -lc 'unclosed"
        );
    }

    #[test]
    fn unreadable_lines_give_nothing() {
        assert_eq!(events(Agent::Codex, "not json", "/r"), vec![]);
        assert_eq!(
            events(Agent::Codex, r#"{"type":"turn.started"}"#, "/r"),
            vec![]
        );
    }
}
