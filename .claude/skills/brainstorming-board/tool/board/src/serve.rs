//! ボードを配り、押された回答を1件ずつ answers/<回答の id>.json に書く（answer.schema.json のインスタンス）。
//!
//! **回答は消さない** ── 何を差し戻したかが、あとから順に読めるようにするためである。
//! 回答の形の検査は、書いたあとに基盤の check が行う。ここでは、書く前に分かる素の形だけを見る。

use crate::net::{Connection, Listener};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// 諾否の3値。
const VERDICTS: [&str; 3] = ["approved", "returned", "unanswered"];

/// 配るときの既定。
pub const DEFAULT_PORT: u16 = 8731;

/// 本文の上限。**受け取る量に上限を置く** ── 置かないと、1件で記憶を使い切れる。
const MAX_BODY: usize = 4_000_000;

/// 回答1件の素の形を検査する。返るのは、直すべきことの一覧（空なら通る）。board は配っているボードの id。
pub fn fault(body: &Value, board: &str) -> Vec<String> {
    let mut bad = Vec::new();
    if body.get("kind").and_then(Value::as_str) != Some("answer") {
        bad.push("kind が answer ではない".to_owned());
    }
    let id = body.get("id").and_then(Value::as_str).unwrap_or_default();
    if id.is_empty() || id.contains('/') || id.contains('\\') || id.starts_with('.') {
        bad.push("id が、ファイルの名前として使えない".to_owned());
    }
    if body.get("board").and_then(Value::as_str) != Some(board) {
        bad.push(format!("board が、配っているボード（{board}）と違う"));
    }
    let Some(answers) = body
        .get("answers")
        .and_then(Value::as_array)
        .filter(|list| !list.is_empty())
    else {
        bad.push("answers が、1件以上の配列ではない".to_owned());
        return bad;
    };
    let mut topics = Vec::new();
    for (index, answer) in answers.iter().enumerate() {
        let at = format!("answers[{index}]");
        let topic = answer
            .get("topic")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if topic.is_empty() {
            bad.push(format!("{at}.topic が無い"));
        }
        topics.push(topic);
        let verdict = answer
            .get("verdict")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !VERDICTS.contains(&verdict) {
            bad.push(format!(
                "{at}.verdict が {VERDICTS:?} のどれでもない（{verdict:?}）"
            ));
        }
        let has_reason = answer
            .get("reason")
            .and_then(Value::as_str)
            .is_some_and(|reason| !reason.trim().is_empty());
        // 理由は自由文である。決められた値に限ると、言えない理由が出たときに書けない
        if verdict == "returned" && !has_reason {
            bad.push(format!("{at}.reason が空（差し戻すなら理由を書く）"));
        }
    }
    // 論点に対して決定は1つなので、同じ論点が2件入っていたら受け取らない
    let mut repeated: Vec<&str> = topics
        .iter()
        .filter(|topic| topics.iter().filter(|other| other == topic).count() > 1)
        .copied()
        .collect();
    repeated.sort_unstable();
    repeated.dedup();
    if !repeated.is_empty() {
        bad.push(format!(
            "同じ論点が2件以上入っている（{}）── 論点に対して決定は1つである",
            repeated.join(" ・ ")
        ));
    }
    bad
}

/// 配っているボード（dir/board.json）の id と、回答が指すスキーマの相対パス。
fn board_of(dir: &Path) -> Result<(String, String), String> {
    let text = fs::read_to_string(dir.join("board.json"))
        .map_err(|error| format!("board.json を読めない ── {error}"))?;
    let board: Value = serde_json::from_str(&text)
        .map_err(|error| format!("board.json が JSON として読めない ── {error}"))?;
    let id = board
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let schema = board
        .get("$schema")
        .and_then(Value::as_str)
        .unwrap_or("board.schema.json");
    // answers/ は board.json の1つ下なので、../ を足して同じ references の answer.schema.json を指す
    let answer_schema = format!(
        "../{}",
        match schema.rsplit_once('/') {
            Some((dir, _)) => format!("{dir}/answer.schema.json"),
            None => "answer.schema.json".to_owned(),
        }
    );
    Ok((id, answer_schema))
}

/// 回答を受け取り、answers/<id>.json に書いて、置いた場所を返す。
/// **同じ内容が直前に届いていたら、積まない** ── 押し直しや二重発火で同じ回答が並ぶと、どれが最後の返答かが読めなくなる。
pub fn accept(dir: &Path, body: &Value) -> Result<Value, (u16, Value)> {
    let (board, answer_schema) = board_of(dir).map_err(|error| (500, json!({"error": error})))?;
    let bad = fault(body, &board);
    if !bad.is_empty() {
        return Err((422, json!({"error": "形が合っていない", "faults": bad})));
    }
    let answers = dir.join("answers");
    fs::create_dir_all(&answers).map_err(|error| {
        (
            500,
            json!({"error": format!("answers を作れない ── {error}")}),
        )
    })?;
    let mut instance = body.clone();
    if let Some(object) = instance.as_object_mut() {
        object.shift_insert(0, "$schema".to_owned(), Value::String(answer_schema));
    }
    let count = body
        .get("answers")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let mut kept: Vec<PathBuf> = fs::read_dir(&answers)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
                .collect()
        })
        .unwrap_or_default();
    kept.sort_by_key(|path| fs::metadata(path).and_then(|meta| meta.modified()).ok());
    if let Some(last) = kept.last() {
        let same = fs::read_to_string(last)
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .is_some_and(|previous| previous.get("answers") == instance.get("answers"));
        if same {
            return Ok(
                json!({"saved": last.display().to_string(), "count": count, "next": "直前と同じ内容だったので、積まずに済ませた"}),
            );
        }
    }
    let id = body.get("id").and_then(Value::as_str).unwrap_or_default();
    let out = answers.join(format!("{id}.json"));
    let text = serde_json::to_string_pretty(&instance).unwrap_or_default() + "\n";
    fs::write(&out, text).map_err(|error| {
        (
            500,
            json!({"error": format!("{}: 書けない ── {error}", out.display())}),
        )
    })?;
    Ok(
        json!({"saved": out.display().to_string(), "count": count, "next": "Claude に一言送ると、この回答が読まれます"}),
    )
}

/// 拡張子から、返す種類を決める。文字コードを言わないと、日本語が化ける。
fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .unwrap_or_default()
        .as_str()
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        _ => "text/plain; charset=utf-8",
    }
}

/// `%XX` を戻す。
fn percent_decoded(piece: &str) -> String {
    let bytes = piece.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(value) = u8::from_str_radix(&piece[i + 1..i + 3], 16) {
                out.push(value);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// 求められた道を、配るフォルダの中の場所へ解決する。**外へ出させない。** `/` は board.html を指す。
fn resolve(root: &Path, target: &str) -> Option<PathBuf> {
    let target = target.split('?').next().unwrap_or(target);
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
    let real = fs::canonicalize(&at).ok()?;
    let inside = fs::canonicalize(root).ok()?;
    real.starts_with(inside).then_some(real)
}

fn send(stream: &mut Connection, code: u16, reason: &str, kind: &str, body: &[u8]) {
    let head = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n",
        body.len()
    );
    stream.send(&[head.as_bytes(), body]);
}

fn send_json(stream: &mut Connection, code: u16, reason: &str, payload: &Value) {
    let body = serde_json::to_vec(payload).unwrap_or_default();
    send(
        stream,
        code,
        reason,
        "application/json; charset=utf-8",
        &body,
    );
}

fn handle(stream: &mut Connection, root: &Path) {
    let Some(line) = stream.read_line() else {
        return;
    };
    let mut parts = line.split_whitespace();
    let (method, target) = (
        parts.next().unwrap_or_default().to_owned(),
        parts.next().unwrap_or("/").to_owned(),
    );
    let mut length = 0usize;
    while let Some(header) = stream
        .read_line()
        .filter(|header| !header.trim().is_empty())
    {
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
                &json!({"error": "そのあて先は受け付けていない", "path": target}),
            );
            return;
        }
        if length == 0 || length > MAX_BODY {
            send_json(
                stream,
                400,
                "Bad Request",
                &json!({"error": "本文が空、または大きすぎる"}),
            );
            return;
        }
        let Some(raw) = stream.read_exact(length) else {
            send_json(
                stream,
                400,
                "Bad Request",
                &json!({"error": "本文を読めない"}),
            );
            return;
        };
        let Ok(body) = serde_json::from_slice::<Value>(&raw) else {
            send_json(
                stream,
                400,
                "Bad Request",
                &json!({"error": "JSON として読めない"}),
            );
            return;
        };
        match accept(root, &body) {
            Ok(payload) => send_json(stream, 200, "OK", &payload),
            Err((code, payload)) => send_json(stream, code, "Unprocessable", &payload),
        }
        eprintln!("受け取った: {target}");
        return;
    }
    match resolve(root, &target).and_then(|path| fs::read(&path).ok().map(|body| (path, body))) {
        Some((path, body)) => send(stream, 200, "OK", content_type(&path), &body),
        None => send(
            stream,
            404,
            "Not Found",
            "text/plain; charset=utf-8",
            "その場所は無い".as_bytes(),
        ),
    }
}

/// ボードのディレクトリを配り、押された回答を書く。**止められるまで返らない。**
pub fn run(dir: &str, port: u16) -> Result<(), String> {
    let root = fs::canonicalize(dir)
        .map_err(|error| format!("配るディレクトリが無い: {dir} ── {error}"))?;
    board_of(&root)?;
    let listener = Listener::bind("127.0.0.1", port)
        .map_err(|error| format!("待ち受けられない: 127.0.0.1:{port} ── {error}"))?;
    println!("配る  : {}", root.display());
    println!("回答  : {}", root.join("answers").display());
    println!("開く  : http://127.0.0.1:{port}/");
    for mut stream in listener.incoming() {
        handle(&mut stream, &root);
    }
    Ok(())
}
