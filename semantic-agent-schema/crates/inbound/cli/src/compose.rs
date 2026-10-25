//! 起動時の結合（outbound を application へ渡す1か所）

use crate::commands::mechanical::{Mechanical, parse};
use semantic_agent_schema_application::ports::changes::Changes;
use semantic_agent_schema_application::semantic_agent_schema_change::values::change_id::ChangeId;
use semantic_agent_schema_application::use_cases::{
    Outcome, Ports, create, history, judge, patch, record, search, validate,
};
use semantic_agent_schema_fs::files::Files;
use semantic_agent_schema_git::changes::Git;
use semantic_agent_schema_sqlite::change_store::LocalDb;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// 引数を受け取って1つのコマンドを動かし、終了コードを返す（0 正常 ・ 1 検出あり ・ 2 使い方の誤り）
pub fn run(args: &[String]) -> ExitCode {
    let parsed = match parse(args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("使い方の誤り：{e}");
            return ExitCode::from(2);
        }
    };
    let root = std::env::current_dir().unwrap_or_default();
    let files = Files::new(root.clone());
    let git = Git::new(root.clone());
    let db = match LocalDb::open(&db_path(&root, git.shared_dir())) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let ports = Ports {
        contracts: &files,
        instances: &files,
        records: &db,
        judgments: &db,
    };
    let outcome = match &parsed.command {
        Mechanical::Create {
            path,
            owner,
            kind,
            value,
        } => create::create(&ports, path, owner, kind, value),
        Mechanical::Get { path, pointer } => search::get(&files, path, pointer),
        Mechanical::CreateInstance {
            path,
            schema,
            id,
            value,
        } => {
            let id = id.clone().unwrap_or_else(|| new_ulid().to_lowercase());
            create::create_instance(&ports, path, schema, &id, value)
        }
        Mechanical::Patch {
            path,
            ops,
            if_match,
        } => patch::patch(&ports, path, ops, if_match.as_deref()),
        Mechanical::Validate { path } => validate::validate(&ports, path),
        Mechanical::Record {
            path,
            pointers,
            intent,
            purpose,
            writer,
            replaces,
            id,
            session,
        } => {
            let id = id.clone().unwrap_or_else(new_ulid);
            record::record(
                &ports,
                &record::RecordRequest {
                    path,
                    pointers,
                    intent,
                    purpose,
                    writer,
                    replaces: replaces.as_deref(),
                    id: &id,
                    session: session.as_deref(),
                },
            )
        }
        Mechanical::Judge {
            change_id,
            result,
            meaning,
            reason,
        } => judge::judge_intent(&ports, change_id, result, meaning, reason),
        Mechanical::History { path } => history::history(&ports, path),
        Mechanical::Summarize { id } => {
            let id = id.clone().unwrap_or_else(new_ulid);
            record::summarize(&ports, &git, &files, &id)
        }
    };
    print(&outcome, parsed.json);
    if outcome.ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

/// 手元の DB の置き場所。起動の設定（環境変数 SEMANTIC_AGENT_SCHEMA_DB）が無ければ、Git のすべての worktree が共有する
/// 置き場所（git common dir）の下に置く。Git の外なら、利用者の状態のフォルダ（$XDG_STATE_HOME か ~/.local/state）の下に、
/// 作業の根ごとに1つ置く。どちらもコミットされない
fn db_path(root: &Path, shared: Option<PathBuf>) -> PathBuf {
    if let Some(p) = std::env::var_os("SEMANTIC_AGENT_SCHEMA_DB") {
        return PathBuf::from(p);
    }
    if let Some(dir) = shared {
        return dir.join("semantic-agent-schema").join("local.db");
    }
    let state = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state")))
        .unwrap_or_else(|| root.join(".semantic-agent-schema-state"));
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in root.to_string_lossy().bytes() {
        hash = (hash ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
    }
    state
        .join("semantic-agent-schema")
        .join(format!("{hash:016x}.db"))
}

/// 打ち合わせなしで一意な id（ULID）。時刻と乱数を読むのは inbound だけ
fn new_ulid() -> String {
    use std::hash::{BuildHasher, Hasher};
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let millis = u64::try_from(now.as_millis()).unwrap_or(u64::MAX);
    let mut random: u128 = 0;
    for salt in 0..2u64 {
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u64(salt ^ u64::from(now.subsec_nanos()) ^ u64::from(std::process::id()));
        random = (random << 64) | u128::from(h.finish());
    }
    ChangeId::from_parts(millis, random).to_string()
}

/// MCP の面を起動する（段5で書く。いまは使い方の誤りを返す）
pub fn run_mcp() -> ExitCode {
    eprintln!("MCP の面はまだ無い");
    ExitCode::from(2)
}

fn print(outcome: &Outcome, as_json: bool) {
    if as_json {
        let findings: Vec<_> = outcome.findings.iter().map(|l| l.to_json()).collect();
        println!(
            "{}",
            json!({"ok": outcome.ok, "findings": findings, "data": outcome.data})
        );
        return;
    }
    for l in &outcome.findings {
        println!("{}\t{}\t{}\t{}", l.result, l.rule, l.pointer, l.reason);
    }
    println!("{}", if outcome.ok { "ok" } else { "ng" });
}
