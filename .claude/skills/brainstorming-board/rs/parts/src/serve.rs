// SPDX-License-Identifier: MIT
//! ブレストボードを配り、押された回答を1件ずつファイルへ書き出す。
//!
//! 画面からの `POST /answer` を受け、本文（JSON）を `<蓄積先>/<日時>.json` に書く。蓄積先に
//! `{board}` を書くと、本文の `board` の値へ置き換わる ── 回答は、そのブレストボードの持ち物だからである。
//!
//! **回答は消さない** ── 何を差し戻したかが、あとから順に読めるようにするためである。
//!
//! 形は `references/answer-sheet.schema.json` が決めるが、ここでは**素の形だけ**を検査する
//! （必須の欄が在るか、諾否が決められた値か）。中身が妥当かは機械には分からない。

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

/// 諾否の3値。
const VERDICTS: [&str; 3] = ["approved", "returned", "unanswered"];

/// 配るときの既定。
pub const DEFAULT_PORT: u16 = 8731;

/// 本文の上限。**受け取る量に上限を置く** ── 置かないと、1件で記憶を使い切れる。
const MAX_BODY: usize = 4_000_000;

/// 回答1件の素の形を検査する。返るのは、直すべきことの一覧（空なら通る）。
#[must_use]
pub fn fault(body: &Value) -> Vec<String> {
    let mut bad = Vec::new();
    let Some(map) = body.as_object() else {
        return vec!["いちばん外側が object ではない".to_owned()];
    };
    for key in ["board", "answeredAt", "answers"] {
        if !map.contains_key(key) {
            bad.push(format!("{key} が無い"));
        }
    }
    let answers = map.get("answers").and_then(|x| x.as_array());
    let Some(answers) = answers.filter(|x| !x.is_empty()) else {
        bad.push("answers が、1件以上の配列ではない".to_owned());
        return bad;
    };
    for (i, a) in answers.iter().enumerate() {
        let where_ = format!("answers[{i}]");
        let Some(a) = a.as_object() else {
            bad.push(format!("{where_} が object ではない"));
            continue;
        };
        if !a
            .get("questionId")
            .and_then(|x| x.as_str())
            .is_some_and(|x| !x.is_empty())
        {
            bad.push(format!("{where_}.questionId が無い"));
        }
        let verdict = a
            .get("verdict")
            .and_then(|x| x.as_str())
            .unwrap_or_default();
        if !VERDICTS.contains(&verdict) {
            bad.push(format!(
                "{where_}.verdict が {:?} のどれでもない（{verdict:?}）",
                VERDICTS
            ));
        }
        let reason = a.get("returnReason");
        let has_reason = reason
            .and_then(|x| x.as_str())
            .is_some_and(|x| !x.trim().is_empty());
        // 理由は自由文である。**決められた値に限ると、言えない理由が出たときに書けない。**
        if verdict == "returned" && !has_reason {
            bad.push(format!(
                "{where_}.returnReason が空（差し戻すなら理由を書く）"
            ));
        }
        if verdict != "returned" && has_reason {
            bad.push(format!(
                "{where_}.returnReason は、差し戻し以外では空でなければならない"
            ));
        }
    }
    // 論点に対して決定は1つなので、同じ問いが2件入っていたら受け取らない
    let ids: Vec<&str> = answers
        .iter()
        .filter_map(|a| a.get("questionId").and_then(|x| x.as_str()))
        .collect();
    let mut dup: Vec<&str> = ids
        .iter()
        .filter(|id| ids.iter().filter(|x| x == id).count() > 1)
        .copied()
        .collect();
    dup.sort_unstable();
    dup.dedup();
    if !dup.is_empty() {
        bad.push(format!(
            "同じ問いが2件以上入っている（{}）── 論点に対して決定は1つである",
            dup.join("・")
        ));
    }
    bad
}

/// 蓄積先を決める。`{board}` は本文の `board` の値へ置き換える。
///
/// **置き場所の名前になるので、区切りや親への遡りは受け取らない。**
#[must_use]
pub fn dir_for(pattern: &str, body: &Value) -> Option<PathBuf> {
    if !pattern.contains("{board}") {
        return Some(PathBuf::from(pattern));
    }
    let board = body
        .get("board")
        .and_then(|x| x.as_str())
        .unwrap_or_default();
    if board.is_empty() || board.contains('/') || board.contains('\\') || board.starts_with('.') {
        return None;
    }
    Some(PathBuf::from(pattern.replace("{board}", board)))
}

/// 拡張子から、返す種類を決める。
fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .map(|x| x.to_string_lossy().to_lowercase())
        .unwrap_or_default()
        .as_str()
    {
        // 文字コードを言わないと、日本語が化ける
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "woff2" => "font/woff2",
        _ => "text/plain; charset=utf-8",
    }
}

/// 頼まれた道を、配るフォルダの中の場所へ解決する。**外へ出させない。**
fn resolve(root: &Path, target: &str) -> Option<PathBuf> {
    let target = target.split('?').next().unwrap_or(target);
    // `/` を board.html へ向ける ── **index.html を書き出さないため**である。
    // 複製を置くと、board.html を作り直したときに片方が古くなる
    let target = if target == "/" || target.is_empty() {
        "/board.html"
    } else {
        target
    };
    let mut at = root.to_path_buf();
    for piece in target.trim_start_matches('/').split('/') {
        if piece.is_empty() || piece == "." {
            continue;
        }
        if piece == ".." {
            return None;
        }
        at.push(percent_decoded(piece));
    }
    let real = std::fs::canonicalize(&at).ok()?;
    let inside = std::fs::canonicalize(root).ok()?;
    real.starts_with(inside).then_some(real)
}

/// `%XX` を戻す。
fn percent_decoded(piece: &str) -> String {
    let bytes = piece.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&piece[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn send(stream: &mut TcpStream, code: u16, reason: &str, kind: &str, body: &[u8], store: bool) {
    let mut head = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    if store {
        head.push_str("Cache-Control: no-store\r\n");
    }
    head.push_str("\r\n");
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
}

fn send_json(stream: &mut TcpStream, code: u16, reason: &str, payload: &Value) {
    let body = serde_json::to_vec(payload).unwrap_or_default();
    send(
        stream,
        code,
        reason,
        "application/json; charset=utf-8",
        &body,
        false,
    );
}

fn now() -> String {
    Command::new("date")
        .arg("+%Y%m%dT%H%M%S")
        .stdin(Stdio::null())
        .output()
        .ok()
        .map_or_else(String::new, |o| {
            String::from_utf8_lossy(&o.stdout).trim().to_owned()
        })
}

/// 回答を受け取り、置いた場所を返す。
///
/// **同じ内容が直前に届いていたら、積まない** ── 押し直しや二重発火で同じ回答が並ぶと、
/// どれが最後の返答かを読む側が判断できなくなる。
///
/// # Errors
///
/// 形が合っていないときと、書けないときに、返す値と番号を返す。
pub fn accept(pattern: &str, body: &Value) -> Result<Value, (u16, Value)> {
    let bad = fault(body);
    if !bad.is_empty() {
        return Err((
            422,
            serde_json::json!({ "error": "形が合っていない", "faults": bad }),
        ));
    }
    let Some(dir) = dir_for(pattern, body) else {
        return Err((
            422,
            serde_json::json!({
                "error": "board の値が、置き場所の名前として使えない",
                "board": body.get("board"),
            }),
        ));
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return Err((
            500,
            serde_json::json!({ "error": format!("{}: 作れない ── {e}", dir.display()) }),
        ));
    }
    let count = body
        .get("answers")
        .and_then(|x| x.as_array())
        .map_or(0, Vec::len);
    let mut kept: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "json"))
                .collect()
        })
        .unwrap_or_default();
    kept.sort();
    if let Some(last) = kept.last() {
        if std::fs::read_to_string(last)
            .ok()
            .and_then(|x| serde_json::from_str::<Value>(&x).ok())
            .is_some_and(|x| x == *body)
        {
            return Ok(serde_json::json!({
                "saved": last.display().to_string(), "count": count,
                "next": "直前と同じ内容だったので、積まずに済ませた",
            }));
        }
    }
    let out = dir.join(format!("{}.json", now()));
    let text = serde_json::to_string_pretty(body).unwrap_or_default() + "\n";
    if let Err(e) = std::fs::write(&out, text) {
        return Err((
            500,
            serde_json::json!({ "error": format!("{}: 書けない ── {e}", out.display()) }),
        ));
    }
    Ok(serde_json::json!({
        "saved": out.display().to_string(), "count": count,
        "next": "Claude に一言送ると、この回答が読まれます",
    }))
}

fn handle(stream: &mut TcpStream, root: &Path, pattern: &str) {
    let mut reader = BufReader::new(match stream.try_clone() {
        Ok(x) => x,
        Err(_) => return,
    });
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let mut parts = line.split_whitespace();
    let (method, target) = (
        parts.next().unwrap_or_default().to_owned(),
        parts.next().unwrap_or("/").to_owned(),
    );
    let mut length = 0usize;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).is_err() || header.trim().is_empty() {
            break;
        }
        if let Some(value) = header
            .to_lowercase()
            .strip_prefix("content-length:")
            .map(str::trim)
        {
            length = value.parse().unwrap_or(0);
        }
    }
    if method == "POST" {
        if target.trim_end_matches('/') != "/answer" {
            send_json(
                stream,
                404,
                "Not Found",
                &serde_json::json!({ "error": "そのあて先は受け付けていない", "path": target }),
            );
            return;
        }
        if length == 0 || length > MAX_BODY {
            send_json(
                stream,
                400,
                "Bad Request",
                &serde_json::json!({ "error": "本文が空、または大きすぎる" }),
            );
            return;
        }
        let mut raw = vec![0u8; length];
        if reader.read_exact(&mut raw).is_err() {
            send_json(
                stream,
                400,
                "Bad Request",
                &serde_json::json!({ "error": "本文を読めない" }),
            );
            return;
        }
        let Ok(body) = serde_json::from_slice::<Value>(&raw) else {
            send_json(
                stream,
                400,
                "Bad Request",
                &serde_json::json!({ "error": "JSON として読めない" }),
            );
            return;
        };
        match accept(pattern, &body) {
            Ok(payload) => send_json(stream, 200, "OK", &payload),
            Err((code, payload)) => send_json(stream, code, "Unprocessable", &payload),
        }
        // **配信のたびに1行出すのは、うるさい** ── 受け取ったときだけ出す
        eprintln!("受け取った: {}", target);
        return;
    }
    let Some(path) = resolve(root, &target) else {
        send(
            stream,
            404,
            "Not Found",
            "text/plain; charset=utf-8",
            b"\xe3\x81\x9d\xe3\x81\xae\xe5\xa0\xb4\xe6\x89\x80\xe3\x81\xaf\xe7\x84\xa1\xe3\x81\x84",
            false,
        );
        return;
    };
    match std::fs::read(&path) {
        Ok(body) => {
            let kind = content_type(&path);
            let store = kind.starts_with("text/html");
            send(stream, 200, "OK", kind, &body, store);
        }
        Err(_) => send(
            stream,
            404,
            "Not Found",
            "text/plain; charset=utf-8",
            b"\xe8\xaa\xad\xe3\x82\x81\xe3\x81\xaa\xe3\x81\x84",
            false,
        ),
    }
}

/// 配って、押された回答を蓄積する。**止められるまで返らない。**
///
/// # Errors
///
/// 配るフォルダが無いときと、待ち受けられないときに返す。
pub fn run(root: &str, answers: &str, port: u16) -> Result<Vec<String>, String> {
    let root = std::fs::canonicalize(root)
        .map_err(|e| format!("配るディレクトリが無い: {root} ── {e}"))?;
    if !root.is_dir() {
        return Err(format!("配るディレクトリが無い: {}", root.display()));
    }
    let pattern = if answers.contains("{board}") {
        answers.to_owned()
    } else {
        std::fs::canonicalize(answers)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| answers.to_owned())
    };
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|e| format!("待ち受けられない: 127.0.0.1:{port} ── {e}"))?;
    let lines = vec![
        format!("配る  : {}", root.display()),
        format!("蓄積  : {pattern}"),
        format!("開く  : http://127.0.0.1:{port}/"),
    ];
    for line in &lines {
        println!("{line}");
    }
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => handle(&mut stream, &root, &pattern),
            Err(_) => continue,
        }
    }
    Ok(lines)
}
