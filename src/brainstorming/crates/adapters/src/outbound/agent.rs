//! port の Agents の実装。AI の CLI をシェルを通さずに起動し、標準出力を1行ずつ渡す。
//! 止めるときは子孫のプロセスまで止める（Linux ・ macOS ・ WSL はプロセスグループ、Windows は taskkill /T）。

use brainstorming_core::domain::agent::Plan;
use brainstorming_core::ports::{Agents, Next, Running};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// 標準エラーのうち、失敗の説明に使う末尾の長さ（文字）。
const TAIL: usize = 2000;

pub struct RealAgents;

/// PATH から実行ファイルを探す。
pub fn which(program: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    let suffixes: &[&str] = if cfg!(windows) {
        &[".exe", ".cmd", ".bat", ""]
    } else {
        &[""]
    };
    std::env::split_paths(&paths).find_map(|dir| {
        suffixes
            .iter()
            .map(|suffix| dir.join(format!("{program}{suffix}")))
            .find(|candidate| candidate.is_file())
    })
}

pub struct RealRun {
    child: Arc<Mutex<Child>>,
    lines: Receiver<String>,
    stderr: Arc<Mutex<String>>,
    reader: Option<thread::JoinHandle<()>>,
}

#[cfg(unix)]
fn own_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn own_group(_command: &mut Command) {}

#[cfg(unix)]
fn kill_tree(pid: u32) {
    let _ = Command::new("kill")
        .args(["-TERM", &format!("-{pid}")])
        .status();
    thread::sleep(Duration::from_millis(500));
    let _ = Command::new("kill")
        .args(["-KILL", &format!("-{pid}")])
        .status();
}

#[cfg(not(unix))]
fn kill_tree(pid: u32) {
    let _ = Command::new("taskkill")
        .args(["/T", "/F", "/PID", &pid.to_string()])
        .status();
}

impl Running for RealRun {
    fn next(&mut self, wait: Duration) -> Next {
        match self.lines.recv_timeout(wait) {
            Ok(line) => Next::Line(line),
            Err(RecvTimeoutError::Timeout) => Next::Quiet,
            Err(RecvTimeoutError::Disconnected) => {
                let code = self
                    .child
                    .lock()
                    .ok()
                    .and_then(|mut child| child.wait().ok())
                    .and_then(|status| status.code())
                    .unwrap_or(-1);
                if let Some(reader) = self.reader.take() {
                    let _ = reader.join();
                }
                let tail = self.stderr.lock().map(|s| s.clone()).unwrap_or_default();
                let start = tail.chars().count().saturating_sub(TAIL);
                Next::Ended(code, tail.chars().skip(start).collect())
            }
        }
    }
    fn stop(&mut self) {
        if let Ok(mut child) = self.child.lock() {
            kill_tree(child.id());
            let _ = child.kill();
        }
    }
}

impl Agents for RealAgents {
    type Started = RealRun;
    fn start(&self, plan: &Plan, cwd: &Path) -> Result<RealRun, String> {
        let program = which(&plan.program)
            .ok_or(format!("{} が見つからない（PATH に無い）", plan.program))?;
        let mut command = Command::new(&program);
        command
            .args(&plan.args)
            .current_dir(cwd)
            .stdin(if plan.stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        own_group(&mut command);
        let mut child = command
            .spawn()
            .map_err(|error| format!("{} を起動できない：{error}", plan.program))?;
        if let (Some(text), Some(mut stdin)) = (&plan.stdin, child.stdin.take()) {
            stdin
                .write_all(text.as_bytes())
                .map_err(|error| format!("依頼を渡せない：{error}"))?;
        }
        let stdout = child.stdout.take().ok_or("標準出力を受け取れない")?;
        let mut err = child.stderr.take().ok_or("標準エラーを受け取れない")?;
        let (send, lines) = channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if send.send(line).is_err() {
                    break;
                }
            }
        });
        let stderr = Arc::new(Mutex::new(String::new()));
        let sink = Arc::clone(&stderr);
        let reader = thread::spawn(move || {
            let mut text = String::new();
            let _ = err.read_to_string(&mut text);
            if let Ok(mut all) = sink.lock() {
                *all = text;
            }
        });
        Ok(RealRun {
            child: Arc::new(Mutex::new(child)),
            lines,
            stderr,
            reader: Some(reader),
        })
    }
    fn found(&self, program: &str) -> bool {
        which(program).is_some()
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn plan(args: &[&str], stdin: Option<&str>) -> Plan {
        Plan {
            program: "sh".to_owned(),
            args: args.iter().map(|a| (*a).to_owned()).collect(),
            stdin: stdin.map(str::to_owned),
        }
    }

    fn drain(run: &mut RealRun) -> Vec<Next> {
        std::iter::from_fn(|| Some(run.next(Duration::from_secs(5))))
            .take_while(|next| !matches!(next, Next::Ended(..)))
            .collect()
    }

    #[test]
    fn lines_come_one_by_one_then_the_exit_code() {
        let mut run = RealAgents
            .start(
                &plan(
                    &["-c", "read x; echo \"$x\"; echo two; echo err >&2; exit 3"],
                    Some("one\n"),
                ),
                Path::new("/"),
            )
            .unwrap();
        assert_eq!(
            drain(&mut run),
            vec![Next::Line("one".to_owned()), Next::Line("two".to_owned())]
        );
        assert_eq!(
            run.next(Duration::from_secs(5)),
            Next::Ended(3, "err\n".to_owned())
        );
    }

    #[test]
    fn stop_ends_the_child_and_its_children() {
        let mut run = RealAgents
            .start(
                &plan(&["-c", "sleep 30 & sleep 30; echo late"], None),
                Path::new("/"),
            )
            .unwrap();
        assert_eq!(run.next(Duration::from_millis(200)), Next::Quiet);
        run.stop();
        assert!(matches!(run.next(Duration::from_secs(5)), Next::Ended(..)));
    }

    #[test]
    fn a_missing_program_is_reported() {
        let missing = Plan {
            program: "no-such-cli-xyz".to_owned(),
            args: vec![],
            stdin: None,
        };
        assert!(RealAgents.start(&missing, Path::new("/")).is_err());
        assert!(!RealAgents.found("no-such-cli-xyz"));
    }
}
