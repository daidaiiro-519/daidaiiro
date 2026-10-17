//! AI に聞くユースケース。serve が、ログイン済みの AI の CLI を子のプロセスとして呼び、出来事を共通の形で画面へ送る。
//!
//! 1回の依頼で行うこと：
//! 1. 起動の前に、ボードの本体とフォルダを .serve/runs/<run>/ へ写し、リポジトリのファイルの指紋を取る
//! 2. 新しいセッションなら Skill の使い方を指す1文を付けて、続きならセッションの ID で起動する
//! 3. 出来事を共通の形にして画面へ送る。止める指示 ・ 全体の時間 ・ 出来事が来ない時間で止める
//! 4. 終わったら、AI の「検査した」は使わず、serve が自分で check を走らせる
//! 5. 書き換えてよい範囲の外で変わったファイルを探して知らせる（戻すかは利用者が選ぶ。黙って戻さない）
//! 6. ボードが変わっていれば描き直し、やりとりをチャットの記録に残す

use crate::application::engine::check_dir;
use crate::application::{relative, render, Env, Skill};
use crate::domain::agent::{self, Agent, Event, Request};
use crate::domain::board::{list, pretty, round, text, valid_id, Layout};
use crate::domain::scope::{outside, porcelain};
use crate::ports::{Agents, Clock, Files, Next, Programs, Running};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// チャットの記録に残すやりとりの数の上限。
pub const KEEP: usize = 200;

/// 時間切れの決まり。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limits {
    /// 全体の時間（ミリ秒）
    pub total_ms: u64,
    /// 出来事が来ない時間（ミリ秒）
    pub quiet_ms: u64,
    /// 1回に待つ時間
    pub wait: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            total_ms: 20 * 60 * 1000,
            quiet_ms: 5 * 60 * 1000,
            wait: Duration::from_millis(500),
        }
    }
}

/// 画面からの依頼。
#[derive(Clone, Debug, PartialEq)]
pub struct Turn {
    pub board: String,
    pub topic: String,
    pub agent: String,
    pub text: String,
}

fn chat_log<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    layout: &Layout,
    board: &str,
    agent: Agent,
) -> Value {
    env.read_json(&layout.chat_file(board, agent.name()))
        .unwrap_or_else(|_| {
            json!({
                "$schema": "brainstorming/chat@v1",
                "kind": "chat",
                "id": format!("{board}-{}", agent.name()),
                "board": board,
                "agent": agent.name(),
                "messages": [],
            })
        })
}

/// やりとりの記録。
pub fn history<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    root: &Path,
    board: &str,
    agent: &str,
) -> Result<Value, String> {
    let agent = Agent::parse(agent).ok_or("AI の名前が違う")?;
    if !valid_id(board) {
        return Err("ボードの名前が違う".to_owned());
    }
    let log = chat_log(env, &Layout::new(root), board, agent);
    Ok(json!({"messages": list(&log, "messages")}))
}

/// リポジトリのうち git が変わったと言うファイルの指紋。git が使えなければ None。
fn fingerprint<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    repo: &Path,
) -> Option<BTreeMap<String, String>> {
    let args: Vec<String> = ["status", "--porcelain=v1", "-z", "--untracked-files=all"]
        .iter()
        .map(|a| (*a).to_owned())
        .collect();
    let run = env.programs.run(Path::new("git"), &args, Some(repo)).ok()?;
    (run.code == 0).then_some(())?;
    Some(
        porcelain(&run.stdout)
            .into_iter()
            .map(|path| {
                let digest = env
                    .files
                    .read_bytes(&repo.join(&path))
                    .map(|bytes| {
                        Sha256::digest(&bytes)
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>()
                    })
                    .unwrap_or_else(|_| "消えた".to_owned());
                (path, digest)
            })
            .collect(),
    )
}

fn copy_into<F: Files>(files: &F, from: &Path, to: &Path) -> Result<(), String> {
    for path in files.tree(from) {
        let rest = path.strip_prefix(from).map_err(|e| e.to_string())?;
        files.write(&to.join(rest), &files.read(&path)?)?;
    }
    Ok(())
}

/// 起動の前の写しを取る。
fn snapshot<F: Files>(files: &F, layout: &Layout, board: &str, run: &Path) -> Result<(), String> {
    files.write(
        &run.join("board.json"),
        &files.read(&layout.board_file(board))?,
    )?;
    copy_into(files, &layout.board_dir(board), &run.join("dir"))
}

/// 1回の依頼を行う。出来事は sink へ1つずつ渡す。
#[allow(clippy::too_many_arguments)]
pub fn turn<F: Files, P: Programs, C: Clock, A: Agents>(
    env: &Env<F, P, C>,
    agents: &A,
    skill: &Skill,
    root: &Path,
    request: &Turn,
    limits: &Limits,
    cancel: &AtomicBool,
    sink: &mut impl FnMut(Value),
) -> Result<(), String> {
    let agent = Agent::parse(&request.agent).ok_or("AI の名前が違う")?;
    if !valid_id(&request.board) {
        return Err("ボードの名前が違う".to_owned());
    }
    if request.text.trim().is_empty() {
        return Err("依頼が空".to_owned());
    }
    if !agents.found(agent.program()) {
        return Err(format!("{} が見つからない（PATH に無い）", agent.program()));
    }
    let layout = Layout::new(root);
    let repo = layout.repo();
    let board_file = layout.board_file(&request.board);
    let board = env.read_json(&board_file)?;
    let run_id = env.clock.uuid();
    let run_dir = layout.runs().join(&run_id);
    snapshot(&env.files, &layout, &request.board, &run_dir)?;
    let before_board = env.files.read(&board_file)?;
    let before = fingerprint(env, &repo);

    let mut log = chat_log(env, &layout, &request.board, agent);
    let session = text(&log, "session").to_owned();
    let fresh = env.clock.uuid();
    let ask = Request {
        skill_md: relative(&skill.skill_md(), &repo),
        entry: relative(&skill.entry(), &repo),
        board_file: relative(&board_file, &repo),
        board_dir: format!("{}/", relative(&layout.board_dir(&request.board), &repo)),
        root: repo.to_string_lossy().into_owned(),
        round: round(&board),
        topic: request.topic.clone(),
        text: request.text.clone(),
    };
    let plan = agent::plan(
        agent,
        (!session.is_empty()).then_some(session.as_str()),
        &fresh,
        &repo.to_string_lossy(),
        &ask,
    );
    let mut messages: Vec<Value> = list(&log, "messages").to_vec();
    messages.push(json!({"role": "user", "text": request.text, "at": env.clock.now()}));
    sink(json!({"type": "start", "run": run_id, "agent": agent.name(), "new": session.is_empty()}));

    let mut started = agents.start(&plan, &repo)?;
    let (begin, mut last) = (env.clock.elapsed_ms(), env.clock.elapsed_ms());
    let (mut said, mut items, mut found_session, mut failure) = (
        String::new(),
        Vec::<Value>::new(),
        String::new(),
        None::<String>,
    );
    loop {
        if cancel.load(Ordering::SeqCst) {
            started.stop();
            failure = Some("止めた".to_owned());
            break;
        }
        let now = env.clock.elapsed_ms();
        if now.saturating_sub(begin) > limits.total_ms || now.saturating_sub(last) > limits.quiet_ms
        {
            started.stop();
            failure = Some(format!(
                "時間切れで止めた（全体 {} 分 ・ 出来事が来ない時間 {} 分まで）",
                limits.total_ms / 60000,
                limits.quiet_ms / 60000
            ));
            break;
        }
        match started.next(limits.wait) {
            Next::Quiet => continue,
            Next::Line(line) => {
                last = env.clock.elapsed_ms();
                for event in agent::events(agent, &line, &repo.to_string_lossy()) {
                    match event {
                        Event::Text(chunk) => {
                            said.push_str(&chunk);
                            sink(json!({"type": "text", "text": chunk}));
                        }
                        Event::Work(act, target) => {
                            items.push(json!({"act": act, "target": target}));
                            sink(json!({"type": "work", "act": act, "target": target}));
                        }
                        Event::Session(id) if !id.is_empty() => found_session = id,
                        Event::Session(_) => {}
                        Event::Denied(tool) => sink(
                            json!({"type": "work", "act": "run", "target": format!("{tool}（許していないので断られた）")}),
                        ),
                        Event::Done(false, why) => {
                            failure = Some(if why.is_empty() {
                                "AI が失敗を返した".to_owned()
                            } else {
                                why
                            })
                        }
                        Event::Done(true, _) => {}
                    }
                }
            }
            Next::Ended(code, tail) => {
                if code != 0 && failure.is_none() {
                    failure = Some(format!(
                        "{} が終了コード {code} で終わった：{}",
                        agent.program(),
                        tail.trim()
                    ));
                }
                break;
            }
        }
    }

    // 4. serve が自分で検査する
    let check = check_dir(env, skill, root);
    let (ok, target) = match &check {
        Ok(0) => (true, "check 合格".to_owned()),
        Ok(n) => (false, format!("check 不合格（{n}件）")),
        Err(error) => (false, error.clone()),
    };
    items.push(json!({"act": "check", "target": target, "ok": ok}));
    sink(json!({"type": "check", "ok": ok, "target": target}));

    // 5. 書き換えてよい範囲の外
    let ranges: Vec<String> = [
        layout.board_file(&request.board),
        layout.board_dir(&request.board),
        layout.chat_dir(),
        layout.runs(),
        layout.site(),
        layout.dashboard(),
    ]
    .iter()
    .map(|path| relative(path, &repo))
    .collect();
    let mut notices = Vec::new();
    match (&before, fingerprint(env, &repo)) {
        (Some(before), Some(after)) => {
            let files = outside(before, &after, &ranges);
            if !files.is_empty() {
                notices.push(format!("ボードの外が変わった：{}", files.join(" ・ ")));
                sink(json!({"type": "outside", "files": files, "run": run_id}));
            }
        }
        _ => {
            notices
                .push("git が使えないので、ボードの外が変わったかを確かめられなかった".to_owned());
            sink(json!({"type": "notice", "text": notices[0]}));
        }
    }

    // 6. 描き直しと記録
    if env.files.read(&board_file).ok().as_deref() != Some(before_board.as_str()) {
        match render::run(env, skill, root) {
            Ok(_) => sink(json!({"type": "changed", "run": run_id})),
            Err(error) => {
                sink(json!({"type": "error", "text": format!("描き直せなかった：{error}")}))
            }
        }
    }
    if let Some(why) = &failure {
        sink(json!({"type": "error", "text": why}));
        notices.push(why.clone());
    }
    let at = env.clock.now();
    if !items.is_empty() {
        messages.push(json!({"role": "work", "items": items, "at": at}));
    }
    if !said.is_empty() {
        messages.push(json!({"role": "assistant", "text": said, "at": at}));
    }
    messages.extend(
        notices
            .iter()
            .map(|text| json!({"role": "notice", "text": text, "at": at})),
    );
    let keep = messages.len().saturating_sub(KEEP);
    log["messages"] = Value::Array(messages.split_off(keep));
    if !found_session.is_empty() {
        log["session"] = json!(found_session);
    } else if session.is_empty() && agent == Agent::Claude && failure.is_none() {
        log["session"] = json!(fresh);
    }
    env.files.write(
        &layout.chat_file(&request.board, agent.name()),
        &pretty(&log),
    )?;
    sink(json!({"type": "done", "ok": failure.is_none()}));
    Ok(())
}

/// 起動の前の写しへ、ボードの本体とフォルダを戻す。
pub fn restore<F: Files, P: Programs, C: Clock>(
    env: &Env<F, P, C>,
    skill: &Skill,
    root: &Path,
    board: &str,
    run: &str,
) -> Result<Value, String> {
    if !valid_id(board) || run.is_empty() || run.contains(['/', '\\', '.']) {
        return Err("戻す先の名前が違う".to_owned());
    }
    let layout = Layout::new(root);
    let saved: PathBuf = layout.runs().join(run);
    if !env.files.exists(&saved.join("board.json")) {
        return Err("その写しは無い".to_owned());
    }
    env.files.write(
        &layout.board_file(board),
        &env.files.read(&saved.join("board.json"))?,
    )?;
    env.files.remove_dir(&layout.board_dir(board))?;
    copy_into(&env.files, &saved.join("dir"), &layout.board_dir(board))?;
    render::run(env, skill, root)?;
    Ok(json!({"restored": run}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{env_with_git, skill, FakeAgents, MemoryFiles};

    const BOARD: &str = r#"{"$schema":"brainstorming/board@v1","kind":"board","id":"b","title":"t","status":"answer","round":2,"topics":[]}"#;

    fn request(text: &str) -> Turn {
        Turn {
            board: "b".to_owned(),
            topic: "Q1".to_owned(),
            agent: "codex".to_owned(),
            text: text.to_owned(),
        }
    }

    fn events(lines: &[&str], code: i32) -> (Vec<Value>, crate::tests::Env3) {
        let env = env_with_git(MemoryFiles::with(&[("/r/.brainstorming/b.json", BOARD)]));
        let agents = FakeAgents::new(lines, code);
        let mut got = Vec::new();
        turn(
            &env,
            &agents,
            &skill(),
            Path::new("/r/.brainstorming"),
            &request("前提を足して"),
            &Limits::default(),
            &AtomicBool::new(false),
            &mut |e| got.push(e),
        )
        .unwrap();
        (got, env)
    }

    fn kinds(events: &[Value]) -> Vec<String> {
        events.iter().map(|e| text(e, "type").to_owned()).collect()
    }

    #[test]
    fn a_turn_streams_common_events_then_checks_by_itself() {
        let (got, env) = events(
            &[
                r#"{"type":"thread.started","thread_id":"T1"}"#,
                r#"{"type":"item.started","item":{"type":"command_execution","command":"/bin/bash -lc 'ls'"}}"#,
                r#"{"type":"item.completed","item":{"type":"agent_message","text":"足しました"}}"#,
                r#"{"type":"turn.completed"}"#,
            ],
            0,
        );
        assert_eq!(kinds(&got), vec!["start", "work", "text", "check", "done"]);
        let log = env
            .read_json(Path::new("/r/.brainstorming/.chat/b/codex.json"))
            .unwrap();
        assert_eq!(log["session"], "T1");
        assert_eq!(
            list(&log, "messages")
                .iter()
                .map(|m| text(m, "role"))
                .collect::<Vec<_>>(),
            vec!["user", "work", "assistant"]
        );
        assert!(env
            .files
            .exists(Path::new("/r/.brainstorming/.serve/runs/U1/board.json")));
    }

    #[test]
    fn the_first_request_points_at_the_skill_and_the_next_resumes() {
        let (_, env) = events(&[r#"{"type":"thread.started","thread_id":"T1"}"#], 0);
        let agents = FakeAgents::new(&[], 0);
        turn(
            &env,
            &agents,
            &skill(),
            Path::new("/r/.brainstorming"),
            &request("続き"),
            &Limits::default(),
            &AtomicBool::new(false),
            &mut |_| {},
        )
        .unwrap();
        let plan = agents.started();
        assert_eq!(
            plan.args[..3],
            ["exec".to_owned(), "resume".to_owned(), "--json".to_owned()]
        );
        assert!(plan
            .stdin
            .as_deref()
            .unwrap_or("")
            .starts_with("作業するフォルダは /r。対象は .brainstorming/b.json"));
    }

    #[test]
    fn a_change_outside_the_board_is_reported_and_not_undone() {
        let env = env_with_git(MemoryFiles::with(&[("/r/.brainstorming/b.json", BOARD)]));
        let agents = FakeAgents::writing(
            &env.files,
            &[
                ("/r/notes.txt", "x"),
                ("/r/.brainstorming/b/answers/x.json", "{}"),
            ],
        );
        let mut got = Vec::new();
        turn(
            &env,
            &agents,
            &skill(),
            Path::new("/r/.brainstorming"),
            &request("x"),
            &Limits::default(),
            &AtomicBool::new(false),
            &mut |e| got.push(e),
        )
        .unwrap();
        let outside: Vec<&Value> = got
            .iter()
            .filter(|e| text(e, "type") == "outside")
            .collect();
        assert_eq!(outside[0]["files"], json!(["notes.txt"]));
        assert!(env.files.exists(Path::new("/r/notes.txt")));
    }

    #[test]
    fn a_failing_cli_and_a_stop_both_end_with_an_error_event() {
        let (got, _) = events(&[], 3);
        assert_eq!(kinds(&got), vec!["start", "check", "error", "done"]);
        let env = env_with_git(MemoryFiles::with(&[("/r/.brainstorming/b.json", BOARD)]));
        let mut stopped = Vec::new();
        turn(
            &env,
            &FakeAgents::quiet(),
            &skill(),
            Path::new("/r/.brainstorming"),
            &request("x"),
            &Limits::default(),
            &AtomicBool::new(true),
            &mut |e| stopped.push(e),
        )
        .unwrap();
        assert_eq!(
            stopped
                .iter()
                .find(|e| text(e, "type") == "error")
                .map(|e| e["text"].clone()),
            Some(json!("止めた"))
        );
    }

    #[test]
    fn quiet_too_long_times_out() {
        let env = env_with_git(MemoryFiles::with(&[("/r/.brainstorming/b.json", BOARD)]));
        let limits = Limits {
            total_ms: 100_000,
            quiet_ms: 3_000,
            wait: Duration::from_millis(1),
        };
        let mut got = Vec::new();
        turn(
            &env,
            &FakeAgents::quiet(),
            &skill(),
            Path::new("/r/.brainstorming"),
            &request("x"),
            &limits,
            &AtomicBool::new(false),
            &mut |e| got.push(e),
        )
        .unwrap();
        let error = got
            .iter()
            .find(|e| text(e, "type") == "error")
            .map(|e| text(e, "text").to_owned())
            .unwrap_or_default();
        assert!(error.starts_with("時間切れで止めた"));
    }

    #[test]
    fn restore_puts_back_the_board_and_its_folder() {
        let env = env_with_git(MemoryFiles::with(&[
            ("/r/.brainstorming/b.json", BOARD),
            ("/r/.brainstorming/b/rounds/1.json", "{}"),
        ]));
        let agents = FakeAgents::writing(
            &env.files,
            &[
                ("/r/.brainstorming/b.json", "{}"),
                ("/r/.brainstorming/b/rounds/9.json", "{}"),
            ],
        );
        turn(
            &env,
            &agents,
            &skill(),
            Path::new("/r/.brainstorming"),
            &request("x"),
            &Limits::default(),
            &AtomicBool::new(false),
            &mut |_| {},
        )
        .unwrap();
        restore(&env, &skill(), Path::new("/r/.brainstorming"), "b", "U1").unwrap();
        assert_eq!(
            env.files
                .read(Path::new("/r/.brainstorming/b.json"))
                .unwrap(),
            BOARD
        );
        assert!(!env
            .files
            .exists(Path::new("/r/.brainstorming/b/rounds/9.json")));
        assert!(env
            .files
            .exists(Path::new("/r/.brainstorming/b/rounds/1.json")));
        assert_eq!(
            restore(&env, &skill(), Path::new("/r/.brainstorming"), "b", "a/b").unwrap_err(),
            "戻す先の名前が違う"
        );
    }
}
