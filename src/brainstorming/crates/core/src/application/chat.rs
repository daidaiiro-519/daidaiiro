//! AI を動かすユースケース。serve が、ログイン済みの AI の CLI を子のプロセスとして呼び、出来事を共通の形で画面へ送る。
//! 依頼は2種類ある。AI に聞く（ask）と、回答を送る（send）である。送ると、回答を書いてから AI がボードを更新する。
//!
//! 1回の依頼で行うこと：
//! 0. ボードのロックを取る（取れなければ断る）。終わったら外す。動いているあいだは beat を書き直す
//! 1. 起動の前に、ボードの本体とフォルダを .serve/runs/<run>/ へ写し、リポジトリのファイルの指紋を取る
//! 2. 新しいセッションなら Skill の使い方を指す1文を付けて、続きならセッションの ID で起動する
//! 3. 出来事を共通の形にして画面へ送る。止める指示 ・ 全体の時間 ・ 出来事が来ない時間で止める
//! 4. 終わったら、AI の「検査した」は使わず、serve が自分で check を走らせる
//! 5. 書き換えてよい範囲の外で変わったファイルを探して知らせる（戻すかは利用者が選ぶ。黙って戻さない）
//! 6. ボードが変わっていれば描き直し、やりとりをチャットの記録に残す

use crate::application::engine::check_board;
use crate::application::{answer, lock, relative, render, Env, Skill};
use crate::domain::agent::{self, Agent, Event, Request};
use crate::domain::board::{list, pretty, round, text, valid_id, Layout};
use crate::domain::scope::{outside, porcelain, sort_out, Sorted};
use crate::ports::{Agents, Clock, Files, Next, Programs, Running};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

/// 止める合図の理由。0 は止めない。
pub const RUNNING: u8 = 0;
/// 画面の「止める」が押された。
pub const STOPPED_BY_USER: u8 = 1;
/// 画面とのつながりが切れた（読み込み直した ・ タブを閉じた）。
pub const DISCONNECTED: u8 = 2;

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

/// 依頼の種類。
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// AI に聞く。text が依頼の文
    Ask,
    /// 回答を送る。回答を書いてから、AI がボードを更新する
    Send { round: u64, answers: Vec<Value> },
}

/// 画面からの依頼。
#[derive(Clone, Debug, PartialEq)]
pub struct Turn {
    pub board: String,
    pub topic: String,
    pub agent: String,
    pub text: String,
    pub kind: Kind,
}

/// serve が beat を書き直す間隔（ミリ秒）。domain::lock::SERVE_STALE より十分短くする。
const BEAT_MS: u64 = 10_000;

/// 送った回答の要約（チャットの記録に残す文）。
fn sent_summary(answers: &[Value]) -> String {
    let pick = |decision: &str| -> Vec<String> {
        answers
            .iter()
            .filter(|a| text(a, "decision") == decision)
            .map(|a| text(a, "topic").to_owned())
            .collect()
    };
    let mut parts = Vec::new();
    for (decision, label) in [("approve", "承認"), ("return", "差し戻し")] {
        let ids = pick(decision);
        if !ids.is_empty() {
            parts.push(format!("{label} {}", ids.join(" ・ ")));
        }
    }
    format!("回答を送った：{}", parts.join("、"))
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

/// git の中のファイルか（取ってきたリポジトリの写しなど）。写しと戻しの対象から外す。
fn in_git(rest: &Path) -> bool {
    rest.components().any(|part| part.as_os_str() == ".git")
}

/// from の下のファイルを、バイトのまま to の下へ写す。git の中は写さない。
fn copy_into<F: Files>(files: &F, from: &Path, to: &Path) -> Result<(), String> {
    for path in files.tree(from) {
        let rest = path.strip_prefix(from).map_err(|e| e.to_string())?;
        if in_git(rest) {
            continue;
        }
        files.write_bytes(&to.join(rest), &files.read_bytes(&path)?)?;
    }
    Ok(())
}

/// dir の下のファイルのうち、git の中でないものを消す（フォルダごとは消さない）。
fn clear_outside_git<F: Files>(files: &F, dir: &Path) -> Result<(), String> {
    for path in files.tree(dir) {
        let rest = path.strip_prefix(dir).map_err(|e| e.to_string())?;
        if !in_git(rest) {
            files.remove_file(&path)?;
        }
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

/// 1回の依頼を行う。出来事は sink へ1つずつ渡す。ボードのロックを取ってから動かし、終わったら外す。
#[allow(clippy::too_many_arguments)]
pub fn turn<F: Files, P: Programs, C: Clock, A: Agents>(
    env: &Env<F, P, C>,
    agents: &A,
    skill: &Skill,
    root: &Path,
    request: &Turn,
    limits: &Limits,
    cancel: &AtomicU8,
    sink: &mut impl FnMut(Value),
) -> Result<(), String> {
    let agent = Agent::parse(&request.agent).ok_or("AI の名前が違う")?;
    if !valid_id(&request.board) {
        return Err("ボードの名前が違う".to_owned());
    }
    let what = match &request.kind {
        Kind::Ask if request.text.trim().is_empty() => return Err("依頼が空".to_owned()),
        Kind::Ask => "ask",
        Kind::Send { .. } if agent == Agent::Kiro => {
            return Err(
                "Kiro は読むだけなので、回答を送ってもボードを更新できない。Claude Code か Codex を選んで送る"
                    .to_owned(),
            )
        }
        Kind::Send { .. } => "send",
    };
    if !agents.found(agent.program()) {
        return Err(format!("{} が見つからない（PATH に無い）", agent.program()));
    }
    let layout = Layout::new(root);
    let run_id = env.clock.uuid();
    lock::acquire(env, &layout, &request.board, &run_id, what)?;
    let done = work(
        env, agents, skill, root, request, agent, &run_id, limits, cancel, sink,
    );
    lock::release(env, &layout, &request.board, &run_id);
    done
}

#[allow(clippy::too_many_arguments)]
fn work<F: Files, P: Programs, C: Clock, A: Agents>(
    env: &Env<F, P, C>,
    agents: &A,
    skill: &Skill,
    root: &Path,
    request: &Turn,
    agent: Agent,
    run_id: &str,
    limits: &Limits,
    cancel: &AtomicU8,
    sink: &mut impl FnMut(Value),
) -> Result<(), String> {
    let layout = Layout::new(root);
    let repo = layout.repo();
    let board_file = layout.board_file(&request.board);
    let (said_by_user, instruction) = match &request.kind {
        Kind::Ask => (request.text.clone(), request.text.clone()),
        Kind::Send { round, answers } => {
            answer::submit_all(env, skill, root, &request.board, *round, answers)?;
            let summary = sent_summary(answers);
            let instruction = format!(
                "画面から回答を送った（{}）。{}/ にある第{round}回への回答を読み、{} の Step 5 のとおりにボードを直し、Step 3 の inspect と check を0件にしてから、advance と render まで行う。",
                summary.trim_start_matches("回答を送った："),
                relative(&layout.answers(&request.board), &repo),
                relative(&skill.skill_md(), &repo),
            );
            (summary, instruction)
        }
    };
    let board = env.read_json(&board_file)?;
    let run_dir = layout.runs().join(run_id);
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
        writable: agent != Agent::Kiro,
        round: round(&board),
        topic: request.topic.clone(),
        text: instruction,
        run: run_id.to_owned(),
        checker: env
            .files
            .exists(&skill.checker())
            .then(|| relative(&skill.checker(), &repo)),
    };
    let plan = agent::plan(
        agent,
        (!session.is_empty()).then_some(session.as_str()),
        &fresh,
        &repo.to_string_lossy(),
        &ask,
    );
    let mut messages: Vec<Value> = list(&log, "messages").to_vec();
    messages.push(json!({"role": "user", "text": said_by_user, "at": env.clock.now()}));
    sink(json!({"type": "start", "run": run_id, "agent": agent.name(), "new": session.is_empty()}));

    sink(json!({"type": "phase", "text": format!("{} を起動している", agent.program())}));
    let mut started = agents.start(&plan, &repo)?;
    let (begin, mut last) = (env.clock.elapsed_ms(), env.clock.elapsed_ms());
    let (mut said, mut items, mut found_session, mut failure) = (
        String::new(),
        Vec::<Value>::new(),
        String::new(),
        None::<String>,
    );
    let mut usage: Option<Value> = None;
    let mut beaten = begin;
    loop {
        if env.clock.elapsed_ms().saturating_sub(beaten) > BEAT_MS {
            lock::beat(env, &layout, &request.board, run_id);
            beaten = env.clock.elapsed_ms();
        }
        let reason = cancel.load(Ordering::SeqCst);
        if reason != RUNNING {
            started.stop();
            failure = Some(match reason {
                STOPPED_BY_USER => "画面の「止める」で止めた".to_owned(),
                _ => "画面とのつながりが切れたので止めた（画面を読み込み直したか、閉じたとき）"
                    .to_owned(),
            });
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
                        Event::Usage(input, cached, output) => {
                            usage =
                                Some(json!({"input": input, "cached": cached, "output": output}));
                            sink(
                                json!({"type": "usage", "input": input, "cached": cached, "output": output}),
                            );
                        }
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

    // 送ったのに回が進んでいなければ、更新しなかったと知らせる
    if let Kind::Send { round: sent, .. } = &request.kind {
        let now = env.read_json(&board_file).map(|b| round(&b)).unwrap_or(0);
        if failure.is_none() && now <= *sent {
            failure = Some(
                "AI がボードを更新しなかった（送った回答は残っている。もう一度送るか、AI に聞いて確かめる）"
                    .to_owned(),
            );
        }
    }

    // 4. serve が自分で検査する
    sink(json!({"type": "phase", "text": "serve が検査している"}));
    let check = check_board(env, skill, root, &board_file, &request.board);
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
        layout.serve_dir(),
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
                let edited: Vec<String> = items
                    .iter()
                    .filter(|item| item["act"] == "edit")
                    .filter_map(|item| item["target"].as_str().map(str::to_owned))
                    .collect();
                let ran = items.iter().any(|item| item["act"] == "run");
                let data_dir = relative(root, &repo);
                let sorted = sort_out(&files, &edited, &data_dir);
                notices.extend(outside_notices(&sorted, ran));
                sink(json!({
                    "type": "outside",
                    "files": files,
                    "edited": sorted.edited,
                    "boards": sorted.boards,
                    "unknown": sorted.unknown,
                    "ran": ran,
                    "run": run_id
                }));
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
        sink(json!({"type": "phase", "text": "画面を描き直している"}));
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
        let mut message = json!({"role": "assistant", "text": said, "at": at});
        if let Some(used) = &usage {
            message["usage"] = used.clone();
        }
        messages.push(message);
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

/// 範囲の外で変わったファイルの知らせ。誰が変えたかの手がかりごとに1件にする。
fn outside_notices(sorted: &Sorted, ran: bool) -> Vec<String> {
    let mut out = Vec::new();
    if !sorted.edited.is_empty() {
        out.push(format!(
            "AI がボードの外を直した：{}",
            sorted.edited.join(" ・ ")
        ));
    }
    if !sorted.boards.is_empty() {
        out.push(format!(
            "ほかのボードが変わった（AI の作業の記録に無い。同じ時間に別のセッションが動かしたことが多い）：{}",
            sorted.boards.join(" ・ ")
        ));
    }
    if !sorted.unknown.is_empty() {
        let who = if ran {
            "AI の作業の記録に無い。AI が実行したコマンドか、同じ時間のほかの作業が変えた"
        } else {
            "AI の作業の記録に無い。同じ時間のほかの作業が変えた"
        };
        out.push(format!(
            "ボードの外が変わった（{who}）：{}",
            sorted.unknown.join(" ・ ")
        ));
    }
    out
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
    clear_outside_git(&env.files, &layout.board_dir(board))?;
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
            kind: Kind::Ask,
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
            &AtomicU8::new(RUNNING),
            &mut |e| got.push(e),
        )
        .unwrap();
        (got, env)
    }

    fn kinds(events: &[Value]) -> Vec<String> {
        events.iter().map(|e| text(e, "type").to_owned()).collect()
    }

    const OPEN_BOARD: &str = r#"{"$schema":"brainstorming/board@v1","kind":"board","id":"b","title":"t","round":2,"topics":[{"id":"Q1","status":"open"}]}"#;

    fn send(agent: &str) -> Turn {
        Turn {
            agent: agent.to_owned(),
            text: String::new(),
            kind: Kind::Send {
                round: 2,
                answers: vec![json!({"topic": "Q1", "decision": "approve"})],
            },
            ..request("")
        }
    }

    fn run_turn(
        env: &crate::tests::Env3,
        agents: &FakeAgents,
        turn_req: &Turn,
    ) -> (Result<(), String>, Vec<Value>) {
        let mut got = Vec::new();
        let done = turn(
            env,
            agents,
            &skill(),
            Path::new("/r/.brainstorming"),
            turn_req,
            &Limits::default(),
            &AtomicU8::new(RUNNING),
            &mut |e| got.push(e),
        );
        (done, got)
    }

    #[test]
    fn sending_writes_the_answers_tells_the_ai_and_says_when_the_board_was_not_updated() {
        let env = env_with_git(MemoryFiles::with(&[(
            "/r/.brainstorming/b.json",
            OPEN_BOARD,
        )]));
        let agents = FakeAgents::new(&[], 0);
        let (done, got) = run_turn(&env, &agents, &send("codex"));
        done.unwrap();
        assert!(env
            .files
            .exists(Path::new("/r/.brainstorming/b/answers/2-Q1-1.json")));
        let told = agents.started().stdin.unwrap_or_default();
        assert!(told.contains("画面から回答を送った（承認 Q1）"));
        assert!(told.contains(".brainstorming/b/answers/ にある第2回への回答を読み"));
        assert_eq!(agents.started().env[0].1, "U1");
        let errors: Vec<&str> = got
            .iter()
            .filter(|e| text(e, "type") == "error")
            .map(|e| text(e, "text"))
            .collect();
        assert_eq!(
            errors,
            vec!["AI がボードを更新しなかった（送った回答は残っている。もう一度送るか、AI に聞いて確かめる）"]
        );
        let log = env
            .read_json(Path::new("/r/.brainstorming/.chat/b/codex.json"))
            .unwrap();
        assert_eq!(list(&log, "messages")[0]["text"], "回答を送った：承認 Q1");
        assert!(!env
            .files
            .exists(Path::new("/r/.brainstorming/.serve/locks/b.json")));
    }

    #[test]
    fn a_board_that_is_locked_refuses_both_asking_and_sending() {
        let env = env_with_git(MemoryFiles::with(&[(
            "/r/.brainstorming/b.json",
            OPEN_BOARD,
        )]));
        lock::acquire(
            &env,
            &Layout::new(Path::new("/r/.brainstorming")),
            "b",
            "cli",
            "cli",
        )
        .unwrap();
        let agents = FakeAgents::new(&[], 0);
        let (asked, _) = run_turn(&env, &agents, &request("聞く"));
        assert!(asked
            .unwrap_err()
            .starts_with("このボードは更新中（ターミナルの AI が直している"));
        let (sent, _) = run_turn(&env, &agents, &send("codex"));
        assert!(sent.is_err());
        assert!(!env
            .files
            .exists(Path::new("/r/.brainstorming/b/answers/2-Q1-1.json")));
    }

    #[test]
    fn kiro_cannot_send_because_it_only_reads() {
        let env = env_with_git(MemoryFiles::with(&[(
            "/r/.brainstorming/b.json",
            OPEN_BOARD,
        )]));
        let (sent, _) = run_turn(&env, &FakeAgents::new(&[], 0), &send("kiro"));
        assert!(sent.unwrap_err().starts_with("Kiro は読むだけなので"));
    }

    #[test]
    fn the_snapshot_copies_bytes_and_leaves_git_folders_alone() {
        let files = MemoryFiles::with(&[("/r/.brainstorming/b.json", OPEN_BOARD)]);
        files
            .write_bytes(
                Path::new("/r/.brainstorming/b/figures/a.png"),
                &[0xff, 0xd8, 0x00],
            )
            .unwrap();
        files
            .write_bytes(
                Path::new("/r/.brainstorming/b/sources/repo/.git/index"),
                &[0xc3, 0x28],
            )
            .unwrap();
        let env = env_with_git(files);
        let layout = Layout::new(Path::new("/r/.brainstorming"));
        snapshot(&env.files, &layout, "b", Path::new("/r/run")).unwrap();
        assert_eq!(
            env.files
                .read_bytes(Path::new("/r/run/dir/figures/a.png"))
                .unwrap(),
            vec![0xff, 0xd8, 0x00]
        );
        assert!(!env
            .files
            .exists(Path::new("/r/run/dir/sources/repo/.git/index")));
        clear_outside_git(&env.files, &layout.board_dir("b")).unwrap();
        assert!(!env
            .files
            .exists(Path::new("/r/.brainstorming/b/figures/a.png")));
        assert!(env
            .files
            .exists(Path::new("/r/.brainstorming/b/sources/repo/.git/index")));
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
        assert_eq!(
            kinds(&got),
            vec!["start", "phase", "work", "text", "usage", "phase", "check", "done"]
        );
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
            &AtomicU8::new(RUNNING),
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
                ("/r/.brainstorming/other.json", "{}"),
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
            &AtomicU8::new(RUNNING),
            &mut |e| got.push(e),
        )
        .unwrap();
        let outside: Vec<&Value> = got
            .iter()
            .filter(|e| text(e, "type") == "outside")
            .collect();
        assert_eq!(
            outside[0]["files"],
            json!([".brainstorming/other.json", "notes.txt"])
        );
        assert_eq!(outside[0]["boards"], json!([".brainstorming/other.json"]));
        assert_eq!(outside[0]["unknown"], json!(["notes.txt"]));
        assert_eq!(outside[0]["edited"], json!([]));
        assert!(env.files.exists(Path::new("/r/notes.txt")));
    }

    #[test]
    fn a_failing_cli_and_a_stop_both_end_with_an_error_event() {
        let (got, _) = events(&[], 3);
        assert_eq!(
            kinds(&got),
            vec!["start", "phase", "phase", "check", "error", "done"]
        );
        let env = env_with_git(MemoryFiles::with(&[("/r/.brainstorming/b.json", BOARD)]));
        let mut stopped = Vec::new();
        turn(
            &env,
            &FakeAgents::quiet(),
            &skill(),
            Path::new("/r/.brainstorming"),
            &request("x"),
            &Limits::default(),
            &AtomicU8::new(STOPPED_BY_USER),
            &mut |e| stopped.push(e),
        )
        .unwrap();
        assert_eq!(
            stopped
                .iter()
                .find(|e| text(e, "type") == "error")
                .map(|e| e["text"].clone()),
            Some(json!("画面の「止める」で止めた"))
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
            &AtomicU8::new(RUNNING),
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
            &AtomicU8::new(RUNNING),
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
