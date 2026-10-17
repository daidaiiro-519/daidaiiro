//! 起動時の結合。port の実装を application に渡す。結合はここ1か所に置く。

use crate::inbound::cli::{self, Action};
use crate::inbound::http::{self, Server};
use crate::outbound::agent::RealAgents;
use crate::outbound::clock::{random, RealClock};
use crate::outbound::fs::RealFiles;
use crate::outbound::process::RealPrograms;
use brainstorming_core::application::{render, Env, Skill};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

/// 本物の入出力で動く port の組。
pub fn env() -> Env<RealFiles, RealPrograms, RealClock> {
    Env {
        files: RealFiles,
        programs: RealPrograms,
        clock: RealClock::default(),
    }
}

/// brainstorming 自身の Skill のフォルダ。実行ファイル（scripts/bin/brainstorming）の場所から決める。
pub fn own_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.parent()?.parent()?.to_path_buf()))
        .unwrap_or_default()
}

/// 引数を受けて動かし、終了コードを返す。serve は止めるまで戻らない。
pub fn main(args: &[String]) -> i32 {
    let skill = Skill::new(&own_dir());
    let env = env();
    match cli::run(args, &skill, &env) {
        Action::Done(code, out) => {
            println!("{out}");
            code
        }
        Action::Serve { dir, port } => {
            if let Err(reason) = render::run(&env, &skill, &dir) {
                println!("{}", json!({"ok": false, "reason": reason}));
                return 1;
            }
            let (port, listener) = match http::bind(port) {
                Ok(bound) => bound,
                Err(reason) => {
                    println!("{}", json!({"ok": false, "reason": reason}));
                    return 1;
                }
            };
            let token: String = random::<16>().iter().map(|b| format!("{b:02x}")).collect();
            let url = format!("http://127.0.0.1:{port}/?k={token}");
            println!(
                "{}",
                json!({"ok": true, "data": {"url": url, "dir": dir, "port": port}})
            );
            let server = Arc::new(Server::new(env, RealAgents, skill, dir, port, token));
            http::serve(server, listener);
            0
        }
    }
}
