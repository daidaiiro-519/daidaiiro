// SPDX-License-Identifier: MIT
//! 1段目の検査 ── **実行ファイルを起動して、振る舞いを見る。** どの言語で書いた Skill でも同じである。
//!
//! 読むのは2つのファイルだけである ── `tool.json`（CLI の起動のコマンドと外部の道具）と
//! `mcp.json`（MCP の起動のコマンド。ホストの形式）。ソースは読まない ── ソースを読む検査は、
//! 言語の組が持つ（2段目）。

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::data_access::files;
use crate::data_access::process::{self, Failed, Session};

/// 起動のコマンドの中の、プロジェクトの場所を指す書き方。**ホスト（Claude Code）が展開する。**
pub const PROJECT_DIR: &str = "${CLAUDE_PROJECT_DIR:-.}";

/// 利用者が実行時に指定する外部の道具を表す名前。**command を持たない。**
pub const USER_CHOSEN: &str = "*";

/// 契約の版2 で、どの Skill も持つ references の道具。
pub const REFS_TOOLS: [&str; 4] = ["get", "validate", "view", "import"];

/// 契約の版を読む。**`tool.json` の contract の欄。無ければ 1 である。**
#[must_use]
pub fn contract_version(root: &Path) -> u64 {
    read_json(&root.join("tool.json"))
        .ok()
        .and_then(|d| d.get("contract").and_then(Value::as_u64))
        .unwrap_or(1)
}

/// `validate` を起動し、references がスキーマに合うかを見る。
fn references_pass(command: &Path, args: &[String]) -> Verdict {
    let mut with = args.to_vec();
    with.extend(["validate".to_owned(), "--json".to_owned()]);
    match output(command, &with, &files::temp_dir()) {
        Ok((0, _)) => Verdict::Pass("references の JSON がスキーマに合う".to_owned()),
        Ok((_, text)) => {
            let found: Vec<String> = serde_json::from_str::<Value>(text.trim())
                .ok()
                .and_then(|d| d.get("findings").and_then(Value::as_array).cloned())
                .unwrap_or_default()
                .iter()
                .filter_map(|x| x.as_str().map(str::to_owned))
                .collect();
            Verdict::Fail(format!(
                "references がスキーマに合わない ── {}",
                found.join(" ／ ")
            ))
        }
        Err(why) => Verdict::Fail(why),
    }
}

/// 実行ファイル1回の起動に待つ時間。**止まった実行ファイルを、永久に待たない。**
const LIMIT: Duration = Duration::from_secs(30);

/// 道具の一覧に無い旗の名前。**検査のためだけの名前で、どの道具も受けない。**
const NO_SUCH_FLAG: &str = "--no-such-flag-for-check";

/// 検査1件の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Verdict {
    /// 満たしている。
    Pass(String),
    /// 満たしていない。
    Fail(String),
}

/// `tool.json` と `mcp.json` を読み、書き方だけを見る。**起動はしない。**
#[must_use]
pub fn declaration(root: &Path) -> Vec<Verdict> {
    let mut out = Vec::new();
    match read_json(&root.join("tool.json")) {
        Err(why) => out.push(Verdict::Fail(format!(
            "tool.json が無いか読めない ── {why}（CLI の起動のコマンドと外部の道具を書く）"
        ))),
        Ok(doc) => {
            // **版1 の免除は無い**（ACDR 0075）── 道具を持つ Skill は、どれも版2 に従う
            if doc.get("contract").and_then(Value::as_u64).unwrap_or(1) < 2 {
                out.push(Verdict::Fail(
                    "契約の版が2でない: tool.json に \"contract\": 2 が無い ── 道具を持つ Skill は、references を JSON Schema と JSON で持ち、get ・ validate ・ view ・ import を持つ".to_owned(),
                ));
            }
            let cli = doc
                .pointer("/cli/command")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if cli.is_empty() {
                out.push(Verdict::Fail(
                    "tool.json に cli.command が無い ── CLI の起動のコマンドを書く".to_owned(),
                ));
            } else if is_absolute(cli) {
                out.push(Verdict::Fail(format!(
                    "登録に絶対パスがある: tool.json の cli ── {cli}（別の場所では存在しない場所を指す）"
                )));
            }
            for item in doc
                .get("external")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let field = |k: &str| item.get(k).and_then(Value::as_str).unwrap_or_default();
                let name = field("name");
                if field("reason").trim().is_empty() {
                    out.push(Verdict::Fail(format!(
                        "外部の道具に理由が無い: {name}（tool.json の external）── 目的に不可欠である理由を書く"
                    )));
                }
                if name != USER_CHOSEN && field("command").trim().is_empty() {
                    out.push(Verdict::Fail(format!(
                        "外部の道具に起動するコマンドが無い: {name}（tool.json の external）"
                    )));
                }
            }
        }
    }
    match read_json(&root.join("mcp.json")) {
        Err(why) => out.push(Verdict::Fail(format!("mcp.json が無いか読めない ── {why}"))),
        Ok(doc) => {
            for (name, server) in doc
                .get("mcpServers")
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
            {
                let command = server
                    .get("command")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if is_absolute(command) {
                    out.push(Verdict::Fail(format!(
                        "登録に絶対パスがある: mcp.json の {name} ── {command}（別の場所では存在しない場所を指す）"
                    )));
                }
            }
        }
    }
    if !out.iter().any(|v| matches!(v, Verdict::Fail(_))) {
        out.push(Verdict::Pass(
            "tool.json と mcp.json が在り、経路が絶対パスでない".to_owned(),
        ));
    }
    out
}

/// 実行ファイルを起動して、振る舞いを見る。**書き方に誤りが在れば、起動しない。**
#[must_use]
pub fn run(root: &Path) -> Vec<Verdict> {
    let mut out = declaration(root);
    if out.iter().any(|v| matches!(v, Verdict::Fail(_))) {
        return out;
    }
    let (Ok(tool), Ok(mcp)) = (
        read_json(&root.join("tool.json")),
        read_json(&root.join("mcp.json")),
    ) else {
        return out;
    };
    let cli = launch(root, tool.get("cli").unwrap_or(&Value::Null));
    let catalog = match cli
        .as_ref()
        .map_err(Clone::clone)
        .and_then(|(c, a)| catalog(c, a))
    {
        Err(why) => {
            out.push(Verdict::Fail(why));
            return out;
        }
        Ok(catalog) => catalog,
    };
    let names = tool_names(&catalog);
    out.push(Verdict::Pass(format!(
        "動詞なしの --json で道具の一覧を返す（道具 {}件）",
        names.len()
    )));
    out.push(skill_root_matches(root, &catalog));
    if let (Ok((command, args)), Some(first)) = (&cli, names.first()) {
        out.push(refuses_unknown_flag(command, args, first));
    }
    // **版2 の規則**（ACDR 0043）── references の4つの道具を持ち、references がスキーマに合う
    if contract_version(root) >= 2 {
        let missing: Vec<&str> = REFS_TOOLS
            .iter()
            .copied()
            .filter(|t| !names.iter().any(|n| n == t))
            .collect();
        if missing.is_empty() {
            out.push(Verdict::Pass(
                "references の4つの道具（get ・ validate ・ view ・ import）を持つ".to_owned(),
            ));
            if let Ok((command, args)) = &cli {
                out.push(references_pass(command, args));
            }
        } else {
            out.push(Verdict::Fail(format!(
                "references の道具が無い: {} ── 契約の版2 は get ・ validate ・ view ・ import を求める",
                missing.join(" ・ ")
            )));
        }
    }
    let servers: Vec<(String, Value)> = mcp
        .get("mcpServers")
        .and_then(Value::as_object)
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    if servers.is_empty() {
        out.push(Verdict::Fail(
            "mcp.json に mcpServers が無い ── MCP の起動のコマンドを書く".to_owned(),
        ));
    }
    for (name, server) in servers {
        out.push(match launch(root, &server) {
            Err(why) => Verdict::Fail(why),
            Ok((command, args)) => same_tools(&name, &command, &args, &catalog),
        });
    }
    out
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = files::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| format!("JSON でない ── {e}"))
}

/// Skill の CLI の起動のコマンドと引数を、`tool.json` から解く。**突き合わせ（conform）も使う。**
///
/// # Errors
///
/// `tool.json` が読めないとき、起動のコマンドの経路が無いときに返す。
pub fn cli_of(root: &Path) -> Result<(PathBuf, Vec<String>), String> {
    let tool = read_json(&root.join("tool.json"))
        .map_err(|e| format!("{} を読めない ── {e}", root.join("tool.json").display()))?;
    launch(root, tool.get("cli").unwrap_or(&Value::Null))
}

/// 絶対パスか。**Windows の書き方も含める** ── `C:\\…` ・ `C:/…`
#[must_use]
pub fn is_absolute(command: &str) -> bool {
    let bytes = command.as_bytes();
    command.starts_with('/')
        || command.starts_with('~')
        || (bytes.len() > 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
}

/// 起動のコマンドと引数を、この機械の上の経路へ解く。**プロジェクトの場所は、Skill の
/// フォルダから上へたどり、経路が実在する場所を採る** ── ホストが展開する値を、
/// 検査の側で推測しない。**引数の中の同じ値も解く** ── 処理系を介して起動する組
/// （`uv run … cli.py`）は、経路を引数に持つ。
fn launch(root: &Path, entry: &Value) -> Result<(PathBuf, Vec<String>), String> {
    let command = entry
        .get("command")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let raw: Vec<String> = entry
        .get("args")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|x| x.as_str().map(str::to_owned))
        .collect();
    // **絶対の経路へ解く** ── 実行ファイルは別の作業場所から起動するので、相対のままだと解けない
    let base = files::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let resolve = |value: &str| -> Option<Option<PathBuf>> {
        let rest = value.strip_prefix(PROJECT_DIR)?.trim_start_matches('/');
        Some(
            base.ancestors()
                .map(|dir| dir.join(rest))
                .find(|p| files::is_file(p) || files::is_dir(p)),
        )
    };
    let mut args = Vec::new();
    for a in raw {
        match resolve(&a) {
            None => args.push(a),
            Some(Some(p)) => args.push(p.display().to_string()),
            Some(None) => {
                return Err(format!(
                    "実行ファイルを起動できない: 引数 {a} の経路が無い（組み立ててから検査する）"
                ))
            }
        }
    }
    match resolve(command) {
        None => Ok((PathBuf::from(command), args)),
        Some(Some(p)) => Ok((p, args)),
        Some(None) => Err(format!(
            "実行ファイルを起動できない: {command} ── 実行ファイルが無い（組み立ててから検査する）"
        )),
    }
}

/// 起動して、終了コードと標準出力を返す。**標準入力は閉じ、制限時間で止める**（データアクセス層）。
fn output(command: &Path, args: &[String], cwd: &Path) -> Result<(i32, String), String> {
    process::run(command, args, cwd, LIMIT)
        .map(|ran| (ran.code, ran.stdout))
        .map_err(|why| failed(command, &why))
}

/// 起動の失敗を、検査の文にする。
fn failed(command: &Path, why: &Failed) -> String {
    match why {
        Failed::Timeout => format!(
            "実行ファイルが {} 秒で終わらない: {}",
            LIMIT.as_secs(),
            command.display()
        ),
        Failed::Spawn(e) => format!("実行ファイルを起動できない: {} ── {e}", command.display()),
        Failed::Pipe(e) => format!("実行ファイルと受け渡せない: {} ── {e}", command.display()),
        _ => format!("実行ファイルを起動できない: {}", command.display()),
    }
}

/// 動詞なしの `--json` で道具の一覧を読む。**別の作業場所から起動する** ── Skill のフォルダを
/// 作業場所に頼って求めていれば、ここで分かる。
fn catalog(command: &Path, args: &[String]) -> Result<Value, String> {
    let away = files::temp_dir();
    let mut with = args.to_vec();
    with.push("--json".to_owned());
    let (code, text) = output(command, &with, &away)?;
    if code != 0 {
        return Err(format!(
            "動詞なしの --json が道具の一覧を返さない: 終了コード {code}（道具の一覧を JSON で返し、0 で終える）"
        ));
    }
    let doc: Value = serde_json::from_str(text.trim())
        .map_err(|e| format!("動詞なしの --json の出力が JSON でない ── {e}"))?;
    if doc
        .pointer("/data/tools")
        .and_then(Value::as_array)
        .is_none()
    {
        return Err("動詞なしの --json の出力に data.tools が無い".to_owned());
    }
    Ok(doc)
}

fn tool_names(catalog: &Value) -> Vec<String> {
    catalog
        .pointer("/data/tools")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|t| t.get("name").and_then(Value::as_str).map(str::to_owned))
        .collect()
}

fn arg_names(tool: &Value) -> Vec<String> {
    let mut names: Vec<String> = tool
        .get("args")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|a| a.get("name").and_then(Value::as_str).map(str::to_owned))
        .collect();
    names.sort();
    names
}

fn skill_root_matches(root: &Path, catalog: &Value) -> Verdict {
    let said = catalog
        .pointer("/data/skill_root")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let same = |p: &Path| files::canonicalize(p).ok();
    if !said.is_empty() && same(Path::new(said)) == same(root) {
        Verdict::Pass("別の作業場所から起動しても、Skill のフォルダを求められる".to_owned())
    } else {
        Verdict::Fail(format!(
            "Skill のフォルダを求められない: 道具の一覧の skill_root が「{said}」── 渡された値か、実行ファイル自身の位置から求める"
        ))
    }
}

fn refuses_unknown_flag(command: &Path, args: &[String], verb: &str) -> Verdict {
    let mut with = args.to_vec();
    with.extend([verb.to_owned(), NO_SUCH_FLAG.to_owned(), "1".to_owned()]);
    match output(command, &with, &files::temp_dir()) {
        Ok((2, _)) => Verdict::Pass("道具の一覧に無い旗を、終了コード 2 で断る".to_owned()),
        Ok((code, _)) => Verdict::Fail(format!(
            "道具の一覧に無い旗を断らない: {verb} {NO_SUCH_FLAG} が終了コード {code}（2 で断る）"
        )),
        Err(why) => Verdict::Fail(why),
    }
}

/// MCP の実行ファイルを起動し、`tools/list` を CLI の道具の一覧と突き合わせる。**道具の名前と引数の
/// 名前が一致すること**を見る ── 能力を2か所に書いた実装は、ここで食い違う。
fn same_tools(server: &str, command: &Path, args: &[String], catalog: &Value) -> Verdict {
    let listed = match tools_list(command, args) {
        Ok(listed) => listed,
        Err(why) => return Verdict::Fail(format!("MCP の {server} ── {why}")),
    };
    let mut want: Vec<(String, Vec<String>)> = catalog
        .pointer("/data/tools")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|t| {
            (
                t.get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                arg_names(t),
            )
        })
        .collect();
    let mut got: Vec<(String, Vec<String>)> = listed
        .iter()
        .map(|t| {
            let mut props: Vec<String> = t
                .pointer("/inputSchema/properties")
                .and_then(Value::as_object)
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();
            props.sort();
            (
                t.get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                props,
            )
        })
        .collect();
    want.sort();
    got.sort();
    if want == got {
        Verdict::Pass(format!(
            "MCP の {server} の tools/list が、CLI の道具の一覧と一致する（{}件）",
            got.len()
        ))
    } else {
        Verdict::Fail(format!(
            "MCP の {server} の tools/list が、CLI の道具の一覧と食い違う ── CLI {want:?} ／ MCP {got:?}"
        ))
    }
}

/// MCP の実行ファイルと、初期化から `tools/list` までをやりとりする。**応答を読んだら止める。**
fn tools_list(command: &Path, args: &[String]) -> Result<Vec<Value>, String> {
    let mut session =
        Session::open(command, args, files::temp_dir()).map_err(|why| failed(command, &why))?;
    converse(&mut session).map_err(|why| match why {
        Talk::Pipe(e) => failed(command, &e),
        Talk::Silent(id) => {
            format!("応答が無い（id {id}）── 標準出力に MCP の通信以外を書いていないかを確かめる")
        }
    })
}

/// 対話の失敗。
enum Talk {
    Pipe(Failed),
    Silent(u64),
}

fn converse(session: &mut Session) -> Result<Vec<Value>, Talk> {
    let send = |s: &mut Session, v: Value| s.send(&v.to_string()).map_err(Talk::Pipe);
    send(
        session,
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {},
        "clientInfo": {"name": "skills-creator-check", "version": "1"}}}),
    )?;
    reply(session, 1)?;
    send(
        session,
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    )?;
    send(
        session,
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
    )?;
    let listed = reply(session, 2)?;
    Ok(listed
        .pointer("/result/tools")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

/// 識別子が一致する応答を待つ。**通知や別の応答、JSON でない行は読み飛ばす。**
fn reply(session: &Session, id: u64) -> Result<Value, Talk> {
    let deadline = Instant::now() + LIMIT;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        let line = session.recv(left).ok_or(Talk::Silent(id))?;
        let Ok(v) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if v.get("id").and_then(Value::as_u64) == Some(id) {
            return Ok(v);
        }
    }
}
